use crate::cap::CapPtr;
use crate::error::Error;

/// CSpaceService is responsible for managing capability slots.
pub trait CSpaceService {
    fn alloc(&mut self, provider: &mut dyn CSpaceProvider) -> Result<CapPtr, Error>;
    fn reserve_slots(
        &mut self,
        provider: &mut dyn CSpaceProvider,
        count: usize,
    ) -> Result<(), Error>;
    fn free(&mut self, slot: CapPtr);
}

pub trait CSpaceProvider {
    fn alloc_cnode(&mut self, dest: CapPtr) -> Result<(), Error>;
    fn free_cnode(&mut self, addr: CapPtr) -> Result<(), Error>;
}
