use super::{CNode, CapPtr, Frame, VSpace, tcbmethod};
use crate::cap::Endpoint;
use crate::error::Error;
use crate::ipc::UTCB;
use crate::set_mrs;

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
        let utcb_ptr = unsafe { UTCB::get() };
        set_mrs!(
            utcb_ptr,
            cspace.cap().bits(),
            vspace.cap().bits(),
            utcb.cap().bits(),
            trapframe.cap().bits(),
            kstack.cap().bits()
        );
        self.0.invoke(tcbmethod::CONFIGURE)
    }

    pub fn set_priority(&self, priority: u8) -> Result<(), Error> {
        let utcb = unsafe { UTCB::get() };
        set_mrs!(utcb, priority);
        self.0.invoke(tcbmethod::SET_PRIORITY)
    }

    pub fn set_entrypoint(&self, pc: usize, sp: usize, tp: usize) -> Result<(), Error> {
        let utcb = unsafe { UTCB::get() };
        set_mrs!(utcb, pc, sp, tp);
        self.0.invoke(tcbmethod::SET_ENTRYPOINT)
    }

    pub fn set_fault_handler(&self, fault_ep: Endpoint, native: bool) -> Result<(), Error> {
        let utcb = unsafe { UTCB::get() };
        set_mrs!(utcb, fault_ep.cap().bits(), native);
        self.0.invoke(tcbmethod::SET_FAULT_HANDLER)
    }

    pub fn set_registers(&self) -> Result<(), Error> {
        self.0.invoke(tcbmethod::SET_REGISTERS)
    }

    pub fn resume(&self) -> Result<(), Error> {
        self.0.invoke(tcbmethod::RESUME)
    }

    pub fn suspend(&self) -> Result<(), Error> {
        self.0.invoke(tcbmethod::SUSPEND)
    }
}
