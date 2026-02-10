use crate::arch::runtime::panic_break;
use crate::arch::syscall::syscall;
use crate::cap::{CapPtr, Endpoint};
use crate::error::Error;
use crate::ipc::{MsgFlags, MsgTag, UTCB};
use crate::print;
use crate::protocol;
use crate::set_mrs;

pub const MONITOR_SLOT: CapPtr = CapPtr::from(4);
pub const MONITOR_CAP: Endpoint = Endpoint::from(MONITOR_SLOT);

pub fn sbrk(size: usize) -> Result<usize, ()> {
    let tag = MsgTag::new(protocol::RESOURCE_PROTO, protocol::resource::SBRK, MsgFlags::NONE);
    let mut utcb = unsafe { UTCB::new() };
    utcb.clear();
    set_mrs!(utcb, size);
    utcb.set_msg_tag(tag);
    if MONITOR_CAP.call(&mut utcb).is_ok() {
        let ret = utcb.get_mr(0);
        if ret > 0 { Ok(ret) } else { Err(()) }
    } else {
        Err(())
    }
}

pub fn exit(code: usize) -> ! {
    let tag = MsgTag::new(protocol::PROCESS_PROTO, protocol::process::EXIT, MsgFlags::NONE);
    let mut utcb = unsafe { UTCB::new() };
    utcb.clear();
    set_mrs!(utcb, code);
    utcb.set_msg_tag(tag);
    let _ = MONITOR_CAP.send(&mut utcb);
    print!("Failed to exit with code {}", code);
    loop {
        unsafe {
            panic_break();
        }
    }
}

#[inline(always)]
pub fn sys_invoke(cptr: usize, method: usize, _utcb: &mut UTCB) -> Result<(), Error> {
    let ret = unsafe { syscall(cptr, method) };
    if Error::from(ret) == Error::Success { Ok(()) } else { Err(Error::from(ret)) }
}
