use super::linked_list::LockedLinkedListAllocator;
use crate::sync::mutex::Mutex;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::NonNull;

const SLAB_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

struct FreeNode {
    next: Option<NonNull<FreeNode>>,
}

struct Slab {
    free_list: Option<NonNull<FreeNode>>,
}

unsafe impl Send for Slab {}

impl Slab {
    const fn new() -> Self {
        Self { free_list: None }
    }
}

pub struct SlabAllocator {
    slabs: [Mutex<Slab>; 9],
    fallback: LockedLinkedListAllocator,
}

impl SlabAllocator {
    pub const fn new() -> Self {
        Self {
            slabs: [
                Mutex::new(Slab::new()),
                Mutex::new(Slab::new()),
                Mutex::new(Slab::new()),
                Mutex::new(Slab::new()),
                Mutex::new(Slab::new()),
                Mutex::new(Slab::new()),
                Mutex::new(Slab::new()),
                Mutex::new(Slab::new()),
                Mutex::new(Slab::new()),
            ],
            fallback: LockedLinkedListAllocator::new(),
        }
    }

    pub fn init(&self) {
        // Fallback initialized on use via sbrk
    }

    pub fn add_free_region(&self, addr: usize, size: usize) {
        self.fallback.lock().add_region(addr, size);
    }
}

unsafe impl GlobalAlloc for SlabAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        // 找到合适的 Slab
        for (i, &slab_size) in SLAB_SIZES.iter().enumerate() {
            if size <= slab_size && align <= slab_size {
                let mut slab = self.slabs[i].lock();
                if let Some(node) = slab.free_list {
                    slab.free_list = unsafe { node.as_ref().next };
                    return node.as_ptr() as *mut u8;
                } else {
                    // Slab 空了，从 fallback 分配一个新的页来填充 Slab
                    let layout = Layout::from_size_align(4096, 4096).unwrap();
                    let page = unsafe { self.fallback.alloc(layout) };
                    if page.is_null() {
                        return core::ptr::null_mut();
                    }

                    // 将页切分为多个 Slab 块
                    let mut current = page as *mut FreeNode;
                    for _ in 0..(4096 / slab_size - 1) {
                        let next = (current as usize + slab_size) as *mut FreeNode;
                        unsafe {
                            (*current).next = Some(NonNull::new_unchecked(next));
                        }
                        current = next;
                    }
                    unsafe {
                        (*current).next = None;
                        slab.free_list = Some(NonNull::new_unchecked(
                            (page as usize + slab_size) as *mut FreeNode,
                        ));
                    }
                    return page;
                }
            }
        }

        // 超过 Slab 大小，使用 Fallback (LinkedList)
        unsafe { self.fallback.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size();
        let align = layout.align();

        for (i, &slab_size) in SLAB_SIZES.iter().enumerate() {
            if size <= slab_size && align <= slab_size {
                let mut slab = self.slabs[i].lock();
                let node = ptr as *mut FreeNode;
                unsafe {
                    (*node).next = slab.free_list;
                    slab.free_list = Some(NonNull::new_unchecked(node));
                }
                return;
            }
        }

        unsafe { self.fallback.dealloc(ptr, layout) }
    }
}
