use crate::cap::{CNode, CapPtr};
use crate::error::Error;
use crate::ipc::{Badge, MsgArgs};

/// ProcessService provides high-level process control.
pub trait ProcessService {
    fn get_pid(&mut self, pid: Badge) -> Result<usize, Error>;
    fn get_ppid(&mut self, pid: Badge) -> Result<usize, Error>;
    fn spawn(&mut self, pid: Badge, path: &str) -> Result<usize, Error>;
    fn fork(&mut self, pid: Badge) -> Result<usize, Error>;
    fn exit(&mut self, pid: Badge, code: usize) -> Result<(), Error>;
    fn kill(&mut self, pid: Badge, target: usize) -> Result<(), Error>;
    fn exec(&mut self, pid: Badge, path: &str) -> Result<(usize, usize), Error>;
    fn get_cnode(&mut self, pid: Badge, target: Badge, recv: CapPtr) -> Result<CNode, Error>;
}

/// ThreadService provided operations for thread management.
pub trait ThreadService {
    fn thread_create(
        &mut self,
        pid: Badge,
        entry: usize,
        arg: usize,
        stack_top: usize,
        tls: usize,
    ) -> Result<usize, Error>;
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
    fn handle_syscall(&mut self, badge: usize, args: MsgArgs) -> Result<(), Error>;
}
