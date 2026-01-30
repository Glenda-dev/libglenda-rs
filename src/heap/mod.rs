#[cfg(feature = "dynamic-heap")]
mod dynamic;
#[cfg(feature = "dynamic-heap")]
pub use dynamic::init;
#[cfg(all(feature = "fixed-heap", not(feature = "dynamic-heap")))]
mod fixed;
#[cfg(all(feature = "fixed-heap", not(feature = "dynamic-heap")))]
pub use fixed::init;
