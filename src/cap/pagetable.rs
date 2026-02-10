use super::{CapPtr, pagetablemethod};
use crate::error::Error;
use crate::ipc::UTCB;
use crate::set_mrs;

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

    pub fn map_table(&self, table: PageTable, vaddr: usize, level: usize) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, table.cap().bits(), vaddr, level);
        self.0.invoke(pagetablemethod::MAP_TABLE, &mut utcb)
    }
}
