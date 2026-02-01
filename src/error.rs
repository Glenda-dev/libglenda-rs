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
    UntypeOOM = 8,
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
    Unknown = 255,
}

impl From<usize> for Error {
    fn from(val: usize) -> Self {
        match val {
            0 => Error::Success,
            1 => Error::InvalidCap,
            2 => Error::PermissionDenied,
            3 => Error::InvalidEndpoint,
            4 => Error::InvalidObjType,
            5 => Error::InvalidMethod,
            6 => Error::MappingFailed,
            7 => Error::InvalidSlot,
            8 => Error::UntypeOOM,
            9 => Error::InvalidArgs,
            10 => Error::InvalidProtocol,
            11 => Error::InvalidParam,
            12 => Error::CNodeFull,
            13 => Error::NotSupported,
            14 => Error::Timeout,
            15 => Error::Interrupted,
            16 => Error::Busy,
            17 => Error::NotFound,
            18 => Error::AlreadyExists,
            19 => Error::Io,
            20 => Error::BufferOverflow,
            21 => Error::NotImplemented,
            _ => Error::Unknown,
        }
    }
}
