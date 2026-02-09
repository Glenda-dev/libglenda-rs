use crate::cap::{CapPtr, CapType, Frame};
use crate::error::Error;
use crate::ipc::Badge;
use crate::protocol::resource::InitCap;
use alloc::string::String;

/// ResourceHelper is responsible for allocating kernel objects from untyped memory.
pub trait ResourceService {
    fn alloc(
        &mut self,
        pid: Badge,
        obj_type: CapType,
        flags: usize,
        recv: CapPtr,
    ) -> Result<CapPtr, Error>;

    fn free(&mut self, pid: Badge, cap: CapPtr) -> Result<(), Error>;
}

pub trait InitResourceService {
    fn get_cap(&self, pid: Badge, cap: InitCap) -> Result<CapPtr, Error>;

    fn get_file(&mut self, pid: Badge, name: &String) -> Result<Frame, Error>;
}
