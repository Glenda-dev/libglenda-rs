use crate::cap::{CapPtr, CapType, Endpoint};
use crate::error::Error;
use crate::interface::{ResourceService, SystemClient};
use crate::ipc::{Badge, MsgFlags, MsgTag, UTCB};
use crate::protocol::{RESOURCE_PROTO, resource};

pub struct ResourceClient {
    endpoint: Endpoint,
}

impl ResourceClient {
    pub const fn new(endpoint: Endpoint) -> Self {
        Self { endpoint }
    }
}

impl SystemClient for ResourceClient {
    fn connect(&mut self, ep: Endpoint, _reply: CapPtr) -> Result<(), Error> {
        self.endpoint = ep;
        Ok(())
    }

    fn disconnect(&mut self) {}

    fn send(&mut self, info: MsgTag) -> Result<(), Error> {
        self.endpoint.send(info)
    }
}

impl ResourceService for ResourceClient {
    fn alloc(&mut self, _pid: Badge, obj_type: CapType, flags: usize) -> Result<CapPtr, Error> {
        let tag = MsgTag::new(RESOURCE_PROTO, resource::ALLOC, MsgFlags::NONE);

        // Use CALL to wait for response
        let utcb = unsafe { UTCB::get() };
        utcb.mrs_regs[0] = obj_type as usize;
        utcb.mrs_regs[1] = flags;

        self.endpoint.call(tag)?;

        // Check return code in UTCB if needed, but invoke already returns Result<(), Error>
        // derived from the syscall return value.
        Ok(utcb.recv_window)
    }

    fn free(&mut self, _pid: Badge, cap: CapPtr) -> Result<(), Error> {
        let tag = MsgTag::new(RESOURCE_PROTO, resource::FREE, MsgFlags::NONE);

        let utcb = unsafe { UTCB::get() };

        utcb.mrs_regs[0] = cap.bits();

        self.endpoint.call(tag)
    }
}
