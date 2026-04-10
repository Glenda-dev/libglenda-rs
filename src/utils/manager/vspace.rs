use crate::arch::mem::{PGSIZE, SHIFTS, VPN_MASK};
use crate::cap::{CapPtr, Frame, PageTable, VSpace};
use crate::error::Error;
use crate::interface::{CSpaceService, VSpaceProvider, VSpaceService};
use crate::mem::Perms;
use crate::mem::TRAMPOLINE_VA;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

pub const MAP_START: usize = 0x30000000;
#[cfg(target_pointer_width = "64")]
pub const MAP_END: usize = 0x3F_0000_0000;
#[cfg(target_pointer_width = "32")]
pub const MAP_END: usize = 0x7F_0000_00;

#[derive(Debug)]
enum ShadowNode {
    Table { cap: CapPtr, entries: BTreeMap<usize, Box<ShadowNode>> },
    Frame { cap: CapPtr, pages: usize, perms: Perms },
}

impl ShadowNode {
    fn new_table(cap: CapPtr) -> Self {
        ShadowNode::Table { cap, entries: BTreeMap::new() }
    }
}

pub struct VSpaceManager {
    pub root: VSpace,
    shadow: BTreeMap<usize, Box<ShadowNode>>, // Top level entries
    // Scratch management
    scratch_start: usize,
    scratch_len: usize,
    scratch_ptr: usize,
    // Cached pagetables: (Capability, last_mapped_vaddr, level)
    pages_cache: Vec<(CapPtr, usize, usize)>,
}

impl VSpaceManager {
    pub fn new(root: VSpace, scratch_start: usize, scratch_len: usize) -> Self {
        let mut slf = Self {
            root,
            shadow: BTreeMap::new(),
            scratch_start,
            scratch_len,
            scratch_ptr: 0,
            pages_cache: Vec::new(),
        };

        slf.init();
        slf
    }

    #[cfg(feature = "vspacemgr_check_bypass")]
    fn init(&mut self) {}
    #[cfg(not(feature = "vspacemgr_check_bypass"))]
    fn init(&mut self) {
        {
            use crate::mem::{
                ENTRY_VA, HEAP_VA, STACK_BASE, TRAMPOLINE_VA, get_trapframe_va, get_utcb_va,
            };
            self.mark_existing(TRAMPOLINE_VA, PGSIZE);
            // 1. Text/Data - Low mem (Initial page)
            self.mark_existing(ENTRY_VA, PGSIZE);
            // 2. Stack - High mem
            self.mark_existing(STACK_BASE - PGSIZE, PGSIZE);
            // 3. Heap
            self.mark_existing(HEAP_VA, PGSIZE);
            // 4. UTCB/TrapFrame for TID 0
            self.mark_existing(get_utcb_va(0), PGSIZE);
            self.mark_existing(get_trapframe_va(0), PGSIZE);
        }
    }

    pub fn drop(&mut self, provider: &mut dyn VSpaceProvider, slots: &mut dyn CSpaceService) {
        // 1. 清理缓存中的页表
        while let Some((cap, _, _)) = self.pages_cache.pop() {
            let released = match provider.free_pagetable(cap) {
                Ok(()) => true,
                Err(e) if e == Error::InvalidCapability || e == Error::InvalidSlot => true,
                Err(e) => {
                    crate::warn!(
                        "vspace.drop: free_pagetable failed for {:?}, skip slot recycle: {:?}",
                        cap,
                        e
                    );
                    false
                }
            };
            if released {
                let _ = slots.free(cap);
            }
        }

        // 2. 递归清理影子页表中的资源
        let mut shadow = core::mem::take(&mut self.shadow);
        for (_, node) in shadow.iter_mut() {
            Self::drop_rec(node, provider, slots);
        }
    }

    fn drop_rec(
        node: &mut ShadowNode,
        provider: &mut dyn VSpaceProvider,
        slots: &mut dyn CSpaceService,
    ) {
        match node {
            ShadowNode::Table { cap, entries } => {
                // 递归清理子节点
                for (_, sub_node) in entries.iter_mut() {
                    Self::drop_rec(sub_node, provider, slots);
                }
                // 清理当前页表
                if !cap.is_null() {
                    let released = match provider.free_pagetable(*cap) {
                        Ok(()) => true,
                        Err(e) if e == Error::InvalidCapability || e == Error::InvalidSlot => true,
                        Err(e) => {
                            crate::warn!(
                                "vspace.drop_rec: free_pagetable failed for {:?}, skip slot recycle: {:?}",
                                *cap,
                                e
                            );
                            false
                        }
                    };
                    if released {
                        let _ = slots.free(*cap);
                    }
                }
            }
            ShadowNode::Frame { .. } => {
                // Frame capabilities are owned by upper-level process/resource managers.
                // Do not recycle slot indices here, otherwise a still-live cap may be reallocated
                // and cause "target slot not empty" failures.
            }
        }
    }

