pub mod channel;
pub mod condvar;
pub mod mutex;
pub mod rwlock;
pub mod semaphore;
pub mod spinlock;

use crate::cap::{CapPtr, Endpoint};
use crate::ipc::{MsgFlags, MsgTag};

/// Park the current thread (block until unparked).
/// Uses the thread-local notification endpoint.
pub fn park() {
    let ep = current_thread_park_endpoint();
    // Block receiving a message. We use a null reply/badge slot as we don't expect complex IPC.
    // In a real loop we might handle spurious wakeups.
    let _ = ep.recv(CapPtr::from(0));
}

/// Unpark a specific thread via its endpoint.
pub fn unpark(endpoint: Endpoint) {
    let tag = MsgTag::new(0, 0, MsgFlags::NONE);
    let _ = endpoint.send(tag);
}

// Placeholder for referencing the current thread's parker endpoint.
// In a real system, this comes from TLS.
pub fn current_thread_park_endpoint() -> Endpoint {
    // TODO: Get from TLS or TCB
    // For now returning a dummy or panicking if used without setup
    unimplemented!("Need TLS support to get current thread endpoint")
}
