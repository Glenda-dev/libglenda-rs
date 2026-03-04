#[cfg(feature = "rt-bare")]
pub mod bare;

#[cfg(feature = "rt-service")]
pub mod service;

#[cfg(all(feature = "rt-bare", not(feature = "rt-service")))]
pub use bare::*;

#[cfg(feature = "rt-service")]
pub use service::*;
