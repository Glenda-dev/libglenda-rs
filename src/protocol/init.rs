pub const START: usize = 1;
pub const STOP: usize = 2;
pub const RESTART: usize = 3;
pub const RELOAD: usize = 4;
pub const QUERY: usize = 5;
pub const LIST: usize = 6;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ServiceStatus {
    pub running: bool,
    pub pid: usize,
}
