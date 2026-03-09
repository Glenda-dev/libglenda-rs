pub const PGSIZE: usize = 4096;
pub const VA_MAX: usize = usize::MAX;
pub const EMPTY_VA: usize = 0x0;
pub const VPN_MASK: usize = 0x3FF;
pub const SHIFTS: [usize; 2] = [12, 22];
pub const USER_VA: usize = 0x10000;
pub const KSTACK_PAGES: usize = 4;
pub const PT_LEVELS: usize = 2;
