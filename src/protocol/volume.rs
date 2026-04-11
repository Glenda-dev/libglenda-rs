use crate::io::uring::{IOURING_OP_READ, IOURING_OP_SYNC, IOURING_OP_WRITE, IoUringSqe};

pub const GET_DEVICE: usize = 0x01;
pub const GET_INFO: usize = 0x02;

pub const SETUP_RING: usize = 0x10;
pub const ACQUIRE_SHM: usize = 0x11;
pub const REGISTER_SHM: usize = 0x12;

pub const PROBE_DEVICE: usize = 0x20;
pub const MOUNT_PARTITION: usize = 0x21;
pub const LIST_PARTITIONS: usize = 0x22;
pub const REPORT_STATE: usize = 0x23;

#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct VolumeInfo {
    pub size: usize,
    pub block_size: u32,
    pub fs_type: [u8; 16],
}

pub fn sqe_read(offset: usize, addr: usize, len: u32, user_data: usize) -> IoUringSqe {
    IoUringSqe { opcode: IOURING_OP_READ, off: offset, addr, len, user_data, ..Default::default() }
}

pub fn sqe_write(offset: usize, addr: usize, len: u32, user_data: usize) -> IoUringSqe {
    IoUringSqe { opcode: IOURING_OP_WRITE, off: offset, addr, len, user_data, ..Default::default() }
}

pub fn sqe_sync(user_data: usize) -> IoUringSqe {
    IoUringSqe { opcode: IOURING_OP_SYNC, user_data, ..Default::default() }
}
