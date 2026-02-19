use core::arch::asm;

/// Memory fence for all instructions.
#[inline(always)]
pub fn fence() {
    unsafe {
        asm!("fence", options(nostack));
    }
}

/// Memory fence for I/O and memory.
#[inline(always)]
pub fn fence_io() {
    unsafe {
        asm!("fence i, o", options(nostack));
    }
}

/// Fence for instruction cache.
#[inline(always)]
pub fn fence_i() {
    unsafe {
        asm!("fence.i", options(nostack));
    }
}
