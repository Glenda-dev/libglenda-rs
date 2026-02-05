pub mod device;
pub mod fs;
pub mod generic;
pub mod init;
pub mod kernel;
pub mod network;
pub mod process;

// Protocol ID
pub const NETWORK_PROTO: usize = 0x0600;
pub const FS_PROTO: usize = 0x0500;
pub const INIT_PROTOCOL: usize = 0x0400;
pub const DEVICE_PROTO: usize = 0x0300;
pub const PROCESS_PROTO: usize = 0x0200;
pub const KERNEL_PROTO: usize = 0x0100;
pub const GENERIC_PROTO: usize = 0x0000;
