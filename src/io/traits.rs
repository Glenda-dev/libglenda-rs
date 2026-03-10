use crate::error::Error;

/// Standard Seek from end/current/start.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeekFrom {
    Start(usize),
    End(isize),
    Current(isize),
}

/// A trait for objects that can be read from.
pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error>;
}

/// A trait for objects that can be written to.
pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error>;
    fn flush(&mut self) -> Result<(), Error>;
}

/// A trait for objects that support seeking.
pub trait Seek {
    fn seek(&mut self, pos: SeekFrom) -> Result<usize, Error>;
}
