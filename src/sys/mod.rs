use crate::arch::runtime::panic_break;
use crate::arch::syscall::syscall;
use crate::cap::MONITOR_CAP;
use crate::console::{ANSI_RED, ANSI_RESET};
use crate::error::Error;
use crate::ipc::{MsgFlags, MsgTag, UTCB};
#[cfg(not(feature = "rt-hosted"))]
use crate::print;
use crate::protocol;
use crate::set_mrs;
#[cfg(feature = "rt-hosted")]
use std::print;

#[cfg(not(feature = "rt-bare"))]
pub fn sbrk(incr: isize) -> Result<usize, Error> {
    let tag = MsgTag::new(protocol::RESOURCE_PROTO, protocol::resource::SBRK, MsgFlags::NONE);
    let mut utcb = unsafe { UTCB::new() };
    utcb.clear();
    set_mrs!(utcb, incr as usize);
    utcb.set_msg_tag(tag);
    MONITOR_CAP.call(&mut utcb)?;
    let ret = utcb.get_mr(0);
    if ret != 0 { Ok(ret) } else { Err(Error::OutOfMemory) }
}

#[cfg(feature = "rt-bare")]
pub fn sbrk(_incr: isize) -> Result<usize, Error> {
    Err(Error::OutOfMemory)
}

pub fn exit(code: usize) -> ! {
    let tag = MsgTag::new(protocol::PROCESS_PROTO, protocol::process::EXIT, MsgFlags::NONE);
    let mut utcb = unsafe { UTCB::new() };
    utcb.clear();
    set_mrs!(utcb, code);
    utcb.set_msg_tag(tag);
    if MONITOR_CAP.send(&mut utcb).is_err() {
        crate::println!("Failed to exit with code {}", code);
    }

    // Do not invoke TCB::YIELD here.
    // A yield syscall can leave a transient cloned TCB capability alive across
    // context switch, which interferes with monitor-side teardown refcount checks.
    // Spinning in user mode is sufficient: timer preemption will hand control
    // back to the monitor to perform process cleanup.
    loop {
        core::hint::spin_loop();
    }
}

#[inline(always)]
pub fn sys_invoke(cptr: usize, method: usize, _utcb: &mut UTCB) -> Result<(), Error> {
    let ret = unsafe { syscall(cptr, method) };
    if Error::from(ret) == Error::Success { Ok(()) } else { Err(Error::from(ret)) }
}
