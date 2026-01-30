use super::{CNode, CapPtr, CapType, untypedmethod};

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
        dirty: bool,
    ) -> usize {
        self.0.invoke(
            untypedmethod::RETYPE,
            [
                obj_type as usize,
                flags,
                dest_cnode.cap().bits(),
                dest_slot.bits(),
                dirty as usize,
                0,
                0,
            ],
        )
    }

    #[inline(always)]
    pub fn retype_untyped(&self, pages: usize, dest_cnode: CNode, dest_slot: CapPtr) -> usize {
        self.retype(CapType::Untyped, pages, dest_cnode, dest_slot, false)
    }

    #[inline(always)]
    pub fn retype_tcb(&self, dest_cnode: CNode, dest_slot: CapPtr) -> usize {
        self.retype(CapType::TCB, 0, dest_cnode, dest_slot, false)
    }

    #[inline(always)]
    pub fn retype_pagetable(&self, level: usize, dest_cnode: CNode, dest_slot: CapPtr) -> usize {
        self.retype(CapType::PageTable, level, dest_cnode, dest_slot, false)
    }

    #[inline(always)]
    pub fn retype_cnode(&self, dest_cnode: CNode, dest_slot: CapPtr) -> usize {
        self.retype(CapType::CNode, 0, dest_cnode, dest_slot, false)
    }

    #[inline(always)]
    pub fn retype_frame(&self, pages: usize, dest_cnode: CNode, dest_slot: CapPtr) -> usize {
        self.retype(CapType::Frame, pages, dest_cnode, dest_slot, false)
    }

    #[inline(always)]
    pub fn retype_vspace(&self, dest_cnode: CNode, dest_slot: CapPtr) -> usize {
        self.retype(CapType::VSpace, 0, dest_cnode, dest_slot, false)
    }

    #[inline(always)]
    pub fn retype_endpoint(&self, dest_cnode: CNode, dest_slot: CapPtr) -> usize {
        self.retype(CapType::Endpoint, 0, dest_cnode, dest_slot, false)
    }
}
