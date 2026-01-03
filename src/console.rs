use crate::cap::{CapPtr, method::consolemethod};
use crate::ipc::utcb;
use core::fmt;
use spin::Mutex;

pub struct Console {
    cap: Option<CapPtr>,
}

impl Console {
    pub const fn new() -> Self {
        Self { cap: None }
    }

    pub fn init(&mut self, cap: CapPtr) {
        self.cap = Some(cap);
    }
}

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if let Some(cap) = self.cap {
            utcb::get().append_str(s);
            // Invoke syscall: PUT_STR(offset, len)
            cap.invoke(consolemethod::PUT_STR, [0, s.len(), 0, 0, 0, 0, 0]);
        }
        Ok(())
    }
}

pub static GLOBAL_CONSOLE: Mutex<Console> = Mutex::new(Console::new());

pub fn init(cap: CapPtr) {
    GLOBAL_CONSOLE.lock().init(cap);
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    GLOBAL_CONSOLE.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
