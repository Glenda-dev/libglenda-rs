use crate::arch::runtime::panic_break;
use crate::arch::syscall::syscall;
use crate::cap::{CapPtr, Endpoint};
use crate::error::Error;
use crate::ipc::{MsgArgs, MsgFlags, MsgTag};
use crate::ipc::{proto, utcb};

pub const MONITOR_SLOT: CapPtr = CapPtr::from(4);
pub const MONITOR_CAP: Endpoint = Endpoint::from(MONITOR_SLOT);

pub fn sbrk(size: usize) -> Result<usize, ()> {
    let tag = MsgTag::new(proto::PROCESS_PROTO, proto::process::SBRK, MsgFlags::NONE);
    if MONITOR_CAP.send(tag, [size, 0, 0, 0, 0, 0, 0]).is_ok() {
        let utcb = unsafe { utcb::get() };
        let ret = utcb.mrs_regs[0];
        if ret > 0 { Ok(ret) } else { Err(()) }
    } else {
        Err(())
    }
}

#[cfg(not(feature = "nosys"))]
pub fn exit(code: usize) -> ! {
    let tag = MsgTag::new(proto::PROCESS_PROTO, proto::process::EXIT, MsgFlags::NONE);
    let _ = MONITOR_CAP.send(tag, [code, 0, 0, 0, 0, 0, 0]);
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
pub fn sys_invoke(cptr: usize, method: usize, args: MsgArgs) -> Result<(), Error> {
    let utcb = unsafe { utcb::get() };
    utcb.mrs_regs = args;
    let ret = unsafe { syscall(cptr, method) };
    if Error::from(ret) == Error::Success { Ok(()) } else { Err(Error::from(ret)) }
}
