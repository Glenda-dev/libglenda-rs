use crate::cap::{CapPtr, CapType, Endpoint};
use crate::error::Error;
use crate::interface::ResourceService;
use crate::ipc::{Badge, MsgFlags, MsgTag, UTCB};
use crate::protocol::{RESOURCE_PROTO, resource};
use crate::set_mrs;

pub struct ResourceClient {
    endpoint: Endpoint,
}

impl ResourceClient {
    pub const fn new(endpoint: Endpoint) -> Self {
        Self { endpoint }
    }
}

impl ResourceService for ResourceClient {
    fn alloc(
        &mut self,
        _pid: Badge,
        obj_type: CapType,
        flags: usize,
        recv: CapPtr,
    ) -> Result<CapPtr, Error> {
        let tag = MsgTag::new(RESOURCE_PROTO, resource::ALLOC, MsgFlags::NONE);

        // Use CALL to wait for response
        let mut utcb = unsafe { UTCB::new() };
        set_mrs!(utcb, obj_type, flags);
        utcb.set_msg_tag(tag);
        utcb.set_recv_window(recv);

        self.endpoint.call(&mut utcb)?;

        // Check return code in UTCB if needed, but invoke already returns Result<(), Error>
        // derived from the syscall return value.
        Ok(recv)
    }

    fn free(&mut self, _pid: Badge, cap: CapPtr) -> Result<(), Error> {
        let tag = MsgTag::new(RESOURCE_PROTO, resource::FREE, MsgFlags::NONE);

        let mut utcb = unsafe { UTCB::new() };

        set_mrs!(utcb, cap.bits());
        utcb.set_msg_tag(tag);

        self.endpoint.call(&mut utcb)?;
        Ok(())
    }
}
