use crate::cap::CNODE_SIZE;
use crate::cap::{CNode, CapPtr, CapType};
use crate::error::Error;
use crate::interface::{CSpaceService, ResourceService};

const L0_DIRECT_LIMIT: usize = 64;
const L1_START_SLOT: usize = L0_DIRECT_LIMIT + 1;
const L1_SLOTS: usize = CNODE_SIZE - L1_START_SLOT; // 192

/// CSpaceManager manages the allocation of capability slots in a process's CSpace.
/// It supports multi-level CNode hierarchy.
pub struct CSpaceManager {
    root_cnode: CNode,
    next_index: usize,
    l1_cnodes: [bool; L1_SLOTS],
}

impl CSpaceManager {
    pub fn new(root: CNode, start_index: usize) -> Self {
        Self { root_cnode: root, next_index: start_index, l1_cnodes: [false; L1_SLOTS] }
    }
}

impl CSpaceService for CSpaceManager {
    fn alloc(&mut self, objects: &mut dyn ResourceService) -> Result<CapPtr, Error> {
        let index = self.next_index;
        self.next_index += 1;

        if index < L0_DIRECT_LIMIT {
            // Level 0 Direct Mapping
            Ok(CapPtr::from(index))
        } else {
            // Level 1 Mapping
            let relative_index = index - L0_DIRECT_LIMIT;
            let l0_idx = L1_START_SLOT + (relative_index / 256);
            let l1_idx = relative_index % 256;

            if l0_idx >= 256 {
                return Err(Error::CNodeFull);
            }

            // Ensure L1 CNode exists
            let l1_cache_idx = l0_idx - L1_START_SLOT;
            if !self.l1_cnodes[l1_cache_idx] {
                let l0_cptr = CapPtr::from(l0_idx);
                objects
                    .alloc(CapType::CNode, 1, self.root_cnode, l0_cptr)
                    .map_err(|_| Error::OutOfMemory)?;
                self.l1_cnodes[l1_cache_idx] = true;
            }

            // Construct 2-level CapPtr: l0_idx | (l1_idx << 8)
            Ok(CapPtr::from(l0_idx | (l1_idx << 8)))
        }
    }
}
