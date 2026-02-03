use crate::error::Error;

/// ProcessService provides high-level process control.
pub trait ProcessService {
    fn spawn(&mut self, name: &str) -> Result<usize, Error>;
    fn fork(&mut self, pid: usize) -> Result<usize, Error>;
    fn exit(&mut self, pid: usize, code: usize) -> Result<(), Error>;
    fn load_image(&mut self, pid: usize, elf_data: &[u8]) -> Result<(usize, usize), Error>;
}

/// FaultService handles faults for processes.
pub trait FaultService {
    fn handle_page_fault(
        &mut self,
        pid: usize,
        vaddr: usize,
        error_code: usize,
    ) -> Result<(), Error>;
}
