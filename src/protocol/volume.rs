pub const GET_DEVICE: usize = 0x01;
pub const GET_INFO: usize = 0x02;
pub const PROBE_DEVICE: usize = 0x40;
pub const MOUNT_PARTITION: usize = 0x41;
pub const LIST_PARTITIONS: usize = 0x42;

#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct VolumeInfo {
    pub size: u64,
    pub block_size: u32,
    pub fs_type: [u8; 16],
}
