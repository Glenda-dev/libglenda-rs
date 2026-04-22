use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::cap::{CapPtr, Endpoint};
use crate::error::Error;
use crate::interface::FileHandleService;
use crate::ipc::{Badge, IPC_BUFFER_SIZE, MsgFlags, MsgTag, UTCB};
use crate::protocol;
use crate::protocol::fs;
use crate::sync::mutex::Mutex;

type HandleId = u64;

pub trait FsNamespace: Send {
    type Handle: FileHandleService + Send;

    fn open(
        &mut self,
        path: &str,
        flags: fs::OpenFlags,
        mode: u32,
        _badge: Badge,
    ) -> Result<Self::Handle, Error>;

    fn mkdir(&mut self, path: &str, mode: u32, _badge: Badge) -> Result<(), Error>;

    fn unlink(&mut self, path: &str, _badge: Badge) -> Result<(), Error>;

    fn stat_path(&mut self, path: &str, _badge: Badge) -> Result<fs::Stat, Error>;

    fn readlink_path(&mut self, path: &str, _badge: Badge) -> Result<alloc::string::String, Error>;

    fn create_pipe(&mut self, _badge: Badge) -> Result<usize, Error> {
        Err(Error::NotSupported)
    }
}

pub struct FsRpcServer<N: FsNamespace> {
    backend: Mutex<N>,
    handles: Mutex<BTreeMap<HandleId, N::Handle>>,
}

impl<N: FsNamespace> FsRpcServer<N> {
    pub fn new(backend: N) -> Self {
        Self { backend: Mutex::new(backend), handles: Mutex::new(BTreeMap::new()) }
    }

    pub fn run(&self, ep: Endpoint, reply_slot: CapPtr, recv_slot: CapPtr) -> Result<(), Error> {
        loop {
            let mut utcb = unsafe { UTCB::new() };
            utcb.clear();
            utcb.set_reply_window(reply_slot);
            utcb.set_recv_window(recv_slot);

            if ep.recv(&mut utcb).is_err() {
                continue;
            }

            let badge = utcb.get_badge();
            match self.dispatch(&mut utcb, badge) {
                Ok(()) => {
                    let _ = crate::cap::Reply::from(reply_slot).reply(&mut utcb);
                }
                Err(Error::Success) => {}
                Err(e) => {
                    utcb.set_msg_tag(MsgTag::err());
                    utcb.set_mr(0, e as usize);
                    let _ = crate::cap::Reply::from(reply_slot).reply(&mut utcb);
                }
            }
        }
    }

    fn handle_key_from_badge(badge: Badge) -> HandleId {
        badge.bits() as u64
    }

    pub fn dispatch(&self, utcb: &mut UTCB, badge: Badge) -> Result<(), Error> {
        let tag = utcb.get_msg_tag();
        if tag.proto() != protocol::FS_PROTO {
            return Err(Error::InvalidProtocol);
        }
        match tag.label() {
            fs::OPEN => self.handle_open(utcb, badge),
            fs::MKDIR => self.handle_mkdir(utcb, badge),
            fs::UNLINK => self.handle_unlink(utcb, badge),
            fs::STAT_PATH | fs::LSTAT_PATH => self.handle_stat_path(utcb, badge),
            fs::READLINK_PATH => self.handle_readlink(utcb, badge),
            fs::CLOSE => self.handle_close(utcb, badge),
            fs::STAT => self.handle_stat(utcb, badge),
            fs::READ_SYNC => self.handle_read(utcb, badge),
            fs::WRITE_SYNC => self.handle_write(utcb, badge),
            fs::GETDENTS => self.handle_getdents(utcb, badge),
            fs::SEEK => self.handle_seek(utcb, badge),
            fs::TRUNCATE => self.handle_truncate(utcb, badge),
            fs::IOCTL => self.handle_ioctl(utcb, badge),
            fs::IOCTL_EX => self.handle_ioctl_ex(utcb, badge),
            fs::PIPE_CREATE => self.handle_pipe_create(utcb, badge),
            _ => Err(Error::NotSupported),
        }
    }

    fn ok_reply(utcb: &mut UTCB) {
        utcb.set_msg_tag(MsgTag::new(
            protocol::GENERIC_PROTO,
            protocol::generic::REPLY,
            MsgFlags::OK,
        ));
    }

