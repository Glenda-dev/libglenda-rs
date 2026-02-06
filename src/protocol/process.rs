// Protocol ID

// Process Lifecycle
pub const SPAWN: usize = 0x01;
pub const EXIT: usize = 0x02;
pub const WAIT: usize = 0x03;
pub const KILL: usize = 0x04;
pub const FORK: usize = 0x05;
// Thread Control
pub const THREAD_CREATE: usize = 0x10;
pub const THREAD_EXIT: usize = 0x11;
pub const THREAD_JOIN: usize = 0x12;
pub const FUTEX_WAIT: usize = 0x13;
pub const FUTEX_WAKE: usize = 0x14;
pub const YIELD: usize = 0x15;
pub const SLEEP: usize = 0x16;

// Debugging & Inspection
pub const GET_PID: usize = 0x20;
pub const PS: usize = 0x21;
