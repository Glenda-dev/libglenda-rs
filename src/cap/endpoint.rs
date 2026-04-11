use super::CapPtr;
use super::ipcmethod;
use crate::error::Error;
use crate::ipc::{Badge, UTCB};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Endpoint(CapPtr);

impl Endpoint {
    pub const fn from(cap: CapPtr) -> Self {
        Self(cap)
    }

    pub fn cap(&self) -> CapPtr {
        self.0
    }

    pub fn send(&self, utcb: &mut UTCB) -> Result<(), Error> {
        self.0.invoke(ipcmethod::SEND, utcb)
    }

    pub fn recv(&self, utcb: &mut UTCB) -> Result<(), Error> {
        self.0.invoke(ipcmethod::RECV, utcb)
    }

    pub fn call(&self, utcb: &mut UTCB) -> Result<(), Error> {
        self.0.invoke(ipcmethod::CALL, utcb)?;
        utcb.error_check()
    }

    pub fn notify(&self, badge: Badge) -> Result<(), Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.set_badge(badge);
        self.0.invoke(ipcmethod::NOTIFY, &mut utcb)
    }

    pub fn proxy(&self, utcb: &mut UTCB) -> Result<(), Error> {
        self.0.invoke(ipcmethod::PROXY, utcb)
    }
}
