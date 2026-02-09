use crate::error::Error;
use crate::ipc::Badge;
use crate::protocol::device::DeviceNode;
use crate::protocol::device::fb::FbInfo;
use crate::protocol::device::input::InputEvent;
use crate::protocol::device::net::MacAddress;
use crate::protocol::device::pci::PciAddress;
use crate::protocol::device::usb::UsbSetupPacket;
use crate::protocol::device::wifi::WifiApInfo;
use crate::utils::platform::PlatformInfo;
use alloc::string::String;

/// DeviceService provides hardware discovery and management.
pub trait DeviceService {
    fn scan_platform(&mut self, badge: Badge, info: &PlatformInfo) -> Result<(), Error>;
    fn find_compatible(&self, badge: Badge, compat: String) -> Result<DeviceNode, Error>;
}

/// DmaService provides DMA-safe memory allocation.
pub trait DmaService {
    fn alloc_dma(&mut self, size: usize) -> Result<usize, Error>;
    fn free_dma(&mut self, paddr: usize, size: usize);
}

/// PciDevice provides PCI config space access.
pub trait PciDevice {
    fn read_config(&self, offset: usize, size: usize) -> Result<u32, Error>;
    fn write_config(&self, offset: usize, value: u32, size: usize) -> Result<(), Error>;
    fn enable_bus_master(&self) -> Result<(), Error>;
    fn enable_msi(&self, vector: u8, dest_id: u32) -> Result<(), Error>;
    fn get_address(&self) -> PciAddress;
}

/// BlockDevice provides block-level access to storage.
pub trait BlockDevice {
    fn capacity(&self) -> u64;
    fn block_size(&self) -> u32;
    fn read_blocks(&mut self, sector: u64, buf: &mut [u8]) -> Result<usize, Error>;
    fn write_blocks(&mut self, sector: u64, buf: &[u8]) -> Result<usize, Error>;
    fn sync(&mut self) -> Result<(), Error>;
}

/// NetDevice provides network packet transmission.
pub trait NetDevice {
    fn mac_address(&self) -> MacAddress;
    fn send(&mut self, buf: &[u8]) -> Result<(), Error>;
    fn recv(&mut self, buf: &mut [u8]) -> Result<usize, Error>;
}

/// UartDevice provides serial communication.
pub trait UartDevice {
    fn put_char(&mut self, c: u8);
    fn get_char(&mut self) -> Option<u8>;
    fn put_str(&mut self, s: &str);
}

/// WifiDevice provides wireless network management.
pub trait WifiDevice {
    fn scan(&mut self) -> Result<(), Error>;
    fn get_scan_results(&self, buf: &mut [WifiApInfo]) -> Result<usize, Error>;
    fn connect(&mut self, ssid: &str, password: &str, security: u8) -> Result<(), Error>;
    fn disconnect(&mut self) -> Result<(), Error>;
    fn status(&self) -> Result<u8, Error>;
}

/// InputDevice provides HID events (keyboard, mouse).
pub trait InputDevice {
    fn poll_event(&mut self) -> Option<InputEvent>;
}

/// FrameBufferDevice provides display management.
pub trait FrameBufferDevice {
    fn get_info(&self) -> FbInfo;
    fn flush(&mut self, x: u32, y: u32, w: u32, h: u32) -> Result<(), Error>;
}

/// UsbHostDevice provides USB bus management.
pub trait UsbHostDevice {
    /// Send a control packet (Setup stage + optional Data stage)
    fn control_transfer(
        &mut self,
        addr: u8,
        ep: u8,
        setup: UsbSetupPacket,
        data: &mut [u8],
    ) -> Result<usize, Error>;

    /// Perform a bulk transfer
    fn bulk_transfer(&mut self, addr: u8, ep: u8, data: &mut [u8]) -> Result<usize, Error>;

    /// Reset a root hub port
    fn reset_port(&mut self, port: u8) -> Result<(), Error>;
}

/// GpioDevice provides pin control.
pub trait GpioDevice {
    fn set_mode(&mut self, pin: u32, mode: u8) -> Result<(), Error>;
    fn write(&mut self, pin: u32, value: bool) -> Result<(), Error>;
    fn read(&self, pin: u32) -> Result<bool, Error>;
}

/// RngDevice provides random numbers.
pub trait RngDevice {
    fn get_random_bytes(&mut self, buf: &mut [u8]) -> Result<usize, Error>;
}

/// SpiDevice provides SPI bus access.
pub trait SpiDevice {
    /// Full-duplex transfer. Data in place modification.
    fn transfer(&mut self, buf: &mut [u8]) -> Result<(), Error>;
    fn send(&mut self, buf: &[u8]) -> Result<(), Error>;
    fn recv(&mut self, buf: &mut [u8]) -> Result<(), Error>;
}

/// I2cDevice provides I2C bus access.
pub trait I2cDevice {
    fn read(&mut self, addr: u16, buf: &mut [u8]) -> Result<(), Error>;
    fn write(&mut self, addr: u16, buf: &[u8]) -> Result<(), Error>;
    /// Write command then read response (Atomic repeated start if supported)
    fn write_read(&mut self, addr: u16, w_buf: &[u8], r_buf: &mut [u8]) -> Result<(), Error>;
}

/// IommuDevice provides DMA remapping.
pub trait IommuDevice {
    fn map(&mut self, iova: usize, paddr: usize, size: usize, flags: u32) -> Result<(), Error>;
    fn unmap(&mut self, iova: usize, size: usize) -> Result<(), Error>;
    fn flush(&mut self) -> Result<(), Error>;
}
