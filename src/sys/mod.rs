use crate::arch::syscall::syscall;
use crate::cap::{CapPtr, Endpoint};
use crate::error::Error;
use crate::ipc::{MsgFlags, MsgTag, UTCB};
use crate::protocol;

pub const MONITOR_SLOT: CapPtr = CapPtr::from(4);
pub const MONITOR_CAP: Endpoint = Endpoint::from(MONITOR_SLOT);

pub fn sbrk(size: usize) -> Result<usize, ()> {
    let tag = MsgTag::new(protocol::RESOURCE_PROTO, protocol::resource::SBRK, MsgFlags::NONE);
    let utcb = unsafe { UTCB::get() };
    utcb.mrs_regs = [size, 0, 0, 0, 0, 0, 0, 0];
    if MONITOR_CAP.send(tag).is_ok() {
        let utcb = unsafe { UTCB::get() };
        let ret = utcb.mrs_regs[0];
        if ret > 0 { Ok(ret) } else { Err(()) }
    } else {
        Err(())
    }
}

pub fn exit(code: usize) -> ! {
    let tag = MsgTag::new(protocol::PROCESS_PROTO, protocol::process::EXIT, MsgFlags::NONE);
    let utcb = unsafe { UTCB::get() };
    utcb.mrs_regs = [code, 0, 0, 0, 0, 0, 0, 0];
    let _ = MONITOR_CAP.send(tag);
    unreachable!("Failed to exit with code {}", code);
}

#[inline(always)]
pub fn sys_invoke(cptr: usize, method: usize) -> Result<(), Error> {
    let ret = unsafe { syscall(cptr, method) };
    if Error::from(ret) == Error::Success { Ok(()) } else { Err(Error::from(ret)) }
}
