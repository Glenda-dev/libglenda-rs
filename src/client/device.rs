use crate::arch::mem::PGSIZE;
use crate::cap::{CSPACE_CAP, CapPtr, CapType, Endpoint, IrqHandler, Page, RECV_SLOT, Rights};
use crate::client::ResourceClient;
use crate::error::Error;
use crate::interface::device::DeviceService;
use crate::interface::{CSpaceService, ResourceService, VSpaceService};
use crate::ipc::{Badge, MsgFlags, MsgTag, UTCB};
use crate::mem::Perms;
use crate::protocol;
use crate::protocol::device::DeviceDescNode;
use crate::protocol::init::ServiceState;
use crate::utils::manager::{CSpaceManager, VSpaceManager};
use alloc::string::String;
use alloc::vec::Vec;

pub struct DeviceClient {
    endpoint: Endpoint,
}

impl DeviceClient {
    pub const fn new(endpoint: Endpoint) -> Self {
        Self { endpoint }
    }

    pub fn report_frame_cap(
        &mut self,
        _badge: Badge,
        frame: CapPtr,
        byte_len: usize,
    ) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_mr(0, byte_len);
        utcb.set_cap_transfer(frame);
        let tag =
            MsgTag::new(protocol::DEVICE_PROTO, protocol::device::REPORT_FRAME, MsgFlags::HAS_CAP);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)
    }

    pub fn report_via_frame(
        &mut self,
        _badge: Badge,
        desc: Vec<DeviceDescNode>,
        res_client: &mut ResourceClient,
        vspace_mgr: &mut VSpaceManager,
        cspace_mgr: &mut CSpaceManager,
        map_vaddr: usize,
    ) -> Result<(), Error> {
        let bytes = postcard::to_allocvec(&desc).map_err(|_| Error::InvalidType)?;
        let byte_len = bytes.len();
        if byte_len == 0 {
            return self.report(Badge::null(), desc);
        }

        let pages = (byte_len + PGSIZE - 1) / PGSIZE;
        let page_level = CapType::page_pages_to_level(pages).ok_or(Error::InvalidArgs)?;
        let frame_slot = cspace_mgr.alloc(res_client)?;
        res_client.alloc(Badge::null(), CapType::Page, page_level, frame_slot)?;

        vspace_mgr.map_page(
            Page::from(frame_slot),
            map_vaddr,
            Perms::READ | Perms::WRITE,
            pages,
            res_client,
            cspace_mgr,
        )?;

        unsafe {
            core::ptr::copy_nonoverlapping(bytes.as_ptr(), map_vaddr as *mut u8, byte_len);
        }

        let call_res = self.report_frame(Badge::null(), frame_slot, byte_len);

        let _ = vspace_mgr.unmap(map_vaddr, pages);
        let _ = res_client.free(Badge::null(), frame_slot);
        cspace_mgr.free(frame_slot);

        call_res
    }
}

