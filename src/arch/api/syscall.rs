use core::arch::asm;

// 系统调用
#[inline(always)]
pub unsafe fn syscall(cptr: usize, method: usize) -> usize {
    unsafe {
        asm!("");
    }
    0
}

// 系统调用/带返回值
#[inline(always)]
pub unsafe fn syscall_recv(cptr: usize, method: usize) -> (usize, usize) {
    unsafe { asm!("") };
    (0, 0)
}
