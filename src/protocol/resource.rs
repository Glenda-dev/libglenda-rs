// Resouces allocation
pub const ALLOC: usize = 0x01;
pub const FREE: usize = 0x02;
pub const GET_RESOURCE: usize = 0x03;
pub const GET_CAP: usize = 0x04;
// VSpace management
pub const MAP: usize = 0x10;
pub const UNMAP: usize = 0x11;
pub const CLONE: usize = 0x12;
// Memory management
pub const MMAP: usize = 0x20;
pub const MUNMAP: usize = 0x21;
pub const SBRK: usize = 0x22;

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitCap {
    Console = 1,
    Initrd = 2,
    Untyped = 3,
    Mmio = 4,
    Irq = 5,
}

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitResource {
    BootArgs = 1,
    PlatformInfo = 2,
    InitrdInfo = 3,
    UntypedInfo = 4,
    MmioInfo = 5,
    IrqInfo = 6,
}
