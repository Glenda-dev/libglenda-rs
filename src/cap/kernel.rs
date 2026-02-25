use super::{CapPtr, kernelmethod};
use crate::error::Error;
use crate::ipc::UTCB;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Kernel(CapPtr);

impl Kernel {
    pub const fn from(cap: CapPtr) -> Self {
        Self(cap)
    }

    pub fn cap(&self) -> CapPtr {
        self.0
    }

    pub const fn null() -> Self {
        Self(CapPtr::null())
    }

    pub fn shell(&self) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        self.0.invoke(kernelmethod::SHELL, &mut utcb)
    }

    pub fn get_irq(&self, irq: usize, dest_cptr: CapPtr) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_mr(0, irq);
        utcb.set_mr(1, dest_cptr.bits());
        self.0.invoke(kernelmethod::GET_IRQ, &mut utcb)
    }

    pub fn get_mmio(&self, paddr: usize, pages: usize, dest_cptr: CapPtr) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_mr(0, paddr);
        utcb.set_mr(1, pages);
        utcb.set_mr(2, dest_cptr.bits());
        self.0.invoke(kernelmethod::GET_MMIO, &mut utcb)
    }

    pub fn set_alarm(&self, ms: usize, ntfn_cptr: CapPtr) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_mr(0, ms);
        utcb.set_mr(1, ntfn_cptr.bits());
        self.0.invoke(kernelmethod::SET_ALARM, &mut utcb)
    }
}
