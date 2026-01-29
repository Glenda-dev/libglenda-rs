// 页大小
pub const PGSIZE: usize = 4096;
// 虚拟地址最高处
pub const VA_MAX: usize = 1 << 63;
// 空地址
pub const EMPTY_VA: usize = 0x0;
pub const VPN_MASK: usize = 0x0;
pub const SHIFTS: [usize; 1] = [0];
// 权限位定义
pub mod perms {
    pub const VALID: usize = 1;
    pub const READ: usize = 1;
    pub const WRITE: usize = 1;
    pub const EXECUTE: usize = 1;
    pub const USER: usize = 1;
    pub const GLOBAL: usize = 1;
    pub const ACCESSED: usize = 1;
    pub const DIRTY: usize = 1;
}
