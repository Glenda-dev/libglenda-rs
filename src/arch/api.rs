#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use core::arch::{asm, global_asm};

global_asm!("");

#[inline(always)]
pub unsafe fn syscall(cptr: usize, method: usize) -> usize {
    unsafe {
        asm!("");
    }
    0
}

#[inline(always)]
pub unsafe fn syscall_recv(cptr: usize, method: usize) -> (usize, usize) {
    unsafe { asm!("") };
    (0, 0)
}

#[inline(always)]
pub unsafe fn panic_break() {
    unsafe {
        asm!("");
    }
}

pub fn backtrace() {
    unreachable!("backtrace not implemented for this arch");
}
