use crate::arch::runtime::panic_break;
use crate::arch::syscall::syscall;
use crate::cap::{CapPtr, Endpoint};
use crate::error::code;
use crate::ipc::{MsgFlags, MsgTag, utcb};
use crate::protocol::process;

pub const MONITOR_SLOT: CapPtr = CapPtr::from(4);
pub const MONITOR_CAP: Endpoint = Endpoint::from(MONITOR_SLOT);

pub fn sbrk(size: usize) -> Result<usize, ()> {
    let tag = MsgTag::new(process::PROCESS_PROTO, process::SBRK, MsgFlags::NONE);
    let ret = MONITOR_CAP.send(tag, [size, 0, 0, 0, 0, 0, 0]);
    if ret == code::SUCCESS {
        let utcb = unsafe { utcb::get() };
        let ret = utcb.mrs_regs[0];
        if ret > 0 { Ok(ret) } else { Err(()) }
    } else {
        Err(())
    }
}

#[cfg(not(feature = "nosys"))]
pub fn exit(code: usize) -> ! {
    let tag = MsgTag::new(process::PROCESS_PROTO, process::EXIT, MsgFlags::NONE);
    MONITOR_CAP.send(tag, [code, 0, 0, 0, 0, 0, 0]);
    loop {
        unsafe {
            panic_break();
        }
    }
}

#[cfg(feature = "nosys")]
pub fn exit(code: usize) -> ! {
    use crate::println;
    println!("Program exited with code: {}\n", code);
    unsafe {
        loop {
            panic_break();
        }
    }
}

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
