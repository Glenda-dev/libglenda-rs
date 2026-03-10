//! UART Device Protocol

/// Write a byte buffer (Synchronous)
pub const WRITE: usize = 0x01;
/// Read a byte buffer (Synchronous/Polled)
pub const READ: usize = 0x02;
/// Configuration
pub const SET_BAUD_RATE: usize = 0x04;
pub const GET_CONFIG: usize = 0x05;

/// Setup io_uring (Primary IO Channel).
/// Args: sq_entries, cq_entries
/// Resp: Cap Transfer (Frame)
pub const SETUP_RING: usize = 0x10;
/// Setup shared memory buffer for IO data.
/// Args: pages (if creating) or empty (if requesting)
/// Resp: Cap Transfer (Frame)
pub const SETUP_BUFFER: usize = 0x11;
/// Notify the driver that new requests are in the SQ.
pub const NOTIFY_SQ: usize = 0x12;

/// Async notification for IO completion
pub const NOTIFY_IO: usize = 0x20;

use crate::io::uring::{IOSQE_MULTISHOT, IOURING_OP_READ, IOURING_OP_WRITE, IoUringSqe};

pub fn sqe_read(addr: usize, len: u32, user_data: usize) -> IoUringSqe {
    IoUringSqe { opcode: IOURING_OP_READ, addr, len, user_data, ..Default::default() }
}

pub fn sqe_read_multishot(addr: usize, len: u32, user_data: usize) -> IoUringSqe {
    IoUringSqe {
        opcode: IOURING_OP_READ,
        flags: IOSQE_MULTISHOT,
        addr,
        len,
        user_data,
        ..Default::default()
    }
}

pub fn sqe_write(addr: usize, len: u32, user_data: usize) -> IoUringSqe {
    IoUringSqe { opcode: IOURING_OP_WRITE, addr, len, user_data, ..Default::default() }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct UartConfig {
    pub baud_rate: u32,
    pub data_bits: u8,
    pub stop_bits: u8,
    pub parity: u8,
}
