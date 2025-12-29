#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PosixSyscall {
    Open = 1,
    Read = 2,
    Write = 3,
    Close = 4,
    Fork = 5,
    Exec = 6,
    Wait = 7,
    Exit = 8,
    GetPid = 9,
}
