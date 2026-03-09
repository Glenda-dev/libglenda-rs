use core::arch::asm;

#[inline(always)]
pub fn get_thread_pointer() -> usize {
    let tp: usize;
    unsafe {
        asm!("mv {}, tp", out(reg) tp);
    }
    tp
}

#[inline(always)]
pub fn set_thread_pointer(tp: usize) {
    unsafe {
        asm!("mv tp, {}", in(reg) tp);
    }
}
