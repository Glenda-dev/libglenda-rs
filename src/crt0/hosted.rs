use std::ffi::{c_char, c_int};

pub fn crt0_init() {
    crate::arch::hosted::crt0_init();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::arch::hosted::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {
        $crate::arch::hosted::_print(format_args!("{}\n", format_args!($($arg)*)))
    };
}
