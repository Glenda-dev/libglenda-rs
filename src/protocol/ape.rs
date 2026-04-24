// APE Subsystem IPC Methods (APE_PROTO: 0x0D00)

// Process Management
pub const GET_BOOTSTRAP_STATE: usize = 0x00;
pub const YIELD: usize = 0x01;
pub const CLONE_PROCESS: usize = 0x02;
pub const EXIT_PROCESS: usize = 0x03;
pub const WAIT_PROCESS: usize = 0x04;
pub const DELIVER_SIGNAL: usize = 0x05;
pub const EXECVE: usize = 0x06;

// Shared State
pub const SET_PGID: usize = 0x10;
pub const GET_SID: usize = 0x11;
pub const SET_SID: usize = 0x12;
pub const GET_PGID: usize = 0x13;

// Identity
pub const SET_IDENTITY: usize = 0x20;
pub const GET_IDENTITY: usize = 0x21;
