use crate::cap::{CapPtr, CapType, Frame};
use crate::error::Error;
use crate::ipc::Badge;
use crate::protocol::resource::ResourceType;

/// ResourceHelper is responsible for allocating kernel objects from untyped memory.
pub trait ResourceService {
    fn alloc(
        &mut self,
        pid: Badge,
        obj_type: CapType,
        flags: usize,
        recv: CapPtr,
    ) -> Result<CapPtr, Error>;

    fn dma_alloc(
        &mut self,
        pid: Badge,
        pages: usize,
        recv: CapPtr,
    ) -> Result<(usize, Frame), Error>;

    fn free(&mut self, pid: Badge, cap: CapPtr) -> Result<(), Error>;

    fn get_cap(
        &mut self,
        pid: Badge,
        cap: ResourceType,
        id: usize,
        recv: CapPtr,
    ) -> Result<CapPtr, Error>;

    fn register_cap(
        &mut self,
        pid: Badge,
        cap_type: ResourceType,
        id: usize,
        cap: CapPtr,
    ) -> Result<(), Error>;

    fn get_config(&mut self, pid: Badge, name: &str, recv: CapPtr)
    -> Result<(Frame, usize), Error>;
}
