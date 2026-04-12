use crate::cap::{CapPtr, Endpoint, Frame};
use crate::error::Error;
use crate::interface::{
    FileHandleService, FileSystemService, PipeService, VirtualFileSystemService,
};
use crate::ipc::{Badge, MsgFlags, MsgTag, UTCB};
use crate::protocol::FS_PROTO;
use crate::protocol::fs;
use crate::protocol::fs::{OpenFlags, Stat};
use crate::set_mrs;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy)]
pub struct FsClient {
    endpoint: Endpoint,
}

impl FsClient {
    pub const fn new(endpoint: Endpoint) -> Self {
        Self { endpoint }
    }

    pub const fn endpoint(&self) -> Endpoint {
        self.endpoint
    }
}

impl PipeService for FsClient {
    fn pipe(&mut self, _pid: Badge) -> Result<(usize, usize), Error> {
        Err(Error::NotImplemented)
    }
}

impl FileSystemService for FsClient {
    fn open(
        &mut self,
        _pid: Badge,
        path: &str,
        flags: OpenFlags,
        mode: u32,
        recv_slot: CapPtr,
    ) -> Result<usize, Error> {
        let tag = MsgTag::new(FS_PROTO, fs::OPEN, MsgFlags::HAS_BUFFER);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        unsafe { utcb.write_str(&path)? };
        set_mrs!(utcb, flags.bits(), mode);
        utcb.set_recv_window(recv_slot);
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;
        Ok(utcb.get_mr(0))
    }

    fn mkdir(&mut self, _pid: Badge, path: &str, mode: u32) -> Result<(), Error> {
        let tag = MsgTag::new(FS_PROTO, fs::MKDIR, MsgFlags::HAS_BUFFER);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        unsafe { utcb.write_str(&path)? };
        set_mrs!(utcb, mode);
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;
        Ok(())
    }

    fn unlink(&mut self, _pid: Badge, path: &str) -> Result<(), Error> {
        let tag = MsgTag::new(FS_PROTO, fs::UNLINK, MsgFlags::HAS_BUFFER);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        unsafe { utcb.write_str(&path)? };
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;
        Ok(())
    }

    fn rename(&mut self, _pid: Badge, old_path: &str, new_path: &str) -> Result<(), Error> {
        let tag = MsgTag::new(FS_PROTO, fs::RENAME, MsgFlags::HAS_BUFFER);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        unsafe { utcb.write_postcard(&(old_path, new_path))? };
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;
        Ok(())
    }

    fn stat_path(&mut self, _pid: Badge, path: &str) -> Result<Stat, Error> {
        let tag = MsgTag::new(FS_PROTO, fs::STAT_PATH, MsgFlags::HAS_BUFFER);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        unsafe { utcb.write_str(&path)? };
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;
        unsafe { utcb.read_obj::<Stat>().map_err(|_| Error::Unknown) }
    }

    fn lstat_path(&mut self, _pid: Badge, path: &str) -> Result<Stat, Error> {
        let tag = MsgTag::new(FS_PROTO, fs::LSTAT_PATH, MsgFlags::HAS_BUFFER);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        unsafe { utcb.write_str(&path)? };
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;
        unsafe { utcb.read_obj::<Stat>().map_err(|_| Error::Unknown) }
    }

    fn readlink_path(&mut self, _pid: Badge, path: &str) -> Result<String, Error> {
        let tag = MsgTag::new(FS_PROTO, fs::READLINK_PATH, MsgFlags::HAS_BUFFER);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        unsafe { utcb.write_str(&path)? };
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;
        unsafe { utcb.read_str().map_err(|_| Error::Unknown) }
    }
}

impl VirtualFileSystemService for FsClient {
    fn mount(&mut self, _pid: Badge, path: &str, target: Endpoint) -> Result<(), Error> {
        let tag = MsgTag::new(FS_PROTO, fs::MOUNT, MsgFlags::HAS_BUFFER | MsgFlags::HAS_CAP);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        unsafe { utcb.write_str(&path)? };
        utcb.set_cap_transfer(target.cap());
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;
        Ok(())
    }

    fn unmount(&mut self, _pid: Badge, path: &str) -> Result<(), Error> {
        let tag = MsgTag::new(FS_PROTO, fs::UNMOUNT, MsgFlags::HAS_BUFFER);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        unsafe { utcb.write_str(&path)? };
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;
        Ok(())
    }

