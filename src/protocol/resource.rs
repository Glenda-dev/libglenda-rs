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
// Init resources
pub const GET_CAP: usize = 0x30;
pub const MAP_FILE: usize = 0x31;

use num_enum::FromPrimitive;

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
pub enum InitCap {
    Kernel = 1,
    Untyped = 2,
    Mmio = 3,
    Irq = 4,
    #[num_enum(default)]
    Unknown = 0,
}
