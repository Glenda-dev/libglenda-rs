use crate::cap::{CapPtr, CapType, Endpoint, Frame};
use crate::error::Error;
use crate::interface::{MemoryService, ResourceService};
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
        utcb.clear();
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
        utcb.clear();

        set_mrs!(utcb, cap.bits());
        utcb.set_msg_tag(tag);

        self.endpoint.call(&mut utcb)?;
        Ok(())
    }

    fn get_cap(
        &mut self,
        _pid: Badge,
        cap: resource::ResourceType,
        id: usize,
        recv: CapPtr,
    ) -> Result<CapPtr, Error> {
        let tag = MsgTag::new(RESOURCE_PROTO, resource::GET_CAP, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, cap as usize, id);
        utcb.set_recv_window(recv);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        Ok(utcb.get_recv_window())
    }

    fn register_cap(
        &mut self,
        _pid: Badge,
        cap_type: resource::ResourceType,
        id: usize,
        cap: CapPtr,
    ) -> Result<(), Error> {
        let tag = MsgTag::new(RESOURCE_PROTO, resource::REGISTER_CAP, MsgFlags::HAS_CAP);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, cap_type as usize, id);
        utcb.set_cap_transfer(cap);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        Ok(())
    }

    fn get_config(
        &mut self,
        _pid: Badge,
        name: &str,
        recv: CapPtr,
    ) -> Result<(Frame, usize), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();

        // Serialize string to IPC buffer
        unsafe {
            utcb.write_str(name)?;
        }

        // Set tag with HAS_BUFFER to enable kernel copy
        let tag = MsgTag::new(RESOURCE_PROTO, resource::GET_CONFIG, MsgFlags::HAS_BUFFER);
        utcb.set_recv_window(recv);
        utcb.set_msg_tag(tag);

        self.endpoint.call(&mut utcb)?;
        let frame = Frame::from(recv);
        let size = utcb.get_mr(0);
        Ok((frame, size))
    }
}

impl MemoryService for ResourceClient {
    fn brk(&mut self, _pid: Badge, increment: isize) -> Result<usize, Error> {
        let tag = MsgTag::new(RESOURCE_PROTO, resource::SBRK, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, increment as usize);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        let new_brk = utcb.get_mr(0);
        Ok(new_brk)
    }
    fn mmap(&mut self, _pid: Badge, frame: Frame, addr: usize, len: usize) -> Result<usize, Error> {
        let tag = MsgTag::new(RESOURCE_PROTO, resource::MMAP, MsgFlags::HAS_CAP);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_cap_transfer(frame.cap());
        set_mrs!(utcb, frame.cap().bits(), addr, len);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        let new_addr = utcb.get_mr(0);
        Ok(new_addr)
    }
    fn munmap(&mut self, _pid: Badge, addr: usize, len: usize) -> Result<(), Error> {
        let tag = MsgTag::new(RESOURCE_PROTO, resource::MUNMAP, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        set_mrs!(utcb, addr, len);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        Ok(())
    }
}
