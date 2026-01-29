pub mod perms {
    pub const VALID: usize = 1 << 0;
    pub const READ: usize = 1 << 1;
    pub const WRITE: usize = 1 << 2;
    pub const EXECUTE: usize = 1 << 3;
    pub const USER: usize = 1 << 4;
    pub const GLOBAL: usize = 1 << 5;
    pub const ACCESSED: usize = 1 << 6;
    pub const DIRTY: usize = 1 << 7;
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Perms(usize);

impl Perms {
    pub const fn from(bits: usize) -> Self {
        let perm = bits & 0xFF;
        Self(perm)
    }

    pub const fn bits(&self) -> usize {
        self.0
    }
}

use super::{CapPtr, Frame, pagetablemethod};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageTable(CapPtr);

impl PageTable {
    pub const fn from(cap: CapPtr) -> Self {
        Self(cap)
    }

    pub fn cap(&self) -> CapPtr {
        self.0
    }

    pub fn map_table(&self, table: PageTable, vaddr: usize, level: usize) -> usize {
        self.0.invoke(pagetablemethod::MAP_TABLE, [table.cap().bits(), vaddr, level, 0, 0, 0, 0])
    }
}
