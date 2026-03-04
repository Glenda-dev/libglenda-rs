use crate::cap::{CapPtr, CapType};
use crate::error::Error;

pub trait UntypedService {
    fn alloc(&mut self, obj_type: CapType, flags: usize, dest: CapPtr) -> Result<usize, Error>;

    fn free(&mut self, cap: CapPtr) -> Result<(), Error>;
}
