#[cfg(not(feature = "fixed-heap"))]
mod dynamic;
#[cfg(not(feature = "fixed-heap"))]
pub use dynamic::init;
#[cfg(feature = "fixed-heap")]
mod fixed;
#[cfg(feature = "fixed-heap")]
pub use fixed::init;
