pub mod platform;

use crate::arch::runtime::panic_break;
use crate::cap::{CapPtr, Endpoint, Frame, Kernel};
use crate::error::code;
use crate::ipc::utcb;
use crate::ipc::{MsgFlags, MsgTag};
use crate::protocol::process;

pub const MONITOR_SLOT: CapPtr = CapPtr::from(4);
pub const PLATFORM_SLOT: CapPtr = CapPtr::from(6);
pub const KERNEL_SLOT: CapPtr = CapPtr::from(5);
pub const MONITOR_CAP: Endpoint = Endpoint::from(MONITOR_SLOT);
pub const PLATFORM_CAP: Frame = Frame::from(PLATFORM_SLOT);
pub const KERNEL_CAP: Kernel = Kernel::from(KERNEL_SLOT);
pub const FAULT_SLOT: CapPtr = CapPtr::from(4);
pub const FAULT_CAP: Endpoint = Endpoint::from(FAULT_SLOT);

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
