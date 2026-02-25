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

    fn retype(&self, obj_type: CapType, flags: usize, dest: CapPtr) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, obj_type, flags, dest.bits(), 0);
        self.0.invoke(untypedmethod::RETYPE, &mut utcb)
    }

    #[inline(always)]
    pub fn retype_untyped(&self, pages: usize, dest: CapPtr) -> Result<(), Error> {
        self.retype(CapType::Untyped, pages, dest)
    }

    #[inline(always)]
    pub fn retype_tcb(&self, dest: CapPtr) -> Result<(), Error> {
        self.retype(CapType::TCB, 0, dest)
    }

    #[inline(always)]
    pub fn retype_pagetable(&self, level: usize, dest: CapPtr) -> Result<(), Error> {
        self.retype(CapType::PageTable, level, dest)
    }

    #[inline(always)]
    pub fn retype_cnode(&self, dest: CapPtr) -> Result<(), Error> {
        self.retype(CapType::CNode, 0, dest)
    }

    #[inline(always)]
    pub fn retype_frame(&self, pages: usize, dest: CapPtr) -> Result<(), Error> {
        self.retype(CapType::Frame, pages, dest)
    }

    #[inline(always)]
    pub fn retype_vspace(&self, dest: CapPtr) -> Result<(), Error> {
        self.retype(CapType::VSpace, 0, dest)
    }

    #[inline(always)]
    pub fn retype_endpoint(&self, dest: CapPtr) -> Result<(), Error> {
        self.retype(CapType::Endpoint, 0, dest)
    }

    pub fn merge(&self, other: &Untyped) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, other.0.bits(), 0);
        self.0.invoke(untypedmethod::MERGE, &mut utcb)
    }
}
