use super::{CapPtr, Endpoint, irqmethod};
use crate::error::Error;
use crate::ipc::UTCB;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IrqHandler(CapPtr);

impl IrqHandler {
    pub const fn from(cap: CapPtr) -> Self {
        Self(cap)
    }

    pub fn cap(&self) -> CapPtr {
        self.0
    }

    pub fn ack(&self) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        self.0.invoke(irqmethod::ACK, &mut utcb)
    }

    pub fn set_notification(&self, notification: Endpoint) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_mr(0, notification.cap().bits());
        self.0.invoke(irqmethod::SET_NOTIFICATION, &mut utcb)
    }

    pub fn clear_notification(&self) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        self.0.invoke(irqmethod::CLEAR_NOTIFICATION, &mut utcb)
    }

    pub fn set_priority(&self, irq: usize, prio: usize) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_mr(0, irq);
        utcb.set_mr(1, prio);
        self.0.invoke(irqmethod::SET_PRIORITY, &mut utcb)
    }

    pub fn set_threshold(&self, cpu: usize, threshold: usize) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_mr(0, cpu);
        utcb.set_mr(1, threshold);
        self.0.invoke(irqmethod::SET_THRESHOLD, &mut utcb)
    }
}
