use crate::cap::{CNode, CapPtr, CapType, Frame};
use crate::error::Error;
use crate::mem::Perms;

/// CSpaceService is responsible for managing capability slots.
pub trait CSpaceService {
    fn alloc(&mut self, objects: &mut dyn UntypedService) -> Result<CapPtr, Error>;
}
/// VSpaceService is responsible for managing virtual memory mappings.
pub trait VSpaceService {
    fn map_frame(
        &mut self,
        frame: Frame,
        vaddr: usize,
        perms: Perms,
        pages: usize,
        objects: &mut dyn UntypedService,
        slots: &mut dyn CSpaceService,
        dest_cnode: CNode,
    ) -> Result<(), Error>;

    /// Unmap memory and free resources
    fn unmap(
        &mut self,
        vaddr: usize,
        pages: usize,
        objects: &mut dyn UntypedService,
        cnode: CNode,
    ) -> Result<(), Error>;

    /// Map a frame into the scratch region and return the virtual address
    fn map_scratch(
        &mut self,
        frame: Frame,
        perms: Perms,
        pages: usize,
        objects: &mut dyn UntypedService,
        slots: &mut dyn CSpaceService,
        dest_cnode: CNode,
    ) -> Result<usize, Error>;

    /// Unmap memory from the scratch region
    fn unmap_scratch(&mut self, vaddr: usize, pages: usize) -> Result<(), Error>;

    fn is_mapped(&self, vaddr: usize, level: usize) -> bool;
}

pub trait UntypedService {
    fn alloc(
        &mut self,
        obj_type: CapType,
        flags: usize,
        dest_cnode: CNode,
        dest_slot: CapPtr,
    ) -> Result<(), Error>;

    fn free(&mut self, cap: CapPtr) -> Result<(), Error>;
}
