use crate::arch::mem::PGSIZE;
use crate::mem::HEAP_VA;
use crate::runtime::sbrk;
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

/// 初始化初始堆空间
pub fn init() {
    unsafe {
        HEAP_ALLOCATOR.inner.lock().init(HEAP_VA, 0);
    }
}

/// 动态扩展堆内存
pub fn expand(size: usize) -> Result<(), ()> {
    // ...existing code...
    if let Ok(old_break) = sbrk(size) {
        unsafe {
            HEAP_ALLOCATOR.inner.lock().add_to_heap(old_break, old_break + size);
        }
        Ok(())
    } else {
        Err(())
    }
}
