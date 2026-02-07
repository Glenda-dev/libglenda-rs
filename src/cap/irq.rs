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
        self.0.invoke(irqmethod::ACK)
    }

    pub fn set_notification(&self, notification: Endpoint) -> Result<(), Error> {
        let utcb = unsafe { UTCB::get() };
        utcb.mrs_regs[0] = notification.cap().bits();
        self.0.invoke(irqmethod::SET_NOTIFICATION)
    }
}
