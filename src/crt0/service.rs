use crate::arch::mem::PGSIZE;
use crate::arch::runtime::backtrace;
use crate::console;
use crate::console::{ANSI_RED, ANSI_RESET};
use crate::mem::{HEAP_SIZE, HEAP_VA};
use crate::println_unsynced;
use crate::sys::{exit, sbrk};
use buddy_system_allocator::LockedHeap;
use core::alloc::{GlobalAlloc, Layout};

struct DynamicAllocator {
    inner: LockedHeap<32>,
}

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
    exit(ret);
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println_unsynced!("{}PANIC{}: {}", ANSI_RED, ANSI_RESET, info);
    backtrace();
    exit(usize::MAX)
}

pub fn init_heap() {
    unsafe {
        HEAP_ALLOCATOR.inner.lock().init(HEAP_VA, HEAP_SIZE);
    }
}
