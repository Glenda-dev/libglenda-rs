// Protocol ID
pub const FACTOTUM_PROTO: usize = 0x0100;

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
pub const GET_PPID: usize = 17;
pub const PS: usize = 18;

// Capability Brokerage
pub const REQUEST_CAP: usize = 19;

// Admin / Loading
pub const INIT_RESOURCES: usize = 20; // arg0: start_slot, arg1: count
pub const INIT_IRQ: usize = 23; // arg0: start_slot, arg1: count
pub const PROCESS_LOAD_IMAGE: usize = 21; // arg0: pid, arg1: frame_cap, arg2: offset, arg3: len, arg4: load_addr
pub const PROCESS_START: usize = 22; // arg0: pid, arg1: entry, arg2: stack
