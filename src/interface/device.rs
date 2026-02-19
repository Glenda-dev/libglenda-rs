use crate::cap::{CapPtr, Endpoint, Frame, IrqHandler};
use crate::error::Error;
use crate::ipc::Badge;
use crate::protocol::device::{DeviceDesc, DeviceDescNode, DeviceQuery, LogicDeviceDesc};
use alloc::string::String;
use alloc::vec::Vec;

/// DeviceService provides hardware discovery and management.
pub trait DeviceService {
    fn scan_platform(&mut self, badge: Badge) -> Result<(), Error>;
    fn get_mmio(&mut self, badge: Badge, id: usize) -> Result<(Frame, usize, usize), Error>;
    fn get_irq(&mut self, badge: Badge, id: usize) -> Result<IrqHandler, Error>;
    fn report(&mut self, badge: Badge, desc: Vec<DeviceDescNode>) -> Result<(), Error>;
    fn update(&mut self, badge: Badge, compatible: Vec<alloc::string::String>)
    -> Result<(), Error>;

    /// Report a logical device (e.g., a disk partition, a network interface)
    fn register_logic(
        &mut self,
        badge: Badge,
        desc: LogicDeviceDesc,
        endpoint: CapPtr,
    ) -> Result<(), Error>;

    /// Allocate/find a logical device matching the criteria
    fn alloc_logic(
        &mut self,
        badge: Badge,
        dev_type: u32,
        criteria: &str,
    ) -> Result<Endpoint, Error>;

    /// Query devices matching criteria. Returns a list of device names.
    fn query(&mut self, badge: Badge, query: DeviceQuery) -> Result<Vec<String>, Error>;

    /// Get description of a device by name.
    fn get_desc(&mut self, badge: Badge, name: &str) -> Result<DeviceDesc, Error>;
}
