use crate::cap::{CNode, CapPtr, CapType, Frame};
use crate::error::Error;
use crate::ipc::Badge;
use crate::protocol::resource::{InitCap, InitResource};
use alloc::boxed::Box;
use alloc::string::String;
use core::any::Any;

/// ResourceHelper is responsible for allocating kernel objects from untyped memory.
pub trait ResourceService {
    fn alloc(
        &mut self,
        pid: Badge,
        obj_type: CapType,
        flags: usize,
        dest_cnode: CNode,
        dest_slot: CapPtr,
    ) -> Result<(), Error>;

    fn free(&mut self, pid: Badge, cap: CapPtr) -> Result<(), Error>;
}

pub trait InitResourceService {
    fn get_cap(&self, cap: InitCap) -> Result<CapPtr, Error>;

    fn get_resource(&self, res: InitResource) -> Result<Box<dyn Any>, Error>;

    fn get_file(&self, name: &String) -> Result<Frame, Error>;
}
