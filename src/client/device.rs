use crate::cap::{CapPtr, Endpoint, Frame, IrqHandler};
use crate::error::Error;
use crate::interface::device::DeviceService;
use crate::ipc::{Badge, MsgFlags, MsgTag, UTCB};
use crate::protocol;
use crate::protocol::device::DeviceDescNode;
use alloc::string::String;
use alloc::vec::Vec;

pub struct DeviceClient {
    endpoint: Endpoint,
}

impl DeviceClient {
    pub const fn new(endpoint: Endpoint) -> Self {
        Self { endpoint }
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
    ) -> Result<(Frame, usize, usize), Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag = MsgTag::new(protocol::DEVICE_PROTO, protocol::device::GET_MMIO, MsgFlags::NONE);
        utcb.set_recv_window(recv);
        utcb.set_mr(0, id);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;

        let addr = utcb.get_mr(0);
        let size = utcb.get_mr(1);
        Ok((Frame::from(recv), addr, size))
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
        let mut utcb = unsafe { UTCB::new() };
        let tag = MsgTag::new(
            protocol::DEVICE_PROTO,
            protocol::device::REGISTER_LOGIC,
            MsgFlags::HAS_BUFFER | MsgFlags::HAS_CAP,
        );
        unsafe {
            utcb.write_postcard(&desc)?;
        }
        utcb.set_cap_transfer(endpoint);
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
        let mut utcb = unsafe { UTCB::new() };
        let tag = MsgTag::new(
            protocol::DEVICE_PROTO,
            protocol::device::HOOK,
            MsgFlags::HAS_BUFFER | MsgFlags::HAS_CAP,
        );
        unsafe {
            utcb.write_postcard(&target)?;
        }
        utcb.set_cap_transfer(endpoint);
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
