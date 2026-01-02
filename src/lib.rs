#![no_std]

extern crate alloc;

pub mod allocator;
pub mod bootinfo;
pub mod cap;
pub mod crt0;
pub mod elf;
pub mod error;
pub mod factotum;
pub mod initrd;
pub mod ipc;
pub mod log;
pub mod posix;
pub mod syscall;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe {
        core::arch::asm!("ebreak");
        loop {
            core::arch::asm!("wfi");
        }
    }
}
