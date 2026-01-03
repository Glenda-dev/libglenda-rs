use super::MsgTag;

pub const UTCB_ADDR: usize = 0x7FFF_F000;
pub const PGSIZE: usize = 4096;
pub const BUFFER_MAX_SIZE: usize = 3 * 1024; // 3KB

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UTCB {
    pub msg_tag: MsgTag,
    pub mrs_regs: [usize; 7],
    pub cap_transfer: usize,
    pub recv_window: usize,
    pub tls: usize,
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
        Some((0, len))
    }
    pub fn set_bytes(&mut self, bytes: &[u8]) -> Option<(usize, usize)> {
        let len = bytes.len();
        if len > BUFFER_MAX_SIZE {
            return None;
        }
        self.ipc_buffer[0..len].copy_from_slice(bytes);
        Some((0, len))
    }
    pub fn clear(&mut self) {
        // No-op since we don't track cursor yet
    }
}

pub fn get() -> &'static mut UTCB {
    unsafe { &mut *(UTCB_ADDR as *mut UTCB) }
}