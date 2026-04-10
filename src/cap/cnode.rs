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

    pub const fn cap(&self) -> CapPtr {
        self.0
    }

    #[inline(always)]
    fn normalize_dest_cnode(&self, dest_cnode: CapPtr) -> CapPtr {
        if dest_cnode == self.0 { CapPtr::null() } else { dest_cnode }
    }

    pub fn mint(
        &self,
        src: CapPtr,
        dest_cnode: CapPtr,
        dest_slot: CapPtr,
        badge: Badge,
        rights: Rights,
    ) -> Result<(), Error> {
        let dest_cnode = self.normalize_dest_cnode(dest_cnode);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(
            utcb,
            src.bits(),
            dest_cnode.bits(),
            dest_slot.bits(),
            badge.bits(),
            rights.bits()
        );
        self.0.invoke(cnodemethod::MINT, &mut utcb)
    }

    #[inline(always)]
    pub fn mint_self(
        &self,
        src: CapPtr,
        dest_slot: CapPtr,
        badge: Badge,
        rights: Rights,
    ) -> Result<(), Error> {
        self.mint(src, CapPtr::null(), dest_slot, badge, rights)
    }

    pub fn copy(
        &self,
        src: CapPtr,
        dest_cnode: CapPtr,
        dest_slot: CapPtr,
        rights: Rights,
    ) -> Result<(), Error> {
        let dest_cnode = self.normalize_dest_cnode(dest_cnode);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, src.bits(), dest_cnode.bits(), dest_slot.bits(), rights.bits());
        self.0.invoke(cnodemethod::COPY, &mut utcb)
    }

    #[inline(always)]
    pub fn copy_self(&self, src: CapPtr, dest_slot: CapPtr, rights: Rights) -> Result<(), Error> {
        self.copy(src, CapPtr::null(), dest_slot, rights)
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

    pub fn transfer(
        &self,
        src: CapPtr,
        dest_cnode: CapPtr,
        dest_slot: CapPtr,
    ) -> Result<(), Error> {
        let dest_cnode = self.normalize_dest_cnode(dest_cnode);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, src.bits(), dest_cnode.bits(), dest_slot.bits());
        self.0.invoke(cnodemethod::TRANSFER, &mut utcb)
    }

    #[inline(always)]
    pub fn transfer_self(&self, src: CapPtr, dest_slot: CapPtr) -> Result<(), Error> {
        self.transfer(src, CapPtr::null(), dest_slot)
    }

    pub fn debug_print(&self) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        self.0.invoke(cnodemethod::DEBUG_PRINT, &mut utcb)
    }
}
