// Protocol ID
pub const PROCESS_PROTO: usize = 0x0100;

// Process Lifecycle
pub const SPAWN: usize = 1;
pub const EXIT: usize = 2;
pub const WAIT: usize = 3;
pub const KILL: usize = 4;
pub const FORK: usize = 5;

// Memory Management
pub const SBRK: usize = 6;
pub const MMAP: usize = 7;
pub const MUNMAP: usize = 8;

// Thread Control
pub const THREAD_CREATE: usize = 9;
pub const THREAD_EXIT: usize = 10;
pub const THREAD_JOIN: usize = 11;
pub const FUTEX_WAIT: usize = 12;
pub const FUTEX_WAKE: usize = 13;
pub const YIELD: usize = 14;
pub const SLEEP: usize = 15;

// Debugging & Inspection
pub const GET_PID: usize = 16;
pub const PS: usize = 18;

// Admin / Loading
pub const SPAWN_SERVICE: usize = 25; // arg0: name_len. Name in UTCB buffer. Cap: manifest_frame.
