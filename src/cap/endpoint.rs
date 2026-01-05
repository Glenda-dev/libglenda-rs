use super::{Args, CapPtr, ipcmethod};
use crate::ipc::{MsgTag, utcb};
use crate::syscall::sys_invoke_recv;

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

    pub fn send(&self, msg_info: MsgTag, args: Args) -> usize {
        let utcb = utcb::get();
        utcb.msg_tag = msg_info;
        self.0.invoke(ipcmethod::SEND, args)
    }

    pub fn recv(&self) -> usize {
        let (ret, badge) = sys_invoke_recv(self.0.bits(), ipcmethod::RECV, 0, 0, 0, 0, 0, 0, 0);
        if ret == 0 {
            badge
        } else {
            // TODO: Return Result
            0
        }
    }

    pub fn call(&self, msg_info: MsgTag, args: Args) -> usize {
        let utcb = utcb::get();
        utcb.msg_tag = msg_info;
        self.0.invoke(ipcmethod::CALL, args)
    }

    pub fn notify(&self, badge: usize) -> usize {
        self.0.invoke(ipcmethod::NOTIFY, [badge, 0, 0, 0, 0, 0, 0])
    }
}
