use super::{CSpaceProvider, UntypedService};
use crate::cap::{CapPtr, CapType};
use crate::error::Error;

pub struct DummyProvider;
impl UntypedService for DummyProvider {
    fn alloc(&mut self, _t: CapType, _f: usize, _d: CapPtr) -> Result<usize, Error> {
        Err(Error::OutOfMemory)
    }
    fn free(&mut self, _c: CapPtr) -> Result<(), Error> {
        Ok(())
    }
    fn as_cspace_provider(&mut self) -> &mut dyn CSpaceProvider {
        self
    }
}
impl CSpaceProvider for DummyProvider {
    fn alloc_cnode(&mut self, _d: CapPtr) -> Result<(), Error> {
        Err(Error::OutOfMemory)
    }
}
