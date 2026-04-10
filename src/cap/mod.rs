mod cnode;
mod console;
mod endpoint;
mod frame;
mod irq;
mod kernel;
mod method;
mod pagetable;
mod reply;
mod tcb;
mod untyped;
mod virt;
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
pub use virt::{VCPU, VMSpace};
pub use vspace::VSpace;

use crate::arch::mem::PGSIZE;
use crate::error::Error;
use crate::ipc::UTCB;
use crate::sys::sys_invoke;
use core::fmt::{Debug, Display};
use num_enum::FromPrimitive;

const SLOT_SIZE: usize = 64; // 每个 Slot 占用 64 字节（与内核 CNode 布局保持一致）
pub const CNODE_BITS: usize = 8;
pub const CNODE_SIZE: usize = SLOT_SIZE * (1 << CNODE_BITS);
pub const CNODE_PAGES: usize = (CNODE_SIZE + PGSIZE - 1) / PGSIZE;
pub const CNODE_SLOTS: usize = 1 << CNODE_BITS;
pub const CNODE_MASK: usize = CNODE_SLOTS - 1;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

    pub fn level_idx(&self, level: usize) -> usize {
        (self.0 >> (level * CNODE_BITS)) & CNODE_MASK
    }

    // --- Generic Invocation ---
    #[inline(always)]
    pub fn invoke(&self, method: usize, utcb: &mut UTCB) -> Result<(), Error> {
        sys_invoke(self.0, method, utcb)
    }
    pub const fn len(&self) -> usize {
        self.effective_bits() / CNODE_BITS
    }

    pub const fn concat(root: CapPtr, ptr: CapPtr) -> CapPtr {
        if root.0 == 0 {
            return ptr;
        }
        if ptr.0 == 0 {
            return root;
        }
        let root_bits = root.effective_bits();
        CapPtr(root.0 | (ptr.0 << root_bits))
    }

    pub const fn effective_bits(&self) -> usize {
        if self.0 == 0 {
            return 0;
        }
        if self.0 <= 0xFF {
            return 8;
        }
        if self.0 <= 0xFFFF {
            return 16;
        }
        if self.0 <= 0xFF_FFFF {
            return 24;
        }
        if self.0 <= 0xFFFF_FFFF {
            return 32;
        }
        #[cfg(target_pointer_width = "64")]
        if self.0 <= 0xFF_FFFF_FFFF {
            return 40;
        }
        #[cfg(target_pointer_width = "64")]
        if self.0 <= 0xFFFF_FFFF_FFFF {
            return 48;
        }
        #[cfg(target_pointer_width = "64")]
        if self.0 <= 0xFF_FFFF_FFFF_FFFF {
            return 56;
        }
        #[cfg(target_pointer_width = "64")]
        {
            return 64;
        }
        #[cfg(target_pointer_width = "32")]
        {
            return 32;
        }
    }

    pub const fn relative(root: CapPtr, abs: CapPtr) -> CapPtr {
        if root.0 == CSPACE_SLOT.0 {
            return abs;
        }
        if abs.0 == 0 {
            return CapPtr::null();
        }
        let root_bits = root.effective_bits();
        if abs.0 % (1 << root_bits) != root.0 {
            return CapPtr::null(); // Not a descendant
        }
        CapPtr(abs.0 >> root_bits)
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
    VCPU = 12,
    VMSpace = 13,
    #[num_enum(default)]
    Unknown = 255,
}

impl Into<usize> for CapType {
    fn into(self) -> usize {
        self as usize
    }
}

impl CapType {
    pub fn pages(&self, flags: usize) -> Result<usize, Error> {
        let pages = match self {
            CapType::Untyped => flags, // 由 flags 决定
            CapType::TCB => 1,
            CapType::Endpoint => 1,
            CapType::Reply => 1,
            CapType::Frame => flags, // 由 flags 决定
            CapType::PageTable => 1,
            CapType::CNode => CNODE_PAGES,
            CapType::VSpace => 1,
            CapType::VCPU => 1,
            CapType::VMSpace => 1,
            _ => 0,
        };
        if pages == 0 { Err(Error::InvalidArgs) } else { Ok(pages) }
    }
}

bitflags::bitflags! {
    #[derive(Copy, Clone, Debug)]
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
pub const ARENA_CSPACE_SLOT: CapPtr = CapPtr::from(10);

pub const CSPACE_CAP: CNode = CNode::from(CSPACE_SLOT);
pub const VSPACE_CAP: VSpace = VSpace::from(VSPACE_SLOT);
pub const TCB_CAP: TCB = TCB::from(TCB_SLOT);
pub const MONITOR_CAP: Endpoint = Endpoint::from(MONITOR_SLOT);
pub const CONSOLE_CAP: Console = Console::from(CONSOLE_SLOT);
pub const REPLY_CAP: Reply = Reply::from(REPLY_SLOT);
pub const ENDPOINT_CAP: Endpoint = Endpoint::from(ENDPOINT_SLOT);
