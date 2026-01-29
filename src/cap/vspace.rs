use super::{CapPtr, Frame, PageTable, vspacemethod};
use crate::mem::Perms;

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

    pub fn map(&self, frame: Frame, vaddr: usize, perms: Perms) -> usize {
        self.0.invoke(vspacemethod::MAP, [frame.cap().bits(), vaddr, perms.bits(), 0, 0, 0, 0])
    }

    pub fn map_table(&self, table: PageTable, vaddr: usize, level: usize) -> usize {
        self.0.invoke(vspacemethod::MAP_TABLE, [table.cap().bits(), vaddr, level, 0, 0, 0, 0])
    }

    pub fn unmap(&self, vaddr: usize, size: usize) -> usize {
        self.0.invoke(vspacemethod::UNMAP, [vaddr, size, 0, 0, 0, 0, 0])
    }

    pub fn setup(&self) -> usize {
        self.0.invoke(vspacemethod::SETUP, [0, 0, 0, 0, 0, 0, 0])
    }

    pub fn debug_print(&self) -> usize {
        self.0.invoke(vspacemethod::DEBUG_PRINT, [0, 0, 0, 0, 0, 0, 0])
    }
}
