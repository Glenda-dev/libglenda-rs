pub mod auth;
pub mod device;
pub mod fs;
pub mod generic;
#[cfg(feature = "hosted")]
pub mod hosted;
pub mod init;
pub mod input;
pub mod kernel;
pub mod network;
pub mod process;
pub mod resource;
pub mod terminal;
pub mod time;
pub mod volume;

// Protocol ID
pub const INPUT_PROTO: usize = 0x0C00;
pub const TERMINAL_PROTO: usize = 0x0B00;
pub const TIME_PROTO: usize = 0x0A00;
pub const VOLUME_PROTO: usize = 0x0900;
pub const AUTH_PROTO: usize = 0x0800;
pub const NETWORK_PROTO: usize = 0x0700;
pub const FS_PROTO: usize = 0x0600;
pub const INIT_PROTO: usize = 0x0500;
pub const DEVICE_PROTO: usize = 0x0400;
pub const RESOURCE_PROTO: usize = 0x0300;
pub const PROCESS_PROTO: usize = 0x0200;
pub const KERNEL_PROTO: usize = 0x0100;
pub const GENERIC_PROTO: usize = 0x0000;