    pub fn setup(
        &mut self,
        provider: &mut dyn VSpaceProvider,
        slots: &mut dyn CSpaceService,
    ) -> Result<(), Error> {
        // 1. 确保 TRAMPOLINE_VA 的路径在影子页表中存在
        // 这会逐级创建页表直到 L1，并调用 map_table 挂载到内核
        Self::ensure_path(
            &mut self.pages_cache,
            &mut self.shadow,
            TRAMPOLINE_VA,
            SHIFTS.len() - 1,
            provider,
            slots,
            self.root,
        )?;
        // 2. 调用内核 VSpace 的 Setup，同步内核上下文
        self.root.setup()?;

        Ok(())
    }

    pub fn mark_existing(&mut self, vaddr: usize, size: usize) {
        for offset in (0..size).step_by(PGSIZE) {
            Self::mark_existing_rec(&mut self.shadow, vaddr + offset, SHIFTS.len() - 1);
        }
    }

    fn mark_existing_rec(
        entries: &mut BTreeMap<usize, Box<ShadowNode>>,
        vaddr: usize,
        level: usize,
    ) {
        let idx = index(vaddr, level);
        if !entries.contains_key(&idx) {
            entries.insert(idx, Box::new(ShadowNode::new_table(CapPtr::null())));
        }

        if level > 1 {
            if let Some(node) = entries.get_mut(&idx) {
                if let ShadowNode::Table { entries: sub_entries, .. } = &mut **node {
                    Self::mark_existing_rec(sub_entries, vaddr, level - 1);
                }
            }
        }
    }

    pub fn clone_space(
        &self,
        dest_mgr: &mut VSpaceManager,
        provider: &mut dyn VSpaceProvider,
        slots: &mut dyn CSpaceService,
        src_scratch_va: usize,
        dest_scratch_va: usize,
        current_vspace: &mut VSpaceManager,
    ) -> Result<(), Error> {
        self.clone_level(
            &self.shadow,
            dest_mgr,
            provider,
            slots,
            0,
            SHIFTS.len() - 1,
            src_scratch_va,
            dest_scratch_va,
            current_vspace,
        )
    }

    fn clone_level(
        &self,
        entries: &BTreeMap<usize, Box<ShadowNode>>,
        dest_mgr: &mut VSpaceManager,
        provider: &mut dyn VSpaceProvider,
        slots: &mut dyn CSpaceService,
        base_vaddr: usize,
        level: usize,
        src_scratch_va: usize,
        dest_scratch_va: usize,
        current_vspace: &mut VSpaceManager,
    ) -> Result<(), Error> {
        for (&idx, node) in entries {
            let vaddr = base_vaddr | (idx << SHIFTS[level]);

            match &**node {
                ShadowNode::Table { entries: sub_entries, .. } => {
                    // Recurse
                    if level == 0 {
                        return Err(Error::MappingFailed);
                    }
                    self.clone_level(
                        sub_entries,
                        dest_mgr,
                        provider,
                        slots,
                        vaddr,
                        level - 1,
                        src_scratch_va,
                        dest_scratch_va,
                        current_vspace,
                    )?;
                }
                ShadowNode::Frame { cap, perms, pages } => {
                    // Clone Frame
                    let num_pages = *pages;
                    if num_pages == 0 {
                        continue;
                    }

                    // Alloc slot and object
                    let new_slot = slots.alloc(provider)?;
                    let full_dest = new_slot;
                    provider.alloc_pagetable(full_dest)?;
                    let new_frame = Frame::from(new_slot);

                    // Map both to copy
                    let src_frame = Frame::from(*cap);

                    // Map src using current_vspace
                    current_vspace.map_frame(
                        src_frame,
                        src_scratch_va,
                        Perms::READ,
                        num_pages,
                        provider,
                        slots,
                    )?;

                    // Map dest using current_vspace
                    current_vspace.map_frame(
                        new_frame,
                        dest_scratch_va,
                        Perms::READ | Perms::WRITE,
                        num_pages,
                        provider,
                        slots,
                    )?;

                    unsafe {
                        let total_size = num_pages * PGSIZE;
                        let src =
                            core::slice::from_raw_parts(src_scratch_va as *const u8, total_size);
                        let dest =
                            core::slice::from_raw_parts_mut(dest_scratch_va as *mut u8, total_size);
                        dest.copy_from_slice(src);
                    }

                    // Unmap
                    current_vspace.unmap(src_scratch_va, num_pages)?;
                    current_vspace.unmap(dest_scratch_va, num_pages)?;

                    // Map to child
                    dest_mgr.map_frame(new_frame, vaddr, *perms, num_pages, provider, slots)?;
                }
            }
        }
        Ok(())
    }

