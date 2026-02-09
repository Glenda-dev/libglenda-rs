use crate::cap::{CapPtr, CapType, Endpoint};
use crate::error::Error;
use crate::interface::{InitResourceService, ResourceService};
use crate::ipc::{Badge, MsgFlags, MsgTag, UTCB};
use crate::protocol::{RESOURCE_PROTO, resource};
use crate::set_mrs;
use alloc::string::String;

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

impl InitResourceService for ResourceClient {
    fn get_cap(&self, _pid: Badge, cap: resource::InitCap, recv: CapPtr) -> Result<CapPtr, Error> {
        let tag = MsgTag::new(RESOURCE_PROTO, resource::GET_CAP, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        set_mrs!(utcb, cap as usize);
        utcb.set_recv_window(recv);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        Ok(utcb.get_recv_window())
    }
    fn map_file(&mut self, _pid: Badge, name: &String, address: usize) -> Result<usize, Error> {
        let tag = MsgTag::new(RESOURCE_PROTO, resource::MAP_FILE, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.set_msg_tag(tag);
        unsafe { utcb.write_str(name)? };
        utcb.set_mr(0, address);
        self.endpoint.call(&mut utcb)?;
        let size = utcb.get_mr(0);
        Ok(size)
    }
}
