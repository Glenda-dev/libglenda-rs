use crate::arch::mem::PGSIZE;
use crate::arch::runtime::{backtrace, panic_break};
use crate::console;
use crate::console::{ANSI_RED, ANSI_RESET};
use crate::println_unsynced;
use buddy_system_allocator::LockedHeap;

pub const HEAP_PAGES: usize = 256; // 用户堆页面数 256 * 4KB = 1MB
pub const HEAP_SIZE: usize = HEAP_PAGES * PGSIZE; // 1MB
pub const HEAP_VA: usize = 0x2000_0000; // 用户堆地址

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::empty();

#[unsafe(no_mangle)]
unsafe extern "C" fn glenda_start() -> ! {
    unsafe extern "C" {
        static mut __bss_start: u8;
        static mut __bss_end: u8;
    }

    unsafe {
        let start = &raw mut __bss_start;
        let end = &raw mut __bss_end;
        let len = end as usize - start as usize;
        core::ptr::write_bytes(start, 0, len);
    }

    unsafe extern "Rust" {
        fn main() -> usize;
    }
    console::init();

    unsafe {
        HEAP_ALLOCATOR.lock().init(HEAP_VA, HEAP_SIZE);
    }

    let ret = unsafe { main() };
    panic!("Root Task exited with code: {}", ret);
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println_unsynced!("{}PANIC{}: {}", ANSI_RED, ANSI_RESET, info);
    backtrace();
    loop {
        unsafe {
            panic_break();
        }
    }
}
