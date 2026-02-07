use crate::cap::{CapPtr, Endpoint};
use crate::error::Error;
use crate::ipc::{Badge, MsgTag};

/// SystemService interfaces for the system services.
pub trait SystemService {
    fn init(&mut self) -> Result<(), Error>;
    fn listen(&mut self, ep: Endpoint, reply: CapPtr) -> Result<(), Error>;
    fn run(&mut self) -> Result<(), Error>;
    fn dispatch(&mut self, badge: Badge, info: MsgTag) -> Result<(), Error>;
    fn reply(&mut self, info: MsgTag) -> Result<(), Error>;
    fn stop(&mut self);
}

/// SystemClient interfaces for system services.
pub trait SystemClient {
    fn connect(&mut self, ep: Endpoint, reply: CapPtr) -> Result<(), Error>;
    fn disconnect(&mut self);
    fn send(&mut self, info: MsgTag) -> Result<(), Error>;
}
