use crate::cap::{CapPtr, CapType, Endpoint, Page};
use crate::error::Error;
use crate::interface::{CSpaceProvider, ResourceService, VSpaceProvider};
use crate::ipc::{Badge, MsgFlags, MsgTag, UTCB};
use crate::protocol::{RESOURCE_PROTO, resource};
use crate::set_mrs;

#[derive(Clone)]
pub struct ResourceClient {
    endpoint: Endpoint,
}

impl ResourceClient {
    pub const fn new(endpoint: Endpoint) -> Self {
        Self { endpoint }
    }

    pub fn brk(&mut self, increment: isize) -> Result<usize, Error> {
        let tag = MsgTag::new(RESOURCE_PROTO, resource::SBRK, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, increment as usize);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        let new_brk = utcb.get_mr(0);
        Ok(new_brk)
    }
}

impl CSpaceProvider for ResourceClient {
    fn alloc_cnode(&mut self, dest: CapPtr) -> Result<(), Error> {
        self.alloc(Badge::null(), CapType::CNode, 0, dest).map(|_| ())
    }

    fn free_cnode(&mut self, addr: CapPtr) -> Result<(), Error> {
        self.free(Badge::null(), addr)
    }
}

impl VSpaceProvider for ResourceClient {
    fn alloc_pagetable(&mut self, dest: CapPtr) -> Result<(), Error> {
        self.alloc(Badge::null(), CapType::PageTable, 0, dest).map(|_| ())
    }

    fn free_pagetable(&mut self, addr: CapPtr) -> Result<(), Error> {
        self.free(Badge::null(), addr)
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
        set_mrs!(utcb, obj_type, flags, recv.bits());
        utcb.set_msg_tag(tag);

        self.endpoint.call(&mut utcb)?;

        Ok(recv)
    }

    fn dma_alloc(
        &mut self,
        _pid: Badge,
        pages: usize,
        recv: CapPtr,
    ) -> Result<(usize, Page), Error> {
        let tag = MsgTag::new(RESOURCE_PROTO, resource::DMA_ALLOC, MsgFlags::NONE);

        // Use CALL to wait for response
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, pages, recv.bits());
        utcb.set_msg_tag(tag);

        self.endpoint.call(&mut utcb)?;

        // The paddr is returned in MR0
        Ok((utcb.get_mr(0), Page::from(recv)))
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
        set_mrs!(utcb, cap as usize, id, recv.bits());
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        Ok(CapPtr::from(utcb.get_mr(0)))
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
    ) -> Result<(Page, usize), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();

        // Serialize string to IPC buffer
        unsafe {
            utcb.write_str(name)?;
        }
        set_mrs!(utcb, recv.bits());

        // Set tag with HAS_BUFFER to enable kernel copy
        let tag = MsgTag::new(RESOURCE_PROTO, resource::GET_CONFIG, MsgFlags::HAS_BUFFER);
        utcb.set_msg_tag(tag);

        self.endpoint.call(&mut utcb)?;
        let frame = Page::from(recv);
        let size = utcb.get_mr(0);
        Ok((frame, size))
    }

    fn status(&mut self, _pid: Badge) -> Result<resource::WarrenStatus, Error> {
        let tag = MsgTag::new(RESOURCE_PROTO, resource::GET_STATUS, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_msg_tag(tag);

        self.endpoint.call(&mut utcb)?;

        Ok(resource::WarrenStatus {
            memory: resource::MemoryStatus {
                available_bytes: utcb.get_mr(0),
                total_bytes: utcb.get_mr(1),
            },
        })
    }
}
