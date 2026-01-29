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

#[inline(always)]
pub unsafe fn syscall_recv(cptr: usize, method: usize) -> (usize, usize) {
    let mut ret;
    let mut badge;
    unsafe {
        asm!(
            "ecall",
            in("a0") cptr,
            in("a7") method,
            lateout("a0") ret,
            lateout("a1") badge,
        );
    }
    (ret, badge)
}
