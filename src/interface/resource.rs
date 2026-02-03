use crate::cap::{CNode, CapPtr, CapType};
use crate::error::Error;

/// ResourceService is responsible for allocating kernel objects from untyped memory.
pub trait ResourceService {
    fn alloc(
        &mut self,
        obj_type: CapType,
        flags: usize,
        dest_cnode: CNode,
        dest_slot: CapPtr,
    ) -> Result<(), Error>;

    fn free(&mut self, cap: CapPtr) -> Result<(), Error>;
}
