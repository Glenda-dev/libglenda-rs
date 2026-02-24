use super::CapPtr;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frame(CapPtr);

impl Frame {
    pub const fn from(cap: CapPtr) -> Self {
        Self(cap)
    }

    pub fn cap(&self) -> CapPtr {
        self.0
    }
}

impl From<CapPtr> for Frame {
    fn from(cap: CapPtr) -> Self {
        Self(cap)
    }
}

impl From<Frame> for CapPtr {
    fn from(frame: Frame) -> Self {
        frame.0
    }
}
