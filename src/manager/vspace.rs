use super::interface::{IResourceManager, ISlotManager, IVSpaceManager};
use crate::arch::mem::{PGSIZE, SHIFTS, VPN_MASK};
use crate::cap::{CNode, CapPtr, CapType, Frame, PageTable, VSpace};
use crate::error::Error;
use crate::mem::Perms;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;

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
}

impl VSpaceManager {
    pub fn new(root: VSpace) -> Self {
        Self { root, shadow: BTreeMap::new() }
    }

    pub fn setup(&self) -> Result<(), Error> {
        if self.root.setup().is_err() {
            return Err(Error::MappingFailed);
        }
        Ok(())
    }

    pub fn mark_existing(&mut self, vaddr: usize) {
        Self::mark_existing_rec(&mut self.shadow, vaddr, SHIFTS.len() - 1);
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
        objects: &mut dyn IResourceManager,
        slots: &mut dyn ISlotManager,
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
        objects: &mut dyn IResourceManager,
        slots: &mut dyn ISlotManager,
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
                    let new_slot = slots.alloc(objects)?;
                    objects.alloc(CapType::Frame, 1, root_cnode, new_slot)?;
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
        objects: &mut dyn IResourceManager,
        slots: &mut dyn ISlotManager,
        dest_cnode: CNode,
        pivot_root: VSpace,
    ) -> Result<&'a mut BTreeMap<usize, Box<ShadowNode>>, Error> {
        let idx = index(vaddr, level);

        if level == 0 {
            return Ok(entries);
        }

        if !entries.contains_key(&idx) {
            let slot = slots.alloc(objects)?;
            let target_level = level - 1;
            objects.alloc(CapType::PageTable, target_level, dest_cnode, slot)?;
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
        objects: &mut dyn IResourceManager,
        cnode: CNode,
    ) {
        let idx = index(vaddr, level);
        if let Some(node) = entries.get_mut(&idx) {
            match &mut **node {
                ShadowNode::Table { entries: sub_entries, .. } => {
                    if level == 0 {
                        // Leaf level (Frame) - Handled below, but structurally we are at Table pointing to Frame?
                        // Actually in this structure, level 0 is Page Table, entries point to Frames.
                        if let Some(removed_node) = sub_entries.remove(&index(vaddr, 0)) {
                            // Free Frame Capability
                            if let ShadowNode::Frame { cap, .. } = *removed_node {
                                let _ = cnode.delete(cap);
                                let _ = objects.free(cap);
                            }
                        }
                    } else {
                        Self::unmap_rec(sub_entries, vaddr, level - 1, objects, cnode);
                    }
                }
                _ => {}
            }
        }
    }
}

impl IVSpaceManager for VSpaceManager {
    fn map_frame(
        &mut self,
        frame: Frame,
        vaddr: usize,
        perms: Perms,
        pages: usize,
        objects: &mut dyn IResourceManager,
        slots: &mut dyn ISlotManager,
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
            if leaf_map.contains_key(&idx0) {
                return Err(Error::MappingFailed);
            }
        }

        if self.root.map(frame, vaddr, perms).is_err() {
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

    fn unmap(
        &mut self,
        vaddr: usize,
        pages: usize,
        objects: &mut dyn IResourceManager,
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
}

fn index(vaddr: usize, level: usize) -> usize {
    (vaddr >> SHIFTS[level]) & VPN_MASK
}
