//! Framebuffer Protocol (0x409)

pub const GET_INFO: usize = 0x1;
pub const FLUSH: usize = 0x2; // arg0: x, arg1: y, arg2: w, arg3: h
pub const SET_SCANOUT: usize = 0x3; // arg0: paddr, arg1: resource_id (optional)

pub const SETUP_RING: usize = 0x10;
pub const SETUP_BUFFER: usize = 0x11;
pub const NOTIFY_SQ: usize = 0x12;

use crate::io::uring::{IOURING_OP_SYNC, IoUringSqe};

pub const IOURING_OP_FB_FLUSH: u8 = 0x10;

pub fn sqe_flush(x: usize, y: usize, w: usize, h: usize, user_data: usize) -> IoUringSqe {
    IoUringSqe {
        opcode: IOURING_OP_FB_FLUSH,
        off: (x as usize) << 32 | (y as usize),
        addr: (w as usize) << 32 | (h as usize),
        user_data,
        ..Default::default()
    }
}

pub const FB_FORMAT_RGB565: usize = 1;
pub const FB_FORMAT_XRGB8888: usize = 2;
pub const FB_FORMAT_ARGB8888: usize = 3;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct FbInfo {
    pub width: usize,
    pub height: usize,
    pub pitch: usize,
    pub format: usize,
    pub bpp: usize,
    pub paddr: usize,
    pub size: usize,
}
