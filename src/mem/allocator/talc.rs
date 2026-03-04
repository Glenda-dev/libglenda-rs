use crate::sync::mutex::Mutex;
use crate::sys::sbrk;
use core::alloc::{GlobalAlloc, Layout};
use talc::*;

pub struct TalcAllocator {
    inner: Talc<Mutex<ClaimOnOom>>,
}

unsafe impl GlobalAlloc for TalcAllocator {
    #[inline(always)]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.inner.lock().alloc(layout)
    }
    #[inline(always)]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.inner.lock().dealloc(ptr, layout)
    }
}

struct SbrkHandler;

impl OomHandler for SbrkHandler {
    fn handle_oom(talc: &mut Talc<Self>, layout: core::alloc::Layout) -> Result<(), ()> {
        // 1. 通过 sbrk 申请更多内存

        let size = layout.size();
        let prev_brk = sbrk(size as isize).map_err(|_| ())? as *mut u8;

        // 2. 将新申请的内存范围告知 Talc
        unsafe {
            let span = Span::from_slice(core::ptr::slice_from_raw_parts_mut(prev_brk, size));
            // claim 会将这段内存合并到现有的堆中
            talc.claim(span)?;
        }

        Ok(())
    }
}
