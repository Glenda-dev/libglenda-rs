use crate::cap::{CapPtr, CapType};
use crate::error::Error;
use crate::interface::{CSpaceProvider, UntypedService, VSpaceProvider};

pub struct DummyProvider;

impl UntypedService for DummyProvider {
    fn alloc(&mut self, _t: CapType, _f: usize, _d: CapPtr) -> Result<usize, Error> {
        Err(Error::OutOfMemory)
    }
    fn free(&mut self, _c: CapPtr) -> Result<(), Error> {
        Ok(())
    }
}
impl CSpaceProvider for DummyProvider {
    fn alloc_cnode(&mut self, _d: CapPtr) -> Result<(), Error> {
        Err(Error::OutOfMemory)
    }
    fn free_cnode(&mut self, _d: CapPtr) -> Result<(), Error> {
        Ok(())
    }
}

impl VSpaceProvider for DummyProvider {
    fn alloc_pagetable(&mut self, _d: CapPtr) -> Result<(), Error> {
        Err(Error::OutOfMemory)
    }
    fn free_pagetable(&mut self, _d: CapPtr) -> Result<(), Error> {
        Ok(())
    }
}
