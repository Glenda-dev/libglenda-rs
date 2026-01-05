use super::{CapPtr, Endpoint, irqmethod};

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

    pub fn ack(&self) -> usize {
        self.0.invoke(irqmethod::ACK, [0, 0, 0, 0, 0, 0, 0])
    }

    pub fn set_notification(&self, notification: Endpoint) -> usize {
        self.0.invoke(irqmethod::SET_NOTIFICATION, [notification.0.bits(), 0, 0, 0, 0, 0, 0])
    }
}
