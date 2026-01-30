mod cnode;
#[cfg(feature = "kernel-console")]
mod console;
mod endpoint;
mod frame;
mod irq;
mod method;
pub mod pagetable;
mod reply;
mod tcb;
mod untyped;
mod vspace;

pub use cnode::CNode;
#[cfg(feature = "kernel-console")]
pub use console::Console;
pub use endpoint::Endpoint;
pub use frame::Frame;
pub use irq::IrqHandler;
pub use method::*;
pub use pagetable::PageTable;
pub use reply::Reply;
pub use tcb::TCB;
pub use untyped::Untyped;
pub use vspace::VSpace;

use crate::ipc::MAX_MRS;
use crate::syscall::{sys_invoke, sys_invoke_recv};
use core::fmt::Display;

pub const CNODE_BITS: usize = 8;
pub const CNODE_PAGES: usize = 4;
pub const CPTR_LEN: usize = 64;
pub const ROOT_CSPACE_SHIFT: usize = CPTR_LEN - CNODE_BITS;
pub const L1_CSPACE_SHIFT: usize = ROOT_CSPACE_SHIFT - CNODE_BITS;
pub const MAX_SLOTS: usize = (1 << CNODE_BITS) - 1;
pub const ROOT_CSPACE_GUARD: usize = CPTR_LEN;
pub const L1_CSPACE_GUARD: usize = ROOT_CSPACE_GUARD - CNODE_BITS;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CapPtr(usize);
pub type Args = [usize; MAX_MRS];

impl Display for CapPtr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
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
    Console = 9,
    MMIO = 10,
    VSpace = 11,
}

impl CapPtr {
    pub const fn null() -> Self {
        Self(0)
    }

    pub const fn new(root: usize, l1: usize) -> Self {
        Self((root << ROOT_CSPACE_SHIFT) | (l1 << L1_CSPACE_SHIFT))
    }

    pub fn bits(&self) -> usize {
        self.0
    }

    // --- Generic Invocation ---
    pub(crate) fn invoke(&self, method: usize, args: Args) -> usize {
        sys_invoke(self.0, method, args[0], args[1], args[2], args[3], args[4], args[5], args[6])
    }

    pub(crate) fn invoke_recv(&self, method: usize, args: Args) -> (usize, usize) {
        sys_invoke_recv(
            self.0, method, args[0], args[1], args[2], args[3], args[4], args[5], args[6],
        )
    }
}

// General Slots
pub const CSPACE_SLOT: usize = 1;
pub const VSPACE_SLOT: usize = 2;
pub const TCB_SLOT: usize = 3;
pub const FAULT_SLOT: usize = 4;
#[cfg(feature = "kernel-console")]
pub const CONSOLE_SLOT: usize = 5;
#[cfg(feature = "rt-bare")]
pub use crate::runtime::{IRQ_SLOT, MMIO_SLOT, PLATFORM_SLOT, UNTYPED_SLOT};

bitflags::bitflags! {
    pub struct Rights: u8 {
        const NONE  = 0;
        const READ  = 1 << 0;
        const WRITE = 1 << 1;
        const GRANT = 1 << 2;
        const SEND  = 1 << 3;
        const RECV  = 1 << 4;
        const CALL  = 1 << 5;
        const ALL   = 0xFF;
    }
}

pub const CSPACE_CAP: CNode = CNode::from(CapPtr::new(CSPACE_SLOT, 0));
pub const VSPACE_CAP: VSpace = VSpace::from(CapPtr::new(VSPACE_SLOT, 0));
pub const TCB_CAP: TCB = TCB::from(CapPtr::new(TCB_SLOT, 0));
pub const FAULT_CAP: Endpoint = Endpoint::from(CapPtr::new(FAULT_SLOT, 0));
#[cfg(feature = "kernel-console")]
pub const CONSOLE_CAP: Console = Console::from(CapPtr::new(CONSOLE_SLOT, 0));
#[cfg(feature = "rt-bare")]
pub use crate::runtime::{IRQ_CAP, MMIO_CAP, PLATFORM_CAP, UNTYPED_CAP};
