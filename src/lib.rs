#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
pub mod allocator;
pub mod arch;
pub mod cap;
pub mod console;
pub mod crt0;
pub mod error;
pub mod ipc;
pub mod manifest;
pub mod mem;
pub mod protocol;
pub mod runtime;
mod syscall;
