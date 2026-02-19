use core::arch::asm;

/// Memory fence for all instructions.
#[inline(always)]
pub fn fence() {}

/// Memory fence for I/O and memory.
#[inline(always)]
pub fn fence_io() {}

/// Fence for instruction cache.
#[inline(always)]
pub fn fence_i() {
    unsafe {
        asm!("cpuid", options(nostack));
    }
}
