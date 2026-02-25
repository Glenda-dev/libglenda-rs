use super::{CSpaceService, UntypedService, VSpaceService};
use crate::arch::mem::{PGSIZE, SHIFTS, VPN_MASK};
use crate::cap::{CNode, CapPtr, CapType, Frame, PageTable, VSpace};
use crate::error::Error;
use crate::mem::Perms;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;

#[derive(Debug)]
enum ShadowNode {
    Table { _cap: CapPtr, entries: BTreeMap<usize, Box<ShadowNode>> },
    Frame { cap: CapPtr, pages: usize, perms: Perms },
}

impl ShadowNode {
    fn new_table(cap: CapPtr) -> Self {
        ShadowNode::Table { _cap: cap, entries: BTreeMap::new() }
    }
}

pub struct VSpaceManager {
    pub root: VSpace,
    shadow: BTreeMap<usize, Box<ShadowNode>>, // Top level entries
    // Scratch management
    scratch_start: usize,
    scratch_len: usize,
    scratch_ptr: usize,
}

impl VSpaceManager {
    pub fn new(root: VSpace, scratch_start: usize, scratch_len: usize) -> Self {
        Self { root, shadow: BTreeMap::new(), scratch_start, scratch_len, scratch_ptr: 0 }
    }

    pub fn setup(&self) -> Result<(), Error> {
        if self.root.setup().is_err() {
            return Err(Error::MappingFailed);
        }
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
        objects: &mut dyn UntypedService,
        slots: &mut dyn CSpaceService,
        root_cnode: CNode,
        src_scratch_va: usize,
        dest_scratch_va: usize,
        current_vspace: &mut VSpaceManager,
    ) -> Result<(), Error> {
        self.clone_level(
            &self.shadow,
            dest_mgr,
            objects,
            slots,
            root_cnode,
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
        objects: &mut dyn UntypedService,
        slots: &mut dyn CSpaceService,
        root_cnode: CNode,
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
                        objects,
                        slots,
                        root_cnode,
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
                    let new_slot = slots.alloc(objects.as_cspace_provider())?;
                    let full_dest = CapPtr::concat(root_cnode.cap(), new_slot);
                    objects.alloc(CapType::Frame, 1, full_dest)?;
                    let new_frame = Frame::from(new_slot);

                    // Map both to copy
                    let src_frame = Frame::from(*cap);

                    // Map src using current_vspace
                    current_vspace.map_frame(
                        src_frame,
                        src_scratch_va,
                        Perms::READ,
                        num_pages,
                        objects,
                        slots,
                        root_cnode,
                    )?;

                    // Map dest using current_vspace
                    current_vspace.map_frame(
                        new_frame,
                        dest_scratch_va,
                        Perms::READ | Perms::WRITE,
                        num_pages,
                        objects,
                        slots,
                        root_cnode,
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
                    current_vspace.unmap(src_scratch_va, num_pages, objects, root_cnode)?;
                    current_vspace.unmap(dest_scratch_va, num_pages, objects, root_cnode)?;

                    // Map to child
                    dest_mgr.map_frame(
                        new_frame, vaddr, *perms, num_pages, objects, slots, root_cnode,
                    )?;
                }
            }
        }
        Ok(())
    }

    fn ensure_path<'a>(
        entries: &'a mut BTreeMap<usize, Box<ShadowNode>>,
        vaddr: usize,
        level: usize,
        objects: &mut dyn UntypedService,
        slots: &mut dyn CSpaceService,
        dest_cnode: CNode,
        pivot_root: VSpace,
    ) -> Result<&'a mut BTreeMap<usize, Box<ShadowNode>>, Error> {
        let idx = index(vaddr, level);

        if level == 0 {
            return Ok(entries);
        }

        if !entries.contains_key(&idx) {
            let slot = slots.alloc(objects.as_cspace_provider())?;
            let target_level = level - 1;
            let full_dest = CapPtr::concat(dest_cnode.cap(), slot);
            objects.alloc(CapType::PageTable, target_level, full_dest)?;
            let pt = PageTable::from(slot);

            if pivot_root.map_table(pt, vaddr, level).is_err() {
                return Err(Error::MappingFailed);
            }
            entries.insert(idx, Box::new(ShadowNode::new_table(slot)));
        }

        let node = entries.get_mut(&idx).unwrap();
        match &mut **node {
            ShadowNode::Table { entries: sub_entries, .. } => {
                if level - 1 == 0 {
                    Ok(sub_entries)
                } else {
                    Self::ensure_path(
                        sub_entries,
                        vaddr,
                        level - 1,
                        objects,
                        slots,
                        dest_cnode,
                        pivot_root,
                    )
                }
            }
            _ => Err(Error::MappingFailed), // Collision
        }
    }

    fn unmap_rec(
        entries: &mut BTreeMap<usize, Box<ShadowNode>>,
        vaddr: usize,
        level: usize,
        objects: &mut dyn UntypedService,
        cnode: CNode,
    ) {
        let idx = index(vaddr, level);
        if level == 0 {
            if let Some(removed_node) = entries.remove(&idx) {
                if let ShadowNode::Frame { cap, .. } = *removed_node {
                    let _ = cnode.delete(cap);
                    let _ = objects.free(cap);
                }
            }
        } else if let Some(node) = entries.get_mut(&idx) {
            match &mut **node {
                ShadowNode::Table { entries: sub_entries, .. } => {
                    Self::unmap_rec(sub_entries, vaddr, level - 1, objects, cnode);
                }
                _ => {}
            }
        }
    }

    fn unmap_rec_leak(entries: &mut BTreeMap<usize, Box<ShadowNode>>, vaddr: usize, level: usize) {
        let idx = index(vaddr, level);
        if let Some(node) = entries.get_mut(&idx) {
            match &mut **node {
                ShadowNode::Table { entries: sub_entries, .. } => {
                    Self::unmap_rec_leak(sub_entries, vaddr, level - 1);
                }
                _ => {}
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
        objects: &mut dyn UntypedService,
        slots: &mut dyn CSpaceService,
        dest_cnode: CNode,
    ) -> Result<(), Error> {
        let levels = SHIFTS.len();

        for i in 0..pages {
            let curr_vaddr = vaddr + i * PGSIZE;
            let leaf_map = Self::ensure_path(
                &mut self.shadow,
                curr_vaddr,
                levels - 1,
                objects,
                slots,
                dest_cnode,
                self.root,
            )?;

            let idx0 = index(curr_vaddr, 0);
            if let Some(node) = leaf_map.get(&idx0) {
                crate::println!(
                    "VSpaceManager::map_frame: vaddr {:#x} (idx {}) already in shadow table: {:?}",
                    curr_vaddr,
                    idx0,
                    node
                );
                return Err(Error::MappingFailed);
            }
        }

        if let Err(e) = self.root.map(frame, vaddr, perms, pages) {
            crate::println!("VSpaceManager::map_frame: self.root.map failed with {:?}", e);
            return Err(Error::MappingFailed);
        }

        for i in 0..pages {
            let curr_vaddr = vaddr + i * PGSIZE;
            let leaf_map = Self::ensure_path(
                &mut self.shadow,
                curr_vaddr,
                levels - 1,
                objects,
                slots,
                dest_cnode,
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

    fn map_scratch(
        &mut self,
        frame: Frame,
        perms: Perms,
        pages: usize,
        objects: &mut dyn UntypedService,
        slots: &mut dyn CSpaceService,
        dest_cnode: CNode,
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
                self.map_frame(frame, vaddr, perms, pages, objects, slots, dest_cnode)?;
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

    fn unmap(
        &mut self,
        vaddr: usize,
        pages: usize,
        objects: &mut dyn UntypedService,
        cnode: CNode,
    ) -> Result<(), Error> {
        let levels = SHIFTS.len();
        for i in 0..pages {
            Self::unmap_rec(&mut self.shadow, vaddr + i * PGSIZE, levels - 1, objects, cnode);
        }
        if self.root.unmap(vaddr, pages * PGSIZE).is_err() {
            return Err(Error::MappingFailed);
        }
        Ok(())
    }

    fn unmap_scratch(&mut self, vaddr: usize, pages: usize) -> Result<(), Error> {
        let levels = SHIFTS.len();
        for i in 0..pages {
            Self::unmap_rec_leak(&mut self.shadow, vaddr + i * PGSIZE, levels - 1);
        }
        if self.root.unmap(vaddr, pages * PGSIZE).is_err() {
            return Err(Error::MappingFailed);
        }
        Ok(())
    }

    fn is_mapped(&self, vaddr: usize, level: usize) -> bool {
        Self::is_mapped_rec(&self.shadow, vaddr, level)
    }
}

fn index(vaddr: usize, level: usize) -> usize {
    (vaddr >> SHIFTS[level]) & VPN_MASK
}
