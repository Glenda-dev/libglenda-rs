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

    pub fn retype(
        &self,
        obj_type: CapType,
        pages: usize,
        n_objs: usize,
        dest_cnode: CNode,
        dest_slot: usize,
        dirty: bool,
    ) -> usize {
        self.0.invoke(
            untypedmethod::RETYPE,
            [obj_type as usize, pages, n_objs, dest_cnode.0.bits(), dest_slot, dirty as usize, 0],
        )
    }
}
