use crate::error::Error;
use crate::ipc::{Badge, MsgArgs};

/// ProcessService provides high-level process control.
pub trait ProcessService {
    fn get_pid(&mut self) -> Result<usize, Error>;
    fn spawn(&mut self, name: &str) -> Result<usize, Error>;
    fn fork(&mut self, pid: Badge) -> Result<usize, Error>;
    fn exit(&mut self, pid: Badge, code: usize) -> Result<(), Error>;
    fn load_image(&mut self, pid: Badge, elf_data: &[u8]) -> Result<(usize, usize), Error>;
}

/// FaultService handles faults for processes.
pub trait FaultService {
    fn page_fault(
        &mut self,
        badge: Badge,
        addr: usize,
        pc: usize,
        cause: usize,
    ) -> Result<(), Error>;
    fn unknown_fault(
        &mut self,
        badge: Badge,
        cause: usize,
        value: usize,
        pc: usize,
    ) -> Result<(), Error>;
    fn illegal_instrution(&mut self, badge: Badge, inst: usize, pc: usize) -> Result<(), Error>;
    fn breakpoint(&mut self, badge: Badge, pc: usize) -> Result<(), Error>;
    fn access_fault(&mut self, badge: Badge, addr: usize, pc: usize) -> Result<(), Error>;
    fn access_misaligned(&mut self, badge: Badge, addr: usize, pc: usize) -> Result<(), Error>;
    fn syscall(&mut self, badge: Badge, regs: MsgArgs) -> Result<(), Error>;
}
