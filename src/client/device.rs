use crate::cap::{Endpoint, Frame, IrqHandler};
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

    fn get_mmio(&mut self, _badge: Badge, id: usize) -> Result<(Frame, usize, usize), Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag = MsgTag::new(protocol::DEVICE_PROTO, protocol::device::GET_MMIO, MsgFlags::NONE);
        let recv = utcb.get_recv_window();
        if recv.is_null() {
            return Err(Error::InvalidArgs);
        }
        utcb.set_mr(0, id);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;

        let addr = utcb.get_mr(0);
        let size = utcb.get_mr(1);
        Ok((Frame::from(recv), addr, size))
    }

    fn get_irq(&mut self, _badge: Badge, id: usize) -> Result<IrqHandler, Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag = MsgTag::new(protocol::DEVICE_PROTO, protocol::device::GET_IRQ, MsgFlags::NONE);
        let recv = utcb.get_recv_window();
        if recv.is_null() {
            return Err(Error::InvalidArgs);
        }
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
}
