use crate::cap::Page;
use crate::error::Error;
use crate::ipc::Badge;

/// MemoryService provides system-level memory operations for processes.
pub trait MemoryService {
    fn brk(&mut self, pid: Badge, incr: isize) -> Result<usize, Error>;
    fn mmap(&mut self, pid: Badge, page: Page, addr: usize, len: usize) -> Result<usize, Error>;
    fn munmap(&mut self, pid: Badge, addr: usize, len: usize) -> Result<(), Error>;
}
