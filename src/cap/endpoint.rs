use super::CapPtr;
use super::ipcmethod;
use crate::error::Error;
use crate::ipc::utcb;
use crate::ipc::{MsgArgs, MsgTag};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Endpoint(CapPtr);

impl Endpoint {
    pub const fn from(cap: CapPtr) -> Self {
        Self(cap)
    }

    pub fn cap(&self) -> CapPtr {
        self.0
    }

    pub fn send(&self, msg_info: MsgTag, args: MsgArgs) -> Result<(), Error> {
        let utcb = unsafe { utcb::get() };
        utcb.msg_tag = msg_info;
        self.0.invoke(ipcmethod::SEND, args)
    }

    pub fn recv(&self, reply_slot: CapPtr) -> Result<usize, Error> {
        let utcb = unsafe { utcb::get() };
        utcb.recv_window = Endpoint::from(reply_slot);
        let ret = self.0.invoke(ipcmethod::RECV, [0, 0, 0, 0, 0, 0, 0]);
        if ret.is_ok() { Ok(utcb.mrs_regs[0]) } else { Err(ret.unwrap_err()) }
    }

    pub fn call(&self, msg_info: MsgTag, args: MsgArgs) -> Result<(), Error> {
        let utcb = unsafe { utcb::get() };
        utcb.msg_tag = msg_info;
        self.0.invoke(ipcmethod::CALL, args)
    }

    pub fn notify(&self, badge: usize) -> Result<(), Error> {
        self.0.invoke(ipcmethod::NOTIFY, [badge, 0, 0, 0, 0, 0, 0])
    }
}
