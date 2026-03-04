use crate::arch::runtime::{backtrace, panic_break};
use crate::cap::{CONSOLE_CAP, CapPtr, Frame, IrqHandler, Kernel, Untyped};
use crate::console::KConsole;
use crate::console::{ANSI_RED, ANSI_RESET};
use crate::ipc::ThreadControlBlock;
use crate::mem::allocator::Allocator;
use crate::mem::{HEAP_SIZE, HEAP_VA};
use crate::sync::spinlock::SpinLock;

// Root Task slots provided by the kernel
pub const BOOTINFO_SLOT: CapPtr = CapPtr::from(9);
pub const UNTYPED_SLOT: CapPtr = CapPtr::from(10);
pub const KERNEL_SLOT: CapPtr = CapPtr::from(11);
pub const IRQ_CONTROL_SLOT: CapPtr = CapPtr::from(12);

// Root Task aliases for capabilities provided by the kernel
pub const BOOTINFO_CAP: Frame = Frame::from(BOOTINFO_SLOT);
pub const UNTYPED_CAP: Untyped = Untyped::from(UNTYPED_SLOT);
pub const KERNEL_CAP: Kernel = Kernel::from(KERNEL_SLOT);
pub const IRQ_CONTROL_CAP: IrqHandler = IrqHandler::from(IRQ_CONTROL_SLOT);

#[unsafe(no_mangle)]
static mut MAIN_TCB: ThreadControlBlock = ThreadControlBlock::new();

#[global_allocator]
pub static HEAP_ALLOCATOR: Allocator = Allocator::new();
pub static KERNEL_CONSOLE: SpinLock<KConsole> = SpinLock::new(KConsole::new(CONSOLE_CAP));

unsafe extern "Rust" {
    fn main() -> usize;
}

#[unsafe(no_mangle)]
unsafe extern "C" fn glenda_start() -> ! {
    HEAP_ALLOCATOR.init();
    HEAP_ALLOCATOR.add_free_region(HEAP_VA, HEAP_SIZE);
    unsafe {
        init_tcb();
    }
    let ret = unsafe { main() };
    panic!("Root Task exited with code: {}", ret);
}

unsafe fn init_tcb() {
    unsafe {
        // Main thread
        MAIN_TCB.self_ptr = core::ptr::addr_of_mut!(MAIN_TCB) as usize;
        MAIN_TCB.tid = 0;
        crate::arch::thread::set_thread_pointer(core::ptr::addr_of_mut!(MAIN_TCB) as usize);
    }
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
