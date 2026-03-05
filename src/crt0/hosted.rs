use std::ffi::{c_char, c_int};

// This is the entry point that the Linux dynamic linker/loader will call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn main(_argc: c_int, _argv: *const *const c_char) -> c_int {
    // 1. Initialize hosted shared memory (UTCB at 0x100000)
    crate::arch::hosted::init_utcb();

    // 2. Call the user's main
    unsafe extern "Rust" {
        fn main();
    }
    unsafe {
        main();
    }
    0
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
