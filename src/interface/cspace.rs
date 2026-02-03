use super::ResourceService;
use crate::cap::CapPtr;
use crate::error::Error;

/// CSpaceService is responsible for managing capability slots.
pub trait CSpaceService {
    fn alloc(&mut self, objects: &mut dyn ResourceService) -> Result<CapPtr, Error>;
}