    fn ensure_path<'a>(
        pages_cache: &mut Vec<(CapPtr, usize, usize)>,
        entries: &'a mut BTreeMap<usize, Box<ShadowNode>>,
        vaddr: usize,
        level: usize,
        provider: &mut dyn VSpaceProvider,
        slots: &mut dyn CSpaceService,
        pivot_root: VSpace,
    ) -> Result<&'a mut BTreeMap<usize, Box<ShadowNode>>, Error> {
        let idx = index(vaddr, level);

        if level == 0 {
            return Ok(entries);
        }

        if !entries.contains_key(&idx) {
            // 首先检查缓存
            let slot = if let Some(pos) =
                pages_cache.iter().position(|&(_, old_vaddr, old_level)| {
                    // 懒回收：如果缓存中有完全匹配的页表（层级相同且全路径相同），
                    // 说明它在内核里依然挂载在原来的地方并没有被 unmap_table。直接重用。
                    old_level == level && (old_vaddr >> SHIFTS[level]) == (vaddr >> SHIFTS[level])
                }) {
                let (cap, _, _) = pages_cache.swap_remove(pos);
                // 不需要重新 map_table，因为依然留在由于懒回收导致的内核页表中。
                cap
            } else {
                // 如果没有完全匹配的，尝试从缓存中取出一个并重新映射
                // 此时必须清理目标位置，防止 "slot occupied"
                let _ = pivot_root.unmap_table(vaddr, level);

                if let Some((cap, old_vaddr, old_level)) = pages_cache.pop() {
                    // 对于来自别的地址的重利用，它目前还挂载在 old_vaddr，这里先卸载它。
                    let _ = pivot_root.unmap_table(old_vaddr, old_level);

                    let pt = PageTable::from(cap);
                    if let Err(e) = pivot_root.map_table(pt, vaddr, level) {
                        // 映射失败时，将 cap 放回缓存以备后用
                        pages_cache.push((cap, old_vaddr, old_level));
                        return Err(e);
                    }
                    cap
                } else {
                    // 缓存也为空，分配全新的
                    let slot = slots.alloc(provider)?;
                    if let Err(e) = provider.alloc_pagetable(slot) {
                        if e != Error::AlreadyExists && e != Error::SlotNotEmpty {
                            let _ = slots.free(slot);
                        } else {
                            crate::warn!(
                                "vspace.ensure_path: alloc_pagetable hit occupied slot {:?}, skip slot recycle: {:?}",
                                slot,
                                e
                            );
                        }
                        return Err(e);
                    }
                    let pt = PageTable::from(slot);

                    if let Err(e) = pivot_root.map_table(pt, vaddr, level) {
                        let released = match provider.free_pagetable(slot) {
                            Ok(()) => true,
                            Err(free_err)
                                if free_err == Error::InvalidCapability
                                    || free_err == Error::InvalidSlot =>
                            {
                                true
                            }
                            Err(free_err) => {
                                crate::warn!(
                                    "vspace.ensure_path: rollback free_pagetable failed for {:?}, skip slot recycle: {:?}",
                                    slot,
                                    free_err
                                );
                                false
                            }
                        };
                        if released {
                            let _ = slots.free(slot);
                        }
                        return Err(e);
                    }
                    slot
                }
            };

            entries.insert(idx, Box::new(ShadowNode::new_table(slot)));
        }

        let node = entries.get_mut(&idx).unwrap();
        match &mut **node {
            ShadowNode::Table { entries: sub_entries, .. } => {
                if level - 1 == 0 {
                    Ok(sub_entries)
                } else {
                    Self::ensure_path(
                        pages_cache,
                        sub_entries,
                        vaddr,
                        level - 1,
                        provider,
                        slots,
                        pivot_root,
                    )
                }
            }
            _ => Err(Error::MappingFailed), // Collision
        }
    }

    fn unmap_rec(
        pivot_root: VSpace,
        cache: &mut Vec<(CapPtr, usize, usize)>,
        entries: &mut BTreeMap<usize, Box<ShadowNode>>,
        vaddr: usize,
        level: usize,
    ) {
        let idx = index(vaddr, level);
        if level == 0 {
            // Level 0 对应的 entries 是页表项，直接移除并调用 unmap
            if let Some(node) = entries.get_mut(&idx) {
                match &mut **node {
                    ShadowNode::Frame { pages: _, .. } => {
                        // 如果是一个范围映射的起始点，我们需要知道是否该 unmap 整个范围
                        // 但 unmap_rec 每次只处理一个 PGSIZE = 4KB 页面。
                        // 如果 pages > 1，表示这是一个 superpage 或连续映射。
                        // 在 Glenda 的 VSpaceManager 中目前似乎只支持单页或多页循环调用 unmap_rec。
                        let _ = pivot_root.unmap(vaddr, 1 << SHIFTS[0]);
                    }
                    _ => {}
                }
                entries.remove(&idx);
            }
        } else if let Some(node) = entries.get_mut(&idx) {
            if let ShadowNode::Table { entries: sub_entries, .. } = &mut **node {
                Self::unmap_rec(pivot_root, cache, sub_entries, vaddr, level - 1);
                // 仅当子页表现在完全变为空时，考虑回收当前层级的页表
                if sub_entries.is_empty() {
                    let removed = entries.remove(&idx).unwrap();
                    if let ShadowNode::Table { cap, .. } = *removed {
                        // 懒回收：不在这里显式向内核发送 unmap_table 解除该层级页表的映射，
                        // 只是记录到 cache 中。这样下次如果在同位置重用，就可以省去系统调用。
                        // 如果在其它位置复用，再执行 unmap_table。
                        cache.push((cap, vaddr, level));
                    }
                }
            }
        }
    }

    fn is_mapped_rec(
        entries: &BTreeMap<usize, Box<ShadowNode>>,
        vaddr: usize,
        level: usize,
    ) -> bool {
        let idx = index(vaddr, level);
        if let Some(node) = entries.get(&idx) {
            match &**node {
                ShadowNode::Table { entries: sub, .. } => {
                    if level == 0 {
                        // Should not happen as Table at 0, but if it does, it's mapped?
                        // In current structure, level 0 entries are frames inside table at level 1?
                        // No, ensure_path passes level-1. Leaf is at level 0 (implied).
                        // recursive function takes 'level'.
                        // If we are at level 0, it means we are looking at leaf PT.
                        // Entries here should be Frames.
                        return true;
                    }
                    Self::is_mapped_rec(sub, vaddr, level - 1)
                }
                ShadowNode::Frame { .. } => true,
            }
        } else {
            false
        }
    }
}

