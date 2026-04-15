use core::arch::asm;

#[inline(always)]
pub unsafe fn syscall(cptr: usize, syscall_no: usize) -> usize {
    let mut ret;
    unsafe {
        asm!(
            "ecall",
            inlateout("a0") cptr => ret,
            in("a7") syscall_no,
            options(nostack),
        );
    }
    ret
}

#[inline(always)]
pub unsafe fn syscall_ipc(
    cptr: usize,
    syscall_no: usize,
    msgtag: &mut usize,
    badge: &mut usize,
    mrs: &mut [usize; 4],
) -> usize {
    let mut inout_tag = *msgtag;
    let mut inout_badge = *badge;
    let mut mr0 = mrs[0];
    let mut mr1 = mrs[1];
    let mut mr2 = mrs[2];
    let mut mr3 = mrs[3];
    let mut ret;
    unsafe {
        asm!(
            "ecall",
            inlateout("a0") cptr => ret,
            inout("a1") inout_tag,
            inout("a2") inout_badge,
            inout("a3") mr0,
            inout("a4") mr1,
            inout("a5") mr2,
            inout("a6") mr3,
            in("a7") syscall_no,
            options(nostack),
        );
    }
    *msgtag = inout_tag;
    *badge = inout_badge;
    mrs[0] = mr0;
    mrs[1] = mr1;
    mrs[2] = mr2;
    mrs[3] = mr3;
    ret
}
