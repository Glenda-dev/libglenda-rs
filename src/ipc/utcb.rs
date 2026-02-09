use super::{Badge, MsgTag};
use crate::cap::CapPtr;
use crate::error::Error;
use crate::mem::UTCB_VA;
use alloc::string::String;
use alloc::vec::Vec;
use core::mem::MaybeUninit;
use core::ptr::{read_volatile, write_volatile};
use serde::{Serialize, de::DeserializeOwned};

pub const IPC_BUFFER_SIZE: usize = 3 * 1024; // 3KB
pub const MAX_MRS: usize = 8;

pub type MsgArgs = [usize; MAX_MRS];

#[macro_export]
macro_rules! set_mrs {
    ($ctx:expr, $($arg:expr),* $(,)?) => {
        {
            let mut _i = 0;
            $(
                $ctx.set_mr(_i, $arg as usize);
                _i += 1;
            )*
        }
    };
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UTCB {
    msg_tag: MsgTag,
    mrs_regs: MsgArgs,
    cap_transfer: CapPtr,
    recv_window: CapPtr,
    reply_window: CapPtr,
    badge: Badge,
    head: usize,
    size: usize,
    ipc_buffer: [u8; IPC_BUFFER_SIZE],
}

impl UTCB {
    /// 创建一个新的 IPC 上下文。
    ///
    /// # Safety
    /// 这个函数应该只在线程的某次 IPC 操作开始前调用，不应长期持有返回的对象。
    pub unsafe fn new() -> &'static mut Self {
        unsafe { &mut *(UTCB_VA as *mut Self) }
    }

    pub fn get_msg_tag(&self) -> MsgTag {
        unsafe { read_volatile(&self.msg_tag) }
    }

    pub fn set_msg_tag(&mut self, tag: MsgTag) {
        unsafe { write_volatile(&mut self.msg_tag, tag) }
    }

    pub fn get_mr(&self, index: usize) -> usize {
        if index < MAX_MRS { unsafe { read_volatile(&self.mrs_regs[index]) } } else { 0 }
    }

    pub fn set_mr(&mut self, index: usize, value: usize) {
        if index < MAX_MRS {
            unsafe { write_volatile(&mut self.mrs_regs[index], value) }
        }
    }

    pub fn get_mrs(&self) -> [usize; MAX_MRS] {
        let mut args = [0; MAX_MRS];
        for i in 0..MAX_MRS {
            args[i] = unsafe { read_volatile(&self.mrs_regs[i]) };
        }
        args
    }

    pub fn set_mrs(&mut self, mrs: [usize; MAX_MRS]) {
        for i in 0..MAX_MRS {
            unsafe { write_volatile(&mut self.mrs_regs[i], mrs[i]) };
        }
    }

    pub fn get_badge(&self) -> Badge {
        unsafe { read_volatile(&self.badge) }
    }

    pub fn get_size(&self) -> usize {
        unsafe { read_volatile(&self.size) }
    }

    pub fn set_size(&mut self, size: usize) {
        unsafe { write_volatile(&mut self.size, size) }
    }

    pub fn set_buffer_len(&mut self, len: usize) {
        self.set_size(core::cmp::min(len, IPC_BUFFER_SIZE));
    }

    pub fn get_recv_window(&self) -> CapPtr {
        unsafe { read_volatile(&self.recv_window) }
    }

    pub fn set_recv_window(&mut self, cap: CapPtr) {
        unsafe { write_volatile(&mut self.recv_window, cap) }
    }

    pub fn get_reply_window(&self) -> CapPtr {
        unsafe { read_volatile(&self.reply_window) }
    }

    pub fn set_reply_window(&mut self, cap: CapPtr) {
        unsafe { write_volatile(&mut self.reply_window, cap) }
    }

    pub fn get_cap_transfer(&self) -> CapPtr {
        unsafe { read_volatile(&self.cap_transfer) }
    }

    pub fn set_cap_transfer(&mut self, cap: CapPtr) {
        unsafe { write_volatile(&mut self.cap_transfer, cap) }
    }

    pub fn ipc_buffer(&mut self) -> &mut [u8] {
        &mut self.ipc_buffer
    }

    pub fn buffer_mut(&mut self) -> &mut [u8] {
        &mut self.ipc_buffer
    }

    pub fn buffer(&self) -> &[u8] {
        &self.ipc_buffer[..self.get_size()]
    }

    pub fn clear(&mut self) {
        unsafe {
            write_volatile(&mut self.msg_tag, MsgTag::empty());
            for i in 0..MAX_MRS {
                write_volatile(&mut self.mrs_regs[i], 0);
            }
            write_volatile(&mut self.cap_transfer, CapPtr::null());
            write_volatile(&mut self.recv_window, CapPtr::null());
            write_volatile(&mut self.reply_window, CapPtr::null());
            write_volatile(&mut self.head, 0);
            write_volatile(&mut self.size, 0);
        }
    }

    pub fn available_data(&self) -> usize {
        self.size - self.head
    }

    pub fn available_space(&self) -> usize {
        IPC_BUFFER_SIZE - self.size
    }

    pub fn write(&mut self, data: &[u8]) -> usize {
        let len = core::cmp::min(data.len(), IPC_BUFFER_SIZE);
        self.ipc_buffer[..len].copy_from_slice(&data[..len]);
        self.size = len;
        self.head = 0;
        len
    }

    pub fn append(&mut self, data: &[u8]) -> usize {
        let len = core::cmp::min(data.len(), self.available_space());
        if len > 0 {
            self.ipc_buffer[self.size..self.size + len].copy_from_slice(&data[..len]);
            self.size += len;
        }
        len
    }

    pub fn read(&mut self, data: &mut [u8]) -> usize {
        let len = core::cmp::min(data.len(), self.available_data());
        if len > 0 {
            data[..len].copy_from_slice(&self.ipc_buffer[self.head..self.head + len]);
            self.head += len;
        }
        len
    }

    pub unsafe fn write_obj<T: Sized + Copy>(&mut self, obj: &T) -> Result<usize, Error> {
        let size = core::mem::size_of::<T>();

        // 检查缓冲区空间是否足够
        if self.available_space() < size {
            return Err(Error::BufferOverflow);
        }

        // 将结构体指针转换为字节切片
        let ptr = obj as *const T as *const u8;
        let slice = unsafe { core::slice::from_raw_parts(ptr, size) };

        // 使用 append 确保不会覆盖之前的数据
        let written = self.append(slice);

        if written == size { Ok(written) } else { Err(Error::BufferOverflow) }
    }

    /// 从 IPC 缓冲区反序列化读取对象
    ///
    /// # Safety
    /// `T` 必须符合 IPC 安全传输的要求（见 `write_obj`）。
    pub unsafe fn read_obj<T: Sized + Copy>(&mut self) -> Result<T, Error> {
        let size = core::mem::size_of::<T>();

        if self.available_data() < size {
            return Err(Error::BufferOverflow);
        }

        let mut obj = MaybeUninit::<T>::uninit();
        let ptr = obj.as_mut_ptr() as *mut u8;
        let slice = unsafe { core::slice::from_raw_parts_mut(ptr, size) };

        if self.read(slice) == size {
            Ok(unsafe { obj.assume_init() })
        } else {
            Err(Error::BufferOverflow)
        }
    }

    pub unsafe fn write_vec<T: Sized + Copy>(&mut self, data: &[T]) -> Result<usize, Error> {
        let len = data.len();
        let size_bytes = len * core::mem::size_of::<T>();

        if self.available_space() < core::mem::size_of::<usize>() + size_bytes {
            return Err(Error::BufferOverflow);
        }

        // 写入长度
        (unsafe { self.write_obj(&len) })?;

        // 写入数据
        let ptr = data.as_ptr() as *const u8;
        let slice = unsafe { core::slice::from_raw_parts(ptr, size_bytes) };
        let written = self.append(slice);

        if written == size_bytes {
            Ok(core::mem::size_of::<usize>() + written)
        } else {
            Err(Error::BufferOverflow)
        }
    }

    pub unsafe fn read_vec<T: Sized + Copy>(&mut self) -> Result<Vec<T>, Error> {
        // 读取长度
        let len: usize = unsafe { self.read_obj() }?;

        let size_bytes = len * core::mem::size_of::<T>();
        if self.available_data() < size_bytes {
            return Err(Error::BufferOverflow);
        }

        let mut vec: Vec<T> = Vec::with_capacity(len);
        let ptr = vec.as_mut_ptr() as *mut u8;
        let slice = unsafe { core::slice::from_raw_parts_mut(ptr, size_bytes) };

        if self.read(slice) == size_bytes {
            unsafe {
                vec.set_len(len);
            }
            Ok(vec)
        } else {
            Err(Error::BufferOverflow)
        }
    }

    pub unsafe fn write_postcard<T: Serialize>(&mut self, obj: &T) -> Result<usize, Error> {
        let vec = postcard::to_allocvec(obj).map_err(|_| Error::InvalidObjType)?;
        unsafe { self.write_vec(&vec) }
    }

    pub unsafe fn read_postcard<T: DeserializeOwned>(&mut self) -> Result<T, Error> {
        let vec = unsafe { self.read_vec::<u8>() }?;
        postcard::from_bytes(&vec).map_err(|_| Error::InvalidObjType)
    }

    pub unsafe fn write_str(&mut self, s: &str) -> Result<usize, Error> {
        unsafe { self.write_vec(s.as_bytes()) }
    }

    pub unsafe fn read_str(&mut self) -> Result<String, Error> {
        let vec = unsafe { self.read_vec::<u8>() }?;
        String::from_utf8(vec).map_err(|_| Error::InvalidObjType)
    }
}
