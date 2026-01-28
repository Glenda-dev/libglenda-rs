#![no_std]

extern crate alloc;

pub mod allocator;
mod arch;
pub mod cap;
pub mod console;
pub mod elf;
pub mod error;
pub mod initrd;
pub mod ipc;
pub mod manifest;
pub mod mem;
pub mod posix;
pub mod protocol;
pub mod runtime;
mod syscall;
