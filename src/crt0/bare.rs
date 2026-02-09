use crate::arch::runtime::{backtrace, panic_break};
use crate::cap::KERNEL_CAP;
use crate::console::KConsole;
use crate::console::{ANSI_RED, ANSI_RESET};
use crate::mem::{HEAP_SIZE, HEAP_VA};
use crate::sync::spinlock::SpinLock;
use buddy_system_allocator::LockedHeap;

#[global_allocator]
pub static HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::empty();
pub static KERNEL_CONSOLE: SpinLock<KConsole> = SpinLock::new(KConsole::new(KERNEL_CAP));
unsafe extern "Rust" {
    fn main() -> usize;
}

#[unsafe(no_mangle)]
unsafe extern "C" fn glenda_start() -> ! {
    unsafe {
        HEAP_ALLOCATOR.lock().init(HEAP_VA, HEAP_SIZE);
    }
    let ret = unsafe { main() };
    panic!("Root Task exited with code: {}", ret);
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::crt0::KERNEL_CONSOLE.lock().print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // Prevent deadlock if panic occurs while holding the console lock
    unsafe {
        KERNEL_CONSOLE.force_unlock();
    }
    println!("\n{}PANIC{}: {}", ANSI_RED, ANSI_RESET, info);
    backtrace();
    loop {
        unsafe {
            panic_break();
        }
    }
}
