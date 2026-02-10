use crate::cap::Endpoint;
use crate::error::Error;
use crate::interface::{FileHandleService, FileSystemService, PipeService};
use crate::ipc::{MsgFlags, MsgTag, UTCB};
use crate::protocol::FS_PROTO;
use crate::protocol::fs;
use crate::protocol::fs::{OpenFlags, Stat};
use crate::set_mrs;
use alloc::vec::Vec;

pub struct FsClient {
    endpoint: Endpoint,
}

impl PipeService for FsClient {
    fn pipe(&mut self) -> Result<(usize, usize), Error> {
        let tag = MsgTag::new(FS_PROTO, fs::PIPE, MsgFlags::NONE);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;
        Ok((utcb.get_mr(0), utcb.get_mr(1)))
    }
}

impl FileSystemService for FsClient {
    fn open(&mut self, path: &str, flags: OpenFlags, mode: u32) -> Result<usize, Error> {
        let tag = MsgTag::new(FS_PROTO, fs::OPEN, MsgFlags::NONE);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.write(path.as_bytes());
        set_mrs!(utcb, flags.bits(), mode);
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;
        Ok(utcb.get_mr(0))
    }

    fn mkdir(&mut self, path: &str, mode: u32) -> Result<(), Error> {
        let tag = MsgTag::new(FS_PROTO, fs::MKDIR, MsgFlags::NONE);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.write(path.as_bytes());
        set_mrs!(utcb, mode);
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;
        Ok(())
    }

    fn unlink(&mut self, path: &str) -> Result<(), Error> {
        let tag = MsgTag::new(FS_PROTO, fs::UNLINK, MsgFlags::NONE);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.write(path.as_bytes());
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;
        Ok(())
    }

    fn rename(&mut self, old_path: &str, new_path: &str) -> Result<(), Error> {
        let tag = MsgTag::new(FS_PROTO, fs::RENAME, MsgFlags::NONE);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        // Simple marshaling: old_path\0new_path
        utcb.write(old_path.as_bytes());
        utcb.append(&[0]);
        utcb.append(new_path.as_bytes());
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;
        Ok(())
    }

    fn stat_path(&mut self, path: &str) -> Result<Stat, Error> {
        let tag = MsgTag::new(FS_PROTO, fs::STAT_PATH, MsgFlags::NONE);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.write(path.as_bytes());
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;
        unsafe { utcb.read_obj::<Stat>().map_err(|_| Error::Unknown) }
    }
}

impl FileHandleService for FsClient {
    fn read(&mut self, offset: u64, buf: &mut [u8]) -> Result<usize, Error> {
        let tag = MsgTag::new(FS_PROTO, fs::READ, MsgFlags::NONE);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, buf.len(), offset as usize);

        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;

        let len = utcb.get_mr(0);
        buf[..len].copy_from_slice(&utcb.ipc_buffer()[..len]);
        Ok(len)
    }

    fn write(&mut self, offset: u64, buf: &[u8]) -> Result<usize, Error> {
        let tag = MsgTag::new(FS_PROTO, fs::WRITE, MsgFlags::NONE);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        let len = utcb.write(buf);
        set_mrs!(utcb, len, offset as usize);
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;

        Ok(utcb.get_mr(0)) // Return actual written length from server
    }

    fn close(&mut self) -> Result<(), Error> {
        let tag = MsgTag::new(FS_PROTO, fs::CLOSE, MsgFlags::NONE);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)
    }

    fn stat(&self) -> Result<Stat, Error> {
        let tag = MsgTag::new(FS_PROTO, fs::STAT, MsgFlags::NONE);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;
        unsafe { utcb.read_obj::<Stat>().map_err(|_| Error::Unknown) }
    }

    fn getdents(&mut self, count: usize) -> Result<Vec<fs::DEntry>, Error> {
        let tag = MsgTag::new(FS_PROTO, fs::GETDENTS, MsgFlags::NONE);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, count);
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;
        unsafe { utcb.read_vec::<fs::DEntry>().map_err(|_| Error::Unknown) }
    }

    fn seek(&mut self, offset: i64, whence: usize) -> Result<u64, Error> {
        let tag = MsgTag::new(FS_PROTO, fs::SEEK, MsgFlags::NONE);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, offset as usize, whence);
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;
        Ok(utcb.get_mr(0) as u64)
    }

    fn sync(&mut self) -> Result<(), Error> {
        let tag = MsgTag::new(FS_PROTO, fs::SYNC, MsgFlags::NONE);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)
    }

    fn truncate(&mut self, size: u64) -> Result<(), Error> {
        let tag = MsgTag::new(FS_PROTO, fs::TRUNCATE, MsgFlags::NONE);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, size as usize);
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)
    }
}
