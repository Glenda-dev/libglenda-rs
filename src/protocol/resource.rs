// Resouces allocation
pub const ALLOC: usize = 0x01;
pub const FREE: usize = 0x02;
// VSpace management
pub const MAP: usize = 0x10;
pub const UNMAP: usize = 0x11;
pub const CLONE: usize = 0x12;
// Memory management
pub const MMAP: usize = 0x20;
pub const MUNMAP: usize = 0x21;
pub const SBRK: usize = 0x22;
// Resources
pub const GET_CAP: usize = 0x30;
pub const REGISTER_CAP: usize = 0x31;
pub const GET_CONFIG: usize = 0x32;

use num_enum::FromPrimitive;
#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
pub enum ResourceType {
    Kernel = 1,
    Untyped = 2,
    Bootinfo = 3,
    Mmio = 4,
    Irq = 5,
    Platform = 6,
    Endpoint = 7,
    #[num_enum(default)]
    Unknown = 0,
}

pub const PROCESS_ENDPOINT: usize = 0;
pub const RESOURCE_ENDPOINT: usize = 1;
pub const INIT_ENDPOINT: usize = 2;
pub const DEVICE_ENDPOINT: usize = 3;
pub const NET_ENDPOINT: usize = 4;
pub const FS_ENDPOINT: usize = 5;
