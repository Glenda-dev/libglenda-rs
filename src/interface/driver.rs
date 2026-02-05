use crate::manager::device::DeviceNode;

pub trait DriverService {
    fn init(&mut self, node: DeviceNode);
}
