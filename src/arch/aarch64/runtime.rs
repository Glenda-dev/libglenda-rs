use crate::println;
use core::arch::{asm, global_asm};

#[cfg(not(feature = "no-start"))]
global_asm!(
    r#"
    .section .text.entry
    .globl _start
_start:
    mov x29, xzr
    bl glenda_start
    brk #0
    "#
);

#[inline(always)]
pub unsafe fn panic_break() {
    unsafe {
        asm!("brk #0");
    }
}

#[inline(always)]
fn fp() -> usize {
    let mut fp: usize;
    unsafe {
        asm!("mov {}, x29", out(reg) fp);
    }
    fp
}

pub fn backtrace() {
    println!("--- GLENDA BACKTRACE START (aarch64) ---");
    let mut current_fp = fp();
    let mut depth = 0;
    while current_fp != 0 && depth < 20 {
        unsafe {
            if current_fp % 16 != 0 || current_fp < 0x1000 {
                break;
            }
            let prev_fp = *(current_fp as *const usize);
            let ra = *((current_fp + 8) as *const usize);
            println!("{:>2}: fp={:#x} ra={:#x}", depth, current_fp, ra);
            if prev_fp != 0 && prev_fp <= current_fp {
                break;
            }
            current_fp = prev_fp;
        }
        depth += 1;
    }
    println!("--- GLENDA BACKTRACE END ---");
}
