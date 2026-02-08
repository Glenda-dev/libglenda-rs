use crate::cap::Endpoint;
use crate::error::Error;
use crate::interface::device::{DeviceService, DmaService};
use crate::ipc::{Badge, MsgFlags, MsgTag, UTCB};
use crate::protocol::DEVICE_PROTO;
use crate::protocol::device::{self, DeviceNode};
use crate::set_mrs;
use crate::utils::platform::PlatformInfo;
use alloc::string::String;

pub mod block;
pub mod fb;
pub mod input;
pub mod net;
pub mod pci;
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
        let utcb = unsafe { UTCB::get() };
        let buf = &mut utcb.ipc_buffer;

        // Serialize PlatformInfo into message buffer
        let len = postcard::to_slice(info, buf).map_err(|_| Error::InvalidArgs)?.len();

        let tag = MsgTag::new(DEVICE_PROTO, device::SCAN_PLATFORM, MsgFlags::NONE);
        utcb.size = len;

        self.endpoint.call(tag)
    }

    fn find_compatible(&self, _badge: Badge, compat: String) -> Result<DeviceNode, Error> {
        let utcb = unsafe { UTCB::get() };
        let buf = &mut utcb.ipc_buffer;

        let bytes = compat.as_bytes();
        if bytes.len() > buf.len() {
            return Err(Error::InvalidArgs);
        }

        buf[..bytes.len()].copy_from_slice(bytes);

        let tag = MsgTag::new(DEVICE_PROTO, device::FIND_COMPATIBLE, MsgFlags::NONE);
        utcb.size = bytes.len();

        self.endpoint.call(tag)?;

        let node: DeviceNode =
            postcard::from_bytes(&buf[..utcb.size]).map_err(|_| Error::InvalidProtocol)?;

        Ok(node)
    }
}

impl DmaService for DeviceClient {
    fn alloc_dma(&mut self, size: usize) -> Result<usize, Error> {
        let utcb = unsafe { UTCB::get() };
        let tag = MsgTag::new(DEVICE_PROTO, device::ALLOC_DMA, MsgFlags::NONE);

        set_mrs!(utcb, size);

        self.endpoint.call(tag)?;

        Ok(utcb.mrs_regs[0])
    }

    fn free_dma(&mut self, _paddr: usize, _size: usize) {
        // TODO: Implement free_dma in protocol
    }
}
