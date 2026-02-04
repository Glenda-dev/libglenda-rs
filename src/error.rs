use core::mem::transmute;

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Success = 0,
    InvalidCap = 1,
    PermissionDenied = 2,
    InvalidEndpoint = 3,
    InvalidObjType = 4,
    InvalidMethod = 5,
    MappingFailed = 6,
    InvalidSlot = 7,
    OutOfMemory = 8,
    InvalidArgs = 9,
    InvalidProtocol = 10,
    InvalidParam = 11,
    CNodeFull = 12,
    NotSupported = 13,
    Timeout = 14,
    Interrupted = 15,
    Busy = 16,
    NotFound = 17,
    AlreadyExists = 18,
    Io = 19,
    BufferOverflow = 20,
    NotImplemented = 21,
    OutOfSlots = 22,
    NotInitialized = 23,
    DeviceError = 24,
    Unknown = 255,
}

impl From<usize> for Error {
    fn from(val: usize) -> Self {
        unsafe { transmute::<usize, Error>(val) }
    }
}
