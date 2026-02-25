use crate::cap::{Endpoint, Frame};
use crate::error::Error;
use crate::ipc::Badge;
use crate::protocol::fs::{DEntry, OpenFlags, Stat};
use alloc::vec::Vec;

/// Filesystem Service Interface
pub trait FileSystemService: Send {
    /// Open a file or directory.
    /// Returns a capability pointer (handle) to the open file.
    fn open(&mut self, pid: Badge, path: &str, flags: OpenFlags, mode: u32)
    -> Result<usize, Error>;

    /// Create a directory.
    fn mkdir(&mut self, pid: Badge, path: &str, mode: u32) -> Result<(), Error>;

    /// Remove a directory entry (file or directory).
    fn unlink(&mut self, pid: Badge, path: &str) -> Result<(), Error>;

    /// Rename a file or directory.
    fn rename(&mut self, pid: Badge, old_path: &str, new_path: &str) -> Result<(), Error>;

    /// Get file status by path.
    fn stat_path(&mut self, pid: Badge, path: &str) -> Result<Stat, Error>;
}

/// File Handle Service Interface
pub trait FileHandleService {
    /// Close the file handle.
    fn close(&mut self, pid: Badge) -> Result<(), Error>;

    /// Get file status of the open file.
    fn stat(&self, pid: Badge) -> Result<Stat, Error>;

    /// Read data from file at specified offset.
    fn read(&mut self, pid: Badge, offset: u64, buf: &mut [u8]) -> Result<usize, Error>;

    /// Write data to file at specified offset.
    fn write(&mut self, pid: Badge, offset: u64, buf: &[u8]) -> Result<usize, Error>;

    /// Read directory entries.
    fn getdents(&mut self, pid: Badge, count: usize) -> Result<Vec<DEntry>, Error>;

    /// Move the read/write file offset.
    fn seek(&mut self, pid: Badge, offset: i64, whence: usize) -> Result<u64, Error>;

    /// Synchronize file state with storage device.
    fn sync(&mut self, pid: Badge) -> Result<(), Error>;

    /// Truncate file to specified size.
    fn truncate(&mut self, pid: Badge, size: u64) -> Result<(), Error>;

    /// Setup io_uring for this file handle.
    fn setup_iouring(
        &mut self,
        pid: Badge,
        server_vaddr: usize,
        client_vaddr: usize,
        size: usize,
        frame: Option<Frame>,
    ) -> Result<(), Error>;

    /// Process pending io_uring entries.
    fn process_iouring(&mut self, pid: Badge) -> Result<(), Error>;
}

/// Virtual Filesystem Service Interface (for VFS/Nexus)
pub trait VirtualFileSystemService: FileSystemService {
    /// Mount a filesystem at the specified path.
    fn mount(&mut self, pid: Badge, path: &str, target: Endpoint) -> Result<(), Error>;

    /// Unmount a filesystem from the specified path.
    fn unmount(&mut self, pid: Badge, path: &str) -> Result<(), Error>;
}

/// PipeService provides creating anonymous pipes.
pub trait PipeService {
    /// Create a pipe.
    /// Returns a pair of capability handles (read_end, write_end).
    fn pipe(&mut self, pid: Badge) -> Result<(usize, usize), Error>;
}

/// FileSystemJournalService provides transaction support for file systems.
pub trait FileSystemJournalService {
    /// Start a transaction. Returns transaction ID.
    fn transaction_start(&mut self, pid: Badge) -> Result<u64, Error>;

    /// Commit a transaction.
    fn transaction_commit(&mut self, pid: Badge, tid: u64) -> Result<(), Error>;

    /// Abort a transaction.
    fn transaction_abort(&mut self, pid: Badge, tid: u64) -> Result<(), Error>;

    /// Log a block write operation within a transaction.
    fn log_block(&mut self, pid: Badge, tid: u64, block_num: u64, data: &[u8])
    -> Result<(), Error>;
}
