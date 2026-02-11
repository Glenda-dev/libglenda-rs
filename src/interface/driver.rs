use crate::protocol::device::DeviceNode;

pub trait DriverService {
    fn init(&mut self, node: DeviceNode);
    fn enable(&mut self);
    fn disable(&mut self);
}
