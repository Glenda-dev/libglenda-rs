use crate::error::Error;
use crate::protocol::fs::{DEntry, OpenFlags, Stat};
use alloc::vec::Vec;

/// FileSystemService provides high-level access to the file system (Namespace operations).
pub trait FileSystemService {
    /// Open a file or directory.
    /// Returns a capability pointer (handle) to the open file.
    fn open(&mut self, path: &str, flags: OpenFlags, mode: u32) -> Result<usize, Error>;

    /// Create a directory.
    fn mkdir(&mut self, path: &str, mode: u32) -> Result<(), Error>;

    /// Remove a directory entry (file or directory).
    fn unlink(&mut self, path: &str) -> Result<(), Error>;

    /// Rename a file or directory.
    fn rename(&mut self, old_path: &str, new_path: &str) -> Result<(), Error>;

    /// Get file status by path.
    fn stat_path(&mut self, path: &str) -> Result<Stat, Error>;
}

/// PipeService provides creating anonymous pipes.
pub trait PipeService {
    /// Create a pipe.
    /// Returns a pair of capability handles (read_end, write_end).
    fn pipe(&mut self) -> Result<(usize, usize), Error>;
}

/// FileSystemJournalService provides transaction support for file systems.
pub trait FileSystemJournalService {
    /// Start a transaction. Returns transaction ID.
    fn transaction_start(&mut self) -> Result<u64, Error>;

    /// Commit a transaction.
    fn transaction_commit(&mut self, tid: u64) -> Result<(), Error>;

    /// Abort a transaction.
    fn transaction_abort(&mut self, tid: u64) -> Result<(), Error>;

    /// Log a block write operation within a transaction.
    fn log_block(&mut self, tid: u64, block_num: u64, data: &[u8]) -> Result<(), Error>;
}

/// FileHandleService provides operations on an open file handle.
pub trait FileHandleService {
    /// Read data from file at specified offset.
    fn read(&mut self, offset: u64, buf: &mut [u8]) -> Result<usize, Error>;

    /// Write data to file at specified offset.
    fn write(&mut self, offset: u64, buf: &[u8]) -> Result<usize, Error>;

    /// Close the file handle.
    fn close(&mut self) -> Result<(), Error>;

    /// Get file status of the open file.
    fn stat(&self) -> Result<Stat, Error>;

    /// Read directory entries.
    fn getdents(&mut self, count: usize) -> Result<Vec<DEntry>, Error>;

    /// Move the read/write file offset.
    fn seek(&mut self, offset: i64, whence: usize) -> Result<u64, Error>;

    /// Synchronize file state with storage device.
    fn sync(&mut self) -> Result<(), Error>;

    /// Truncate file to specified size.
    fn truncate(&mut self, size: u64) -> Result<(), Error>;
}
