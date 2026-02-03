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
    fn page_fault(
        &mut self,
        badge: usize,
        addr: usize,
        pc: usize,
        cause: usize,
    ) -> Result<(), Error>;
    fn unknown_fault(
        &mut self,
        pid: usize,
        cause: usize,
        value: usize,
        pc: usize,
    ) -> Result<(), Error>;
    fn illegal_instrution(&mut self, badge: usize, inst: usize, pc: usize) -> Result<(), Error>;
    fn breakpoint(&mut self, badge: usize, pc: usize) -> Result<(), Error>;
    fn access_fault(&mut self, badge: usize, addr: usize, pc: usize) -> Result<(), Error>;
    fn access_misaligned(&mut self, badge: usize, addr: usize, pc: usize) -> Result<(), Error>;
    fn syscall(
        &mut self,
        badge: usize,
        reg0: usize,
        reg1: usize,
        reg2: usize,
        reg3: usize,
        reg4: usize,
        reg5: usize,
        reg6: usize,
    ) -> Result<(), Error>;
}
