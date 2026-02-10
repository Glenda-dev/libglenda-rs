use crate::cap::Endpoint;
use crate::error::Error;
use crate::interface::device::{DeviceService, DmaService};
use crate::ipc::{Badge, MsgFlags, MsgTag, UTCB};
use crate::protocol::DEVICE_PROTO;
use crate::protocol::device::{self, DeviceNode};
use crate::set_mrs;
use crate::utils::platform::PlatformInfo;

pub mod block;
pub mod fb;
pub mod input;
pub mod net;
pub mod pci;
pub mod timer;
pub mod uart;

pub struct DeviceClient {
    endpoint: Endpoint,
}

impl DeviceClient {
    pub const fn new(endpoint: Endpoint) -> Self {
        Self { endpoint }
    }
}

impl DeviceService for DeviceClient {
    fn scan_platform(&mut self, _badge: Badge, info: &PlatformInfo) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        unsafe {
            utcb.write_obj::<PlatformInfo>(info)?;
        }
        let tag = MsgTag::new(DEVICE_PROTO, device::SCAN_PLATFORM, MsgFlags::NONE);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)
    }

    fn find_compatible(&self, _badge: Badge, compat: &str) -> Result<DeviceNode, Error> {
        let mut utcb = unsafe { UTCB::new() };

        unsafe {
            utcb.write_str(compat)?;
        }

        let tag = MsgTag::new(DEVICE_PROTO, device::FIND_COMPATIBLE, MsgFlags::NONE);
        utcb.set_msg_tag(tag);

        self.endpoint.call(&mut utcb)?;

        Ok(unsafe { utcb.read_postcard::<DeviceNode>()? })
    }
}

impl DmaService for DeviceClient {
    fn alloc_dma(&mut self, size: usize) -> Result<usize, Error> {
        let mut utcb = unsafe { UTCB::new() };
        let tag = MsgTag::new(DEVICE_PROTO, device::ALLOC_DMA, MsgFlags::NONE);
        utcb.set_msg_tag(tag);
        set_mrs!(utcb, size);

        self.endpoint.call(&mut utcb)?;

        Ok(utcb.get_mr(0))
    }

    fn free_dma(&mut self, _paddr: usize, _size: usize) {
        // TODO: Implement free_dma in protocol
    }
}
