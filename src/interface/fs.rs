use crate::cap::{CapPtr, Endpoint, Page};
use crate::error::Error;
use crate::ipc::Badge;
use crate::protocol::fs::{DEntry, OpenFlags, Stat};
use alloc::string::String;
use alloc::vec::Vec;

/// Filesystem Service Interface
pub trait FileSystemService: Send {
    /// Open a file or directory.
    /// Returns a capability pointer (handle) to the open file.
    fn open(
        &mut self,
        pid: Badge,
        path: &str,
        flags: OpenFlags,
        mode: u32,
        recv_slot: CapPtr,
    ) -> Result<usize, Error>;

    /// Create a directory.
    fn mkdir(&mut self, pid: Badge, path: &str, mode: u32) -> Result<(), Error>;

    /// Remove a directory entry (file or directory).
    fn unlink(&mut self, pid: Badge, path: &str) -> Result<(), Error>;

    /// Rename a file or directory.
    fn rename(&mut self, pid: Badge, old_path: &str, new_path: &str) -> Result<(), Error>;

    /// Create a hard link from `old_path` to `new_path`.
    fn link(&mut self, _pid: Badge, _old_path: &str, _new_path: &str) -> Result<(), Error> {
        Err(Error::NotSupported)
    }

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

    /// Perform device-specific ioctl operation.
    fn ioctl(&mut self, _pid: Badge, _cmd: u32, _arg: usize) -> Result<usize, Error> {
        Err(Error::NotSupported)
    }

    /// Poll handle readiness with Linux `poll(2)` event mask.
    ///
    /// Default behavior is "always ready" for requested events.
    fn poll(&mut self, _pid: Badge, events: u32) -> Result<u32, Error> {
        Ok(events)
    }

    /// Perform extended ioctl operation with structured payload.
    ///
    /// - `input`: optional serialized input payload copied to backend.
    /// - `out_len`: requested output payload capacity.
    ///
    /// Returns `(ret, out_bytes)` where `ret` is ioctl return value and
    /// `out_bytes` is optional output payload returned by backend.
    fn ioctl_ex(
        &mut self,
        pid: Badge,
        cmd: u32,
        arg: usize,
        input: Option<&[u8]>,
        out_len: usize,
    ) -> Result<(usize, Vec<u8>), Error> {
        if input.map(|b| !b.is_empty()).unwrap_or(false) || out_len != 0 {
            return Err(Error::NotSupported);
        }
        Ok((self.ioctl(pid, cmd, arg)?, Vec::new()))
    }

    /// Configure per-handle io_uring shared region.
    fn setup_iouring(
        &mut self,
        _pid: Badge,
        _client_vaddr: usize,
        _size: usize,
        _frame: Option<Page>,
    ) -> Result<(), Error> {
        Err(Error::NotSupported)
    }

    /// Process queued io_uring requests.
    fn process_iouring(&mut self) -> Result<(), Error> {
        Err(Error::NotSupported)
    }

    /// Request a file page frame capability at `offset`.
    ///
    /// The service should transfer a frame cap into `recv_slot` and return the number
    /// of valid bytes in that page via return value.
    fn map_page(
        &mut self,
        _pid: Badge,
        _offset: usize,
        _recv_slot: CapPtr,
    ) -> Result<usize, Error> {
        Err(Error::NotSupported)
    }

    /// Request multiple contiguous file pages starting at `offset`.
    /// Returns total valid bytes in the returned frame object.
    fn map_pages(
        &mut self,
        pid: Badge,
        offset: usize,
        pages: usize,
        recv_slot: CapPtr,
    ) -> Result<usize, Error> {
        if pages == 1 { self.map_page(pid, offset, recv_slot) } else { Err(Error::NotSupported) }
    }

    /// Release/unpin a previously transferred file page frame.
    fn unmap_page(&mut self, _pid: Badge, _frame: Page) -> Result<(), Error> {
        Err(Error::NotSupported)
    }
}

/// Virtual Filesystem Service Interface (for VFS/Nexus)
pub trait VirtualFileSystemService: FileSystemService {
    /// Mount a filesystem at the specified path.
    ///
    /// Repeated mounts on the same path are layered in stack order.
    /// The most recently mounted layer has the highest priority.
    fn mount(&mut self, pid: Badge, path: &str, target: Endpoint) -> Result<(), Error>;

    /// Unmount a filesystem from the specified path.
    ///
    /// If multiple layers exist on the same path, unmount removes only the top layer.
    fn unmount(&mut self, pid: Badge, path: &str) -> Result<(), Error>;

    /// Create a new filesystem view cloned from caller's current view and set root path.
    fn create_view(&mut self, _pid: Badge, _root: &str) -> Result<usize, Error> {
        Err(Error::NotSupported)
    }

    /// Bind caller process to an existing view.
    fn set_view(&mut self, _pid: Badge, _view_id: usize) -> Result<(), Error> {
        Err(Error::NotSupported)
    }
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
