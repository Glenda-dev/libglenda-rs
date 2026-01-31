pub mod bootinfo;
pub mod initrd;

use crate::arch::mem::PGSIZE;
use crate::arch::runtime::panic_break;
use crate::cap::{CNode, CapPtr, Frame, Kernel};
use crate::println;

pub const KERNEL_SLOT: CapPtr = CapPtr::from(5);
pub const UNTYPED_SLOT: CapPtr = CapPtr::from(7);
pub const MMIO_SLOT: CapPtr = CapPtr::from(8);
pub const IRQ_SLOT: CapPtr = CapPtr::from(9);
pub const PLATFORM_SLOT: CapPtr = CapPtr::from(6);
pub const UNTYPED_CAP: CNode = CNode::from(UNTYPED_SLOT);
pub const PLATFORM_CAP: Frame = Frame::from(PLATFORM_SLOT);
pub const MMIO_CAP: CNode = CNode::from(MMIO_SLOT);
pub const IRQ_CAP: CNode = CNode::from(IRQ_SLOT);
pub const KERNEL_CAP: Kernel = Kernel::from(KERNEL_SLOT);

pub const BOOTINFO_VA: usize = 0x4000_0000;
pub const INITRD_VA: usize = 0x5000_0000;

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
