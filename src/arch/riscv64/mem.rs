pub const PGSIZE: usize = 4096;
pub const VA_MAX: usize = 1 << 38; // 256 GiB 虚拟地址空间上限
pub const EMPTY_VA: usize = 0x0; // 空虚拟地址
pub const VPN_MASK: usize = 0x1FF;
pub const SHIFTS: [usize; 3] = [12, 21, 30]; // L0, L1, L2
