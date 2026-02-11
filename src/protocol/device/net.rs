#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct MacAddress {
    pub octets: [u8; 6],
}
