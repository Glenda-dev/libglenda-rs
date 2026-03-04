use crate::arch::mem::PGSIZE;
use crate::cap::{CNode, CapPtr, CapType, Untyped};
use crate::error::Error;
use crate::interface::{CSpaceProvider, UntypedService, VSpaceProvider};
use crate::utils::BootInfo;
use crate::utils::bootinfo::MAX_UNTYPED_REGIONS;
use alloc::vec::Vec;

#[derive(Clone, Copy, Debug)]
pub struct UntypedBlock {
    pub paddr: usize,
    pub cap: Untyped,
}

pub struct UntypedManager {
    blocks: Vec<UntypedBlock>,
    root: CNode,
}

impl UntypedManager {
    pub fn new(bootinfo: &BootInfo, root: CNode, space: CapPtr) -> Self {
        let mut blocks = Vec::new();

        for i in 0..bootinfo.untyped_count {
            if i >= MAX_UNTYPED_REGIONS {
                break;
            }
            let paddr = bootinfo.untyped_list[i];
            // Slots in the Untyped CNode start at 1
            let cptr = CapPtr::concat(space, CapPtr::from(i + 1));
            blocks.push(UntypedBlock { paddr, cap: Untyped::from(cptr) });
        }

        Self { blocks, root }
    }

    pub fn add_block(&mut self, cap: Untyped, paddr: usize) {
        self.blocks.push(UntypedBlock { paddr, cap });
    }
}

impl CSpaceProvider for UntypedManager {
    fn alloc_cnode(&mut self, dest: CapPtr) -> Result<(), Error> {
        self.alloc(CapType::CNode, 0, dest).map(|_| ())
    }

    fn free_cnode(&mut self, addr: CapPtr) -> Result<(), Error> {
        self.free(addr)
    }
}

impl VSpaceProvider for UntypedManager {
    fn alloc_pagetable(&mut self, dest: CapPtr) -> Result<(), Error> {
        self.alloc(CapType::PageTable, 0, dest).map(|_| ())
    }

    fn free_pagetable(&mut self, addr: CapPtr) -> Result<(), Error> {
        self.free(addr)
    }
}

impl UntypedService for UntypedManager {
    fn alloc(&mut self, obj_type: CapType, flags: usize, dest: CapPtr) -> Result<usize, Error> {
        let pages = obj_type.pages(flags)?;
        for block in self.blocks.iter_mut() {
            let info = block.cap.get_info()?;
            let total_pages = info.0;
            let watermark = info.1;
            if watermark + pages <= total_pages {
                let paddr = block.paddr;
                let obj_paddr = paddr + watermark * PGSIZE;
                // Try to retype
                let ret = match obj_type {
                    CapType::Untyped => block.cap.retype_untyped(flags, dest),
                    CapType::TCB => block.cap.retype_tcb(dest),
                    CapType::PageTable => block.cap.retype_pagetable(flags, dest),
                    CapType::CNode => block.cap.retype_cnode(dest),
                    CapType::Frame => block.cap.retype_frame(flags, dest),
                    CapType::VSpace => block.cap.retype_vspace(dest),
                    CapType::Endpoint => block.cap.retype_endpoint(dest),
                    _ => return Err(Error::NotSupported),
                };

                match ret {
                    Ok(()) => {
                        return Ok(obj_paddr);
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

    fn free(&mut self, cap: CapPtr) -> Result<(), Error> {
        self.root.revoke(cap)?;
        match self.root.recycle(cap) {
            Ok((addr, _)) => {
                self.add_block(Untyped::from(cap), addr);
                Ok(())
            }
            Err(e) => {
                let _ = self.root.delete(cap);
                Err(e)
            }
        }
    }
}
