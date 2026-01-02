use core::arch::asm;

#[inline(always)]
pub fn sys_invoke(
    cptr: usize,
    method: usize,
    arg0: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
) -> usize {
    let mut ret;
    unsafe {
        asm!(
            "ecall",
            in("a0") cptr,
            in("a1") arg0,
            in("a2") arg1,
            in("a3") arg2,
            in("a4") arg3,
            in("a5") arg4,
            in("a6") arg5,
            in("a7") method,
            lateout("a0") ret,
        );
    }
    ret
}

#[inline(always)]
pub fn sys_invoke_recv(
    cptr: usize,
    method: usize,
    arg0: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
) -> (usize, usize) {
    let mut ret;
    let mut badge;
    unsafe {
        asm!(
            "ecall",
            in("a0") cptr,
            in("a1") arg0,
            in("a2") arg1,
            in("a3") arg2,
            in("a4") arg3,
            in("a5") arg4,
            in("a6") arg5,
            in("a7") method,
            lateout("a0") ret,
            lateout("t0") badge,
        );
    }
    (ret, badge)
}
