use crate::cap::{CapPtr, Endpoint};
use crate::error::Error;
use crate::interface::volume::VolumeService;
use crate::ipc::{Badge, MsgFlags, MsgTag, UTCB};
use crate::protocol;
use crate::protocol::volume;

pub struct VolumeClient {
    endpoint: Endpoint,
}

impl VolumeClient {
    pub const fn new(endpoint: Endpoint) -> Self {
        Self { endpoint }
    }
}

impl VolumeService for VolumeClient {
    fn get_device(&mut self, badge: Badge, recv: CapPtr) -> Result<Endpoint, Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();

        let tag = MsgTag::new(protocol::VOLUME_PROTO, volume::GET_DEVICE, MsgFlags::NONE);
        utcb.set_msg_tag(tag);
        utcb.set_mr(0, badge.bits());
        utcb.set_recv_window(recv);
        self.endpoint.call(&mut utcb)?;

        Ok(Endpoint::from(recv))
    }

    fn probe_device(&mut self, _pid: Badge, device_name: &str) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();

        let tag = MsgTag::new(protocol::VOLUME_PROTO, volume::PROBE_DEVICE, MsgFlags::HAS_BUFFER);
        unsafe { utcb.write_str(device_name)? };
        utcb.set_msg_tag(tag);

        self.endpoint.call(&mut utcb)?;
        Ok(())
    }

    fn mount_partition(
        &mut self,
        _pid: Badge,
        partition_name: &str,
        mount_point: &str,
    ) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();

        let tag =
            MsgTag::new(protocol::VOLUME_PROTO, volume::MOUNT_PARTITION, MsgFlags::HAS_BUFFER);
        unsafe {
            let mut writer = utcb.get_buffer_writer();
            writer.write_str(partition_name)?;
            writer.write_str(mount_point)?;
        }
        utcb.set_msg_tag(tag);

        self.endpoint.call(&mut utcb)?;
        Ok(())
    }
}
