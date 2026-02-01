// --- Kernel Protocols (Reserved High Values) ---

pub const PAGE_FAULT: usize = 1;
pub const EXCEPTION: usize = 2;
pub const UNKNOWN_SYSCALL: usize = 3;
pub const CAP_FAULT: usize = 4;
pub const IRQ: usize = 5;
pub const NOTIFY: usize = 6;
pub const FAULT: usize = EXCEPTION;
