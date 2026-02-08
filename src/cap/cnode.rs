use super::cnodemethod;
use super::{CapPtr, Rights};
use crate::error::Error;
use crate::set_mrs;
use crate::ipc::{Badge, UTCB};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CNode(CapPtr);

impl CNode {
    pub const fn from(cap: CapPtr) -> Self {
        Self(cap)
    }

    pub fn cap(&self) -> CapPtr {
        self.0
    }

    pub fn mint(
        &self,
        src: CapPtr,
        dest: CapPtr,
        badge: Badge,
        rights: Rights,
    ) -> Result<(), Error> {
        let utcb = unsafe { UTCB::get() };
        set_mrs!(utcb, src.bits(), dest.bits(), badge.bits(), rights.bits());
        self.0.invoke(cnodemethod::MINT)
    }

    pub fn copy(&self, src: CapPtr, dest: CapPtr, rights: Rights) -> Result<(), Error> {
        let utcb = unsafe { UTCB::get() };
        set_mrs!(utcb, src.bits(), dest.bits(), rights.bits());
        self.0.invoke(cnodemethod::COPY)
    }

    pub fn delete(&self, cptr: CapPtr) -> Result<(), Error> {
        let utcb = unsafe { UTCB::get() };
        set_mrs!(utcb, cptr.bits());
        self.0.invoke(cnodemethod::DELETE)
    }

    pub fn revoke(&self, cptr: CapPtr) -> Result<(), Error> {
        let utcb = unsafe { UTCB::get() };
        set_mrs!(utcb, cptr.bits());
        self.0.invoke(cnodemethod::REVOKE)
    }

    pub fn debug_print(&self) -> Result<(), Error> {
        self.0.invoke(cnodemethod::DEBUG_PRINT)
    }
}