impl DeviceService for DeviceClient {
    fn scan_platform(&mut self, _badge: Badge) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag =
            MsgTag::new(protocol::DEVICE_PROTO, protocol::device::SCAN_PLATFORM, MsgFlags::NONE);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)
    }

    fn get_mmio(
        &mut self,
        _badge: Badge,
        id: usize,
        recv: CapPtr,
    ) -> Result<(Page, usize, usize), Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag = MsgTag::new(protocol::DEVICE_PROTO, protocol::device::GET_MMIO, MsgFlags::NONE);
        utcb.set_recv_window(recv);
        utcb.set_mr(0, id);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;

        let addr = utcb.get_mr(0);
        let size = utcb.get_mr(1);
        Ok((Page::from(recv), addr, size))
    }

    fn get_irq(&mut self, _badge: Badge, id: usize, recv: CapPtr) -> Result<IrqHandler, Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag = MsgTag::new(protocol::DEVICE_PROTO, protocol::device::GET_IRQ, MsgFlags::NONE);
        utcb.set_recv_window(recv);
        utcb.set_mr(0, id);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        Ok(IrqHandler::from(recv))
    }

    fn report_frame(&mut self, badge: Badge, frame: CapPtr, byte_len: usize) -> Result<(), Error> {
        self.report_frame_cap(badge, frame, byte_len)
    }

    fn report(&mut self, _badge: Badge, desc: Vec<DeviceDescNode>) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag =
            MsgTag::new(protocol::DEVICE_PROTO, protocol::device::REPORT, MsgFlags::HAS_BUFFER);
        unsafe {
            utcb.write_postcard(&desc)?;
        }
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)
    }

    fn report_state(&mut self, _badge: Badge, status: ServiceState) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag =
            MsgTag::new(protocol::DEVICE_PROTO, protocol::device::REPORT_STATE, MsgFlags::NONE);
        utcb.set_mr(0, status as usize);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)
    }

    fn update(&mut self, _badge: Badge, compatible: Vec<String>) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag =
            MsgTag::new(protocol::DEVICE_PROTO, protocol::device::UPDATE, MsgFlags::HAS_BUFFER);
        unsafe {
            utcb.write_postcard(&compatible)?;
        }
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)
    }

    fn register_logic(
        &mut self,
        _badge: Badge,
        desc: protocol::device::LogicDeviceDesc,
        endpoint: CapPtr,
    ) -> Result<(), Error> {
        let transfer_slot = RECV_SLOT;
        let _ = CSPACE_CAP.delete(transfer_slot);
        CSPACE_CAP.copy_self(endpoint, transfer_slot, Rights::ALL)?;

        let mut utcb = unsafe { UTCB::new() };
        let tag = MsgTag::new(
            protocol::DEVICE_PROTO,
            protocol::device::REGISTER_LOGIC,
            MsgFlags::HAS_BUFFER | MsgFlags::HAS_CAP,
        );
        unsafe {
            utcb.write_postcard(&desc)?;
        }
        utcb.set_cap_transfer(transfer_slot);
        utcb.set_recv_window(transfer_slot);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)
    }

    fn alloc_logic(
        &mut self,
        _badge: Badge,
        dev_type: protocol::device::LogicDeviceType,
        criteria: &str,
        recv: CapPtr,
    ) -> Result<Endpoint, Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag = MsgTag::new(
            protocol::DEVICE_PROTO,
            protocol::device::ALLOC_LOGIC,
            MsgFlags::HAS_BUFFER,
        );
        utcb.set_recv_window(recv);
        let req = protocol::device::AllocLogicRequest {
            dev_type,
            criteria: alloc::string::String::from(criteria),
        };
        unsafe {
            utcb.write_postcard(&req)?;
        }
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        Ok(Endpoint::from(recv))
    }

    fn query(
        &mut self,
        _badge: Badge,
        query: protocol::device::DeviceQuery,
    ) -> Result<Vec<String>, Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag =
            MsgTag::new(protocol::DEVICE_PROTO, protocol::device::QUERY, MsgFlags::HAS_BUFFER);
        unsafe {
            utcb.write_postcard(&query)?;
        }
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        unsafe { utcb.read_postcard() }
    }

    fn get_desc(
        &mut self,
        _badge: Badge,
        name: &str,
    ) -> Result<protocol::device::DeviceDesc, Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag =
            MsgTag::new(protocol::DEVICE_PROTO, protocol::device::GET_DESC, MsgFlags::HAS_BUFFER);
        unsafe {
            utcb.write_str(&name)?;
        }
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        unsafe { utcb.read_postcard() }
    }

    fn get_logic_desc(
        &mut self,
        _badge: Badge,
        name: &str,
    ) -> Result<(usize, protocol::device::LogicDeviceDesc), Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag = MsgTag::new(
            protocol::DEVICE_PROTO,
            protocol::device::GET_LOGIC_DESC,
            MsgFlags::HAS_BUFFER,
        );
        utcb.set_msg_tag(tag);
        unsafe {
            utcb.write_str(&name)?;
        }
        self.endpoint.call(&mut utcb)?;
        let id = utcb.get_mr(0) as usize;
        let desc = unsafe { utcb.read_postcard()? };
        Ok((id, desc))
    }

    fn hook(
        &mut self,
        _badge: Badge,
        target: crate::protocol::device::HookTarget,
        endpoint: CapPtr,
    ) -> Result<(), Error> {
        let transfer_slot = RECV_SLOT;
        let _ = CSPACE_CAP.delete(transfer_slot);
        CSPACE_CAP.copy_self(endpoint, transfer_slot, Rights::ALL)?;

        let mut utcb = unsafe { UTCB::new() };
        let tag = MsgTag::new(
            protocol::DEVICE_PROTO,
            protocol::device::HOOK,
            MsgFlags::HAS_BUFFER | MsgFlags::HAS_CAP,
        );
        unsafe {
            utcb.write_postcard(&target)?;
        }
        utcb.set_cap_transfer(transfer_slot);
        utcb.set_recv_window(transfer_slot);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)
    }

    fn unhook(
        &mut self,
        _badge: Badge,
        target: crate::protocol::device::HookTarget,
    ) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag =
            MsgTag::new(protocol::DEVICE_PROTO, protocol::device::UNHOOK, MsgFlags::HAS_BUFFER);
        unsafe {
            utcb.write_postcard(&target)?;
        }
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)
    }
}
