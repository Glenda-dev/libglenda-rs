pub fn get_time() -> u64 {
    unsafe { core::arch::x86_64::_rdtsc() }
}
