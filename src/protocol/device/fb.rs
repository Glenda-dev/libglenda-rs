#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct FbInfo {
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub format: u32,
    pub bpp: u32,
    pub paddr: usize,
    pub size: usize,
}
