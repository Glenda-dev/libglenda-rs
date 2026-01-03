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
    pub compatible: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Manifest {
    #[serde(default)]
    pub service: Vec<ServiceEntry>,
    #[serde(default)]
    pub driver: Vec<DriverEntry>,
}

impl Manifest {
    pub fn parse(data: &[u8]) -> Self {
        let mut service = Vec::new();
        let mut driver = Vec::new();
        let content = core::str::from_utf8(data).unwrap_or("");

        let mut current_name = String::new();
        let mut current_binary = String::new();
        let mut current_compatible = None;
        let mut in_service = false;
        let mut in_driver = false;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.starts_with("[[") {
                // Save previous entry
                if !current_name.is_empty() && !current_binary.is_empty() {
                    if in_service {
                        service.push(ServiceEntry {
                            name: current_name.clone(),
                            binary: current_binary.clone(),
                        });
                    } else if in_driver {
                        driver.push(DriverEntry {
                            name: current_name.clone(),
                            binary: current_binary.clone(),
                            compatible: current_compatible.clone(),
                        });
                    }
                }
                current_name.clear();
                current_binary.clear();
                current_compatible = None;

                if line.contains("service") {
                    in_service = true;
                    in_driver = false;
                } else if line.contains("driver") {
                    in_service = false;
                    in_driver = true;
                }
                continue;
            }

            if let Some(idx) = line.find('=') {
                let key = line[..idx].trim();
                let value =
                    line[idx + 1..].trim().trim_matches('"').trim_matches('[').trim_matches(']');

                match key {
                    "name" => current_name = String::from(value),
                    "binary" => current_binary = String::from(value),
                    "compatible" => current_compatible = Some(String::from(value)),
                    _ => {}
                }
            }
        }

        // Save last entry
        if !current_name.is_empty() && !current_binary.is_empty() {
            if in_service {
                service.push(ServiceEntry { name: current_name, binary: current_binary });
            } else if in_driver {
                driver.push(DriverEntry {
                    name: current_name,
                    binary: current_binary,
                    compatible: current_compatible,
                });
            }
        }

        Self { service, driver }
    }
}
