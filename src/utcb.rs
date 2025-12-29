use crate::types::UTCB;

pub const UTCB_ADDR: usize = 0x8000_0000;

pub fn get() -> &'static mut UTCB {
    unsafe { &mut *(UTCB_ADDR as *mut UTCB) }
}
