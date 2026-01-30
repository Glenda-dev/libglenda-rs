use core::arch::asm;

#[inline(always)]
pub unsafe fn syscall(cptr: usize, method: usize) -> usize {
    let mut ret;
    unsafe {
        asm!(
            "ecall",
            in("a0") cptr,
            in("a7") method,
            lateout("a0") ret,
        );
    }
    ret
}
