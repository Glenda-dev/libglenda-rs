pub mod api;

#[cfg(target_arch = "riscv64")]
#[cfg(not(feature = "hosted"))]
pub mod riscv64;
#[cfg(target_arch = "riscv64")]
#[cfg(not(feature = "hosted"))]
pub use riscv64::*;

#[cfg(target_arch = "x86_64")]
#[cfg(not(feature = "hosted"))]
pub mod x86_64;
#[cfg(target_arch = "x86_64")]
#[cfg(not(feature = "hosted"))]
pub use x86_64::*;

#[cfg(feature = "hosted")]
pub mod hosted;
#[cfg(feature = "hosted")]
pub use hosted::*;
