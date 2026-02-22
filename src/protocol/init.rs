use alloc::string::String;
use num_enum::FromPrimitive;
use serde::{Deserialize, Serialize};

pub const START: usize = 0x01;
pub const STOP: usize = 0x02;
pub const RESTART: usize = 0x03;
pub const RELOAD: usize = 0x04;
pub const QUERY: usize = 0x05;
pub const LIST: usize = 0x06;
pub const REPORT: usize = 0x10;

#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub name: String,
    pub running: ServiceState,
    pub pid: usize,
}

impl ServiceStatus {
    pub fn new(name: String, pid: usize) -> Self {
        Self { name, running: ServiceState::Starting, pid }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, FromPrimitive, PartialEq, Eq)]
#[repr(usize)]
pub enum ServiceState {
    Starting = 1,
    Running = 2,
    Stopped = 3,
    Exited = 4,
    #[num_enum(default)]
    Failed = 0,
}
