use crate::cap::{Endpoint, Reply};
use crate::error::Error;
use crate::ipc::{MsgArgs, MsgFlags};

/// SystemService interfaces for the system services.
pub trait SystemService {
    fn init(&mut self) -> Result<(), Error>;
    fn listen(&mut self, ep: Endpoint, reply: Reply) -> Result<(), Error>;
    fn run(&mut self) -> Result<(), Error>;
    fn dispatch(
        &mut self,
        badge: usize,
        label: usize,
        proto: usize,
        flags: MsgFlags,
        msg: MsgArgs,
    ) -> Result<usize, Error>;
    fn reply(
        &mut self,
        label: usize,
        proto: usize,
        flags: MsgFlags,
        msg: MsgArgs,
    ) -> Result<(), Error>;
}
