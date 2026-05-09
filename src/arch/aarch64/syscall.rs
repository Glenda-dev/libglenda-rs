use core::arch::asm;

#[inline(always)]
pub unsafe fn syscall(cptr: usize, syscall_no: usize) -> usize {
    let mut ret;
    unsafe {
        asm!(
            "svc #0",
            inlateout("x0") cptr => ret,
            in("x8") syscall_no,
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
            "svc #0",
            inlateout("x0") cptr => ret,
            inout("x1") inout_tag,
            inout("x2") inout_badge,
            inout("x3") mr0,
            inout("x4") mr1,
            inout("x5") mr2,
            inout("x6") mr3,
            in("x8") syscall_no,
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
