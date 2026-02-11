use crate::cap::{CapPtr, Endpoint};
use crate::error::Error;
use crate::ipc::UTCB;
use alloc::boxed::Box;
use core::any::Any;

/// SystemService interfaces for the system services.
pub trait SystemService {
    fn init(&mut self) -> Result<(), Error>;
    fn listen(&mut self, ep: Endpoint, reply: CapPtr, recv: CapPtr) -> Result<(), Error>;
    fn run(&mut self) -> Result<(), Error>;
    fn dispatch(&mut self, utcb: &mut UTCB) -> Result<(), Error>;
    fn reply(&mut self, utcb: &mut UTCB) -> Result<(), Error>;
    fn stop(&mut self);
}

/// SystemClient interfaces for system services.
pub trait IpcClient {
    fn connect(&mut self, ep: Endpoint, reply: CapPtr, recv: CapPtr) -> Result<(), Error>;
    fn send(&mut self, utcb: &mut UTCB) -> Result<(), Error>;
    fn recv(&mut self, utcb: &mut UTCB) -> Result<Box<dyn Any>, Error>;
    fn call(&mut self, utcb: &mut UTCB, callback: fn() -> ()) -> Result<(), Error>;
    fn notify(&mut self, utcb: &mut UTCB) -> Result<(), Error>;
    fn disconnect(&mut self);
}
