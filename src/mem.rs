use crate::arch::mem::{PGSIZE, USER_VA, VA_MAX};
use bitflags::bitflags;

pub const TRAMPOLINE_VA: usize = VA_MAX - PGSIZE; // Trampoline 映射地址
pub const TRAPFRAME_VA: usize = TRAMPOLINE_VA - PGSIZE; // Trapframe 映射地址
pub const UTCB_VA: usize = TRAPFRAME_VA - PGSIZE; // UTCB 映射地址 0x3FFFFFD000
pub const STACK_VA: usize = UTCB_VA - PGSIZE; // 用户栈映射地址
pub const HEAP_VA: usize = 0x2000_0000; // 用户堆地址
pub const ENTRY_VA: usize = USER_VA; // 用户程序入口地址

/*
用户地址空间布局：
trampoline  (1 page) 映射在最高地址
trapframe   (1 page)
UTCB        (1 page)
ustack      (N pages)
------------
BootInfo    (1 page)  0x40000000
Initrd      (N pages) 0x40001000
————————————
heap        (M pages) 0x20000000
-------------
code + data (N pages)
empty space (1 page) 最低的4096字节 不分配物理页，同时不可访问
*/

bitflags! {
    #[derive(Clone,Copy,Debug)]
    pub struct Perms: usize {
        const READ = 1 << 1;
        const WRITE = 1 << 2;
        const EXECUTE = 1 << 3;
        const USER = 1 << 4;
    }
}
