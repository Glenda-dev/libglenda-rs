// 页大小
pub const PGSIZE: usize = 4096;
// 虚拟地址最高处
pub const VA_MAX: usize = 1 << 63;
// 空地址
pub const EMPTY_VA: usize = 0x0;
pub const VPN_MASK: usize = 0x0;
pub const SHIFTS: [usize; 1] = [0];
// 用户空间地址起始
pub const USER_VA: usize = 0x10000;
