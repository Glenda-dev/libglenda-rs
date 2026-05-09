pub fn get_time() -> usize {
    let value: usize;
    unsafe {
        core::arch::asm!("mrs {}, cntvct_el0", out(reg) value);
    }
    value
}
