pub const START: usize = 1;
pub const STOP: usize = 2;
pub const RESTART: usize = 3;
pub const RELOAD: usize = 4;
pub const QUERY: usize = 5;
pub const LIST: usize = 6;
pub const GET_CAP: usize = 7;
pub const GET_RESOURCE: usize = 8;

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

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ServiceStatus {
    pub running: bool,
    pub pid: usize,
}
