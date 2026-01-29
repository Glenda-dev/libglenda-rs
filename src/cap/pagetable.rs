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

use super::{CapPtr, pagetablemethod};

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
