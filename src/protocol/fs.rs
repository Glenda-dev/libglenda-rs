//! VFS Protocol Definition
//!
//! Protocol ID range: 0x500 - 0x5FF
//!
//! This protocol defines interactions for Virtual File System operations.
//! It covers both namespace operations (open, mkdir) and file handle operations (read, write).

pub const PROTOCOL_ID: usize = 0x500;

// Namespace Operations (Invoked on Root/Current Dir Capability)
pub const OPEN: usize = 1; // args: [flags, mode], cap: [], string: path -> cap: handle
pub const MKDIR: usize = 2; // args: [mode], string: path
pub const UNLINK: usize = 3; // args: [], string: path
pub const RENAME: usize = 4; // args: [], string: old_path | string: new_path (Note: complex marshaling)
pub const STAT_PATH: usize = 5; // args: [], string: path -> buffer: stat

// File Handle Operations (Invoked on open file Capability)
pub const READ: usize = 10; // args: [size, offset] -> bytes
pub const WRITE: usize = 11; // args: [size, offset], bytes -> written
pub const CLOSE: usize = 12; // args: []
pub const STAT: usize = 13; // args: [] -> buffer: stat
pub const GETDENTS: usize = 14; // args: [count] -> buffer: dirents
pub const SEEK: usize = 15; // args: [offset, whence] -> new_offset
pub const SYNC: usize = 16; // args: []
pub const TRUNCATE: usize = 17; // args: [size]

use bitflags::bitflags;

// Flags for OPEN
bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct OpenFlags: usize {
        const O_RDONLY = 0o0;
        const O_WRONLY = 0o1;
        const O_RDWR   = 0o2;
        const O_CREAT  = 0o100;
        const O_EXCL   = 0o200;
        const O_TRUNC  = 0o1000;
        const O_APPEND = 0o2000;
        const O_DIRECTORY = 0o200000;
    }
}

// File Types (for Stat)
bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct FileType: usize {
        const S_IFMT  = 0o170000;
        const S_IFDIR = 0o040000;
        const S_IFREG = 0o100000;
    }
}
// Seek Whence
pub mod seek {
    pub const SEEK_SET: usize = 0;
    pub const SEEK_CUR: usize = 1;
    pub const SEEK_END: usize = 2;
}

// Struct Definitions
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Stat {
    pub dev: u64,
    pub ino: u64,
    pub mode: u32,
    pub nlink: u32,
    pub uid: u32,
    pub gid: u32,
    pub size: u64,
    pub atime_sec: i64,
    pub atime_nsec: i64,
    pub mtime_sec: i64,
    pub mtime_nsec: i64,
    pub ctime_sec: i64,
    pub ctime_nsec: i64,
    pub blksize: i64,
    pub blocks: i64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DEntry {
    pub d_ino: u64,
    pub d_off: i64,
    pub d_reclen: u16,
    pub d_type: u8,
    pub d_name: [u8; 256],
}
