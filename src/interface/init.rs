use crate::error::Error;
use crate::ipc::Badge;
use crate::protocol::init::{ServiceState, ServiceStatus};
use alloc::string::String;
use alloc::vec::Vec;

pub trait InitService {
    fn start_service(&mut self, service: String) -> Result<(), Error>;
    fn stop_service(&mut self, service: String) -> Result<(), Error>;
    fn restart_service(&mut self, service: String) -> Result<(), Error>;
    fn reload_service(&mut self, service: String) -> Result<(), Error>;
    fn query_service(&self, service: String) -> Result<ServiceStatus, Error>;
    fn report_service(&self, badge: Badge, stat: ServiceState) -> Result<(), Error>;
    fn list_services(&self) -> Result<Vec<(String, ServiceStatus)>, Error>;
}
