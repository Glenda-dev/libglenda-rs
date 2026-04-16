use super::CapPtr;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Page(CapPtr);

impl Page {
    pub const fn from(cap: CapPtr) -> Self {
        Self(cap)
    }

    pub fn cap(&self) -> CapPtr {
        self.0
    }
}

impl From<CapPtr> for Page {
    fn from(cap: CapPtr) -> Self {
        Self(cap)
    }
}

impl From<Page> for CapPtr {
    fn from(frame: Page) -> Self {
        frame.0
    }
}
