use crate::cap::Endpoint;

use super::{CNode, CapPtr, Frame, VSpace, tcbmethod};

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
    ) -> usize {
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
            ],
        )
    }

    pub fn set_priority(&self, priority: usize) -> usize {
        self.0.invoke(tcbmethod::SET_PRIORITY, [priority, 0, 0, 0, 0, 0, 0])
    }

    pub fn set_registers(&self, flags: usize, pc: usize, sp: usize) -> usize {
        self.0.invoke(tcbmethod::SET_REGISTERS, [flags, pc, sp, 0, 0, 0, 0])
    }

    pub fn set_fault_handler(&self, fault_ep: Endpoint, native: bool) -> usize {
        self.0.invoke(
            tcbmethod::SET_FAULT_HANDLER,
            [fault_ep.cap().bits(), native as usize, 0, 0, 0, 0, 0],
        )
    }

    pub fn resume(&self) -> usize {
        self.0.invoke(tcbmethod::RESUME, [0, 0, 0, 0, 0, 0, 0])
    }

    pub fn suspend(&self) -> usize {
        self.0.invoke(tcbmethod::SUSPEND, [0, 0, 0, 0, 0, 0, 0])
    }
}
