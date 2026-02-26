pub fn get_time() -> u64 {
    let mut time: u64;
    unsafe {
        core::arch::asm!("rdtime {}", out(reg) time);
    }
    time
}
