mod cnode;
mod console;
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
pub use console::Console;
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
use crate::ipc::UTCB;
use crate::sys::sys_invoke;
use core::fmt::{Debug, Display};
use num_enum::FromPrimitive;

const SLOT_SIZE: usize = 48; // 每个 Slot 占用 48 字节
pub const CNODE_BITS: usize = 8;
pub const CNODE_SIZE: usize = SLOT_SIZE * (1 << CNODE_BITS) + 8;
pub const CNODE_PAGES: usize = (CNODE_SIZE + PGSIZE - 1) / PGSIZE;
pub const CNODE_SLOTS: usize = 1 << CNODE_BITS;
pub const CNODE_MASK: usize = CNODE_SLOTS - 1;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
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
    pub fn invoke(&self, method: usize, utcb: &mut UTCB) -> Result<(), Error> {
        sys_invoke(self.0, method, utcb)
    }

    pub const fn len(&self) -> usize {
        if self.0 == 0 {
            return 0; // NULL
        } else if self.0 >> (CNODE_BITS * 1) == 0 {
            return 1; // L0
        } else if self.0 >> (CNODE_BITS * 2) == 0 {
            return 2; // L1
        } else if self.0 >> (CNODE_BITS * 3) == 0 {
            return 3; // L2
        } else if self.0 >> (CNODE_BITS * 4) == 0 {
            return 4; // L3
        } else if self.0 >> (CNODE_BITS * 5) == 0 {
            return 5; // L4
        } else if self.0 >> (CNODE_BITS * 6) == 0 {
            return 6; // L5
        } else if self.0 >> (CNODE_BITS * 7) == 0 {
            return 7; // L6
        } else {
            return 8; // L7
        }
    }

    pub const fn concat(root: CapPtr, ptr: CapPtr) -> CapPtr {
        let root_len = root.len();
        let ptr_len = ptr.len();
        if root_len + ptr_len > 8 || root.is_null() || ptr.is_null() {
            return CapPtr::null();
        }
        CapPtr::from(root.0 | ptr.0 << (root_len * CNODE_BITS))
    }
}

impl Display for CapPtr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

impl Debug for CapPtr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
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
    Console = 11,
    #[num_enum(default)]
    Unknown = 255,
}

impl Into<usize> for CapType {
    fn into(self) -> usize {
        self as usize
    }
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
        const CUSTOM = 1 << 7;
        const ALL   = 0xFF;
    }
}

pub const CSPACE_SLOT: CapPtr = CapPtr::from(1);
pub const VSPACE_SLOT: CapPtr = CapPtr::from(2);
pub const TCB_SLOT: CapPtr = CapPtr::from(3);
pub const MONITOR_SLOT: CapPtr = CapPtr::from(4);
pub const CONSOLE_SLOT: CapPtr = CapPtr::from(5);
pub const REPLY_SLOT: CapPtr = CapPtr::from(6);
pub const RECV_SLOT: CapPtr = CapPtr::from(7);
pub const ENDPOINT_SLOT: CapPtr = CapPtr::from(8);

pub const CSPACE_CAP: CNode = CNode::from(CSPACE_SLOT);
pub const VSPACE_CAP: VSpace = VSpace::from(VSPACE_SLOT);
pub const TCB_CAP: TCB = TCB::from(TCB_SLOT);
pub const MONITOR_CAP: Endpoint = Endpoint::from(MONITOR_SLOT);
pub const CONSOLE_CAP: Console = Console::from(CONSOLE_SLOT);
pub const REPLY_CAP: Reply = Reply::from(REPLY_SLOT);
pub const ENDPOINT_CAP: Endpoint = Endpoint::from(ENDPOINT_SLOT);
