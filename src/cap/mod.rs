mod cnode;
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

pub const MAX_SLOTS: usize = 255;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CapPtr(usize);
pub type Args = [usize; MAX_MRS];

impl CapPtr {
    pub const fn from(slot: usize) -> Self {
        CapPtr(slot)
    }
    pub fn next(&self) -> Self {
        CapPtr(self.0 + 1)
    }
    pub fn prev(&self) -> Self {
        CapPtr(self.0 - 1)
    }
}

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
    Mmio = 10,
    VSpace = 11,
}

pub mod sizes {
    pub const TCB: usize = 1; // 4 KiB, 1 page
    pub const ENDPOINT: usize = 1; // 256 B, 1 page
    pub const REPLY: usize = 1; // 256 B, 1 page
    pub const FRAME: usize = 1; // 4 KiB, 1 page
    pub const PAGETABLE: usize = 1; // 4 KiB, 1 page
    pub const CNODE: usize = 4; // 16 KiB, 4 pages
    pub const IRQ_HANDLER: usize = 1; // 256 B, 1 page
    pub const CONSOLE: usize = 1; // 256 B, 1 page
    pub const MMIO: usize = 1; // 4 KiB, 1 page
    pub const VSPACE: usize = 1; // 4 KiB, 1 page
}

impl CapPtr {
    pub const fn null() -> Self {
        Self(0)
    }

    pub const fn new(cptr: usize) -> Self {
        Self(cptr)
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
pub const NULL_SLOT: usize = 0;
pub const CSPACE_SLOT: usize = 1;
pub const VSPACE_SLOT: usize = 2;
pub const TCB_SLOT: usize = 3;
pub const MEM_SLOT: usize = 4;
pub const FAULT_SLOT: usize = 5;
pub const CONSOLE_SLOT: usize = 6;

pub mod rights {
    pub const NONE: u8 = 0;
    pub const READ: u8 = 1 << 0;
    pub const WRITE: u8 = 1 << 1;
    pub const GRANT: u8 = 1 << 2;
    pub const SEND: u8 = 1 << 3;
    pub const RECV: u8 = 1 << 4;
    pub const CALL: u8 = 1 << 5;
    pub const ALL: u8 = 0xFF;
    pub const RW: u8 = READ | WRITE;
    pub const MASTER: u8 = ALL;
}

pub const CSPACE_CAP: CNode = CNode::from(CapPtr::from(CSPACE_SLOT));
pub const VSPACE_CAP: VSpace = VSpace::from(CapPtr::from(VSPACE_SLOT));
pub const TCB_CAP: TCB = TCB::from(CapPtr::from(TCB_SLOT));
pub const MEM_CAP: Frame = Frame::from(CapPtr::from(MEM_SLOT));
pub const FAULT_CAP: Endpoint = Endpoint::from(CapPtr::from(FAULT_SLOT));
pub const CONSOLE_CAP: Console = Console::from(CapPtr::from(CONSOLE_SLOT));
