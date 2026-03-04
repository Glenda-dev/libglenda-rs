// Protocol ID

// Process Lifecycle
pub const SPAWN: usize = 0x01;
pub const EXIT: usize = 0x02;
pub const KILL: usize = 0x03;
// Thread Control
pub const THREAD_CREATE: usize = 0x10;
pub const THREAD_EXIT: usize = 0x11;
pub const THREAD_JOIN: usize = 0x12;
pub const YIELD: usize = 0x15;
pub const SLEEP: usize = 0x16;
// Scheduling & Synchronization
pub const WAIT: usize = 0x20;
pub const WAKE: usize = 0x21;
// Debugging & Inspection
pub const GET_PID: usize = 0x30;
pub const GET_PPID: usize = 0x31;
pub const GET_CNODE: usize = 0x32;
