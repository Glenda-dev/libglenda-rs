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
        utcb_frame: Frame,
        trapframe: Frame,
        kstack: Frame,
    ) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(
            utcb,
            cspace.cap().bits(),
            vspace.cap().bits(),
            utcb_frame.cap().bits(),
            trapframe.cap().bits(),
            kstack.cap().bits(),
        );
        self.0.invoke(tcbmethod::CONFIGURE, &mut utcb)
    }

    pub fn set_priority(&self, priority: u8, incr: i8) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, priority as usize, incr as usize);
        self.0.invoke(tcbmethod::SET_PRIORITY, &mut utcb)
    }

    pub fn set_entrypoint(&self, pc: usize, sp: usize, tp: usize) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, pc, sp, tp);
        self.0.invoke(tcbmethod::SET_ENTRYPOINT, &mut utcb)
    }

    pub fn set_address(&self, utcb_va: usize, trapframe_va: usize) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, utcb_va, trapframe_va);
        self.0.invoke(tcbmethod::SET_ADDRESS, &mut utcb)
    }

    pub fn set_fault_handler(&self, fault_ep: Endpoint, native: bool) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, fault_ep.cap().bits(), native as usize);
        self.0.invoke(tcbmethod::SET_FAULT_HANDLER, &mut utcb)
    }

    pub fn set_registers(&self, regs: &[usize]) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        for (i, &reg) in regs.iter().enumerate() {
            utcb.set_mr(i, reg);
        }
        self.0.invoke(tcbmethod::SET_REGISTERS, &mut utcb)
    }

    pub fn resume(&self) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        self.0.invoke(tcbmethod::RESUME, &mut utcb)
    }

    pub fn suspend(&self) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        self.0.invoke(tcbmethod::SUSPEND, &mut utcb)
    }
}
