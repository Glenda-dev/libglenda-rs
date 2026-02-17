use super::cnodemethod;
use super::{CapPtr, Rights};
use crate::error::Error;
use crate::ipc::{Badge, UTCB};
use crate::set_mrs;

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
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, src.bits(), dest.bits(), badge.bits(), rights.bits());
        self.0.invoke(cnodemethod::MINT, &mut utcb)
    }

    pub fn copy(&self, src: CapPtr, dest: CapPtr, rights: Rights) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, src.bits(), dest.bits(), rights.bits());
        self.0.invoke(cnodemethod::COPY, &mut utcb)
    }

    pub fn delete(&self, cptr: CapPtr) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, cptr.bits());
        self.0.invoke(cnodemethod::DELETE, &mut utcb)
    }

    pub fn revoke(&self, cptr: CapPtr) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, cptr.bits());
        self.0.invoke(cnodemethod::REVOKE, &mut utcb)
    }

    pub fn move_cap(&self, src: CapPtr, dest: CapPtr) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, src.bits(), dest.bits());
        self.0.invoke(cnodemethod::MOVE, &mut utcb)
    }

    pub fn debug_print(&self) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        self.0.invoke(cnodemethod::DEBUG_PRINT, &mut utcb)
    }

    pub fn recycle(&self, cptr: CapPtr) -> Result<usize, Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, cptr.bits());
        self.0.invoke(cnodemethod::RECYCLE, &mut utcb)?;
        Ok(utcb.get_mr(0))
    }
}
