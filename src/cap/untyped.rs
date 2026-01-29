use super::sizes;
use super::{CNode, CapPtr, CapType, untypedmethod};
use crate::error::code;

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
        size: usize,
        n_objs: usize,
        dest_cnode: CNode,
        dest_slot: CapPtr,
        dirty: bool,
    ) -> usize {
        self.0.invoke(
            untypedmethod::RETYPE,
            [
                obj_type as usize,
                size,
                n_objs,
                dest_cnode.cap().bits(),
                dest_slot.bits(),
                dirty as usize,
                0,
            ],
        )
    }
}
