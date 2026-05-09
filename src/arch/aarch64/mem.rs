pub const PGSIZE: usize = 4096;
pub const VA_MAX: usize = 1 << 48;
pub const EMPTY_VA: usize = 0x0;
pub const VPN_MASK: usize = 0x1FF;
pub const SHIFTS: [usize; 4] = [12, 21, 30, 39];
pub const USER_VA: usize = 0x400000;
pub const KSTACK_PAGES: usize = 4;
pub const PT_LEVELS: usize = 4;
