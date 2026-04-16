use super::{CSpaceProvider, CSpaceService};
use crate::cap::{CapPtr, Page, PageTable};
use crate::error::Error;
use crate::mem::Perms;

/// VSpaceService is responsible for managing virtual memory mappings.
pub trait VSpaceService {
    fn map_page(
        &mut self,
        frame: Page,
        vaddr: usize,
        perms: Perms,
        pages: usize,
        provider: &mut dyn VSpaceProvider,
        slots: &mut dyn CSpaceService,
    ) -> Result<(), Error>;

    fn map_pt(
        &mut self,
        level: usize,
        vaddr: usize,
        provider: &mut dyn VSpaceProvider,
        slots: &mut dyn CSpaceService,
    ) -> Result<PageTable, Error>;

    /// Unmap memory
    fn unmap(&mut self, vaddr: usize, pages: usize) -> Result<(), Error>;

    /// Map a frame into the scratch region and return the virtual address
    fn map_scratch(
        &mut self,
        frame: Page,
        perms: Perms,
        pages: usize,
        provider: &mut dyn VSpaceProvider,
        slots: &mut dyn CSpaceService,
    ) -> Result<usize, Error>;

    fn is_mapped(&self, vaddr: usize, level: usize) -> bool;
}

pub trait VSpaceProvider: CSpaceProvider {
    fn alloc_pagetable(&mut self, dest: CapPtr) -> Result<(), Error>;
    fn free_pagetable(&mut self, addr: CapPtr) -> Result<(), Error>;
}
