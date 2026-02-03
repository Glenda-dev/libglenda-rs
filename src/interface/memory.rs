use crate::error::Error;

/// MemoryService provides system-level memory operations for processes.
pub trait MemoryService {
    fn brk(&mut self, pid: usize, incr: isize) -> Result<usize, Error>;
    fn mmap(&mut self, pid: usize, args: &[usize]) -> Result<usize, Error>;
    fn munmap(&mut self, pid: usize, args: &[usize]) -> Result<(), Error>;
}
