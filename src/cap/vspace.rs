use super::{CapPtr, Frame, PageTable, vspacemethod};
use crate::error::Error;
use crate::mem::Perms;
use crate::ipc::UTCB;

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

    pub fn map(&self, frame: Frame, vaddr: usize, perms: Perms) -> Result<(), Error> {
        let utcb = unsafe { UTCB::get() };
        utcb.mrs_regs[0] = frame.cap().bits();
        utcb.mrs_regs[1] = vaddr;
        utcb.mrs_regs[2] = perms.bits();
        self.0.invoke(vspacemethod::MAP)
    }

    pub fn map_table(&self, table: PageTable, vaddr: usize, level: usize) -> Result<(), Error> {
        let utcb = unsafe { UTCB::get() };
        utcb.mrs_regs[0] = table.cap().bits();
        utcb.mrs_regs[1] = vaddr;
        utcb.mrs_regs[2] = level;
        self.0.invoke(vspacemethod::MAP_TABLE)
    }

    pub fn unmap(&self, vaddr: usize, size: usize) -> Result<(), Error> {
        let utcb = unsafe { UTCB::get() };
        utcb.mrs_regs[0] = vaddr;
        utcb.mrs_regs[1] = size;
        self.0.invoke(vspacemethod::UNMAP)
    }

    pub fn setup(&self) -> Result<(), Error> {
        self.0.invoke(vspacemethod::SETUP)
    }

    pub fn debug_print(&self) -> Result<(), Error> {
        self.0.invoke(vspacemethod::DEBUG_PRINT)
    }
}
