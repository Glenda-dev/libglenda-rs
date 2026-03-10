pub fn get_time() -> usize {
    // `_rdtsc` returns a `u64` which we shrink to `usize`.  On 64‑bit targets
    // this is a no-op; on 32‑bit we drop the high half which is acceptable for
    // timer purposes.
    unsafe { core::arch::x86_64::_rdtsc() as usize }
}
