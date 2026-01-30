pub mod code {
    pub const SUCCESS: usize = 0;
    pub const INVALID_CAP: usize = 1;
    pub const PERMISSION_DENIED: usize = 2;
    pub const INVALID_ENDPOINT: usize = 3;
    pub const INVALID_OBJ_TYPE: usize = 4;
    pub const INVALID_METHOD: usize = 5;
    pub const MAPPING_FAILED: usize = 6;
    pub const INVALID_SLOT: usize = 7;
    pub const UNTYPE_OOM: usize = 8;
    pub const INVALID_ARGS: usize = 9;
}

#[repr(usize)]
pub enum Error {
    Success = code::SUCCESS,
    InvalidCap = code::INVALID_CAP,
    PermissionDenied = code::PERMISSION_DENIED,
    InvalidEndpoint = code::INVALID_ENDPOINT,
    InvalidObjType = code::INVALID_OBJ_TYPE,
    InvalidMethod = code::INVALID_METHOD,
    MappingFailed = code::MAPPING_FAILED,
    InvalidSlot = code::INVALID_SLOT,
    UntypeOOM = code::UNTYPE_OOM,
    InvalidArgs = code::INVALID_ARGS,
    InvalidParam,
}
