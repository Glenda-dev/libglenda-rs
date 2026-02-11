use crate::cap::Endpoint;
use crate::error::Error;
use crate::interface::drivers::FrameBufferDriver;
use crate::ipc::{MsgFlags, MsgTag, UTCB};
use crate::protocol::device::fb::FbInfo;
use crate::protocol::drivers::{FB_PROTO, fb};
use crate::set_mrs;

pub struct FbClient {
    endpoint: Endpoint,
}

impl FbClient {
    pub const fn new(endpoint: Endpoint) -> Self {
        Self { endpoint }
    }
}

impl FrameBufferDriver for FbClient {
    fn get_info(&self) -> FbInfo {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        let tag = MsgTag::new(FB_PROTO, fb::GET_INFO, MsgFlags::NONE);
        utcb.set_msg_tag(tag);
        if self.endpoint.call(&mut utcb).is_ok() {
            unsafe { utcb.read_obj::<FbInfo>().unwrap_or(FbInfo::default()) }
        } else {
            FbInfo::default()
        }
    }

    fn flush(&mut self, x: u32, y: u32, w: u32, h: u32) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        let tag = MsgTag::new(FB_PROTO, fb::FLUSH, MsgFlags::NONE);
        set_mrs!(utcb, x, y, w, h);
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)
    }
}
