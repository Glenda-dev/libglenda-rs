pub const UTCB_ADDR: usize = 0x8000_0000;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UTCB {
    pub mrs: [usize; 7],
    pub cap_transfer: usize,
    pub recv_window: usize,
    pub tls: usize,
    pub ipc_buffer_size: usize,
}

pub fn get() -> &'static mut UTCB {
    unsafe { &mut *(UTCB_ADDR as *mut UTCB) }
}
