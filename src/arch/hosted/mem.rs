pub const PGSIZE: usize = 4096;
pub const VA_MAX: usize = 1 << 47; // Hosted (x86_64 usually 48-bit, 47-bit user)
pub const EMPTY_VA: usize = 0x0;
pub const VPN_MASK: usize = 0x1FF;
pub const SHIFTS: [usize; 1] = [12]; // Placeholder for compatibility
pub const USER_VA: usize = 0x10000;
pub const KSTACK_PAGES: usize = 4;
pub const PT_LEVELS: usize = 1;

pub const UTCB_VA: usize = 0x100000;
pub const SHM_VA: usize = 0x20000000;
