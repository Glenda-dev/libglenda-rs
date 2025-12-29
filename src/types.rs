pub use super::caps::CapPtr;

pub const CSPACE_SLOT: usize = 0;
pub const VSPACE_SLOT: usize = 1;
pub const TCB_SLOT: usize = 2;
pub const UTCB_SLOT: usize = 3;
pub const MEM_SLOT: usize = 4;
pub const MMIO_SLOT: usize = 5;
pub const IRQ_SLOT: usize = 6;
pub const FAULT_SLOT: usize = 7;

pub mod rights {
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

pub mod ipcmethod {
    pub const SEND: usize = 1;
    pub const RECV: usize = 2;
    pub const CALL: usize = 3;
}

pub mod tcbmethod {
    pub const CONFIGURE: usize = 1;
    pub const SET_PRIORITY: usize = 2;
    pub const SET_REGISTERS: usize = 3;
    pub const RESUME: usize = 4;
    pub const SUSPEND: usize = 5;
}

pub mod pagetablemethod {
    pub const MAP: usize = 1;
    pub const UNMAP: usize = 2;
}

pub mod cnodemethod {
    pub const MINT: usize = 1;
    pub const COPY: usize = 2;
    pub const DELETE: usize = 3;
    pub const REVOKE: usize = 4;
}

pub mod untypedmethod {
    pub const RETYPE: usize = 1;
}

pub mod irqmethod {
    pub const SET_NOTIFICATION: usize = 1;
    pub const ACK: usize = 2;
    pub const CLEAR_NOTIFICATION: usize = 3;
    pub const SET_PRIORITY: usize = 4;
}

#[derive(Debug, Clone, Copy)]
pub struct MsgTag(pub usize);

impl MsgTag {
    pub const FLAG_HAS_CAP: usize = 1 << 4;

    pub fn new(label: usize, length: usize) -> Self {
        Self((label << 16) | (length & 0xF))
    }

    pub fn label(&self) -> usize {
        self.0 >> 16
    }

    pub fn length(&self) -> usize {
        self.0 & 0xF
    }

    pub fn has_cap(&self) -> bool {
        (self.0 & Self::FLAG_HAS_CAP) != 0
    }

    pub fn set_has_cap(&mut self) {
        self.0 |= Self::FLAG_HAS_CAP;
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UTCB {
    pub mrs: [usize; 7],
    pub cap_transfer: usize,
    pub recv_window: usize,
    pub tls: usize,
    pub ipc_buffer_size: usize,
}

#[derive(Debug, Clone, Copy)]
#[repr(usize)]
pub enum CapType {
    CNode = 1,
    TCB = 2,
    Endpoint = 3,
    Frame = 4,
    PageTable = 5,
}
