use super::{Args, CapPtr, replymethod};
use crate::ipc::{MsgTag, utcb};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reply(CapPtr);

impl Reply {
    pub const fn from(cap: CapPtr) -> Self {
        Self(cap)
    }

    pub fn cap(&self) -> CapPtr {
        self.0
    }

    pub fn reply(&self, msg_info: MsgTag, args: Args) -> usize {
        let utcb = utcb::get();
        utcb.msg_tag = msg_info;
        self.0.invoke(replymethod::REPLY, args)
    }
}
