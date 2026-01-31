#![allow(dead_code)]

mod bare;
mod service;
#[cfg(feature = "rt-bare")]
pub use bare::*;
#[cfg(feature = "rt-service")]
pub use service::*;
