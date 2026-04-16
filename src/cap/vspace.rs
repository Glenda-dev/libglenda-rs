use super::{CapPtr, Page, PageTable, vspacemethod};
use crate::error::Error;
use crate::ipc::UTCB;
use crate::mem::Perms;
use crate::set_mrs;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VSpace(CapPtr);

impl VSpace {
    pub const fn from(cap: CapPtr) -> Self {
        Self(cap)
    }

    pub fn cap(&self) -> CapPtr {
        self.0
    }

    pub fn map(&self, frame: Page, vaddr: usize, perms: Perms, pages: usize) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        let perms = perms ^ Perms::SUPERVISOR;
        set_mrs!(utcb, frame.cap().bits(), vaddr, perms.bits() as usize, pages);
        self.0.invoke(vspacemethod::MAP, &mut utcb)
    }

    pub fn map_table(&self, table: PageTable, vaddr: usize, level: usize) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, table.cap().bits(), vaddr, level);
        self.0.invoke(vspacemethod::MAP_TABLE, &mut utcb)
    }

    pub fn unmap_table(&self, vaddr: usize, level: usize) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, vaddr, level);
        self.0.invoke(vspacemethod::UNMAP_TABLE, &mut utcb)
    }

    pub fn unmap(&self, vaddr: usize, size: usize) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, vaddr, size);
        self.0.invoke(vspacemethod::UNMAP, &mut utcb)
    }

    pub fn setup(&self) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        self.0.invoke(vspacemethod::SETUP, &mut utcb)
    }

    pub fn debug_print(&self) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        self.0.invoke(vspacemethod::DEBUG_PRINT, &mut utcb)
    }
}
