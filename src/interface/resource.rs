use crate::cap::{CapPtr, CapType};
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
    fn get_cap(&self, pid: Badge, cap: InitCap, recv: CapPtr) -> Result<CapPtr, Error>;

    fn map_file(&mut self, pid: Badge, name: &String, address: usize) -> Result<usize, Error>;
}
