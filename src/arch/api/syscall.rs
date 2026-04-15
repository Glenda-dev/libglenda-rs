//! Syscall Interface API (Reference Only)
//!
//! This module defines the expected signatures for syscall-related functions
//! that must be implemented by each architecture.

pub unsafe fn syscall(cptr: usize, method: usize) -> usize {
    unimplemented!("This is a reference API, not a concrete implementation.");
}

pub unsafe fn syscall_ipc(
    cptr: usize,
    method: usize,
    msgtag: &mut usize,
    badge: &mut usize,
    mrs: &mut [usize; 4],
) -> usize {
    let _ = (cptr, method, msgtag, badge, mrs);
    unimplemented!("This is a reference API, not a concrete implementation.");
}
