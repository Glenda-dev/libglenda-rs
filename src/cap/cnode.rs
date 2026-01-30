use super::{CapPtr, Rights, cnodemethod};

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

    pub fn mint(&self, src: CapPtr, dest: CapPtr, badge: usize, rights: Rights) -> usize {
        self.0.invoke(
            cnodemethod::MINT,
            [src.bits(), dest.bits(), badge, rights.bits() as usize, 0, 0, 0],
        )
    }

    pub fn copy(&self, src: CapPtr, dest: CapPtr, rights: Rights) -> usize {
        self.0.invoke(
            cnodemethod::COPY,
            [src.bits(), dest.bits(), rights.bits() as usize, 0, 0, 0, 0],
        )
    }

    pub fn delete(&self, cptr: CapPtr) -> usize {
        self.0.invoke(cnodemethod::DELETE, [cptr.bits(), 0, 0, 0, 0, 0, 0])
    }

    pub fn revoke(&self, cptr: CapPtr) -> usize {
        self.0.invoke(cnodemethod::REVOKE, [cptr.bits(), 0, 0, 0, 0, 0, 0])
    }

    pub fn debug_print(&self) -> usize {
        self.0.invoke(cnodemethod::DEBUG_PRINT, [0, 0, 0, 0, 0, 0, 0])
    }
}
