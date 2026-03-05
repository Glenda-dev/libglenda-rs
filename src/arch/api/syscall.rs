//! Syscall Interface API (Reference Only)
//!
//! This module defines the expected signatures for syscall-related functions
//! that must be implemented by each architecture.

pub unsafe fn syscall(cptr: usize, method: usize) -> usize {
    unimplemented!("This is a reference API, not a concrete implementation.");
}
