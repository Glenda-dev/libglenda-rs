use crate::cap::{CSPACE_CAP, CapPtr, Endpoint, Page, RECV_SLOT, Rights};
use crate::error::Error;
use crate::interface::{
    FileHandleService, FileSystemService, PipeService, VirtualFileSystemService,
};
use crate::ipc::{Badge, IPC_BUFFER_SIZE, MsgFlags, MsgTag, UTCB};
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

    fn link(&mut self, _pid: Badge, old_path: &str, new_path: &str) -> Result<(), Error> {
        let tag = MsgTag::new(FS_PROTO, fs::LINK, MsgFlags::HAS_BUFFER);
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
        let transfer_slot = RECV_SLOT;
        let _ = CSPACE_CAP.delete(transfer_slot);
        CSPACE_CAP.copy_self(target.cap(), transfer_slot, Rights::ALL)?;

        let tag = MsgTag::new(FS_PROTO, fs::MOUNT, MsgFlags::HAS_BUFFER | MsgFlags::HAS_CAP);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        unsafe { utcb.write_str(&path)? };
        utcb.set_cap_transfer(transfer_slot);
        utcb.set_recv_window(transfer_slot);
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
    #[warn(deprecated_in_future)]
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
    #[warn(deprecated_in_future)]
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

    fn ioctl(&mut self, _pid: Badge, cmd: u32, arg: usize) -> Result<usize, Error> {
        let (ret, _) = self.ioctl_ex(_pid, cmd, arg, None, 0)?;
        Ok(ret)
    }

    fn poll(&mut self, _pid: Badge, events: u32) -> Result<u32, Error> {
        let tag = MsgTag::new(FS_PROTO, fs::POLL, MsgFlags::NONE);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_mr(1, events as usize);
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;
        Ok(utcb.get_mr(0) as u32)
    }

    fn ioctl_ex(
        &mut self,
        _pid: Badge,
        cmd: u32,
        arg: usize,
        input: Option<&[u8]>,
        out_len: usize,
    ) -> Result<(usize, Vec<u8>), Error> {
        let in_buf = input.unwrap_or(&[]);
        if in_buf.len() > IPC_BUFFER_SIZE || out_len > IPC_BUFFER_SIZE {
            return Err(Error::InvalidArgs);
        }

        let flags = if in_buf.is_empty() { MsgFlags::NONE } else { MsgFlags::HAS_BUFFER };
        let tag = MsgTag::new(FS_PROTO, fs::IOCTL_EX, flags);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_mr(1, cmd as usize);
        utcb.set_mr(2, arg);
        utcb.set_mr(3, in_buf.len());
        utcb.set_mr(4, out_len);
        if !in_buf.is_empty() {
            utcb.write(in_buf);
        }
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;

        let ret = utcb.get_mr(0);
        let out_actual = utcb.get_mr(1);
        if out_actual > IPC_BUFFER_SIZE || out_actual > out_len {
            return Err(Error::InvalidArgs);
        }

        if out_actual == 0 {
            return Ok((ret, Vec::new()));
        }

        if !utcb.get_msg_tag().flags().contains(MsgFlags::HAS_BUFFER) {
            return Err(Error::Unknown);
        }

        let mut out = Vec::with_capacity(out_actual);
        out.extend_from_slice(&utcb.buffer()[..out_actual]);
        Ok((ret, out))
    }

    fn setup_iouring(
        &mut self,
        _pid: Badge,
        client_vaddr: usize,
        size: usize,
        frame: Option<Page>,
    ) -> Result<(), Error> {
        let mut transfer_slot = CapPtr::null();
        if let Some(f) = frame {
            transfer_slot = RECV_SLOT;
            let _ = CSPACE_CAP.delete(transfer_slot);
            CSPACE_CAP.copy_self(f.cap(), transfer_slot, Rights::ALL)?;
        }

        let mut flags = MsgFlags::NONE;
        if !transfer_slot.is_null() {
            flags |= MsgFlags::HAS_CAP;
        }
        let tag = MsgTag::new(FS_PROTO, fs::SETUP_IOURING, flags);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, size, client_vaddr);
        if !transfer_slot.is_null() {
            utcb.set_cap_transfer(transfer_slot);
            utcb.set_recv_window(transfer_slot);
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

    fn map_page(&mut self, pid: Badge, offset: usize, recv_slot: CapPtr) -> Result<usize, Error> {
        self.map_pages(pid, offset, 1, recv_slot)
    }

    fn map_pages(
        &mut self,
        _pid: Badge,
        offset: usize,
        pages: usize,
        recv_slot: CapPtr,
    ) -> Result<usize, Error> {
        if pages == 0 {
            return Err(Error::InvalidArgs);
        }
        let tag = MsgTag::new(FS_PROTO, fs::MAP_PAGE, MsgFlags::NONE);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        set_mrs!(utcb, offset, pages);
        utcb.set_recv_window(recv_slot);
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;
        Ok(utcb.get_mr(0))
    }

    fn unmap_page(&mut self, _pid: Badge, frame: Page) -> Result<(), Error> {
        let transfer_slot = RECV_SLOT;
        let _ = CSPACE_CAP.delete(transfer_slot);
        CSPACE_CAP.copy_self(frame.cap(), transfer_slot, Rights::ALL)?;

        let tag = MsgTag::new(FS_PROTO, fs::UNMAP_PAGE, MsgFlags::HAS_CAP);
        let utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_cap_transfer(transfer_slot);
        utcb.set_recv_window(transfer_slot);
        utcb.set_msg_tag(tag);
        self.endpoint.call(utcb)?;
        Ok(())
    }
}
