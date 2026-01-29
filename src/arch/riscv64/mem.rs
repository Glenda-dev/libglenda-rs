pub const PGSIZE: usize = 4096;
pub const VA_MAX: usize = 1 << 38; // 256 GiB 虚拟地址空间上限
pub const EMPTY_VA: usize = 0x0; // 空虚拟地址
pub const VPN_MASK: usize = 0x1FF;
pub const SHIFTS: [usize; 3] = [12, 21, 30]; // L0, L1, L2
pub mod perms {
    pub const VALID: usize = 1 << 0;
    pub const READ: usize = 1 << 1;
    pub const WRITE: usize = 1 << 2;
    pub const EXECUTE: usize = 1 << 3;
    pub const USER: usize = 1 << 4;
    pub const GLOBAL: usize = 1 << 5;
    pub const ACCESSED: usize = 1 << 6;
    pub const DIRTY: usize = 1 << 7;
}