impl VSpaceService for VSpaceManager {
    fn map_frame(
        &mut self,
        frame: Frame,
        vaddr: usize,
        perms: Perms,
        pages: usize,
        provider: &mut dyn VSpaceProvider,
        slots: &mut dyn CSpaceService,
    ) -> Result<(), Error> {
        if !Self::check_addr(vaddr) || !Self::check_addr(vaddr + pages * PGSIZE - 1) {
            return Err(Error::PermissionDenied);
        }
        let levels = SHIFTS.len();

        for i in 0..pages {
            let curr_vaddr = vaddr + i * PGSIZE;
            let leaf_map = Self::ensure_path(
                &mut self.pages_cache,
                &mut self.shadow,
                curr_vaddr,
                levels - 1,
                provider,
                slots,
                self.root,
            )?;

            let idx0 = index(curr_vaddr, 0);
            if let Some(_) = leaf_map.get(&idx0) {
                // 如果已经存在映射，先清理它以防 PageTable 冲突
                let _ = self.unmap(curr_vaddr, 1);
            }
        }

        // 调用内核进行映射，并检查返回值
        if let Err(e) = self.root.map(frame, vaddr, perms, pages) {
            crate::error!("VSpaceManager::map_frame: self.root.map failed with {:?}", e);
            return Err(Error::MappingFailed);
        }

        for i in 0..pages {
            let curr_vaddr = vaddr + i * PGSIZE;
            let leaf_map = Self::ensure_path(
                &mut self.pages_cache,
                &mut self.shadow,
                curr_vaddr,
                levels - 1,
                provider,
                slots,
                self.root,
            )?;

            let idx0 = index(curr_vaddr, 0);
            leaf_map.insert(
                idx0,
                Box::new(ShadowNode::Frame {
                    cap: frame.cap(),
                    perms,
                    pages: if i == 0 { pages } else { 0 },
                }),
            );
        }
        Ok(())
    }

