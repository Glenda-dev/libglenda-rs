use crate::arch::runtime::panic_break;
use crate::cap::Console;
use core::fmt;
use core::fmt::Write;

const PRINT_BUF_SIZE: usize = 1024;

struct FixedBuf {
    buf: [u8; PRINT_BUF_SIZE],
    len: usize,
    truncated: bool,
}

impl FixedBuf {
    const fn new() -> Self {
        Self { buf: [0; PRINT_BUF_SIZE], len: 0, truncated: false }
    }

    fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.buf[..self.len]) }
    }
}

impl fmt::Write for FixedBuf {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let remaining = PRINT_BUF_SIZE.saturating_sub(self.len);
        if remaining == 0 {
            self.truncated = true;
            return Err(fmt::Error);
        }

        let mut copy_len = s.len().min(remaining);
        while copy_len > 0 && !s.is_char_boundary(copy_len) {
            copy_len -= 1;
        }

        self.buf[self.len..self.len + copy_len].copy_from_slice(&s.as_bytes()[..copy_len]);
        self.len += copy_len;

        if copy_len != s.len() {
            self.truncated = true;
            return Err(fmt::Error);
        }

        Ok(())
    }
}

pub struct KConsole(Console);

impl KConsole {
    pub const fn new(cap: Console) -> Self {
        Self(cap)
    }

    pub const fn null() -> Self {
        Self(Console::null())
    }

    pub fn initialize(&mut self, cap: Console) {
        self.0 = cap;
    }

    pub fn print(&mut self, args: fmt::Arguments) {
        let mut line = FixedBuf::new();
        let _ = line.write_fmt(args);

        match self.0.put_str(line.as_str()) {
            Ok(_) => {}
            Err(_) => unsafe { panic_break() },
        }
    }
}
