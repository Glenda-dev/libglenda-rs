use super::{CapPtr, replymethod};
use crate::error::Error;
use crate::ipc::utcb;
use crate::ipc::{MsgArgs, MsgTag};

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

    pub fn reply(&self, msg_info: MsgTag, args: MsgArgs) -> Result<(), Error> {
        let utcb = unsafe { utcb::get() };
        utcb.msg_tag = msg_info;
        self.0.invoke(replymethod::REPLY, args)
    }
}
