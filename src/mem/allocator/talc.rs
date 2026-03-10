use crate::sync::mutex::Mutex;
use crate::sys::sbrk;
use core::alloc::{GlobalAlloc, Layout};
use talc::*;

pub struct TalcAllocator {
    /// The actual talc heap instance wrapped in our own mutex to provide thread
    /// safety.  The previous implementation attempted to make the *oom handler*
    /// itself a `Mutex`, which triggered compilation errors because `Mutex<T>`
    /// does not implement `talc::OomHandler`.  By moving the lock one level up
    /// we can simply use `SbrkHandler` as the handler (it already implements
    /// `OomHandler`) and protect the entire allocator with a mutex.
    inner: Mutex<Talc<SbrkHandler>>,
}

impl TalcAllocator {
    /// Create a new, empty allocator.  The underlying `Talc` is initialised
    /// with the [`SbrkHandler`] so that more memory can be obtained from the
    /// kernel at OOM time.
    pub const fn new() -> Self {
        // NOTE: `Mutex::new` is a `const fn` so this entire constructor is
        // also `const`.
        TalcAllocator { inner: Mutex::new(Talc::new(SbrkHandler)) }
    }
}

unsafe impl GlobalAlloc for TalcAllocator {
    #[inline(always)]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Acquire the lock and call `malloc` on the contained Talc instance.
        // `malloc` returns a `Result<NonNull<u8>, ()>`; convert to raw pointer.
        unsafe {
            match self.inner.lock().malloc(layout) {
                Ok(nonnull) => nonnull.as_ptr(),
                Err(_) => core::ptr::null_mut(),
            }
        }
    }

    #[inline(always)]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if let Some(nonnull) = core::ptr::NonNull::new(ptr) {
            unsafe {
                self.inner.lock().free(nonnull, layout);
            }
        }
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

// Make the global allocator `Sync` – the internal mutex ensures safe concurrent
// access to the underlying `Talc` instance.
unsafe impl Sync for TalcAllocator {}
