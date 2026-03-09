use bitflags::bitflags;

// --- Namespace Operations (Synchronous) ---
pub const OPEN: usize = 0x1; // args: [flags, mode], str: path -> cap: FileHandle (Badged Endpoint)
pub const MKDIR: usize = 0x2; // args: [mode], str: path -> res: Status
pub const UNLINK: usize = 0x3; // args: [], str: path -> res: Status
pub const STAT_PATH: usize = 0x4; // args: [], str: path -> buf: Stat
pub const RENAME: usize = 0x5; // args: [], str: old|new -> res: Status

// --- File Handle Operations (Synchronous) ---
pub const CLOSE: usize = 0x10; // args: [] -> res: Status
pub const STAT: usize = 0x11; // args: [] -> buf: Stat
pub const SETATTR: usize = 0x12; // args: [mask], buf: Attrs -> res: Status
pub const READ_SYNC: usize = 0x13; // args: [size, offset] -> bytes: data
pub const WRITE_SYNC: usize = 0x14; // args: [offset], bytes: data -> res: written
pub const GETDENTS: usize = 0x15; // args: [count -> buf: DEntry[]
pub const SYNC: usize = 0x16;
pub const SEEK: usize = 0x17; // args: [offset, whence] -> res: new_offset
pub const TRUNCATE: usize = 0x18; // args: [size] -> res: Status
pub const SETUP_IOURING: usize = 0x20; // args: [size], cap: Frame -> res: Status
pub const PROCESS_IOURING: usize = 0x21; // args: [] -> res: Status
pub const LOOP_SETUP: usize = 0x30; // args: [], cap: FileHandle -> res: Endpoint (BlockDevice)

pub const MOUNT: usize = 0x40; // args: [], str: path, cap: FS_ENDPOINT
pub const UNMOUNT: usize = 0x41; // args: [], str: path

// --- Request types for IoUring Entries ---
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    Nop = 0,
    Read = 1,
    Write = 2,
    Flush = 3,
}

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
        const S_IFBLK = 0o060000;
        const S_IFCHR = 0o020000;
    }
}

// Seek Whence
pub mod seek {
    pub const SEEK_SET: usize = 0;
    pub const SEEK_CUR: usize = 1;
    pub const SEEK_END: usize = 2;
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Stat {
    pub dev: usize,
    pub ino: usize,
    pub mode: u32,
    pub nlink: u32,
    pub uid: u32,
    pub gid: u32,
    pub rdev: usize,
    pub size: usize,
    pub blksize: u32,
    pub blocks: usize,
    pub atime: usize,
    pub mtime: usize,
    pub ctime: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DEntry {
    pub d_ino: usize,
    pub d_off: i64,
    pub d_reclen: u16,
    pub d_type: u8,
    pub d_name: [u8; 256],
}
