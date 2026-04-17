// Resouces allocation
pub const ALLOC: usize = 0x01;
pub const FREE: usize = 0x02;
pub const DMA_ALLOC: usize = 0x03;
// VSpace management
pub const MAP: usize = 0x10;
pub const UNMAP: usize = 0x11;
pub const CLONE: usize = 0x12;
// Memory management
pub const SBRK: usize = 0x22;
// Resources
pub const GET_CAP: usize = 0x30;
pub const REGISTER_CAP: usize = 0x31;
pub const GET_CONFIG: usize = 0x32;
pub const GET_STATUS: usize = 0x33;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct MemoryStatus {
    pub available_bytes: usize,
    pub total_bytes: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct WarrenStatus {
    pub memory: MemoryStatus,
}

use num_enum::FromPrimitive;
#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
pub enum ResourceType {
    Kernel = 1,
    Untyped = 2,
    Bootinfo = 3,
    Console = 4,
    Irq = 5,
    Endpoint = 6,
    IrqControl = 7,
    #[num_enum(default)]
    Unknown = 0,
}

pub const PROCESS_ENDPOINT: usize = 0;
pub const RESOURCE_ENDPOINT: usize = 1;
pub const INIT_ENDPOINT: usize = 2;
pub const DEVICE_ENDPOINT: usize = 3;
pub const NET_ENDPOINT: usize = 4;
pub const FS_ENDPOINT: usize = 5;
pub const VOLUME_ENDPOINT: usize = 6;
pub const TIME_ENDPOINT: usize = 7;
pub const VT_ENDPOINT: usize = 8;
pub const APE_ENDPOINT: usize = 9;
pub const CHIMERA_ENDPOINT: usize = 10;
pub const FACTOTUM_ENDPOINT: usize = 11;
