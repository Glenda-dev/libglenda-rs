use crate::println_unsynced;
use core::arch::{asm, global_asm};

global_asm!(
    r#"
    .section .text.entry
    .globl _start
_start:
    .option push
    .option norelax
    la gp, __global_pointer$
    .option pop
    li s0, 0
    # 4. 跳转到 Rust 初始化函数
    call glenda_start

    # 5. 如果返回，死循环
    ebreak
    "#
);

#[inline(always)]
pub unsafe fn panic_break() {
    unsafe {
        asm!("ebreak");
    }
}

#[inline(always)]
fn fp() -> usize {
    let mut fp: usize;
    unsafe {
        asm!("mv {}, s0", out(reg) fp);
    }
    fp
}

pub fn backtrace() {
    println_unsynced!("--- GLENDA BACKTRACE START ---");
    let mut current_fp = fp();
    let mut depth = 0;
    while current_fp != 0 && depth < 20 {
        unsafe {
            // 1. 严格检查对齐和地址范围，防止解引用非法地址导致二次 Trap
            // 假设用户态有效地址空间在 0x1000 到 0x3fffffffff 之间
            if current_fp % 8 != 0 || current_fp < 0x1000 || current_fp > 0x3fffffffff {
                break;
            }

            let ra_ptr = (current_fp as *const usize).offset(-1);
            let prev_fp_ptr = (current_fp as *const usize).offset(-2);

            // 2. 检查指针本身是否在合理范围内
            if (ra_ptr as usize) < 0x1000 || (prev_fp_ptr as usize) < 0x1000 {
                break;
            }

            let ra = *ra_ptr;
            let prev_fp = *prev_fp_ptr;

            println_unsynced!("{:>2}: fp={:#x} ra={:#x}", depth, current_fp, ra);

            // 3. 栈增长方向检查：RISC-V 栈向下增长，prev_fp 必须大于 current_fp
            // 如果 prev_fp <= current_fp，说明栈已损坏或出现环路，必须停止
            if prev_fp != 0 && prev_fp <= current_fp {
                break;
            }

            current_fp = prev_fp;
        }
        depth += 1;
    }
    println_unsynced!("--- GLENDA BACKTRACE END ---");
}
