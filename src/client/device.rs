use crate::cap::{Endpoint, Frame, IrqHandler};
use crate::error::Error;
use crate::interface::device::DeviceService;
use crate::ipc::{Badge, MsgFlags, MsgTag, UTCB};
use crate::protocol;
use crate::utils::platform::DeviceDesc;

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

    fn get_desc(&mut self, _badge: Badge) -> Result<DeviceDesc, Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag = MsgTag::new(protocol::DEVICE_PROTO, protocol::device::GET_DESC, MsgFlags::NONE);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        Ok(unsafe { utcb.read_postcard::<DeviceDesc>()? })
    }

    fn get_mmio(&mut self, _badge: Badge) -> Result<Frame, Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag = MsgTag::new(protocol::DEVICE_PROTO, protocol::device::GET_MMIO, MsgFlags::NONE);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        Ok(Frame::from(utcb.get_recv_window()))
    }

    fn get_irq(&mut self, _badge: Badge) -> Result<IrqHandler, Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag = MsgTag::new(protocol::DEVICE_PROTO, protocol::device::GET_IRQ, MsgFlags::NONE);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        Ok(IrqHandler::from(utcb.get_recv_window()))
    }
}
