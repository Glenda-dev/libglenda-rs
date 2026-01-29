use crate::arch::syscall::{syscall, syscall_recv};
use crate::ipc::utcb;

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
    arg6: usize,
) -> usize {
    let utcb = unsafe { utcb::get() };
    utcb.mrs_regs[0] = arg0;
    utcb.mrs_regs[1] = arg1;
    utcb.mrs_regs[2] = arg2;
    utcb.mrs_regs[3] = arg3;
    utcb.mrs_regs[4] = arg4;
    utcb.mrs_regs[5] = arg5;
    utcb.mrs_regs[6] = arg6;
    unsafe { syscall(cptr, method) }
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
    arg6: usize,
) -> (usize, usize) {
    let utcb = unsafe { utcb::get() };
    utcb.mrs_regs[0] = arg0;
    utcb.mrs_regs[1] = arg1;
    utcb.mrs_regs[2] = arg2;
    utcb.mrs_regs[3] = arg3;
    utcb.mrs_regs[4] = arg4;
    utcb.mrs_regs[5] = arg5;
    utcb.mrs_regs[6] = arg6;
    unsafe { syscall_recv(cptr, method) }
}
