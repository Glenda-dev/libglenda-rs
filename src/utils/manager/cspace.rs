use crate::cap::{CNODE_SLOTS, CNode, CSPACE_CAP, CapPtr};
use crate::error::Error;
use crate::interface::{CSpaceProvider, CSpaceService};
use alloc::vec::Vec;

/// CSpaceManager manages the allocation of capability slots in a process's CSpace.
/// It uses a pure 2-level CNode hierarchy (Root CNode -> Level 1 CNodes).
/// Slots `l1_start_slot` to 255 in the Root CNode are used for indices of Level 1 CNodes.
/// Each Level 1 CNode provides 256 slots (0-255), except slot 0 which is reserved.
pub struct CSpaceManager {
    root_cnode: CNode,
    l1_start_slot: usize,
    next_index: usize,
    l1_cnodes: [bool; CNODE_SLOTS],
    free_list: Vec<usize>,
}

impl CSpaceManager {
    pub fn new(root: CNode, l1_start_slot: usize) -> Self {
        // next_index is now relative to l1_start_slot.
        // next_index = 1 means l1_start_slot * 256 + 1.
        let next_index = 1;
        let l1_cnodes = [false; CNODE_SLOTS];
        Self { root_cnode: root, l1_start_slot, next_index, l1_cnodes, free_list: Vec::new() }
    }

    /// Mark a L1 CNode as already present. Used during initialization.
    pub fn mark_present(&mut self, l0_idx: usize) {
        if l0_idx < CNODE_SLOTS {
            self.l1_cnodes[l0_idx] = true;
        }
    }

    fn get_absolute_cptr(&self, cptr: CapPtr) -> CapPtr {
        if self.root_cnode == CSPACE_CAP {
            cptr
        } else {
            CapPtr::concat(self.root_cnode.cap(), cptr)
        }
    }

    /// Converts a logical index to a capability pointer.
    /// Index 0 is reserved for metadata and should not be allocated for regular capabilities.
    /// Each Level 1 CNode's slot 0 is also skipped.
    /// The index calculations start from the first L1 CNode pointed to by `l1_start_slot`.
    pub fn index_to_cptr(&self, index: usize) -> CapPtr {
        // Calculate internal slot index within L1 CNodes, skipping slot 0 of each L1.
        // Effective slots per L1 = CNODE_SLOTS - 1
        let slots_per_l1 = CNODE_SLOTS - 1;

        // index 1 maps to (l0 = l1_start_slot, l1 = 1)
        // index 2 maps to (l0 = l1_start_slot, l1 = 2)
        // ...
        // index 255 maps to (l0 = l1_start_slot, l1 = 255)
        // index 256 maps to (l0 = l1_start_slot + 1, l1 = 1)

        let l1_offset = (index - 1) / slots_per_l1;
        let l1_slot = (index - 1) % slots_per_l1 + 1; // Start from slot 1

        let l0_idx = self.l1_start_slot + l1_offset;
        let l1_idx = l1_slot;

        let l0 = CapPtr::from(l0_idx);
        let l1 = CapPtr::from(l1_idx);
        self.get_absolute_cptr(CapPtr::concat(l0, l1))
    }

    /// Converts a capability pointer back to a logical index.
    pub fn cptr_to_index(&self, cptr: CapPtr) -> Option<usize> {
        let rel_ptr = CapPtr::relative(self.root_cnode.cap(), cptr);
        if rel_ptr.is_null() {
            return None;
        }
        let l0 = rel_ptr.level_idx(0);
        let l1 = rel_ptr.level_idx(1);

        if l0 < self.l1_start_slot || l1 == 0 {
            return None;
        }

        let l1_offset = l0 - self.l1_start_slot;
        let slots_per_l1 = CNODE_SLOTS - 1;
        Some(l1_offset * slots_per_l1 + l1)
    }

    /// Proactively ensures that the L1 CNode for a future index is mapped.
    fn ensure_margin(&mut self, provider: &mut dyn CSpaceProvider) {
        let index_to_check = self.next_index + 16;
        let slots_per_l1 = CNODE_SLOTS - 1;
        let l1_offset = (index_to_check - 1) / slots_per_l1;
        if (self.l1_start_slot + l1_offset) < CNODE_SLOTS {
            let _ = self.ensure_l1_cnode(provider, index_to_check);
        }
    }

    fn ensure_l1_cnode(
        &mut self,
        provider: &mut dyn CSpaceProvider,
        index: usize,
    ) -> Result<(), Error> {
        let slots_per_l1 = CNODE_SLOTS - 1;
        let l1_offset = (index - 1) / slots_per_l1;
        let l0_idx = self.l1_start_slot + l1_offset;

        if l0_idx >= CNODE_SLOTS {
            crate::error!("CSpaceManager: index {} out of bounds", index);
            return Err(Error::CNodeFull);
        }

        if !self.l1_cnodes[l0_idx] {
            let dest = self.get_absolute_cptr(CapPtr::from(l0_idx));
            provider.alloc_cnode(dest)?;
            self.l1_cnodes[l0_idx] = true;
        }
        Ok(())
    }

    /// Allocate a slot directly without calling a provider.
    pub fn alloc_direct(&mut self) -> Result<CapPtr, Error> {
        let index = if let Some(idx) = self.free_list.pop() {
            idx
        } else {
            let res = self.next_index;
            self.next_index += 1;
            res
        };

        let slots_per_l1 = CNODE_SLOTS - 1;
        let l1_offset = (index - 1) / slots_per_l1;
        let l0_idx = self.l1_start_slot + l1_offset;

        if !self.l1_cnodes[l0_idx] {
            // Cannot allocate if L1 CNode is missing and no provider is available
            self.free_list.push(index);
            crate::error!("CSpaceManager: L1 CNode for index {} not allocated", index);
            return Err(Error::CNodeFull);
        }

        let ptr = self.index_to_cptr(index);
        Ok(ptr)
    }
}

impl CSpaceService for CSpaceManager {
    fn alloc(&mut self, provider: &mut dyn CSpaceProvider) -> Result<CapPtr, Error> {
        self.ensure_margin(provider);

        let index = if let Some(idx) = self.free_list.pop() {
            idx
        } else {
            let res = self.next_index;
            self.next_index += 1;
            res
        };
        self.ensure_l1_cnode(provider, index)?;

        let ptr = self.index_to_cptr(index);
        Ok(ptr)
    }

    fn reserve_slots(
        &mut self,
        provider: &mut dyn CSpaceProvider,
        count: usize,
    ) -> Result<(), Error> {
        for i in 0..count {
            let index = self.next_index + i;
            let slots_per_l1 = CNODE_SLOTS - 1;
            let l1_offset = (index - 1) / slots_per_l1;
            if (self.l1_start_slot + l1_offset) >= CNODE_SLOTS {
                crate::error!(
                    "CSpaceManager: cannot reserve {} slots, index {} out of bounds",
                    count,
                    index
                );
                return Err(Error::CNodeFull);
            }
            self.ensure_l1_cnode(provider, index)?;
        }
        Ok(())
    }

    fn free(&mut self, slot: CapPtr) {
        if let Some(index) = self.cptr_to_index(slot) {
            if index >= self.next_index {
                return;
            }
            self.free_list.push(index);
        }
    }
}
