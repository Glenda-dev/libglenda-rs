use buddy_system_allocator::LockedHeap;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::empty();

pub fn init_heap(start: usize, size: usize) {
    unsafe {
        HEAP_ALLOCATOR.lock().init(start, size);
    }
}
