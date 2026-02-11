use crate::cap::{Frame, IrqHandler};
use crate::error::Error;
use crate::ipc::Badge;
use crate::utils::platform::DeviceDesc;

/// DeviceService provides hardware discovery and management.
pub trait DeviceService {
    fn scan_platform(&mut self, badge: Badge) -> Result<(), Error>;
    fn get_desc(&mut self, badge: Badge) -> Result<DeviceDesc, Error>;
    fn get_mmio(&mut self, badge: Badge) -> Result<Frame, Error>;
    fn get_irq(&mut self, badge: Badge) -> Result<IrqHandler, Error>;
}
