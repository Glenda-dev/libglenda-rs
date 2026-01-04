use crate::allocator;
use crate::console::{ANSI_RED, ANSI_RESET};
use crate::mem::{HEAP_SIZE, HEAP_VA};
use crate::println;
use core::arch::global_asm;

global_asm!(
    r#"
    .section .text.entry
    .globl _start
_start:
    li s0, 0
    # 4. 跳转到 Rust 初始化函数
    call glenda_start

    # 5. 如果返回，死循环
    ebreak
    "#
);

#[unsafe(no_mangle)]
unsafe extern "C" fn glenda_start() -> ! {
    unsafe extern "Rust" {
        fn main() -> usize;
    }

    allocator::init_heap(HEAP_VA, HEAP_SIZE);
    let ret = unsafe { main() };

    exit(ret)
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe { crate::console::force_unlock() };
    println!("{}PANIC{}: {}", ANSI_RED, ANSI_RESET, info);
    backtrace();
    unsafe {
        loop {
            core::arch::asm!("ebreak");
        }
    }
}

#[inline(always)]
fn fp() -> usize {
    let mut fp: usize;
    unsafe {
        core::arch::asm!("mv {}, s0", out(reg) fp);
    }
    fp
}
fn backtrace() {
    unsafe { crate::console::force_unlock() };
    println!("--- GLENDA BACKTRACE START ---");
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

            println!("{:>2}: fp={:#x} ra={:#x}", depth, current_fp, ra);

            // 3. 栈增长方向检查：RISC-V 栈向下增长，prev_fp 必须大于 current_fp
            // 如果 prev_fp <= current_fp，说明栈已损坏或出现环路，必须停止
            if prev_fp != 0 && prev_fp <= current_fp {
                break;
            }

            current_fp = prev_fp;
        }
        depth += 1;
    }
    println!("--- GLENDA BACKTRACE END ---");
}

fn exit(exitcode: usize) -> ! {
    println!("Program Exited with code {}", exitcode);
    unimplemented!()
}
