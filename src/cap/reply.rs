use super::{CapPtr, replymethod};
use crate::error::Error;
use crate::ipc::UTCB;

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

    pub fn reply(&self, utcb: &mut UTCB) -> Result<(), Error> {
        self.0.invoke_ipc(replymethod::REPLY, utcb)
    }
}
