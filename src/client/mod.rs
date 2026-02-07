pub mod fs;
pub mod init;
pub mod network;
pub mod process;
pub mod resource;

pub use fs::FsClient;
pub use init::InitClient;
pub use network::NetworkClient;
pub use process::ProcessClient;
pub use resource::ResourceClient;

use crate::cap::{CapPtr, Endpoint};
use crate::error::Error;
use crate::interface::SystemClient;
use crate::ipc::{MsgArgs, MsgFlags, MsgTag};

pub struct GenericClient {
    endpoint: Endpoint,
}

impl GenericClient {
    pub const fn new(endpoint: Endpoint) -> Self {
        Self { endpoint }
    }

    pub fn endpoint(&self) -> Endpoint {
        self.endpoint
    }
}

impl SystemClient for GenericClient {
    fn connect(&mut self, ep: Endpoint, _reply: CapPtr) -> Result<(), Error> {
        self.endpoint = ep;
        Ok(())
    }

    fn disconnect(&mut self) {
        // Disconnect logic
    }

    fn send(
        &mut self,
        label: usize,
        proto: usize,
        flags: MsgFlags,
        msg: MsgArgs,
    ) -> Result<(), Error> {
        let tag = MsgTag::new(proto, label, flags);
        self.endpoint.send(tag, msg)
    }
}
