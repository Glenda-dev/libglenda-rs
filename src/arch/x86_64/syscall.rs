use core::arch::asm;

#[inline(always)]
pub unsafe fn syscall(cptr: usize, method: usize) -> usize {
    let mut ret;
    unsafe {
        asm!(
            "syscall",
            in("rax") cptr,
            in("rdi") method,
            lateout("rax") ret,
            out("rcx") _,
            out("r11") _,
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
            "syscall",
            in("rax") cptr,
            in("rdi") method,
            lateout("rax") ret,
            lateout("rsi") badge,
            out("rcx") _,
            out("r11") _,
        );
    }
    (ret, badge)
}
