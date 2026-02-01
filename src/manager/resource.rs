use super::interface::IResourceManager;
use crate::cap::{CNODE_BITS, CNODE_PAGES, UNTYPED_SLOT};
use crate::cap::{CNode, CapPtr, CapType, Untyped};
use crate::error::Error;
use crate::error::code;
use crate::utils::BootInfo;
use crate::utils::bootinfo::MAX_UNTYPED_REGIONS;
use crate::utils::bootinfo::UntypedRegion;
use alloc::vec::Vec;

#[derive(Clone, Copy, Debug)]
pub struct UntypedBlock {
    pub cap: Untyped,
    pub desc: UntypedRegion,
}

pub struct ResourceManager {
    blocks: Vec<UntypedBlock>,
}

impl ResourceManager {
    pub fn new(bootinfo: &BootInfo) -> Self {
        let mut blocks = Vec::new();

        for i in 0..bootinfo.untyped_count {
            if i >= MAX_UNTYPED_REGIONS {
                break;
            }
            // Slots in the Untyped CNode start at 1
            let cptr = CapPtr::from((i + 1) << CNODE_BITS | UNTYPED_SLOT.bits());
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

impl IResourceManager for ResourceManager {
    fn alloc(
        &mut self,
        obj_type: CapType,
        flags: usize,
        dest_cnode: CNode,
        dest_slot: CapPtr,
    ) -> Result<(), Error> {
        let pages = self.get_pages(obj_type, flags);
        for block in self.blocks.iter_mut() {
            if block.desc.watermark + pages <= block.desc.pages {
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
                    code::SUCCESS => {
                        block.desc.watermark += pages;
                        return Ok(());
                    }
                    code::UNTYPE_OOM => {
                        // This block is out of memory, try next block
                        continue;
                    }
                    code::INVALID_SLOT => return Err(Error::InvalidCap),
                    _ => {
                        return Err(Error::UntypeOOM);
                    }
                }
            }
        }

        Err(Error::UntypeOOM)
    }
}
