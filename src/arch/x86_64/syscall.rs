use core::arch::asm;

#[inline(always)]
pub unsafe fn syscall(cptr: usize, syscall_no: usize) -> usize {
    let mut ret;
    unsafe {
        asm!(
            "syscall",
            in("rax") syscall_no,
            in("rdi") cptr,
            lateout("rax") ret,
            out("rcx") _,
            out("r11") _,
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
    let _ = (msgtag, badge, mrs);
    unsafe { syscall(cptr, syscall_no) }
}
