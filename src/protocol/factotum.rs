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

// Admin / Loading
pub const INIT_RESOURCES: usize = 20; // arg0: untyped_start_slot, arg1: untyped_count arg2: irq_start_slot, arg3: irq_count arg4: mmio_start_slot, arg5: mmio_count
pub const PROCESS_LOAD_IMAGE: usize = 21; // arg0: pid, arg1: frame_cap, arg2: offset, arg3: len, arg4: load_addr
pub const PROCESS_START: usize = 22; // arg0: pid, arg1: entry, arg2: stack
pub const SPAWN_SERVICE: usize = 24; // arg0: name_len, arg1: binary_name_len. Name in UTCB buffer.
pub const SPAWN_SERVICE_INITRD: usize = 25; // arg0: name_len. Name in UTCB buffer. Cap: manifest_frame.
pub const INIT_MANIFEST: usize = 26; // Cap: manifest_frame.
pub const SPAWN_SERVICE_MANIFEST: usize = 27; // arg0: index.
pub const SHARE_CAP: usize = 28; // arg0: dest_slot, arg1: target_pid. Cap in transfer.
