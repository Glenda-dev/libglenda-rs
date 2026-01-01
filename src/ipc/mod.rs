pub mod utcb;

pub use utcb::UTCB;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct MsgTag(pub usize);

impl MsgTag {
    pub const FLAG_HAS_CAP: usize = 1 << 4;

    pub fn new(label: usize, length: usize) -> Self {
        Self((label << 16) | (length & 0xF))
    }

    pub fn label(&self) -> usize {
        self.0 >> 16
    }

    pub fn length(&self) -> usize {
        self.0 & 0xF
    }

    pub fn has_cap(&self) -> bool {
        (self.0 & Self::FLAG_HAS_CAP) != 0
    }

    pub fn set_has_cap(&mut self) {
        self.0 |= Self::FLAG_HAS_CAP;
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }
}
