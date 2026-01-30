use crate::mem::HEAP_VA;
use crate::runtime::HEAP_SIZE;
use buddy_system_allocator::LockedHeap;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::empty();

pub fn init() {
    unsafe {
        HEAP_ALLOCATOR.lock().init(HEAP_VA, HEAP_SIZE);
    }
}
