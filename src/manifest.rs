use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServiceEntry {
    pub name: String,
    pub binary: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DriverEntry {
    pub name: String,
    pub binary: String,
    pub compatible: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestEntry {
    pub name: String,
    pub binary: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Manifest {
    pub services: Vec<ServiceEntry>,
    pub drivers: Vec<DriverEntry>,
    pub tests: Vec<TestEntry>,
}

impl Manifest {
    /// 从 JSON 字节切片解析 Manifest
    pub fn parse(data: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(data)
    }

    /// 从 JSON 字符串解析 Manifest
    pub fn from_str(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }
}
