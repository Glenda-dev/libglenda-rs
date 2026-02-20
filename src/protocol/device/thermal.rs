use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThermalType {
    Cpu,
    Gpu,
    Board,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TripType {
    Passive,
    Active,
    Hot,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalTrip {
    pub temp: u32,       // Kelvin/10
    pub hysteresis: u32, // Kelvin/10
    pub trip_type: TripType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalZoneInfo {
    pub name: String,
    pub thermal_type: ThermalType,
    pub trips: Vec<ThermalTrip>,
    pub sensor_id: usize, // Identifier within the reporting driver
    pub driver_logic_id: usize, // Logic device ID of the driver reported to Unicorn
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThermalZones {
    pub zones: Vec<ThermalZoneInfo>,
}
