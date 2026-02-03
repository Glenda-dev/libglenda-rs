use crate::cap::Endpoint;

use super::{CNode, CapPtr, Frame, VSpace, tcbmethod};
use crate::error::Error;
use crate::ipc::MsgArgs;
use crate::ipc::utcb::MAX_MRS;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TCB(CapPtr);

impl TCB {
    pub const fn from(cap: CapPtr) -> Self {
        Self(cap)
    }

    pub fn cap(&self) -> CapPtr {
        self.0
    }

    pub fn configure(
        &self,
        cspace: CNode,
        vspace: VSpace,
        utcb: Frame,
        trapframe: Frame,
        kstack: Frame,
    ) -> Result<(), Error> {
        self.0.invoke(
            tcbmethod::CONFIGURE,
            [
                cspace.cap().bits(),
                vspace.cap().bits(),
                utcb.cap().bits(),
                trapframe.cap().bits(),
                kstack.cap().bits(),
                0,
                0,
                0,
            ],
        )
    }

    pub fn set_priority(&self, priority: u8) -> Result<(), Error> {
        self.0.invoke(tcbmethod::SET_PRIORITY, [priority as usize, 0, 0, 0, 0, 0, 0, 0])
    }

    pub fn set_entrypoint(&self, pc: usize, sp: usize, tp: usize) -> Result<(), Error> {
        self.0.invoke(tcbmethod::SET_ENTRYPOINT, [pc, sp, tp, 0, 0, 0, 0, 0])
    }

    pub fn set_fault_handler(&self, fault_ep: Endpoint, native: bool) -> Result<(), Error> {
        self.0.invoke(
            tcbmethod::SET_FAULT_HANDLER,
            [fault_ep.cap().bits(), native as usize, 0, 0, 0, 0, 0, 0],
        )
    }

    pub fn set_registers(&self, regs: MsgArgs) -> Result<(), Error> {
        self.0.invoke(tcbmethod::SET_REGISTERS, regs)
    }

    pub fn resume(&self) -> Result<(), Error> {
        self.0.invoke(tcbmethod::RESUME, [0; MAX_MRS])
    }

    pub fn suspend(&self) -> Result<(), Error> {
        self.0.invoke(tcbmethod::SUSPEND, [0; MAX_MRS])
    }
}
