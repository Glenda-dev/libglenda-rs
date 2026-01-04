#![no_std]

extern crate alloc;

pub mod allocator;
pub mod bootinfo;
pub mod cap;
pub mod console;
pub mod crt0;
pub mod elf;
pub mod error;
pub mod initrd;
pub mod ipc;
pub mod manifest;
pub mod posix;
pub mod protocol;
pub mod syscall;

use console::{ANSI_RED, ANSI_RESET};

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        println!("{}PANIC{}: {}", ANSI_RED, ANSI_RESET, info);
        backtrace();
        loop {
            core::arch::asm!("wfi");
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
    println!("--- GLENDA BACKTRACE START ---");
    let mut current_fp = fp();
    let mut depth = 0;
    while current_fp != 0 && depth < 20 {
        // RISC-V standard frame pointer layout:
        // fp[-1] -> saved ra
        // fp[-2] -> saved fp
        unsafe {
            // Simple sanity check for frame pointer address
            if current_fp < 0x1000 || current_fp % 8 != 0 {
                break;
            }

            let ra_ptr = (current_fp as *const usize).offset(-1);
            let prev_fp_ptr = (current_fp as *const usize).offset(-2);

            let ra = *ra_ptr;
            let prev_fp = *prev_fp_ptr;

            println!("{:>2}: fp={:#x} ra={:#x}", depth, current_fp, ra);

            // In a typical stack, prev_fp should be higher than current_fp
            if prev_fp != 0 && prev_fp <= current_fp {
                // Potential stack corruption or end of stack
                // break;
            }

            current_fp = prev_fp;
        }
        depth += 1;
    }
    println!("--- GLENDA BACKTRACE END ---");
}
