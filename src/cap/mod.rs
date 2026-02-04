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
use crate::error::Error;
use crate::ipc::MsgArgs;
use crate::sys::sys_invoke;
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

    pub const fn is_null(&self) -> bool {
        self.0 == 0
    }

    // --- Generic Invocation ---
    #[inline(always)]
    pub(crate) fn invoke(&self, method: usize, args: MsgArgs) -> Result<(), Error> {
        sys_invoke(self.0, method, args)
    }
}

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
    VSpace = 10,
}

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

pub const CSPACE_SLOT: CapPtr = CapPtr::from(1);
pub const VSPACE_SLOT: CapPtr = CapPtr::from(2);
pub const TCB_SLOT: CapPtr = CapPtr::from(3);
pub const MONITOR_SLOT: CapPtr = CapPtr::from(4);
pub const KERNEL_SLOT: CapPtr = CapPtr::from(5);
pub const PLATFORM_SLOT: CapPtr = CapPtr::from(6);
pub const UNTYPED_SLOT: CapPtr = CapPtr::from(7);
pub const MMIO_SLOT: CapPtr = CapPtr::from(8);
pub const IRQ_SLOT: CapPtr = CapPtr::from(9);
pub const REPLY_SLOT: CapPtr = CapPtr::from(10);

pub const CSPACE_CAP: CNode = CNode::from(CSPACE_SLOT);
pub const VSPACE_CAP: VSpace = VSpace::from(VSPACE_SLOT);
pub const TCB_CAP: TCB = TCB::from(TCB_SLOT);
pub const UNTYPED_CAP: CNode = CNode::from(UNTYPED_SLOT);
pub const MMIO_CAP: CNode = CNode::from(MMIO_SLOT);
pub const IRQ_CAP: CNode = CNode::from(IRQ_SLOT);
pub const KERNEL_CAP: Kernel = Kernel::from(KERNEL_SLOT);
pub const PLATFORM_CAP: Frame = Frame::from(PLATFORM_SLOT);
pub const MONITOR_CAP: Endpoint = Endpoint::from(MONITOR_SLOT);
pub const REPLY_CAP: Reply = Reply::from(REPLY_SLOT);
