use crate::cap::{Endpoint, Frame};
use crate::error::Error;
use crate::ipc::Badge;
use crate::protocol::fs::{DEntry, OpenFlags, Stat};
use alloc::string::String;
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

    /// Get file status by path without following the final symlink.
    fn lstat_path(&mut self, _pid: Badge, _path: &str) -> Result<Stat, Error> {
        Err(Error::NotSupported)
    }

    /// Read symbolic link target by path.
    fn readlink_path(&mut self, _pid: Badge, _path: &str) -> Result<String, Error> {
        Err(Error::NotSupported)
    }
}

/// File Handle Service Interface
pub trait FileHandleService {
    /// Close the file handle.
    fn close(&mut self, pid: Badge) -> Result<(), Error>;

    /// Get file status of the open file.
    fn stat(&self, pid: Badge) -> Result<Stat, Error>;

    /// Read data from file at specified offset.
    fn read(&mut self, pid: Badge, offset: usize, buf: &mut [u8]) -> Result<usize, Error>;

    /// Write data to file at specified offset.
    fn write(&mut self, pid: Badge, offset: usize, buf: &[u8]) -> Result<usize, Error>;

    /// Read directory entries.
    fn getdents(&mut self, pid: Badge, count: usize) -> Result<Vec<DEntry>, Error>;

    /// Move the read/write file offset.
    fn seek(&mut self, pid: Badge, offset: i64, whence: usize) -> Result<usize, Error>;

    /// Synchronize file state with storage device.
    fn sync(&mut self, pid: Badge) -> Result<(), Error>;

    /// Truncate file to specified size.
    fn truncate(&mut self, pid: Badge, size: usize) -> Result<(), Error>;

    /// Configure per-handle io_uring shared region.
    fn setup_iouring(
        &mut self,
        _pid: Badge,
        _client_vaddr: usize,
        _size: usize,
        _frame: Option<Frame>,
    ) -> Result<(), Error> {
        Err(Error::NotSupported)
    }

    /// Process queued io_uring requests.
    fn process_iouring(&mut self) -> Result<(), Error> {
        Err(Error::NotSupported)
    }
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
    fn transaction_start(&mut self, pid: Badge) -> Result<usize, Error>;

    /// Commit a transaction.
    fn transaction_commit(&mut self, pid: Badge, tid: usize) -> Result<(), Error>;

    /// Abort a transaction.
    fn transaction_abort(&mut self, pid: Badge, tid: usize) -> Result<(), Error>;

    /// Log a block write operation within a transaction.
    fn log_block(
        &mut self,
        pid: Badge,
        tid: usize,
        block_num: usize,
        data: &[u8],
    ) -> Result<(), Error>;
}
