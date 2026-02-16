use crate::cap::{Frame, IrqHandler};
use crate::error::Error;
use crate::ipc::Badge;
use crate::protocol::device::DeviceDescNode;
use alloc::vec::Vec;

/// DeviceService provides hardware discovery and management.
pub trait DeviceService {
    fn scan_platform(&mut self, badge: Badge) -> Result<(), Error>;
    fn get_mmio(&mut self, badge: Badge, id: usize) -> Result<(Frame, usize, usize), Error>;
    fn get_irq(&mut self, badge: Badge, id: usize) -> Result<IrqHandler, Error>;
    fn report(&mut self, badge: Badge, desc: Vec<DeviceDescNode>) -> Result<(), Error>;
    fn update(&mut self, badge: Badge, compatible: Vec<alloc::string::String>)
    -> Result<(), Error>;
}
