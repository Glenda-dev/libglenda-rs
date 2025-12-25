#![no_std]

pub mod caps;
pub mod errors;
pub mod syscall;
pub mod types;

pub use types::*;
pub use caps::CapPtr;

pub const UTCB_ADDR: usize = 0x8000_0000;

pub fn get_utcb() -> &'static mut UTCB {
    unsafe { &mut *(UTCB_ADDR as *mut UTCB) }
}

pub mod errcode {
    pub const SUCCESS: usize = 0;
    pub const INVALID_CAP: usize = 1;
    pub const PERMISSION_DENIED: usize = 2;
    pub const INVALID_ENDPOINT: usize = 3;
    pub const INVALID_OBJ_TYPE: usize = 4;
    pub const INVALID_METHOD: usize = 5;
    pub const MAPPING_FAILED: usize = 6;
    pub const INVALID_SLOT: usize = 7;
    pub const UNTYPE_OOM: usize = 8;
}
