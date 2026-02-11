use super::interface::{CSpaceProvider, UntypedService};
use crate::cap::{CNODE_BITS, CNODE_PAGES};
use crate::cap::{CNode, CapPtr, CapType, Untyped};
use crate::error::Error;
use crate::utils::BootInfo;
use crate::utils::bootinfo::MAX_UNTYPED_REGIONS;
use crate::utils::bootinfo::UntypedRegion;
use alloc::vec::Vec;

#[derive(Clone, Copy, Debug)]
pub struct UntypedBlock {
    pub cap: Untyped,
    pub desc: UntypedRegion,
}

pub struct UntypedManager {
    blocks: Vec<UntypedBlock>,
}

impl UntypedManager {
    pub fn new(bootinfo: &BootInfo, root: CapPtr) -> Self {
        let mut blocks = Vec::new();

        for i in 0..bootinfo.untyped_count {
            if i >= MAX_UNTYPED_REGIONS {
                break;
            }
            // Slots in the Untyped CNode start at 1
            let cptr = CapPtr::from((i + 1) << CNODE_BITS | root.bits());
            let desc = bootinfo.untyped_list[i];

            blocks.push(UntypedBlock { cap: Untyped::from(cptr), desc });
        }

        Self { blocks }
    }

    fn get_pages(&mut self, obj_type: CapType, flags: usize) -> usize {
        match obj_type {
            CapType::Untyped => flags,
            CapType::Frame => flags,
            CapType::CNode => CNODE_PAGES,
            _ => 1,
        }
    }
}

impl UntypedService for UntypedManager {
    fn alloc(
        &mut self,
        obj_type: CapType,
        flags: usize,
        dest_cnode: CNode,
        dest_slot: CapPtr,
    ) -> Result<usize, Error> {
        let pages = self.get_pages(obj_type, flags);
        for block in self.blocks.iter_mut() {
            if block.desc.watermark + pages <= block.desc.pages {
                let paddr = block.desc.start + block.desc.watermark * 4096;
                // Try to retype
                let ret = match obj_type {
                    CapType::Untyped => block.cap.retype_untyped(flags, dest_cnode, dest_slot),
                    CapType::TCB => block.cap.retype_tcb(dest_cnode, dest_slot),
                    CapType::PageTable => block.cap.retype_pagetable(flags, dest_cnode, dest_slot),
                    CapType::CNode => block.cap.retype_cnode(dest_cnode, dest_slot),
                    CapType::Frame => block.cap.retype_frame(flags, dest_cnode, dest_slot),
                    CapType::VSpace => block.cap.retype_vspace(dest_cnode, dest_slot),
                    CapType::Endpoint => block.cap.retype_endpoint(dest_cnode, dest_slot),
                    _ => return Err(Error::NotSupported),
                };

                match ret {
                    Ok(()) => {
                        block.desc.watermark += pages;
                        return Ok(paddr as usize);
                    }
                    Err(Error::OutOfMemory) => {
                        // This block is out of memory, try next block
                        continue;
                    }
                    Err(Error::InvalidSlot) => return Err(Error::InvalidCapability),
                    Err(_) => {
                        return Err(Error::OutOfMemory);
                    }
                }
            }
        }

        Err(Error::OutOfMemory)
    }

    fn free(&mut self, _cap: CapPtr) -> Result<(), Error> {
        // TODO: Implement proper memory accounting/redzone freeing.
        // Currently we use a bump pointer allocator for untyped memory,
        // so we cannot easily reclaim allocated pages without a rewrite.
        // The capability itself is deleted by the caller (CNode::delete).
        Ok(())
    }

    fn as_cspace_provider(&mut self) -> &mut dyn CSpaceProvider {
        self
    }
}

impl CSpaceProvider for UntypedManager {
    fn alloc_cnode(&mut self, dest_cnode: CNode, dest_slot: CapPtr) -> Result<(), Error> {
        self.alloc(CapType::CNode, 1, dest_cnode, dest_slot).map(|_| ())
    }
}
