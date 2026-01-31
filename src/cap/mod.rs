mod cnode;
mod endpoint;
mod frame;
mod irq;
mod kernel;
mod method;
pub mod pagetable;
mod reply;
mod tcb;
mod untyped;
mod vspace;

pub use cnode::CNode;
pub use endpoint::Endpoint;
pub use frame::Frame;
pub use irq::IrqHandler;
pub use kernel::Kernel;
pub use method::*;
pub use pagetable::PageTable;
pub use reply::Reply;
pub use tcb::TCB;
pub use untyped::Untyped;
pub use vspace::VSpace;

use crate::arch::mem::PGSIZE;
use crate::ipc::MAX_MRS;
use crate::syscall::sys_invoke;
use core::fmt::Display;

const SLOT_SIZE: usize = 48; // 每个 Slot 占用 48 字节
pub const CNODE_BITS: usize = 8;
pub const CNODE_SIZE: usize = SLOT_SIZE * (1 << CNODE_BITS) + 8;
pub const CNODE_PAGES: usize = (CNODE_SIZE + PGSIZE - 1) / PGSIZE;
pub const CNODE_SLOTS: usize = 1 << CNODE_BITS;
pub const CNODE_MASK: usize = CNODE_SLOTS - 1;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CapPtr(usize);
pub type Args = [usize; MAX_MRS];

impl Display for CapPtr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(usize)]
pub enum CapType {
    Empty = 0,
    Untyped = 1,
    TCB = 2,
    Endpoint = 3,
    Reply = 4,
    Frame = 5,
    PageTable = 6,
    CNode = 7,
    IrqHandler = 8,
    Kernel = 9,
    MMIO = 10,
    VSpace = 11,
}

impl CapPtr {
    pub const fn null() -> Self {
        Self(0)
    }

    pub const fn from(slot: usize) -> Self {
        Self(slot)
    }

    pub fn bits(&self) -> usize {
        self.0
    }

    // --- Generic Invocation ---
    pub(crate) fn invoke(&self, method: usize, args: Args) -> usize {
        sys_invoke(self.0, method, args[0], args[1], args[2], args[3], args[4], args[5], args[6])
    }
}

// General Slots
pub const CSPACE_SLOT: CapPtr = CapPtr::from(1);
pub const VSPACE_SLOT: CapPtr = CapPtr::from(2);
pub const TCB_SLOT: CapPtr = CapPtr::from(3);

bitflags::bitflags! {
    pub struct Rights: u8 {
        const NONE  = 0;
        const READ  = 1 << 0;
        const WRITE = 1 << 1;
        const GRANT = 1 << 2;
        const SEND  = 1 << 3;
        const RECV  = 1 << 4;
        const CALL  = 1 << 5;
        const EXECUTE = 1 << 6; // 允许执行 (仅用于 TCB)
        const ALL   = 0xFF;
    }
}

pub const CSPACE_CAP: CNode = CNode::from(CSPACE_SLOT);
pub const VSPACE_CAP: VSpace = VSpace::from(VSPACE_SLOT);
pub const TCB_CAP: TCB = TCB::from(TCB_SLOT);
