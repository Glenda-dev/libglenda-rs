use crate::println_unsynced;
use core::arch::{asm, global_asm};

global_asm!(
    r#"
    .section .text.entry
    .globl _start
_start:
    # 1. 清空帧指针 (RBP)，标记栈回溯的终点
    xor rbp, rbp

    # 3. 栈对齐 (x86_64 要求函数调用前栈对齐到 16 字节)
    and rsp, -16

    # 4. 跳转到 Rust 初始化函数
    call glenda_start

    # 5. 如果返回，使用 ud2 触发非法指令异常 (比 int3 更确定的崩溃)
    int3
    "#
);

#[inline(always)]
pub unsafe fn syscall(cptr: usize, method: usize) -> usize {
    let mut ret;
    unsafe {
        asm!(
            "syscall",
            in("rax") cptr,
            in("rdi") method,
            lateout("rax") ret,
            out("rcx") _,
            out("r11") _,
        );
    }
    ret
}

#[inline(always)]
pub unsafe fn syscall_recv(cptr: usize, method: usize) -> (usize, usize) {
    let mut ret;
    let mut badge;
    unsafe {
        asm!(
            "syscall",
            in("rax") cptr,
            in("rdi") method,
            lateout("rax") ret,
            lateout("rsi") badge,
            out("rcx") _,
            out("r11") _,
        );
    }
    (ret, badge)
}

#[inline(always)]
pub unsafe fn panic_break() {
    unsafe {
        asm!("int3");
    }
}

#[inline(always)]
fn fp() -> usize {
    let mut fp: usize;
    unsafe {
        asm!("mov {}, rbp", out(reg) fp);
    }
    fp
}

pub fn backtrace() {
    println_unsynced!("--- GLENDA BACKTRACE START (x86_64) ---");
    let mut current_fp = fp();
    let mut depth = 0;
    while current_fp != 0 && depth < 20 {
        unsafe {
            // 基本对齐和范围检查
            if current_fp % 8 != 0 || current_fp < 0x1000 {
                break;
            }

            // x86_64 stack layout:
            // RBP -> [ Old RBP ]
            //        [ Return Address ] (RBP + 8)

            let prev_fp_ptr = current_fp as *const usize;
            let ra_ptr = (current_fp as *const usize).offset(1); // 注意这里是 +1

            let prev_fp = *prev_fp_ptr;
            let ra = *ra_ptr;

            println_unsynced!("{:>2}: fp={:#x} ra={:#x}", depth, current_fp, ra);

            // 栈向上增长检查 (x86 栈向下生长，所以 caller 的 frame 地址更大)
            if prev_fp != 0 && prev_fp <= current_fp {
                break;
            }
            current_fp = prev_fp;
        }
        depth += 1;
    }
    println_unsynced!("--- GLENDA BACKTRACE END ---");
}
