
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct BlockRequest {
    pub sector: u64,
    pub count: u32,
    pub flags: u32,
}
