use crate::cap::Endpoint;
use crate::error::Error;
use crate::interface::device::NetDevice;
use crate::ipc::{MsgFlags, MsgTag, UTCB};
use crate::protocol::device::net::MacAddress;
use crate::protocol::device::{NET_PROTO, net};

pub struct NetClient {
    endpoint: Endpoint,
}

impl NetClient {
    pub const fn new(endpoint: Endpoint) -> Self {
        Self { endpoint }
    }
}

impl NetDevice for NetClient {
    fn mac_address(&self) -> MacAddress {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        let tag = MsgTag::new(NET_PROTO, net::GET_MAC, MsgFlags::NONE);
        utcb.set_msg_tag(tag);

        if self.endpoint.call(&mut utcb).is_ok() {
            unsafe { utcb.read_obj::<MacAddress>().unwrap_or_default() }
        } else {
            MacAddress::default()
        }
    }

    fn send(&mut self, buf: &[u8]) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        let tag = MsgTag::new(NET_PROTO, net::SEND, MsgFlags::NONE);
        utcb.set_msg_tag(tag);
        utcb.write(buf);
        self.endpoint.call(&mut utcb)
    }

    fn recv(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        let tag = MsgTag::new(NET_PROTO, net::RECV, MsgFlags::NONE);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)?;
        Ok(utcb.read(buf))
    }
}
