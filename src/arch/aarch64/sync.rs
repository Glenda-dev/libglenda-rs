use core::arch::asm;

#[inline(always)]
pub fn fence() {
    unsafe {
        asm!("dmb ish", options(nostack));
    }
}

#[inline(always)]
pub fn fence_io() {
    unsafe {
        asm!("dsb sy", options(nostack));
    }
}

#[inline(always)]
pub fn fence_i() {
    unsafe {
        asm!("isb", options(nostack));
    }
}
