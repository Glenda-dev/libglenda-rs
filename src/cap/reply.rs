use super::{CapPtr, replymethod};
use crate::error::Error;
use crate::ipc::{MsgTag, UTCB};

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

    pub fn reply(&self, info: MsgTag) -> Result<(), Error> {
        let utcb = unsafe { UTCB::get() };
        utcb.msg_tag = info;
        self.0.invoke(replymethod::REPLY)
    }
}
