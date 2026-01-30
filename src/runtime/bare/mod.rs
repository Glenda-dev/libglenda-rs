pub mod bootinfo;
pub mod initrd;

use crate::cap::{CNode, CapPtr, Frame};

pub const PLATFORM_SLOT: usize = 6;
pub const UNTYPED_SLOT: usize = 7;
pub const MMIO_SLOT: usize = 8;
pub const IRQ_SLOT: usize = 9;

pub const PLATFORM_CAP: Frame = Frame::from(CapPtr::new(PLATFORM_SLOT, 0));
pub const UNTYPED_CAP: CNode = CNode::from(CapPtr::new(UNTYPED_SLOT, 0));
pub const MMIO_CAP: CNode = CNode::from(CapPtr::new(MMIO_SLOT, 0));
pub const IRQ_CAP: CNode = CNode::from(CapPtr::new(IRQ_SLOT, 0));
