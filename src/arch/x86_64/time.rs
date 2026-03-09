pub fn get_time() -> usize {
    unsafe { core::arch::x86_64::_rdtsc() }
}
