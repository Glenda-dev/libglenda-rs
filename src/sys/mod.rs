use crate::arch::runtime::panic_break;
use crate::arch::syscall::{syscall, syscall_ipc};
use crate::cap::MONITOR_CAP;
use crate::cap::encode_invoke;
use crate::console::{ANSI_RED, ANSI_RESET};
use crate::error::Error;
use crate::ipc::{Badge, MsgFlags, MsgTag, UTCB};
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
    if MONITOR_CAP.call(&mut utcb).is_err() {
        crate::println!("Failed to exit with code {}", code);
    }
    loop {
        core::hint::spin_loop();
    }
}

#[inline(always)]
pub fn sys_invoke(cptr: usize, method: usize, _utcb: &mut UTCB) -> Result<(), Error> {
    #[cfg(feature = "rt-hosted")]
    let syscall_no = method;
    #[cfg(not(feature = "rt-hosted"))]
    let syscall_no = encode_invoke(method) as usize;

    let ret = unsafe { syscall(cptr, syscall_no) };
    if Error::from(ret) == Error::Success { Ok(()) } else { Err(Error::from(ret)) }
}

#[inline(always)]
pub fn sys_invoke_ipc(cptr: usize, method: usize, utcb: &mut UTCB) -> Result<(), Error> {
    #[cfg(feature = "rt-hosted")]
    {
        return sys_invoke(cptr, method, utcb);
    }

    #[cfg(not(feature = "rt-hosted"))]
    {
        let tag = utcb.get_msg_tag();
        let mut flags = tag.flags();
        if utcb.get_mrs_count() > 4 {
            flags |= MsgFlags::HAS_MRS;
        } else {
            flags.remove(MsgFlags::HAS_MRS);
        }

        let mut msgtag = MsgTag::new(tag.proto(), tag.label(), flags).as_usize();
        utcb.set_msg_tag(MsgTag(msgtag));

        let mut badge = utcb.get_badge().bits();
        let mut mrs = [utcb.get_mr(0), utcb.get_mr(1), utcb.get_mr(2), utcb.get_mr(3)];

        let syscall_no = encode_invoke(method) as usize;
        let ret = unsafe { syscall_ipc(cptr, syscall_no, &mut msgtag, &mut badge, &mut mrs) };

        utcb.set_msg_tag(MsgTag(msgtag));
        utcb.set_badge(Badge::new(badge));
        utcb.set_mr(0, mrs[0]);
        utcb.set_mr(1, mrs[1]);
        utcb.set_mr(2, mrs[2]);
        utcb.set_mr(3, mrs[3]);

        if Error::from(ret) == Error::Success { Ok(()) } else { Err(Error::from(ret)) }
    }
}
