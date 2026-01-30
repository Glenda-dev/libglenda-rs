pub mod bootinfo;
pub mod initrd;

use crate::arch::mem::PGSIZE;
use crate::arch::runtime::panic_break;
use crate::cap::{CNode, CapPtr, Frame};
use crate::mem::RES_VA_BASE;
use crate::println;

pub const PLATFORM_SLOT: usize = 6;
pub const UNTYPED_SLOT: usize = 7;
pub const MMIO_SLOT: usize = 8;
pub const IRQ_SLOT: usize = 9;

pub const PLATFORM_CAP: Frame = Frame::from(CapPtr::new(PLATFORM_SLOT));
pub const UNTYPED_CAP: CNode = CNode::from(CapPtr::new(UNTYPED_SLOT));
pub const MMIO_CAP: CNode = CNode::from(CapPtr::new(MMIO_SLOT));
pub const IRQ_CAP: CNode = CNode::from(CapPtr::new(IRQ_SLOT));

pub const BOOTINFO_VA: usize = RES_VA_BASE;
pub const INITRD_VA: usize = BOOTINFO_VA + PGSIZE;

pub const STACK_PAGES: usize = 16; // 用户栈页面数 16 * 4KB = 64KB
pub const STACK_SIZE: usize = STACK_PAGES * PGSIZE; // 64KB
pub const HEAP_PAGES: usize = 64; // 用户堆页面数 64 * 4KB = 256KB
pub const HEAP_SIZE: usize = HEAP_PAGES * PGSIZE; // 256KB

pub fn exit(code: usize) -> ! {
    println!("Program exited with code: {}\n", code);
    unsafe {
        loop {
            panic_break();
        }
    }
}
