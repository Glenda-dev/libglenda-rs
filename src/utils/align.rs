pub const fn align_up(value: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two(), "align must be power of two");
    (value + align - 1) & !(align - 1)
}

pub const fn align_down(value: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two(), "align must be power of two");
    value & !(align - 1)
}
