use bitflags::bitflags;
use core::fmt::Display;

use crate::protocol;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct MsgTag(pub usize);

impl MsgTag {
    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn new(proto: usize, label: usize, flags: MsgFlags) -> Self {
        Self((proto & 0xFFFF) << 24 | (label & 0xFFFF) << 16 | (flags.bits() & 0xFFFF))
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }

    pub fn proto(&self) -> usize {
        (self.0 >> 24) & 0xFFFF
    }

    pub fn label(&self) -> usize {
        (self.0 >> 16) & 0xFFFF
    }

    pub fn flags(&self) -> MsgFlags {
        MsgFlags::from_bits_truncate(self.0 & 0xFFFF)
    }

    pub const fn ok() -> Self {
        Self::new(protocol::GENERIC_PROTO, protocol::generic::REPLY, MsgFlags::OK)
    }

    pub const fn err() -> Self {
        Self::new(protocol::GENERIC_PROTO, protocol::generic::REPLY, MsgFlags::ERROR)
    }
}

bitflags! {
    #[derive(Clone,Copy)]
    pub struct MsgFlags: usize {
        const NONE = 0;
        const OK = 1 << 0;
        const ERROR = 1 << 1;
        const HAS_CAP = 1 << 2;
        const HAS_BUFFER = 1 << 3;
    }
}

impl Display for MsgFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut first = true;
        let perms = [
            (MsgFlags::OK, "OK"),
            (MsgFlags::ERROR, "ERROR"),
            (MsgFlags::HAS_CAP, "HAS_CAP"),
            (MsgFlags::HAS_BUFFER, "HAS_BUFFER"),
        ];
        for (bit, name) in perms.iter() {
            if self.contains(*bit) {
                if !first {
                    write!(f, "|")?;
                }
                write!(f, "{}", name)?;
                first = false;
            }
        }
        if first {
            write!(f, "NONE")?;
        }
        Ok(())
    }
}
