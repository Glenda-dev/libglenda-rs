use crate::cap::{CapPtr, Endpoint};
use crate::error::Error;
use crate::interface::{FileHandleService, FileSystemService, PipeService, SystemClient};
use crate::ipc::{MsgFlags, MsgTag, UTCB};
use crate::protocol::{
    FS_PROTO,
    fs::{self, OpenFlags, Stat},
};
use alloc::vec::Vec;

pub struct FsClient {
    endpoint: Endpoint,
}

impl FsClient {
    pub const fn new(endpoint: Endpoint) -> Self {
        Self { endpoint }
    }
}

impl SystemClient for FsClient {
    fn connect(&mut self, ep: Endpoint, _reply: CapPtr) -> Result<(), Error> {
        self.endpoint = ep;
        Ok(())
    }

    fn disconnect(&mut self) {}

    fn send(&mut self, info: MsgTag) -> Result<(), Error> {
        self.endpoint.send(info)
    }
}

impl PipeService for FsClient {
    fn pipe(&mut self) -> Result<(usize, usize), Error> {
        let tag = MsgTag::new(FS_PROTO, fs::PIPE, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };

        self.endpoint.call(tag)?;

        Ok((utcb.mrs_regs[0], utcb.mrs_regs[1]))
    }
}

impl FileSystemService for FsClient {
    fn open(&mut self, path: &str, flags: OpenFlags, mode: u32) -> Result<usize, Error> {
        let tag = MsgTag::new(FS_PROTO, fs::OPEN, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.write(path.as_bytes());
        utcb.mrs_regs[0] = flags.bits();
        utcb.mrs_regs[1] = mode as usize;

        self.endpoint.call(tag)?;

        Ok(utcb.mrs_regs[0])
    }

    fn mkdir(&mut self, path: &str, mode: u32) -> Result<(), Error> {
        let tag = MsgTag::new(FS_PROTO, fs::MKDIR, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.write(path.as_bytes());
        utcb.mrs_regs[0] = mode as usize;

        self.endpoint.call(tag)
    }

    fn unlink(&mut self, path: &str) -> Result<(), Error> {
        let tag = MsgTag::new(FS_PROTO, fs::UNLINK, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.write(path.as_bytes());

        self.endpoint.call(tag)
    }

    fn rename(&mut self, old_path: &str, new_path: &str) -> Result<(), Error> {
        let tag = MsgTag::new(FS_PROTO, fs::RENAME, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };

        // Simple marshaling: old_path\0new_path
        utcb.write(old_path.as_bytes());
        utcb.append(&[0]);
        utcb.append(new_path.as_bytes());

        self.endpoint.call(tag)
    }

    fn stat_path(&mut self, path: &str) -> Result<Stat, Error> {
        let tag = MsgTag::new(FS_PROTO, fs::STAT_PATH, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.write(path.as_bytes());

        self.endpoint.call(tag)?;

        unsafe { utcb.read_obj::<Stat>().map_err(|_| Error::Unknown) }
    }
}

impl FileHandleService for FsClient {
    fn read(&mut self, offset: u64, buf: &mut [u8]) -> Result<usize, Error> {
        let tag = MsgTag::new(FS_PROTO, fs::READ, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.write(&[]); // Reset size/head if needed, or just rely on manual management. Code below doesn't write.
        // wait, read doesn't send data, it receives.
        // Original code didn't call utcb.write/clear, just set mrs_regs.
        // UTCB state persistence across calls is tricky. Assuming mrs_regs is enough.

        utcb.mrs_regs[0] = buf.len();
        utcb.mrs_regs[1] = offset as usize;

        self.endpoint.call(tag)?;

        let len = utcb.mrs_regs[0];
        buf[..len].copy_from_slice(&utcb.ipc_buffer[..len]);
        Ok(len)
    }

    fn write(&mut self, offset: u64, buf: &[u8]) -> Result<usize, Error> {
        let tag = MsgTag::new(FS_PROTO, fs::WRITE, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        let len = utcb.write(buf);
        utcb.mrs_regs[0] = len;
        utcb.mrs_regs[1] = offset as usize;

        self.endpoint.call(tag)?;

        Ok(utcb.mrs_regs[0])
    }

    fn close(&mut self) -> Result<(), Error> {
        let tag = MsgTag::new(FS_PROTO, fs::CLOSE, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.mrs_regs = [0; 8];

        self.endpoint.call(tag)
    }

    fn stat(&self) -> Result<Stat, Error> {
        let tag = MsgTag::new(FS_PROTO, fs::STAT, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.mrs_regs = [0; 8];

        self.endpoint.call(tag)?;

        unsafe { utcb.read_obj::<Stat>().map_err(|_| Error::Unknown) }
    }

    fn getdents(&mut self, count: usize) -> Result<Vec<fs::DEntry>, Error> {
        let tag = MsgTag::new(FS_PROTO, fs::GETDENTS, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.mrs_regs = [0, count, 0, 0, 0, 0, 0, 0];

        self.endpoint.call(tag)?;

        unsafe { utcb.read_vec::<fs::DEntry>().map_err(|_| Error::Unknown) }
    }

    fn seek(&mut self, offset: i64, whence: usize) -> Result<u64, Error> {
        let tag = MsgTag::new(FS_PROTO, fs::SEEK, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.mrs_regs = [0, offset as usize, (offset >> 32) as usize, whence, 0, 0, 0, 0];

        self.endpoint.call(tag)?;

        Ok(utcb.mrs_regs[0] as u64 | ((utcb.mrs_regs[1] as u64) << 32))
    }

    fn sync(&mut self) -> Result<(), Error> {
        let tag = MsgTag::new(FS_PROTO, fs::SYNC, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.mrs_regs = [0; 8];

        self.endpoint.call(tag)
    }

    fn truncate(&mut self, size: u64) -> Result<(), Error> {
        let tag = MsgTag::new(FS_PROTO, fs::TRUNCATE, MsgFlags::NONE);
        let utcb = unsafe { UTCB::get() };
        utcb.mrs_regs = [0, size as usize, (size >> 32) as usize, 0, 0, 0, 0, 0];

        self.endpoint.call(tag)
    }
}
