use crate::arch::mem::PGSIZE;
use crate::arch::runtime::backtrace;
use crate::cap::{KERNEL_CAP, KERNEL_SLOT, MONITOR_CAP};
use crate::console::KConsole;
use crate::console::{ANSI_RED, ANSI_RESET};
use crate::ipc::{MsgFlags, MsgTag, UTCB};
use crate::mem::{HEAP_SIZE, HEAP_VA};
use crate::println;
use crate::protocol;
use crate::protocol::resource::InitCap;
use crate::set_mrs;
use crate::sync::mutex::Mutex;
use crate::sys::{exit, sbrk};
use buddy_system_allocator::LockedHeap;
use core::alloc::{GlobalAlloc, Layout};

struct DynamicAllocator {
    inner: LockedHeap<32>,
}

pub static KERNEL_CONSOLE: Mutex<KConsole> = Mutex::new(KConsole::null());

#[global_allocator]
static HEAP_ALLOCATOR: DynamicAllocator = DynamicAllocator { inner: LockedHeap::empty() };

unsafe impl GlobalAlloc for DynamicAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { self.inner.alloc(layout) };
        if !ptr.is_null() {
            return ptr;
        }

        // 第一次分配失败，尝试扩展堆
        // 扩展大小建议至少为 layout 的尺寸与页大小（4KB）的较大值
        let expand_size = layout.size().max(PGSIZE);
        if expand(expand_size).is_ok() {
            // 扩展成功后重试分配
            unsafe { self.inner.alloc(layout) }
        } else {
            core::ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { self.inner.dealloc(ptr, layout) };
    }
}

/// 动态扩展堆内存
pub fn expand(size: usize) -> Result<(), ()> {
    if let Ok(old_break) = sbrk(size) {
        unsafe {
            HEAP_ALLOCATOR.inner.lock().add_to_heap(old_break, old_break + size);
        }
        Ok(())
    } else {
        Err(())
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn glenda_start() -> ! {
    unsafe extern "Rust" {
        fn main() -> usize;
    }
    init_console();
    init_heap();

    let ret = unsafe { main() };
    exit(ret);
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    KERNEL_CONSOLE.unlock();
    println!("\n{}PANIC{}: {}", ANSI_RED, ANSI_RESET, info);
    backtrace();
    exit(usize::MAX)
}

pub fn init_heap() {
    unsafe {
        HEAP_ALLOCATOR.inner.lock().init(HEAP_VA, HEAP_SIZE);
    }
}

pub fn init_console() {
    let tag = MsgTag::new(protocol::RESOURCE_PROTO, protocol::resource::GET_CAP, MsgFlags::NONE);
    let mut utcb = unsafe { UTCB::new() };
    utcb.clear();
    set_mrs!(utcb, InitCap::Kernel as usize);
    utcb.set_recv_window(KERNEL_SLOT);
    utcb.set_msg_tag(tag);
    let res = MONITOR_CAP.call(&mut utcb);
    if let Err(_) = res {
        exit(usize::MAX);
    }
    KERNEL_CONSOLE.lock().initialize(KERNEL_CAP);
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ( $crate::crt0::KERNEL_CONSOLE.lock().print(format_args!($($arg)*)) );
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
