use core::arch::asm;

// 系统调用
#[inline(always)]
pub unsafe fn syscall(cptr: usize, method: usize) -> usize {
    unsafe {
        asm!("");
    }
    0
}
