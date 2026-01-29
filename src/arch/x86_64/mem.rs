pub const PGSIZE: usize = 4096;
pub const VA_MAX: usize = 1 << 47; // 256 TB 虚拟地址空间上限
pub const EMPTY_VA: usize = 0x0; // 空虚拟地址
pub const VPN_MASK: usize = 0x1FF;
pub const SHIFTS: [usize; 4] = [12, 21, 30, 39]; // L0, L1, L2, L3
// TODO: Define proper permission bits for x86_64
pub mod perms {}