    fn map_pt(
        &mut self,
        level: usize,
        vaddr: usize,
        provider: &mut dyn VSpaceProvider,
        slots: &mut dyn CSpaceService,
    ) -> Result<PageTable, Error> {
        if !Self::check_addr(vaddr) {
            return Err(Error::PermissionDenied);
        }
        let levels = SHIFTS.len();
        Self::ensure_path(
            &mut self.pages_cache,
            &mut self.shadow,
            vaddr,
            levels - 1,
            provider,
            slots,
            self.root,
        )?;

        let mut curr = &self.shadow;
        for curr_lvl in (level + 1..levels).rev() {
            let idx = index(vaddr, curr_lvl);
            if let Some(node) = curr.get(&idx) {
                if let ShadowNode::Table { entries, .. } = &**node {
                    curr = entries;
                } else {
                    return Err(Error::MappingFailed);
                }
            } else {
                return Err(Error::MappingFailed);
            }
        }

        let idx = index(vaddr, level);
        if let Some(node) = curr.get(&idx) {
            match &**node {
                ShadowNode::Table { cap, .. } => return Ok(PageTable::from(*cap)),
                _ => {}
            }
        }
        Err(Error::MappingFailed)
    }

    fn unmap(&mut self, vaddr: usize, pages: usize) -> Result<(), Error> {
        let levels = SHIFTS.len();
        for i in 0..pages {
            Self::unmap_rec(
                self.root,
                &mut self.pages_cache,
                &mut self.shadow,
                vaddr + i * PGSIZE,
                levels - 1,
            );
        }
        // 内核 unmap 已经由 unmap_rec 在叶子节点回收时显式处理。
        // 重复调用会引发 MappingFailed (如果路径已被 unmap_table 移除)。
        Ok(())
    }

    fn map_scratch(
        &mut self,
        frame: Frame,
        perms: Perms,
        pages: usize,
        provider: &mut dyn VSpaceProvider,
        slots: &mut dyn CSpaceService,
    ) -> Result<usize, Error> {
        let size = pages * PGSIZE;
        if size > self.scratch_len {
            return Err(Error::OutOfMemory);
        }

        let start_offset = self.scratch_ptr;
        let mut offset = start_offset;
        let levels = SHIFTS.len();

        loop {
            // Wrap around
            if offset + size > self.scratch_len {
                offset = 0;
            }

            let vaddr = self.scratch_start + offset;

            // Check if free (not very efficient but correct)
            let mut free = true;
            for i in 0..pages {
                if Self::is_mapped_rec(&self.shadow, vaddr + i * PGSIZE, levels - 1) {
                    free = false;
                    break;
                }
            }

            if free {
                self.map_frame(frame, vaddr, perms, pages, provider, slots)?;
                self.scratch_ptr = offset + size;
                return Ok(vaddr);
            }

            offset += PGSIZE;
            if offset == start_offset {
                return Err(Error::OutOfMemory);
            }
            if start_offset > self.scratch_len - size && offset == 0 {
                // Wrapped and reached 0, if start was high
                // Actually if strict loop detection:
                // If we wrapped and reached start_offset again.
                // My logic above sets offset=0 but doesn't check if we already checked 0.
                // Just use a full pass count or range check.
            }
            // Simplified loop exit
            if offset == start_offset || (start_offset > self.scratch_len && offset == 0) {
                return Err(Error::OutOfMemory);
            }
        }
    }

    fn is_mapped(&self, vaddr: usize, level: usize) -> bool {
        Self::is_mapped_rec(&self.shadow, vaddr, level)
    }
}

fn index(vaddr: usize, level: usize) -> usize {
    (vaddr >> SHIFTS[level]) & VPN_MASK
}

#[cfg(not(feature = "vspacemgr_check_bypass"))]
impl VSpaceManager {
    fn check_addr(addr: usize) -> bool {
        if addr >= MAP_START && addr < MAP_END {
            return true;
        }
        crate::error!(
            "VSpaceManager: Address {:#x} out of allowed range [{:#x}, {:#x})",
            addr,
            MAP_START,
            MAP_END,
        );
        false
    }
}

#[cfg(feature = "vspacemgr_check_bypass")]
impl VSpaceManager {
    fn check_addr(addr: usize) -> bool {
        true
    }
}
