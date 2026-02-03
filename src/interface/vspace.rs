use super::{CSpaceService, ResourceService};
use crate::cap::{CNode, Frame};
use crate::error::Error;
use crate::mem::Perms;

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
}
