use super::CapPtr;
use super::ipcmethod;
use crate::error::Error;
use crate::ipc::{Badge, MsgTag, UTCB};

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

    pub fn send(&self, msg_info: MsgTag) -> Result<(), Error> {
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = msg_info;
        self.0.invoke(ipcmethod::SEND)
    }

    pub fn recv(&self, reply_slot: CapPtr) -> Result<usize, Error> {
        let utcb = unsafe { UTCB::get() };
        utcb.recv_window = reply_slot;
        let ret = self.0.invoke(ipcmethod::RECV);
        if ret.is_ok() { Ok(utcb.mrs_regs[0]) } else { Err(ret.unwrap_err()) }
    }

    pub fn call(&self, msg_info: MsgTag) -> Result<(), Error> {
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = msg_info;
        self.0.invoke(ipcmethod::CALL)
    }

    pub fn notify(&self, badge: Badge) -> Result<(), Error> {
        let utcb = unsafe { UTCB::get() };
        utcb.badge = badge;
        self.0.invoke(ipcmethod::NOTIFY)
    }
}
