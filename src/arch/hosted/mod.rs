pub mod mem;
pub mod runtime;
pub mod sync;
pub mod syscall;
pub mod thread;
pub mod time;

pub use runtime::crt0_init as init_utcb;
pub use runtime::crt0_init;
pub use syscall::syscall;

use std::io::Write;

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    let mut stdout = std::io::stdout();
    let _ = stdout.write_fmt(args);
    let _ = stdout.flush();
}
