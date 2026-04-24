pub mod channel;
pub mod condvar;
pub mod mutex;
pub mod once;
pub mod rwlock;
pub mod semaphore;
pub mod spinlock;

use crate::cap::Endpoint;
use crate::ipc::{Badge, ThreadControlBlock, UTCB};

const PARK_BADGE: Badge = Badge::new(1);

/// Park the current thread (block until unparked).
/// Uses the thread-local notification endpoint.
pub fn park() {
    let ep = current_thread_park_endpoint();
    // Block receiving a message. We use a null reply/badge slot as we don't expect complex IPC.
    // In a real loop we might handle spurious wakeups.
    let mut utcb = unsafe { UTCB::new() };
    let _ = ep.recv(&mut utcb);
}

/// Unpark a specific thread via its endpoint.
pub fn unpark(endpoint: Endpoint) {
    let _ = endpoint.notify(PARK_BADGE);
}

// Placeholder for referencing the current thread's parker endpoint.
// In a real system, this comes from TLS.
pub fn current_thread_park_endpoint() -> Endpoint {
    let tp = crate::arch::thread::get_thread_pointer();
    assert!(tp != 0, "current_thread_park_endpoint requires an initialized thread pointer");
    let tcb = unsafe { &*(tp as *const ThreadControlBlock) };
    assert!(!tcb.park_ep.cap().is_null(), "current thread has no registered park endpoint");
    tcb.park_ep
}
