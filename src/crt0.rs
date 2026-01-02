use crate::allocator;
use core::arch::global_asm;

global_asm!(
    r#"
    .section .text.entry
    .globl _start
_start:

    # 2. 清零 .bss 段
    la t0, __bss_start
    la t1, __bss_end
    bge t0, t1, 2f
1:
    sd zero, (t0)
    addi t0, t0, 8
    blt t0, t1, 1b
2:
    
    # 4. 跳转到 Rust 初始化函数
    call glenda_start

    # 5. 如果返回，死循环
    unimp
    "#
);

pub const HEAP_VA: usize = 0x2000_0000;
pub const HEAP_SIZE: usize = 1024 * 1024; // 1 MB

#[unsafe(no_mangle)]
unsafe extern "C" fn glenda_start() -> ! {
    unsafe extern "Rust" {
        fn main() -> !;
    }

    // TODO: 使用 bootinfo 初始化堆
    allocator::init_heap(HEAP_VA, HEAP_SIZE);

    unsafe {
        main();
    }
}
