use crate::cap::Endpoint;
use crate::error::Error;
use crate::interface::device::FrameBufferDevice;
use crate::ipc::{MsgFlags, MsgTag, UTCB};
use crate::protocol::device::fb::FbInfo;
use crate::protocol::device::{FB_PROTO, fb};

pub struct FbClient {
    endpoint: Endpoint,
}

impl FbClient {
    pub const fn new(endpoint: Endpoint) -> Self {
        Self { endpoint }
    }
}

impl FrameBufferDevice for FbClient {
    fn get_info(&self) -> FbInfo {
        let utcb = unsafe { UTCB::get() };
        let tag = MsgTag::new(FB_PROTO, fb::GET_INFO, MsgFlags::NONE);

        if self.endpoint.call(tag).is_ok() {
            if utcb.size >= core::mem::size_of::<FbInfo>() {
                let mut info = FbInfo::default();
                unsafe {
                    let src = utcb.ipc_buffer.as_ptr();
                    let dst = &mut info as *mut FbInfo as *mut u8;
                    core::ptr::copy_nonoverlapping(src, dst, core::mem::size_of::<FbInfo>());
                }
                return info;
            }
        }
        FbInfo::default()
    }

    fn flush(&mut self, x: u32, y: u32, w: u32, h: u32) -> Result<(), Error> {
        let utcb = unsafe { UTCB::get() };
        let tag = MsgTag::new(FB_PROTO, fb::FLUSH, MsgFlags::NONE);
        utcb.mrs_regs[0] = x as usize;
        utcb.mrs_regs[1] = y as usize;
        utcb.mrs_regs[2] = w as usize;
        utcb.mrs_regs[3] = h as usize;

        self.endpoint.call(tag)
    }
}
