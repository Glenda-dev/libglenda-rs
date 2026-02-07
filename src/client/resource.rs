use crate::cap::{CNode, CapPtr, CapType, Endpoint};
use crate::error::Error;
use crate::interface::{ResourceService, SystemClient};
use crate::ipc::{Badge, MsgArgs, MsgFlags, MsgTag, UTCB};
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

impl ResourceService for ResourceClient {
    fn alloc(
        &mut self,
        pid: Badge,
        obj_type: CapType,
        flags: usize,
        dest_cnode: CNode,
        dest_slot: CapPtr,
    ) -> Result<(), Error> {
        let tag = MsgTag::new(RESOURCE_PROTO, resource::ALLOC, MsgFlags::NONE);
        let args = [
            pid.bits(),
            obj_type as usize,
            flags,
            dest_cnode.cap().bits(),
            dest_slot.bits(),
            0,
            0,
            0,
        ];

        // Use CALL to wait for response
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        utcb.mrs_regs = args;

        self.endpoint.cap().invoke(crate::cap::ipcmethod::CALL, args)?;

        // Check return code in UTCB if needed, but invoke already returns Result<(), Error>
        // derived from the syscall return value.
        Ok(())
    }

    fn free(&mut self, pid: Badge, cap: CapPtr) -> Result<(), Error> {
        let tag = MsgTag::new(RESOURCE_PROTO, resource::FREE, MsgFlags::NONE);
        let args = [pid.bits(), cap.bits(), 0, 0, 0, 0, 0, 0];

        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = tag;
        utcb.mrs_regs = args;

        self.endpoint.cap().invoke(crate::cap::ipcmethod::CALL, args)
    }
}
