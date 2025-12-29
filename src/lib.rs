#![no_std]

pub mod bootinfo;
pub mod caps;
pub mod errors;
pub mod posix;
pub mod syscall;
pub mod types;
pub mod utcb;

pub use caps::CapPtr;
pub use types::*;

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

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe {
        core::arch::asm!("ebreak");
        loop {
            core::arch::asm!("wfi");
        }
    }
}
