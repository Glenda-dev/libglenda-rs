use crate::arch::runtime::panic_break;
use crate::cap::{CapPtr, Endpoint, Frame};
use crate::error::code;
use crate::ipc::utcb;
use crate::ipc::{MsgFlags, MsgTag};
use crate::protocol::process;

pub const MONITOR_SLOT: usize = 4;
pub const PLATFORM_SLOT: usize = 6;
pub const MONITOR_CAP: Endpoint = Endpoint::from(CapPtr::new(MONITOR_SLOT));
pub const PLATFORM_CAP: Frame = Frame::from(CapPtr::new(PLATFORM_SLOT));

pub fn exit(code: usize) -> ! {
    let tag = MsgTag::new(process::PROCESS_PROTO, process::EXIT, MsgFlags::NONE);
    MONITOR_CAP.send(tag, [code, 0, 0, 0, 0, 0, 0]);
    loop {
        unsafe {
            panic_break();
        }
    }
}

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
