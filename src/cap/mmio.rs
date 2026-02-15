use super::CapPtr;
use super::method::mmiomethod;
use crate::error::Error;
use crate::ipc::UTCB;

#[derive(Clone, Copy, Debug)]
pub struct Mmio(CapPtr);

impl Mmio {
    pub const fn from(cptr: CapPtr) -> Self {
        Self(cptr)
    }

    pub fn get_frame(&self, paddr: usize, pages: usize, dest_cptr: CapPtr) -> Result<(), Error> {
        let utcb = unsafe { UTCB::new() };
        utcb.set_mr(0, paddr);
        utcb.set_mr(1, pages);
        utcb.set_mr(2, dest_cptr.bits());
        self.0.invoke(mmiomethod::GET_FRAME, utcb)
    }
}
