use crate::error::Error;
use crate::ipc::Badge;

pub trait TimeService {
    fn time_now(&mut self, _badge: Badge) -> Result<u64, Error>;
    fn mono_now(&mut self, _badge: Badge) -> Result<u64, Error>;
    fn sleep(&mut self, _badge: Badge, ms: usize) -> Result<(), Error>;
    fn adj_time(&mut self, _badge: Badge, absolute_ns: u64, drift_ppb: i64) -> Result<(), Error>;
}
