use crate::arch::mem::{PGSIZE, USER_VA, VA_MAX};

pub const TRAMPOLINE_VA: usize = VA_MAX - PGSIZE; // Trampoline 映射地址

pub const STACK_BASE: usize = TRAMPOLINE_VA; // 用户栈最高地址（起始地址，向低地址生长）
pub const STACK_PAGES: usize = 32;
pub const STACK_SIZE: usize = STACK_PAGES * PGSIZE;
pub const ENTRY_VA: usize = USER_VA; // 用户程序入口地址

#[cfg(target_pointer_width = "64")]
pub const THREAD_AREA_BASE: usize = 0x3F_0000_0000;
#[cfg(target_pointer_width = "32")]
pub const THREAD_AREA_BASE: usize = 0x7F_0000_00;

pub const fn get_utcb_va(tid: usize) -> usize {
    THREAD_AREA_BASE + tid * 2 * PGSIZE
}

pub const fn get_trapframe_va(tid: usize) -> usize {
    THREAD_AREA_BASE + tid * 2 * PGSIZE + PGSIZE
}

pub const HEAP_VA: usize = 0x2000_0000; // 用户堆地址
pub const BOOTINFO_VA: usize = 0x4000_0000;
pub const INITRD_VA: usize = 0x5000_0000;
pub const HEAP_PAGES: usize = 256; // 用户堆页面数 256 * 4KB = 1MB
pub const HEAP_SIZE: usize = HEAP_PAGES * PGSIZE; // 1MB

pub mod allocator;
pub mod pool;
pub mod ringbuf;
pub mod shm;

/*
用户地址空间布局：
trampoline  (1 page) 映射在最高地址
trapframe   (1 page)
UTCB        (1 page)
ustack      (N pages)
------------
BootInfo    (1 page)  0x40000000
Initrd      (N pages) 0x50000000
————————————
heap        (M pages) 0x20000000
-------------
code + data (N pages)
empty space (1 page) 最低的4096字节 不分配物理页，同时不可访问
*/

bitflags::bitflags! {
    #[derive(Clone,Copy,Debug)]
    pub struct Perms: usize {
        const READ = 1 << 1;
        const WRITE = 1 << 2;
        const EXECUTE = 1 << 3;
        const SUPERVISOR = 1 << 4; // 设置该位则用户不可读
        const DEVICE = 1 << 5;
        const FRAMEBUFFER = 1 << 6;
        const GLOBAL = 1 << 7;
    }
}
