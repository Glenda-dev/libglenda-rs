use super::{CapPtr, TCB, vcpumethod, vmspacemethod};
use crate::error::Error;
use crate::ipc::UTCB;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VCPU(CapPtr);

impl VCPU {
    pub const fn from(cap: CapPtr) -> Self {
        Self(cap)
    }

    pub fn cap(&self) -> CapPtr {
        self.0
    }

    pub fn bind_tcb(&self, tcb: TCB) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_mr(0, tcb.cap().bits());
        utcb.set_mr(1, 0);
        self.0.invoke(vcpumethod::BIND_TCB, &mut utcb)
    }

    pub fn bind_tcb_with_vmspace(&self, tcb: TCB, vmspace: VMSpace) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_mr(0, tcb.cap().bits());
        utcb.set_mr(1, vmspace.cap().bits());
        self.0.invoke(vcpumethod::BIND_TCB, &mut utcb)
    }

    pub fn run(&self) -> Result<(), Error> {
        let _ = self.run_exit()?;
        Ok(())
    }

    pub fn run_exit(&self) -> Result<(usize, usize, usize, usize), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        self.0.invoke(vcpumethod::RUN, &mut utcb)?;
        Ok((utcb.get_mr(0), utcb.get_mr(1), utcb.get_mr(2), utcb.get_mr(3)))
    }

    pub fn inject_irq(&self, irq: usize) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_mr(0, irq);
        self.0.invoke(vcpumethod::INJECT_IRQ, &mut utcb)
    }

    pub fn read_reg(&self, reg: usize) -> Result<usize, Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_mr(0, reg);
        self.0.invoke(vcpumethod::READ_REG, &mut utcb)?;
        Ok(utcb.get_mr(0))
    }

    pub fn write_reg(&self, reg: usize, value: usize) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_mr(0, reg);
        utcb.set_mr(1, value);
        self.0.invoke(vcpumethod::WRITE_REG, &mut utcb)
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VMSpace(CapPtr);

impl VMSpace {
    pub const fn from(cap: CapPtr) -> Self {
        Self(cap)
    }

    pub fn cap(&self) -> CapPtr {
        self.0
    }

    pub fn map_stage2(
        &self,
        frame_cptr: CapPtr,
        guest_paddr: usize,
        host_paddr: usize,
        pages: usize,
    ) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_mr(0, frame_cptr.bits());
        utcb.set_mr(1, guest_paddr);
        utcb.set_mr(2, host_paddr);
        utcb.set_mr(3, pages);
        self.0.invoke(vmspacemethod::MAP_STAGE2, &mut utcb)
    }

    pub fn unmap_stage2(&self, guest_paddr: usize, pages: usize) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_mr(0, guest_paddr);
        utcb.set_mr(1, pages);
        self.0.invoke(vmspacemethod::UNMAP_STAGE2, &mut utcb)
    }

    pub fn setup_stage2(&self) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        self.0.invoke(vmspacemethod::SETUP_STAGE2, &mut utcb)
    }
}
