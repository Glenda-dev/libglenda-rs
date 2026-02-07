use crate::cap::Endpoint;
use crate::error::Error;
use crate::interface::device::NetDevice;
use crate::ipc::{MsgFlags, MsgTag, UTCB};
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
    fn mac_address(&self) -> [u8; 6] {
        let utcb = unsafe { UTCB::get() };
        let tag = MsgTag::new(NET_PROTO, net::GET_MAC, MsgFlags::NONE);

        if self.endpoint.call(tag).is_ok() {
            let mut mac = [0; 6];
            if utcb.size >= 6 {
                mac.copy_from_slice(&utcb.ipc_buffer[..6]);
            }
            mac
        } else {
            [0; 6]
        }
    }

    fn send(&mut self, buf: &[u8]) -> Result<(), Error> {
        let utcb = unsafe { UTCB::get() };
        let msg_buf = &mut utcb.ipc_buffer;

        if buf.len() > msg_buf.len() {
            return Err(Error::InvalidArgs);
        }

        msg_buf[..buf.len()].copy_from_slice(buf);
        let tag = MsgTag::new(NET_PROTO, net::SEND, MsgFlags::NONE);
        utcb.size = buf.len();

        self.endpoint.call(tag)
    }

    fn recv(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let utcb = unsafe { UTCB::get() };
        let tag = MsgTag::new(NET_PROTO, net::RECV, MsgFlags::NONE);
        utcb.mrs_regs[0] = buf.len(); // Max length

        self.endpoint.call(tag)?;

        let len = utcb.size;
        let copy_len = core::cmp::min(len, buf.len());
        buf[..copy_len].copy_from_slice(&utcb.ipc_buffer[..copy_len]);

        Ok(copy_len)
    }
}
