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
        n_objs: usize,
        dest_cnode: CNode,
        dest_slot: CapPtr,
        dirty: bool,
    ) -> usize {
        self.0.invoke(
            untypedmethod::RETYPE,
            [
                obj_type as usize,
                flags,
                n_objs,
                dest_cnode.cap().bits(),
                dest_slot.bits(),
                dirty as usize,
                0,
            ],
        )
    }

    #[inline(always)]
    pub fn retype_untyped(
        &self,
        pages: usize,
        n_objs: usize,
        dest_cnode: CNode,
        dest_slot: CapPtr,
    ) -> usize {
        self.retype(CapType::Untyped, pages, n_objs, dest_cnode, dest_slot, false)
    }

    #[inline(always)]
    pub fn retype_tcb(&self, n_objs: usize, dest_cnode: CNode, dest_slot: CapPtr) -> usize {
        self.retype(CapType::TCB, 0, n_objs, dest_cnode, dest_slot, false)
    }

    #[inline(always)]
    pub fn retype_pagetable(
        &self,
        level: usize,
        n_objs: usize,
        dest_cnode: CNode,
        dest_slot: CapPtr,
    ) -> usize {
        self.retype(CapType::PageTable, level, n_objs, dest_cnode, dest_slot, false)
    }

    #[inline(always)]
    pub fn retype_cnode(&self, n_objs: usize, dest_cnode: CNode, dest_slot: CapPtr) -> usize {
        self.retype(CapType::CNode, 0, n_objs, dest_cnode, dest_slot, false)
    }

    #[inline(always)]
    pub fn retype_frame(
        &self,
        pages: usize,
        n_objs: usize,
        dest_cnode: CNode,
        dest_slot: CapPtr,
    ) -> usize {
        self.retype(CapType::Frame, pages, n_objs, dest_cnode, dest_slot, false)
    }

    #[inline(always)]
    pub fn retype_vspace(&self, n_objs: usize, dest_cnode: CNode, dest_slot: CapPtr) -> usize {
        self.retype(CapType::VSpace, 0, n_objs, dest_cnode, dest_slot, false)
    }

    #[inline(always)]
    pub fn retype_endpoint(&self, n_objs: usize, dest_cnode: CNode, dest_slot: CapPtr) -> usize {
        self.retype(CapType::Endpoint, 0, n_objs, dest_cnode, dest_slot, false)
    }
}
