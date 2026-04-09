use crate::cap::{CapPtr, Endpoint};
use crate::error::Error;
use crate::ipc::Badge;

pub trait VolumeService {
    fn get_device(&mut self, pid: Badge, recv: CapPtr) -> Result<Endpoint, Error>;
    fn probe_device(&mut self, pid: Badge, device_name: &str) -> Result<(), Error>;
    fn mount_partition(&mut self, pid: Badge, partition_name: &str) -> Result<Endpoint, Error>;
}
