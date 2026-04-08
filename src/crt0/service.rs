use crate::arch::mem::PGSIZE;
use crate::arch::runtime::backtrace;
use crate::cap::CONSOLE_CAP;
use crate::console::KConsole;
use crate::console::{ANSI_RED, ANSI_RESET};
use crate::ipc::ThreadControlBlock;
use crate::mem::HEAP_VA;
use crate::mem::allocator::Allocator;
use crate::sync::mutex::Mutex;
use crate::sys::exit;

#[unsafe(no_mangle)]
static mut MAIN_TCB: ThreadControlBlock = ThreadControlBlock::new();

pub static KERNEL_CONSOLE: Mutex<KConsole> = Mutex::new(KConsole::null());

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ( $crate::crt0::KERNEL_CONSOLE.lock().print(format_args!($($arg)*)) );
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[global_allocator]
static HEAP_ALLOCATOR: Allocator = Allocator::new();

#[unsafe(no_mangle)]
unsafe extern "C" fn glenda_start(_arg: usize, tid: usize) -> ! {
    unsafe extern "Rust" {
        fn main() -> usize;
    }
    unsafe {
        init_tcb(tid);
    }
    init_console();
    init_heap();

    let ret = unsafe { main() };
    exit(ret);
}

pub unsafe fn init_tcb(tid: usize) {
    let tp = crate::arch::thread::get_thread_pointer();
    unsafe {
        if tp == 0 {
            // Main thread
            MAIN_TCB.self_ptr = core::ptr::addr_of_mut!(MAIN_TCB) as usize;
            MAIN_TCB.tid = tid;
            crate::arch::thread::set_thread_pointer(core::ptr::addr_of_mut!(MAIN_TCB) as usize);
        } else {
            // Thread created by thread_create, tp already set to provided TCB
            let tcb = &mut *(tp as *mut ThreadControlBlock);
            tcb.self_ptr = tp;
            tcb.tid = tid;
        }
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    KERNEL_CONSOLE.unlock();
    println!("\n{}PANIC{}: {}", ANSI_RED, ANSI_RESET, info);
    backtrace();
    exit(usize::MAX)
}

pub fn init_heap() {
    HEAP_ALLOCATOR.init();
}

pub fn init_console() {
    // Console capability is already pre-populated into CONSOLE_SLOT (5)
    // by the monitor (warren) during process creation.
    KERNEL_CONSOLE.lock().initialize(CONSOLE_CAP);
}
