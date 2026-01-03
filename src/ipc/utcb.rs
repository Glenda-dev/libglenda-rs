use super::MsgTag;
use crate::cap::CapPtr;

pub const UTCB_ADDR: usize = 0x3FFFFFD000;
pub const PGSIZE: usize = 4096;
pub const BUFFER_MAX_SIZE: usize = 3 * 1024; // 3KB
pub const MAX_MRS: usize = 7;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UTCB {
    pub msg_tag: MsgTag,
    pub mrs_regs: [usize; MAX_MRS],
    pub cap_transfer: CapPtr,
    pub recv_window: CapPtr,
    pub tls: usize,
    pub cursor: usize,
    pub buffer_size: usize,
    pub ipc_buffer: [u8; BUFFER_MAX_SIZE],
}

impl UTCB {
    pub fn current() -> &'static mut Self {
        unsafe { &mut *(UTCB_ADDR as *mut UTCB) }
    }
    pub fn set_str(&mut self, s: &str) -> Option<(usize, usize)> {
        let len = s.len();
        if len > BUFFER_MAX_SIZE {
            return None;
        }
        // For simplicity, always write at offset 0 for now
        self.ipc_buffer[0..len].copy_from_slice(s.as_bytes());
        self.buffer_size = len;
        Some((0, len))
    }
    pub fn set_bytes(&mut self, bytes: &[u8]) -> Option<(usize, usize)> {
        let len = bytes.len();
        if len > BUFFER_MAX_SIZE {
            return None;
        }
        self.ipc_buffer[0..len].copy_from_slice(bytes);
        self.buffer_size = len;
        Some((0, len))
    }
    pub fn clear(&mut self) {
        self.msg_tag = MsgTag::empty();
        self.mrs_regs = [0; MAX_MRS];
        self.cap_transfer = CapPtr::null();
        self.recv_window = CapPtr::null();
        self.tls = 0;
        self.cursor = 0;
        self.buffer_size = 0;
        for byte in self.ipc_buffer.iter_mut() {
            *byte = 0;
        }
    }
    pub fn get_str(&self, offset: usize, len: usize) -> Option<&str> {
        if offset + len > self.buffer_size {
            return None;
        }
        let slice = &self.ipc_buffer[offset..offset + len];
        core::str::from_utf8(slice).ok()
    }
}

pub fn get() -> &'static mut UTCB {
    unsafe { &mut *(UTCB_ADDR as *mut UTCB) }
}
