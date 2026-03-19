// Protocol ID

// Process Lifecycle
pub const CREATE: usize = 0x01;
pub const SPAWN: usize = 0x02;
pub const EXIT: usize = 0x03;
pub const KILL: usize = 0x04;

// Thread Control
pub const THREAD_CREATE: usize = 0x10;
pub const THREAD_EXIT: usize = 0x11;
pub const THREAD_JOIN: usize = 0x12;
pub const YIELD: usize = 0x15;

// Debugging & Inspection
pub const GET_CNODE: usize = 0x20;
