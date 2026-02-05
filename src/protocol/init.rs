pub const SERVICE_START: usize = 1;
pub const SERVICE_STOP: usize = 2;
pub const SERVICE_RESTART: usize = 3;
pub const SERVICE_RELOAD: usize = 4;
pub const SERVICE_QUERY: usize = 5;
pub const GET_CAP: usize = 6;
pub const GET_RESOURCE: usize = 7;

#[repr(usize)]
pub enum InitCap {
    Console = 1,
    Initrd = 2,
    Untyped = 3,
    Mmio = 4,
    Irq = 5,
}

#[repr(usize)]
pub enum InitResource {
    BootArgs = 1,
    PlatformInfo = 2,
}
