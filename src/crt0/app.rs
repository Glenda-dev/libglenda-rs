use crate::console::ConsoleVT;
use crate::ipc::ThreadControlBlock;
use crate::mem::allocator::Allocator;
use crate::sync::mutex::Mutex;
use crate::sys::exit;

#[unsafe(no_mangle)]
static mut MAIN_TCB: ThreadControlBlock = ThreadControlBlock::new();

pub static APP_CONSOLE: Mutex<Option<ConsoleVT>> = Mutex::new(None);

pub fn init_console(console: ConsoleVT) {
    *APP_CONSOLE.lock() = Some(console);
}

// public printing macros that mirror the other runtimes but route through the
// global `APP_CONSOLE` if present.  If the console has not been initialised
// the macros simply do nothing rather than panic; this keeps early boot code
// from crashing if it tries to print before the terminal is ready.

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        if let Some(ref mut cons) = *$crate::crt0::APP_CONSOLE.lock() {
            cons.print(format_args!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[global_allocator]
static HEAP_ALLOCATOR: Allocator = Allocator::new();

#[unsafe(no_mangle)]
unsafe extern "C" fn glenda_start(arg: usize) -> ! {
    unsafe {
        init_tcb(arg);
    }
    unsafe extern "Rust" {
        fn main() -> usize;
    }

    let ret = unsafe { main() };
    exit(ret);
}

unsafe fn init_tcb(tid: usize) {
    let tp = crate::arch::thread::get_thread_pointer();
    unsafe {
        if tp == 0 {
            MAIN_TCB.self_ptr = core::ptr::addr_of_mut!(MAIN_TCB) as usize;
            MAIN_TCB.tid = tid;
            crate::arch::thread::set_thread_pointer(core::ptr::addr_of_mut!(MAIN_TCB) as usize);
        } else {
            let tcb = &mut *(tp as *mut ThreadControlBlock);
            tcb.self_ptr = tp;
            tcb.tid = tid;
        }
    }
}

// simple panic handler which prints to the console if initialised and then
// terminates the process.

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    if let Some(ref mut cons) = *APP_CONSOLE.lock() {
        cons.print(format_args!("panic: {}\n", info));
    }
    exit(usize::MAX);
}
