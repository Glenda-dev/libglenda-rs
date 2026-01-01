use super::MsgTag;

pub const UTCB_ADDR: usize = 0x7FFF_F000;
pub const PGSIZE: usize = 4096;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UTCB {
    pub msg_tag: MsgTag,
    pub mrs_regs: [usize; 7],
    pub cap_transfer: usize,
    pub recv_window: usize,
    pub tls: usize,
    pub ipc_buffer_size: usize,
}

pub const UTCB_SIZE: usize = core::mem::size_of::<UTCB>();
pub const IPC_BUFFER_SIZE: usize = PGSIZE - UTCB_SIZE;

#[repr(C)]
pub struct IPCBuffer(pub [u8; IPC_BUFFER_SIZE]);

impl IPCBuffer {
    pub fn from_utcb(utcb: &UTCB) -> &mut Self {
        let buf_addr = (utcb as *const UTCB as usize) + UTCB_SIZE;
        unsafe { &mut *(buf_addr as *mut IPCBuffer) }
    }

    pub fn append_str(&mut self, s: &str) -> Option<(usize, usize)> {
        let len = s.len();
        if len > IPC_BUFFER_SIZE {
            return None;
        }
        // For simplicity, always write at offset 0 for now
        self.0[0..len].copy_from_slice(s.as_bytes());
        Some((0, len))
    }
}

pub fn get() -> &'static mut UTCB {
    unsafe { &mut *(UTCB_ADDR as *mut UTCB) }
}

pub fn get_ipc_buffer() -> &'static mut IPCBuffer {
    IPCBuffer::from_utcb(get())
}
