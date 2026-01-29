use core::arch::{asm, global_asm};

// _start汇编
global_asm!("");

// 断点指令
#[inline(always)]
pub unsafe fn panic_break() {
    unsafe {
        asm!("");
    }
}

// Backtrace
pub fn backtrace() {
    unimplemented!()
}
