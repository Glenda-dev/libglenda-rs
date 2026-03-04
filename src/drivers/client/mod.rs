// use crate::cap::{CapPtr, Endpoint, Frame};

pub mod acpi;
pub mod battery;
pub mod block;
pub mod fb;
pub mod input;
pub mod net;
pub mod pci;
pub mod platform;
pub mod thermal;
pub mod timer;
pub mod uart;

pub use crate::io::uring::RingParams;
pub use crate::mem::shm::ShmParams;
