use core::arch::asm;

#[inline(always)]
pub fn get_thread_pointer() -> usize {
    let tp: usize;
    unsafe {
        asm!("mrs {}, tpidr_el0", out(reg) tp);
    }
    tp
}

#[inline(always)]
pub fn set_thread_pointer(tp: usize) {
    unsafe {
        asm!("msr tpidr_el0, {}", in(reg) tp);
    }
}
