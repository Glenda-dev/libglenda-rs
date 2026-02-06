use crate::arch::syscall::syscall;
use crate::cap::{CapPtr, Endpoint};
use crate::error::Error;
use crate::ipc::{MsgArgs, MsgFlags, MsgTag, UTCB};
use crate::protocol;

pub const MONITOR_SLOT: CapPtr = CapPtr::from(4);
pub const MONITOR_CAP: Endpoint = Endpoint::from(MONITOR_SLOT);

pub fn sbrk(size: usize) -> Result<usize, ()> {
    let tag = MsgTag::new(protocol::PROCESS_PROTO, protocol::resource::SBRK, MsgFlags::NONE);
    if MONITOR_CAP.send(tag, [size, 0, 0, 0, 0, 0, 0, 0]).is_ok() {
        let utcb = unsafe { UTCB::get() };
        let ret = utcb.mrs_regs[0];
        if ret > 0 { Ok(ret) } else { Err(()) }
    } else {
        Err(())
    }
}

pub fn exit(code: usize) -> ! {
    let tag = MsgTag::new(protocol::PROCESS_PROTO, protocol::process::EXIT, MsgFlags::NONE);
    let _ = MONITOR_CAP.send(tag, [code, 0, 0, 0, 0, 0, 0, 0]);
    unreachable!("Failed to exit with code {}", code);
}

#[inline(always)]
pub fn sys_invoke(cptr: usize, method: usize, args: MsgArgs) -> Result<(), Error> {
    let utcb = unsafe { UTCB::get() };
    utcb.mrs_regs = args;
    let ret = unsafe { syscall(cptr, method) };
    if Error::from(ret) == Error::Success { Ok(()) } else { Err(Error::from(ret)) }
}
