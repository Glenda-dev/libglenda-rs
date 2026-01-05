use super::{CNode, CapPtr, Frame, PageTable, tcbmethod};

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
        vspace: PageTable,
        utcb: Frame,
        trapframe: Frame,
        kstack: Frame,
    ) -> usize {
        self.0.invoke(
            tcbmethod::CONFIGURE,
            [
                cspace.0.bits(),
                vspace.0.bits(),
                utcb.0.bits(),
                trapframe.0.bits(),
                kstack.0.bits(),
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

    pub fn set_fault_handler(&self, fault_ep: CapPtr) -> usize {
        self.0.invoke(tcbmethod::SET_FAULT_HANDLER, [fault_ep.bits(), 0, 0, 0, 0, 0, 0])
    }

    pub fn resume(&self) -> usize {
        self.0.invoke(tcbmethod::RESUME, [0, 0, 0, 0, 0, 0, 0])
    }

    pub fn suspend(&self) -> usize {
        self.0.invoke(tcbmethod::SUSPEND, [0, 0, 0, 0, 0, 0, 0])
    }
}
