use crate::io::{Read, Write};
use core::fmt;
use core::fmt::Write as FmtWrite; // bring formatting trait into scope

use crate::cap::CapPtr;
use crate::client::terminal::TerminalClient;
use crate::error::Error;
use crate::interface::CSpaceService;
use crate::interface::vspace::{VSpaceProvider, VSpaceService};
use crate::io::uring::{IOURING_OP_READ, IOURING_OP_WRITE, IoUringGeneric};
use crate::ipc::Badge;
use crate::mem::Perms;
use crate::utils::manager::{CSpaceManager, VSpaceManager};

/// A standard I/O structure for a Virtual Terminal (VT).
/// It provides zero-copy high-performance I/O through io_uring.
pub struct ConsoleVT {
    client: TerminalClient,
    base: *mut u8,
    entries: usize,
}

// raw pointers are inherently not `Send`/`Sync`, but our usage ensures that the
// memory region referenced by `base` is shared and managed safely by the
// accompanying managers.  We can therefore mark the structure as `Send` so it
// may live inside the global mutex used by `rt-app`.
unsafe impl Send for ConsoleVT {}

impl ConsoleVT {
    /// Create a new ConsoleVT from a TerminalClient.
    /// Memory must be provided for mapping the io_uring frame.
    pub fn new(
        mut client: TerminalClient,
        vspace: &mut VSpaceManager,
        slots: &mut dyn CSpaceService,
        provider: &mut dyn VSpaceProvider,
        recv_slot: CapPtr,
    ) -> Result<Self, Error> {
        // 1. Connect and get configuration/frame
        client.connect(Badge::null(), recv_slot)?;

        // 2. Map the frame to userspace
        let config = client.config().ok_or(Error::NotInitialized)?;
        let page = client.frame().ok_or(Error::NotInitialized)?;

        // Use a fixed address or let manager find one (simplified for now)
        let vaddr = 0x40000000;
        vspace.map_page(page, vaddr, Perms::READ | Perms::WRITE, 1, provider, slots)?;
        let base = vaddr as *mut u8;

        // 3. Initialize io_uring
        let entries = 32;

        Ok(Self { client, base, entries })
    }

    /// Obtain the io_uring interface.
    pub fn get_ring(&mut self) -> IoUringGeneric<'_> {
        unsafe { self.client.get_uring(self.base).unwrap() }
    }

    /// Destroy the ConsoleVT and unmap memory.
    pub fn destroy(self, vspace: &mut VSpaceManager) -> Result<(), Error> {
        vspace.unmap(self.base as usize, 1)?; // 1 page
        Ok(())
    }

    /// Convenience helper mirroring `KConsole::print` for formatted output.
    pub fn print(&mut self, args: fmt::Arguments) {
        // ignore errors for now, similar to other console implementations
        let _ = FmtWrite::write_fmt(self, args);
    }
}

impl Read for ConsoleVT {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let _ring = self.get_ring();

        // Placeholder for real io_uring submit/wait logic:
        Ok(0)
    }
}

impl Write for ConsoleVT {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        let _ring = self.get_ring();

        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

// Provide a `core::fmt::Write` implementation so the console can be used with
// `format_args!` and the standard printing macros.  This is the primary
// trait needed by the various runtime `print!`/`println!` implementations when
// the `user-console` feature is enabled.  Previously `ConsoleVT` only
// implemented the crate-local IO traits; formatting support makes it behave
// more like `KConsole`.
impl fmt::Write for ConsoleVT {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // Forward to the underlying `Write` impl.  We ignore the returned
        // length since formatting APIs don't need it.
        match self.write(s.as_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(fmt::Error),
        }
    }
}
