use crate::cap::Endpoint;
use crate::error::Error;
use crate::interface::{CSpaceService, VSpaceService};
use crate::ipc::{MsgFlags, MsgTag, UTCB};
use crate::drivers::interface::{ThermalDriver, DriverClient};
use crate::drivers::protocol::{THERMAL_PROTO, thermal};

pub struct ThermalClient {
    endpoint: Endpoint,
}

impl DriverClient for ThermalClient {
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

impl ThermalClient {
    pub const fn new(endpoint: Endpoint) -> Self {
        Self { endpoint }
    }
}

impl ThermalDriver for ThermalClient {
    fn get_temperature(&self, zone: u32) -> Result<u32, Error> {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        let tag = MsgTag::new(THERMAL_PROTO, thermal::GET_TEMPERATURE, MsgFlags::NONE);
        utcb.set_msg_tag(tag);
        utcb.set_mr(0, zone as usize);

        self.endpoint.call(&mut utcb)?;
        Ok(utcb.get_mr(0) as u32)
    }
}