    fn handle_open(&self, utcb: &mut UTCB, badge: Badge) -> Result<(), Error> {
        let path = unsafe { utcb.read_str()? };
        let flags = fs::OpenFlags::from_bits_truncate(utcb.get_mr(0));
        let mode = utcb.get_mr(1) as u32;

        let handle = self.backend.lock().open(&path, flags, mode, badge)?;
        let handle_id = Self::handle_key_from_badge(badge);
        self.handles.lock().insert(handle_id, handle);
        utcb.set_mr(0, handle_id as usize);
        Self::ok_reply(utcb);
        Ok(())
    }

    fn handle_mkdir(&self, utcb: &mut UTCB, badge: Badge) -> Result<(), Error> {
        let path = unsafe { utcb.read_str()? };
        let mode = utcb.get_mr(0) as u32;
        self.backend.lock().mkdir(&path, mode, badge)?;
        Self::ok_reply(utcb);
        Ok(())
    }

    fn handle_unlink(&self, utcb: &mut UTCB, badge: Badge) -> Result<(), Error> {
        let path = unsafe { utcb.read_str()? };
        self.backend.lock().unlink(&path, badge)?;
        Self::ok_reply(utcb);
        Ok(())
    }

    fn handle_stat_path(&self, utcb: &mut UTCB, badge: Badge) -> Result<(), Error> {
        let path = unsafe { utcb.read_str()? };
        let stat = self.backend.lock().stat_path(&path, badge)?;
        unsafe { utcb.write_obj(&stat)? };
        Self::ok_reply(utcb);
        Ok(())
    }

    fn handle_readlink(&self, utcb: &mut UTCB, badge: Badge) -> Result<(), Error> {
        let path = unsafe { utcb.read_str()? };
        let target = self.backend.lock().readlink_path(&path, badge)?;
        unsafe { utcb.write_str(&target)? };
        Self::ok_reply(utcb);
        Ok(())
    }

    fn handle_close(&self, utcb: &mut UTCB, badge: Badge) -> Result<(), Error> {
        let handle_id = Self::handle_key_from_badge(badge);
        if let Some(mut h) = self.handles.lock().remove(&handle_id) {
            h.close(badge)?;
        }
        Self::ok_reply(utcb);
        Ok(())
    }

    fn handle_stat(&self, utcb: &mut UTCB, badge: Badge) -> Result<(), Error> {
        let handle_id = Self::handle_key_from_badge(badge);
        let handles = self.handles.lock();
        let handle = handles.get(&handle_id).ok_or(Error::NotFound)?;
        let stat = handle.stat(badge)?;
        drop(handles);
        unsafe { utcb.write_obj(&stat)? };
        Self::ok_reply(utcb);
        Ok(())
    }

    fn handle_read(&self, utcb: &mut UTCB, badge: Badge) -> Result<(), Error> {
        let size = utcb.get_mr(0);
        let offset = utcb.get_mr(1);
        let handle_id = Self::handle_key_from_badge(badge);

        let mut handles = self.handles.lock();
        let handle = handles.get_mut(&handle_id).ok_or(Error::NotFound)?;
        let read_size = size.min(IPC_BUFFER_SIZE);
        let mut tmp = [0u8; IPC_BUFFER_SIZE];
        let n = handle.read(badge, offset, &mut tmp[..read_size])?;
        drop(handles);
        if n > read_size {
            return Err(Error::InvalidArgs);
        }

        utcb.set_mr(0, n);
        if n == 0 {
            Self::ok_reply(utcb);
        } else {
            utcb.write(&tmp[..n]);
            utcb.set_msg_tag(MsgTag::new(
                protocol::GENERIC_PROTO,
                protocol::generic::REPLY,
                MsgFlags::OK | MsgFlags::HAS_BUFFER,
            ));
        }
        Ok(())
    }

    fn handle_write(&self, utcb: &mut UTCB, badge: Badge) -> Result<(), Error> {
        let offset = utcb.get_mr(0);
        let handle_id = Self::handle_key_from_badge(badge);
        let input_len = utcb.get_size();
        let mut input = Vec::new();
        if input_len > 0 {
            if input_len > utcb.buffer().len() {
                return Err(Error::InvalidArgs);
            }
            input.extend_from_slice(&utcb.buffer()[..input_len]);
        }
        let mut handles = self.handles.lock();
        let handle = handles.get_mut(&handle_id).ok_or(Error::NotFound)?;
        let n = handle.write(badge, offset, &input)?;
        drop(handles);
        utcb.set_mr(0, n);
        Self::ok_reply(utcb);
        Ok(())
    }

