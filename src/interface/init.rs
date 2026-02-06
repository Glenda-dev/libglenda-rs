use crate::cap::CapPtr;
use crate::error::Error;
use crate::protocol::init::{InitCap, InitResource, ServiceStatus};
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

pub trait InitService {
    fn start_service(&mut self, service: String) -> Result<(), Error>;
    fn stop_service(&mut self, service: String) -> Result<(), Error>;
    fn restart_service(&mut self, service: String) -> Result<(), Error>;
    fn reload_service(&mut self, service: String) -> Result<(), Error>;
    fn query_service(&self, service: String) -> Result<ServiceStatus, Error>;
    fn list_service(&self) -> Result<Vec<String>, Error>;
    fn get_cap(&self, cap: InitCap) -> Result<CapPtr, Error>;
    fn get_resource(&self, res: InitResource) -> Result<Box<dyn Any>, Error>;
}
