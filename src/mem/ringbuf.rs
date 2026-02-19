use core::cmp::min;
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};

/// A shared-memory ring buffer header.
/// This structure is placed at the beginning of the shared memory region.
#[repr(C)]
pub struct SharedRingHeader {
    /// Writer updates this (index of next write).
    pub tail: AtomicUsize,
    /// Reader updates this (index of next read).
    pub head: AtomicUsize,
    /// Total data capacity (excluding header).
    pub size: usize,
}

/// A shared-memory ring buffer.
/// This structure is a handle to the ring buffer residing in shared memory.
///
/// NOTE: This implementation is SPSC (Single-Producer Single-Consumer) safe.
/// For multi-producer or multi-consumer scenarios, external synchronization is required.
pub struct SharedRingBuffer {
    header: *mut SharedRingHeader,
    data: *mut u8,
}

impl SharedRingBuffer {
    /// Initialize a new ring buffer in the given memory region.
    ///
    /// # Safety
    /// The memory region must be at least large enough to hold the header.
    pub unsafe fn init(ptr: *mut u8, size: usize) -> Self {
        let header_ptr = ptr as *mut SharedRingHeader;
        let header = unsafe { &mut *header_ptr };
        header.head.store(0, Ordering::Release);
        header.tail.store(0, Ordering::Release);
        header.size = size - core::mem::size_of::<SharedRingHeader>();
        Self {
            header: header_ptr,
            data: unsafe { ptr.add(core::mem::size_of::<SharedRingHeader>()) },
        }
    }

    /// Attach to an existing ring buffer at the given memory region.
    ///
    /// # Safety
    /// The memory region must have been previously initialized with `init`.
    pub unsafe fn attach(ptr: *mut u8) -> Self {
        let header_ptr = ptr as *mut SharedRingHeader;
        Self {
            header: header_ptr,
            data: unsafe { ptr.add(core::mem::size_of::<SharedRingHeader>()) },
        }
    }

    /// Get current amount of data available to read.
    pub fn readable_len(&self) -> usize {
        let header = unsafe { &*self.header };
        let head = header.head.load(Ordering::Acquire);
        let tail = header.tail.load(Ordering::Acquire);
        let size = header.size;

        if tail >= head { tail - head } else { size - head + tail }
    }

    /// Get current amount of space available to write.
    pub fn writable_len(&self) -> usize {
        let header = unsafe { &*self.header };
        let head = header.head.load(Ordering::Acquire);
        let tail = header.tail.load(Ordering::Acquire);
        let size = header.size;

        if tail >= head { size - (tail - head) - 1 } else { head - tail - 1 }
    }

    /// Check if the ring buffer is empty.
    pub fn is_empty(&self) -> bool {
        let header = unsafe { &*self.header };
        header.head.load(Ordering::Acquire) == header.tail.load(Ordering::Acquire)
    }

    /// Check if the ring buffer is full.
    pub fn is_full(&self) -> bool {
        self.writable_len() == 0
    }

    /// Write data to the ring buffer. returns number of bytes written.
    pub fn write(&mut self, buf: &[u8]) -> usize {
        let header = unsafe { &*self.header };
        let head = header.head.load(Ordering::Acquire);
        let tail = header.tail.load(Ordering::Acquire);
        let size = header.size;

        let available = if tail >= head { size - (tail - head) - 1 } else { head - tail - 1 };

        let to_write = min(available, buf.len());
        if to_write == 0 {
            return 0;
        }

        let first_chunk = min(to_write, size - tail);
        unsafe {
            ptr::copy_nonoverlapping(buf.as_ptr(), self.data.add(tail), first_chunk);
            if to_write > first_chunk {
                ptr::copy_nonoverlapping(
                    buf.as_ptr().add(first_chunk),
                    self.data,
                    to_write - first_chunk,
                );
            }
        }

        header.tail.store((tail + to_write) % size, Ordering::Release);
        to_write
    }

    /// Read data from the ring buffer. returns number of bytes read.
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        let header = unsafe { &*self.header };
        let head = header.head.load(Ordering::Acquire);
        let tail = header.tail.load(Ordering::Acquire);
        let size = header.size;

        let available = if tail >= head { tail - head } else { size - head + tail };

        let to_read = min(available, buf.len());
        if to_read == 0 {
            return 0;
        }

        let first_chunk = min(to_read, size - head);
        unsafe {
            ptr::copy_nonoverlapping(self.data.add(head), buf.as_mut_ptr(), first_chunk);
            if to_read > first_chunk {
                ptr::copy_nonoverlapping(
                    self.data,
                    buf.as_mut_ptr().add(first_chunk),
                    to_read - first_chunk,
                );
            }
        }

        header.head.store((head + to_read) % size, Ordering::Release);
        to_read
    }
}

// Ensure SharedRingBuffer can be sent between threads if the underlying memory is shared.
unsafe impl Send for SharedRingBuffer {}
unsafe impl Sync for SharedRingBuffer {}
