//! Block Device Protocol

/// Get device capacity in blocks
pub const GET_CAPACITY: usize = 0x1;
/// Get block size in bytes
pub const GET_BLOCK_SIZE: usize = 0x2;
/// Read blocks
/// Args: start_sector, count
/// Ret: Data (in IPC buffer or Shared Mem)
pub const READ_BLOCKS: usize = 0x3;
/// Write blocks
/// Args: start_sector, count
pub const WRITE_BLOCKS: usize = 0x4;
/// Flush cache
pub const SYNC: usize = 0x5;
