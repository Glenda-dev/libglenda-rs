pub mod msg;
pub mod router;
pub mod server;
pub mod utcb;
#[macro_use]
pub mod macros;

pub use msg::{MsgFlags, MsgTag};
pub use router::{Handler, IpcRouter};
pub use utcb::{IPC_BUFFER_SIZE, MAX_MRS};
pub use utcb::{MsgArgs, UTCB};

use core::cmp::Ord;
use core::fmt::{Debug, Display};

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
#[repr(C)]
pub struct Badge(usize);

impl Badge {
    pub const fn new(badge: usize) -> Self {
        Self(badge)
    }

    pub const fn bits(&self) -> usize {
        self.0
    }

    pub const fn null() -> Self {
        Self(0)
    }
}

impl Debug for Badge {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for Badge {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}
