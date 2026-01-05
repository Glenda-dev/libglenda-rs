use super::{CapPtr, cnodemethod};

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

    pub fn mint(&self, src: CapPtr, dest_slot: usize, badge: usize, rights: u8) -> usize {
        self.0.invoke(cnodemethod::MINT, [src.bits(), dest_slot, badge, rights as usize, 0, 0, 0])
    }

    pub fn copy(&self, src: CapPtr, dest_slot: usize, rights: u8) -> usize {
        self.0.invoke(cnodemethod::COPY, [src.bits(), dest_slot, rights as usize, 0, 0, 0, 0])
    }

    pub fn delete(&self, slot: usize) -> usize {
        self.0.invoke(cnodemethod::DELETE, [slot, 0, 0, 0, 0, 0, 0])
    }

    pub fn revoke(&self, slot: usize) -> usize {
        self.0.invoke(cnodemethod::REVOKE, [slot, 0, 0, 0, 0, 0, 0])
    }

    pub fn debug_print(&self) -> usize {
        self.0.invoke(cnodemethod::DEBUG_PRINT, [0, 0, 0, 0, 0, 0, 0])
    }
}
