use core::sync::atomic::{AtomicUsize, Ordering};

/// A simple lock-free ring buffer designed for Shared Memory (SHM)
/// between a Producer and a Consumer.
#[repr(C)]
pub struct ShmRingBuffer {
    pub head: AtomicUsize, // Managed by Consumer
    pub tail: AtomicUsize, // Managed by Producer
    pub size: usize,
    // Data follows at offset 'data_offset'
}

impl ShmRingBuffer {
    /// Initialize a new ring buffer in the given memory
    pub unsafe fn init(ptr: *mut u8, size: usize) -> &'static mut Self {
        unsafe {
            let header = &mut *(ptr as *mut ShmRingBuffer);
            header.head.store(0, Ordering::Release);
            header.tail.store(0, Ordering::Release);
            header.size = size - core::mem::size_of::<ShmRingBuffer>();
            header
        }
    }

    /// Attach to an existing ring buffer in memory
    pub unsafe fn attach(ptr: *mut u8) -> &'static mut Self {
        unsafe { &mut *(ptr as *mut ShmRingBuffer) }
    }

    pub fn data_ptr(&self) -> *mut u8 {
        unsafe {
            (self as *const ShmRingBuffer as *mut u8).add(core::mem::size_of::<ShmRingBuffer>())
        }
    }

    pub fn push_slice(&self, slice: &[u8]) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        let size = self.size;

        // head == tail means empty, (tail + 1) % size == head means full
        let used = if tail >= head { tail - head } else { size - (head - tail) };
        let available = if size > 0 { size - used - 1 } else { 0 };
        let to_write = core::cmp::min(slice.len(), available);

        if to_write == 0 {
            return 0;
        }

        let data = self.data_ptr();
        for i in 0..to_write {
            let idx = (tail + i) % size;
            unsafe {
                *data.add(idx) = slice[i];
            }
        }

        self.tail.store((tail + to_write) % size, Ordering::Release);
        to_write
    }

    pub fn push_byte(&self, byte: u8) -> bool {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        let size = self.size;
        if size == 0 {
            return false;
        }
        let next_tail = (tail + 1) % size;
        if next_tail == head {
            return false;
        }
        let data = self.data_ptr();
        unsafe {
            *data.add(tail) = byte;
        }
        self.tail.store(next_tail, Ordering::Release);
        true
    }

    pub fn pop_byte(&self) -> Option<u8> {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        let size = self.size;
        if head == tail || size == 0 {
            return None;
        }
        let data = self.data_ptr();
        let val = unsafe { *data.add(head) };
        self.head.store((head + 1) % size, Ordering::Release);
        Some(val)
    }

    pub fn pop_slice(&self, buf: &mut [u8]) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        let size = self.size;

        if head == tail || size == 0 {
            return 0;
        }

        let available = if tail >= head { tail - head } else { size - head + tail };
        let to_read = core::cmp::min(buf.len(), available);

        let data = self.data_ptr();
        for i in 0..to_read {
            let idx = (head + i) % size;
            unsafe {
                buf[i] = *data.add(idx);
            }
        }

        self.head.store((head + to_read) % size, Ordering::Release);
        to_read
    }

    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        if tail >= head { tail - head } else { self.size - head + tail }
    }

    pub fn is_full(&self) -> bool {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        (tail + 1) % self.size == head
    }
}