    fn create_view(&mut self, _pid: Badge, root: &str) -> Result<usize, Error> {
        let tag = MsgTag::new(FS_PROTO, fs::CREATE_VIEW, MsgFlags::HAS_BUFFER);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        unsafe { utcb.write_str(root)? };
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;
        Ok(utcb.get_mr(0))
    }

    fn set_view(&mut self, _pid: Badge, view_id: usize) -> Result<(), Error> {
        let tag = MsgTag::new(FS_PROTO, fs::SET_VIEW, MsgFlags::NONE);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, view_id);
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;
        Ok(())
    }
}

impl FileHandleService for FsClient {
    fn read(&mut self, _pid: Badge, offset: usize, buf: &mut [u8]) -> Result<usize, Error> {
        let tag = MsgTag::new(FS_PROTO, fs::READ_SYNC, MsgFlags::NONE);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, buf.len(), offset as usize);

        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;

        let val = utcb.get_mr(0);

        let len = val;
        if len > buf.len() {
            return Err(Error::InvalidArgs);
        }
        // Copy from UTCB buffer
        buf[..len].copy_from_slice(&utcb.buffer()[..len]);
        Ok(len)
    }

    fn write(&mut self, _pid: Badge, offset: usize, buf: &[u8]) -> Result<usize, Error> {
        let tag = MsgTag::new(FS_PROTO, fs::WRITE_SYNC, MsgFlags::HAS_BUFFER);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();

        // Protocol: args: [offset], bytes: data -> res: written
        set_mrs!(utcb, offset as usize);
        utcb.write(buf); // Write data to UTCB buffer

        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;

        Ok(utcb.get_mr(0))
    }

    fn close(&mut self, _pid: Badge) -> Result<(), Error> {
        let tag = MsgTag::new(FS_PROTO, fs::CLOSE, MsgFlags::NONE);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)
    }

    fn stat(&self, _pid: Badge) -> Result<Stat, Error> {
        let tag = MsgTag::new(FS_PROTO, fs::STAT, MsgFlags::NONE);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;
        unsafe { utcb.read_obj::<Stat>().map_err(|_| Error::Unknown) }
    }

    fn getdents(&mut self, _pid: Badge, count: usize) -> Result<Vec<fs::DEntry>, Error> {
        let tag = MsgTag::new(FS_PROTO, fs::GETDENTS, MsgFlags::NONE);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_mr(1, count);
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;

        unsafe { utcb.read_vec::<fs::DEntry>() }
    }

    fn seek(&mut self, _pid: Badge, offset: i64, whence: usize) -> Result<usize, Error> {
        let tag = MsgTag::new(FS_PROTO, fs::SEEK, MsgFlags::NONE);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_mr(1, offset as usize);
        utcb.set_mr(2, whence);
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;
        Ok(utcb.get_mr(0) as usize)
    }

    fn sync(&mut self, _pid: Badge) -> Result<(), Error> {
        let tag = MsgTag::new(FS_PROTO, fs::SYNC, MsgFlags::NONE);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;
        Ok(())
    }

    fn truncate(&mut self, _pid: Badge, size: usize) -> Result<(), Error> {
        let tag = MsgTag::new(FS_PROTO, fs::TRUNCATE, MsgFlags::NONE);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_mr(1, size as usize);
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;
        Ok(())
    }

    fn setup_iouring(
        &mut self,
        _pid: Badge,
        client_vaddr: usize,
        size: usize,
        frame: Option<Frame>,
    ) -> Result<(), Error> {
        let mut flags = MsgFlags::NONE;
        if frame.is_some() {
            flags |= MsgFlags::HAS_CAP;
        }
        let tag = MsgTag::new(FS_PROTO, fs::SETUP_IOURING, flags);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, size, client_vaddr);
        if let Some(f) = frame {
            utcb.set_cap_transfer(f.cap());
        }
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;
        Ok(())
    }

    fn process_iouring(&mut self) -> Result<(), Error> {
        let tag = MsgTag::new(FS_PROTO, fs::PROCESS_IOURING, MsgFlags::NONE);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;
        Ok(())
    }
}
