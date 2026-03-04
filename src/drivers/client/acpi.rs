use crate::cap::Endpoint;
use crate::drivers::interface::{AcpiDriver, DriverClient};
use crate::drivers::protocol::{ACPI_PROTO, acpi};
use crate::error::Error;
use crate::interface::{CSpaceService, VSpaceService};
use crate::ipc::{MsgFlags, MsgTag, UTCB};
use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub struct AcpiClient {
    endpoint: Endpoint,
}

impl DriverClient for AcpiClient {
    fn connect(
        &mut self,
        _vm: &mut dyn VSpaceService,
        _cm: &mut dyn CSpaceService,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

impl AcpiClient {
    pub const fn new(endpoint: Endpoint) -> Self {
        Self { endpoint }
    }
}

impl AcpiDriver for AcpiClient {
    fn evaluate_method(&mut self, path: &str, args: &[u64]) -> Result<Vec<u64>, Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();

        let args = (path.to_string(), Vec::from(args));

        unsafe { utcb.write_postcard::<(String, Vec<u64>)>(&args) }?;

        let tag = MsgTag::new(ACPI_PROTO, acpi::EVAL_METHOD, MsgFlags::HAS_BUFFER);
        utcb.set_msg_tag(tag);

        self.endpoint.call(&mut utcb)?;
        let results = unsafe { utcb.read_vec::<u64>().unwrap() };
        Ok(results)
    }
}
