use crate::cap::CapPtr;
use crate::error::Error;

pub trait GeneralService {
    fn ping(&mut self, value: usize) -> Result<usize, Error>;
    fn share_memory(&mut self, cap: CapPtr) -> Result<(), Error>;
    fn send_message(&mut self, message: &str) -> Result<(), Error>;
}
