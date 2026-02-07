use super::{CNode, CapPtr, CapType, untypedmethod};
use crate::error::Error;
use crate::ipc::UTCB;

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
        dest_cnode: CNode,
        dest_slot: CapPtr,
    ) -> Result<(), Error> {
        let utcb = unsafe { UTCB::get() };
        utcb.mrs_regs[0] = obj_type as usize;
        utcb.mrs_regs[1] = flags;
        utcb.mrs_regs[2] = dest_cnode.cap().bits();
        utcb.mrs_regs[3] = dest_slot.bits();
        self.0.invoke(untypedmethod::RETYPE)
    }

    #[inline(always)]
    pub fn retype_untyped(
        &self,
        pages: usize,
        dest_cnode: CNode,
        dest_slot: CapPtr,
    ) -> Result<(), Error> {
        self.retype(CapType::Untyped, pages, dest_cnode, dest_slot)
    }

    #[inline(always)]
    pub fn retype_tcb(&self, dest_cnode: CNode, dest_slot: CapPtr) -> Result<(), Error> {
        self.retype(CapType::TCB, 0, dest_cnode, dest_slot)
    }

    #[inline(always)]
    pub fn retype_pagetable(
        &self,
        level: usize,
        dest_cnode: CNode,
        dest_slot: CapPtr,
    ) -> Result<(), Error> {
        self.retype(CapType::PageTable, level, dest_cnode, dest_slot)
    }

    #[inline(always)]
    pub fn retype_cnode(&self, dest_cnode: CNode, dest_slot: CapPtr) -> Result<(), Error> {
        self.retype(CapType::CNode, 0, dest_cnode, dest_slot)
    }

    #[inline(always)]
    pub fn retype_frame(
        &self,
        pages: usize,
        dest_cnode: CNode,
        dest_slot: CapPtr,
    ) -> Result<(), Error> {
        self.retype(CapType::Frame, pages, dest_cnode, dest_slot)
    }

    #[inline(always)]
    pub fn retype_vspace(&self, dest_cnode: CNode, dest_slot: CapPtr) -> Result<(), Error> {
        self.retype(CapType::VSpace, 0, dest_cnode, dest_slot)
    }

    #[inline(always)]
    pub fn retype_endpoint(&self, dest_cnode: CNode, dest_slot: CapPtr) -> Result<(), Error> {
        self.retype(CapType::Endpoint, 0, dest_cnode, dest_slot)
    }
}
