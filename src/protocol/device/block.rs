//! Block Device Protocol

/// Get device capacity in blocks
pub const GET_CAPACITY: usize = 1;
/// Get block size in bytes
pub const GET_BLOCK_SIZE: usize = 2;
/// Read blocks
/// Args: start_sector, count
/// Ret: Data (in IPC buffer or Shared Mem)
pub const READ_BLOCKS: usize = 3;
/// Write blocks
/// Args: start_sector, count
pub const WRITE_BLOCKS: usize = 4;
/// Flush cache
pub const SYNC: usize = 5;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct BlockRequest {
    pub sector: u64,
    pub count: u32,
    pub flags: u32,
}
