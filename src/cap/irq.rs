use super::{CapPtr, Endpoint, irqmethod};
use crate::error::Error;

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
        self.0.invoke(irqmethod::ACK, [0, 0, 0, 0, 0,0, 0, 0])
    }

    pub fn set_notification(&self, notification: Endpoint) -> Result<(), Error> {
        self.0.invoke(irqmethod::SET_NOTIFICATION, [notification.cap().bits(), 0, 0, 0, 0,0, 0, 0])
    }
}