    fn handle_getdents(&self, utcb: &mut UTCB, badge: Badge) -> Result<(), Error> {
        let count = utcb.get_mr(1);
        let handle_id = Self::handle_key_from_badge(badge);
        let mut handles = self.handles.lock();
        let handle = handles.get_mut(&handle_id).ok_or(Error::NotFound)?;
        let entries = handle.getdents(badge, count)?;
        drop(handles);
        unsafe { utcb.write_vec(&entries)? };
        Self::ok_reply(utcb);
        Ok(())
    }

    fn handle_seek(&self, utcb: &mut UTCB, badge: Badge) -> Result<(), Error> {
        let offset = utcb.get_mr(1) as i64;
        let whence = utcb.get_mr(2);
        let handle_id = Self::handle_key_from_badge(badge);
        let mut handles = self.handles.lock();
        let handle = handles.get_mut(&handle_id).ok_or(Error::NotFound)?;
        let new_offset = handle.seek(badge, offset, whence)?;
        drop(handles);
        utcb.set_mr(0, new_offset);
        Self::ok_reply(utcb);
        Ok(())
    }

    fn handle_truncate(&self, utcb: &mut UTCB, badge: Badge) -> Result<(), Error> {
        let size = utcb.get_mr(1);
        let handle_id = Self::handle_key_from_badge(badge);
        let mut handles = self.handles.lock();
        let handle = handles.get_mut(&handle_id).ok_or(Error::NotFound)?;
        handle.truncate(badge, size)?;
        drop(handles);
        Self::ok_reply(utcb);
        Ok(())
    }

    fn handle_ioctl(&self, utcb: &mut UTCB, badge: Badge) -> Result<(), Error> {
        let cmd = utcb.get_mr(1) as u32;
        let arg = utcb.get_mr(2);
        let handle_id = Self::handle_key_from_badge(badge);
        let mut handles = self.handles.lock();
        let handle = handles.get_mut(&handle_id).ok_or(Error::NotFound)?;
        let ret = handle.ioctl(badge, cmd, arg)?;
        drop(handles);
        utcb.set_mr(0, ret);
        Self::ok_reply(utcb);
        Ok(())
    }

    fn handle_ioctl_ex(&self, utcb: &mut UTCB, badge: Badge) -> Result<(), Error> {
        let cmd = utcb.get_mr(1) as u32;
        let arg = utcb.get_mr(2);
        let in_len = utcb.get_mr(3);
        let out_len = utcb.get_mr(4);
        if in_len > utcb.buffer().len() || out_len > IPC_BUFFER_SIZE {
            return Err(Error::InvalidArgs);
        }
        let input = if in_len > 0 { Some(&utcb.buffer()[..in_len]) } else { None };

        let handle_id = Self::handle_key_from_badge(badge);
        let mut handles = self.handles.lock();
        let handle = handles.get_mut(&handle_id).ok_or(Error::NotFound)?;
        let (ret, out) = handle.ioctl_ex(badge, cmd, arg, input, out_len)?;
        drop(handles);

        if out.len() > out_len || out.len() > IPC_BUFFER_SIZE {
            return Err(Error::InvalidArgs);
        }
        utcb.set_mr(0, ret);
        utcb.set_mr(1, out.len());
        if out.is_empty() {
            Self::ok_reply(utcb);
        } else {
            utcb.write(&out);
            utcb.set_msg_tag(MsgTag::new(
                protocol::GENERIC_PROTO,
                protocol::generic::REPLY,
                MsgFlags::OK | MsgFlags::HAS_BUFFER,
            ));
        }
        Ok(())
    }

    fn handle_pipe_create(&self, utcb: &mut UTCB, badge: Badge) -> Result<(), Error> {
        let pipe_id = self.backend.lock().create_pipe(badge)?;
        utcb.set_mr(0, pipe_id);
        Self::ok_reply(utcb);
        Ok(())
    }
}
