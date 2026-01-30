use crate::allocator;
use crate::arch::runtime::{backtrace, panic_break};
use crate::console;
use crate::console::{ANSI_RED, ANSI_RESET};
use crate::mem::{HEAP_SIZE, HEAP_VA};
use crate::{println, println_unsynced};

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
    init_heap();
    let ret = unsafe { main() };

    exit(ret)
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println_unsynced!("{}PANIC{}: {}", ANSI_RED, ANSI_RESET, info);
    backtrace();
    exit(usize::MAX);
}

pub fn exit(code: usize) -> ! {
    println!("Program exited with code: {}\n", code);
    unsafe {
        loop {
            panic_break();
        }
    }
}

fn init_heap() {
    #[cfg(feature = "fixed-heap")]
    allocator::init_heap(HEAP_VA, HEAP_SIZE);
}
