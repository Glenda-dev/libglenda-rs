use crate::cap::Endpoint;
use crate::error::Error;
use crate::interface::device::PciDevice;
use crate::ipc::{MsgFlags, MsgTag, UTCB};
use crate::protocol::device::pci::PciAddress;
use crate::protocol::device::{PCI_PROTO, pci};

pub struct PciClient {
    endpoint: Endpoint,
    address: PciAddress,
}

impl PciClient {
    pub const fn new(endpoint: Endpoint, address: PciAddress) -> Self {
        Self { endpoint, address }
    }
}

impl PciDevice for PciClient {
    fn read_config(&self, offset: usize, size: usize) -> Result<u32, Error> {
        let utcb = unsafe { UTCB::get() };
        let tag = MsgTag::new(PCI_PROTO, pci::READ_CONFIG, MsgFlags::NONE);
        utcb.mrs_regs[0] = offset;
        utcb.mrs_regs[1] = size;

        self.endpoint.call(tag)?;

        Ok(utcb.mrs_regs[0] as u32)
    }

    fn write_config(&self, offset: usize, value: u32, size: usize) -> Result<(), Error> {
        let utcb = unsafe { UTCB::get() };
        let tag = MsgTag::new(PCI_PROTO, pci::WRITE_CONFIG, MsgFlags::NONE);
        utcb.mrs_regs[0] = offset;
        utcb.mrs_regs[1] = value as usize;
        utcb.mrs_regs[2] = size;

        self.endpoint.call(tag)
    }

    fn enable_bus_master(&self) -> Result<(), Error> {
        let tag = MsgTag::new(PCI_PROTO, pci::ENABLE_BUS_MASTER, MsgFlags::NONE);
        self.endpoint.call(tag)
    }

    fn enable_msi(&self, vector: u8, dest_id: u32) -> Result<(), Error> {
        let utcb = unsafe { UTCB::get() };
        let tag = MsgTag::new(PCI_PROTO, pci::ENABLE_MSI, MsgFlags::NONE);
        utcb.mrs_regs[0] = vector as usize;
        utcb.mrs_regs[1] = dest_id as usize;

        self.endpoint.call(tag)
    }

    fn get_address(&self) -> PciAddress {
        self.address
    }
}
