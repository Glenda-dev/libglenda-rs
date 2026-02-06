use crate::cap::{CNode, CapPtr, Frame};
use crate::error::Error;
use crate::interface::ResourceService;
use crate::mem::Perms;

/// CSpaceService is responsible for managing capability slots.
pub trait CSpaceService {
    fn alloc(&mut self, objects: &mut dyn ResourceService) -> Result<CapPtr, Error>;
}
/// VSpaceService is responsible for managing virtual memory mappings.
pub trait VSpaceService {
    fn map_frame(
        &mut self,
        frame: Frame,
        vaddr: usize,
        perms: Perms,
        pages: usize,
        objects: &mut dyn ResourceService,
        slots: &mut dyn CSpaceService,
        dest_cnode: CNode,
    ) -> Result<(), Error>;

    /// Unmap memory and free resources
    fn unmap(
        &mut self,
        vaddr: usize,
        pages: usize,
        objects: &mut dyn ResourceService,
        cnode: CNode,
    ) -> Result<(), Error>;

    /// Map a frame into the scratch region and return the virtual address
    fn map_scratch(
        &mut self,
        frame: Frame,
        perms: Perms,
        pages: usize,
        objects: &mut dyn ResourceService,
        slots: &mut dyn CSpaceService,
        dest_cnode: CNode,
    ) -> Result<usize, Error>;

    fn is_mapped(&self, vaddr: usize, level: usize) -> bool;
}
