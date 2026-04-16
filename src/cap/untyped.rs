use super::{CapPtr, CapType, untypedmethod};
use crate::error::Error;
use crate::ipc::UTCB;
use crate::set_mrs;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Untyped(CapPtr);

impl Untyped {
    pub const fn from(cap: CapPtr) -> Self {
        Self(cap)
    }

    pub fn cap(&self) -> CapPtr {
        self.0
    }

    fn retype(
        &self,
        obj_type: CapType,
        flags: usize,
        dest_cnode: CapPtr,
        dest_slot: CapPtr,
    ) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, obj_type, flags, dest_cnode.bits(), dest_slot.bits());
        self.0.invoke(untypedmethod::RETYPE, &mut utcb)
    }

    #[inline(always)]
    pub fn retype_untyped(
        &self,
        pages: usize,
        dest_cnode: CapPtr,
        dest_slot: CapPtr,
    ) -> Result<(), Error> {
        self.retype(CapType::Untyped, pages, dest_cnode, dest_slot)
    }

    #[inline(always)]
    pub fn retype_tcb(&self, dest_cnode: CapPtr, dest_slot: CapPtr) -> Result<(), Error> {
        self.retype(CapType::TCB, 0, dest_cnode, dest_slot)
    }

    #[inline(always)]
    pub fn retype_pagetable(
        &self,
        level: usize,
        dest_cnode: CapPtr,
        dest_slot: CapPtr,
    ) -> Result<(), Error> {
        self.retype(CapType::PageTable, level, dest_cnode, dest_slot)
    }

    #[inline(always)]
    pub fn retype_cnode(&self, dest_cnode: CapPtr, dest_slot: CapPtr) -> Result<(), Error> {
        self.retype(CapType::CNode, 0, dest_cnode, dest_slot)
    }

    #[inline(always)]
    pub fn retype_page(
        &self,
        level: usize,
        dest_cnode: CapPtr,
        dest_slot: CapPtr,
    ) -> Result<(), Error> {
        self.retype(CapType::Page, level, dest_cnode, dest_slot)
    }

    #[inline(always)]
    pub fn retype_vspace(&self, dest_cnode: CapPtr, dest_slot: CapPtr) -> Result<(), Error> {
        self.retype(CapType::VSpace, 0, dest_cnode, dest_slot)
    }

    #[inline(always)]
    pub fn retype_endpoint(&self, dest_cnode: CapPtr, dest_slot: CapPtr) -> Result<(), Error> {
        self.retype(CapType::Endpoint, 0, dest_cnode, dest_slot)
    }

    pub fn get_info(&self) -> Result<(usize, usize), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        self.0.invoke(untypedmethod::GET_INFO, &mut utcb)?;
        Ok((utcb.get_mr(0), utcb.get_mr(1)))
    }

    pub fn recycle(&self) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        self.0.invoke(untypedmethod::RECYCLE, &mut utcb)
    }
}
