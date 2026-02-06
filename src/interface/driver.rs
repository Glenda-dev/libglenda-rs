use crate::protocol::device::DeviceNode;

pub trait DriverService {
    fn init(&mut self, node: DeviceNode);
}
