use super::{CSpaceProvider, CSpaceService};
use crate::cap::{CNODE_BITS, CNODE_SLOTS, CNode, CapPtr};
use crate::error::Error;
use alloc::vec::Vec;

const L0_DIRECT_LIMIT: usize = 192;
const L1_START_SLOT: usize = L0_DIRECT_LIMIT + 1;
const L1_SLOTS: usize = CNODE_SLOTS - L1_START_SLOT; // 63

/// CSpaceManager manages the allocation of capability slots in a process's CSpace.
/// It supports multi-level CNode hierarchy.
pub struct CSpaceManager {
    #[allow(dead_code)]
    root_cnode: CNode,
    next_index: usize,
    l1_cnodes: [bool; L1_SLOTS],
    free_list: Vec<usize>,
}

impl CSpaceManager {
    pub fn new(root: CNode, start_index: usize) -> Self {
        Self {
            root_cnode: root,
            next_index: start_index,
            l1_cnodes: [false; L1_SLOTS],
            free_list: Vec::new(),
        }
    }

    /*
    pub fn free(&mut self, slot: CapPtr) -> Result<(), Error> {
        crate::log!("Freeing slot: {}", slot);
        let l0_idx = slot.bits() & 0xFF;
        let l1_idx = (slot.bits() >> CNODE_BITS) & 0xFF;

        let index = if l1_idx == 0 {
            l0_idx
        } else {
            if l0_idx < L1_START_SLOT {
                return Err(Error::InvalidArgs);
            }
            L1_START_SLOT + (l0_idx - L1_START_SLOT) * 255 + (l1_idx - 1)
        };
        self.free_list.push(index);
        Ok(())
    }
    */

    // FIXME: 完善回收逻辑
    pub fn free(&mut self, _slot: CapPtr) -> Result<(), Error> {
        Ok(())
    }
}

impl CSpaceService for CSpaceManager {
    fn alloc(&mut self, provider: &mut dyn CSpaceProvider) -> Result<CapPtr, Error> {
        let index = if let Some(idx) = self.free_list.pop() {
            idx
        } else {
            let res = self.next_index;
            self.next_index += 1;
            res
        };

        if index <= L0_DIRECT_LIMIT {
            // Level 0 Direct Mapping
            Ok(CapPtr::from(index))
        } else {
            // Level 1 Mapping
            // We skip slot 0 in the L1 CNode as it is reserved for metadata.
            // Each L1 CNode has 255 usable slots (1-255).
            let relative_index = index - L1_START_SLOT;
            let l0_idx = L1_START_SLOT + (relative_index / 255);
            let l1_idx = (relative_index % 255) + 1;

            if l0_idx >= CNODE_SLOTS {
                return Err(Error::CNodeFull);
            }

            // Ensure L1 CNode exists
            let l1_cache_idx = l0_idx - L1_START_SLOT;
            if !self.l1_cnodes[l1_cache_idx] {
                let l0_cptr = CapPtr::from(l0_idx);
                let full_dest = CapPtr::concat(self.root_cnode.cap(), l0_cptr);
                provider.alloc_cnode(full_dest).map_err(|_| Error::OutOfMemory)?;
                self.l1_cnodes[l1_cache_idx] = true;
            }

            // Construct 2-level CapPtr: l0_idx | (l1_idx << 8)
            Ok(CapPtr::from(l0_idx | (l1_idx << CNODE_BITS)))
        }
    }

    fn free(&mut self, slot: CapPtr) -> Result<(), Error> {
        self.free(slot)
    }

    fn root(&mut self) -> CNode {
        self.root_cnode
    }
}
