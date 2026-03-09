pub fn get_time() -> u64 {
    let mut time: usize;
    unsafe {
        core::arch::asm!("rdtime {}", out(reg) time);
    }
    time as u64
}
