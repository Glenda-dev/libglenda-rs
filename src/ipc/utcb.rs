use super::{Badge, MsgTag};
use crate::cap::CapPtr;
use crate::mem::UTCB_VA;
use alloc::vec::Vec;
use core::mem::MaybeUninit;
use serde::{Serialize, de::DeserializeOwned};

pub const BUFFER_MAX_SIZE: usize = 3 * 1024; // 3KB
pub const MAX_MRS: usize = 8;

pub type MsgArgs = [usize; MAX_MRS];

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UTCB {
    pub msg_tag: MsgTag,
    pub mrs_regs: [usize; MAX_MRS],
    pub cap_transfer: CapPtr,
    pub recv_window: CapPtr,
    pub badge: Badge,
    pub head: usize,
    pub size: usize,
    pub ipc_buffer: [u8; BUFFER_MAX_SIZE],
}

impl UTCB {
    pub unsafe fn get() -> &'static mut Self {
        unsafe { &mut *(UTCB_VA as *mut UTCB) }
    }

    pub unsafe fn from(addr: usize) -> &'static mut Self {
        unsafe { &mut *(addr as *mut UTCB) }
    }

    pub fn available_data(&self) -> usize {
        self.size - self.head
    }

    pub fn available_space(&self) -> usize {
        BUFFER_MAX_SIZE - self.size
    }

    pub fn write(&mut self, data: &[u8]) -> usize {
        let len = core::cmp::min(data.len(), BUFFER_MAX_SIZE);
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

    pub fn clear(&mut self) {
        self.msg_tag = MsgTag::empty();
        self.mrs_regs = [0; MAX_MRS];
        self.cap_transfer = CapPtr::null();
        self.recv_window = CapPtr::null();
        self.size = 0;
        self.head = 0;
    }

    pub unsafe fn write_obj<T: Sized + Copy>(&mut self, obj: &T) -> Result<usize, ()> {
        let size = core::mem::size_of::<T>();

        // 检查缓冲区空间是否足够
        if self.available_space() < size {
            return Err(());
        }

        // 将结构体指针转换为字节切片
        let ptr = obj as *const T as *const u8;
        let slice = unsafe { core::slice::from_raw_parts(ptr, size) };

        // 使用 append 确保不会覆盖之前的数据
        let written = self.append(slice);

        if written == size { Ok(written) } else { Err(()) }
    }

    /// 从 IPC 缓冲区反序列化读取对象
    ///
    /// # Safety
    /// `T` 必须符合 IPC 安全传输的要求（见 `write_obj`）。
    pub unsafe fn read_obj<T: Sized + Copy>(&mut self) -> Result<T, ()> {
        let size = core::mem::size_of::<T>();

        if self.available_data() < size {
            return Err(());
        }

        let mut obj = MaybeUninit::<T>::uninit();
        let ptr = obj.as_mut_ptr() as *mut u8;
        let slice = unsafe { core::slice::from_raw_parts_mut(ptr, size) };

        if self.read(slice) == size { Ok(unsafe { obj.assume_init() }) } else { Err(()) }
    }

    pub unsafe fn write_vec<T: Sized + Copy>(&mut self, data: &[T]) -> Result<usize, ()> {
        let len = data.len();
        let size_bytes = len * core::mem::size_of::<T>();

        if self.available_space() < core::mem::size_of::<usize>() + size_bytes {
            return Err(());
        }

        // 写入长度
        self.write_obj(&len)?;

        // 写入数据
        let ptr = data.as_ptr() as *const u8;
        let slice = unsafe { core::slice::from_raw_parts(ptr, size_bytes) };
        let written = self.append(slice);

        if written == size_bytes { Ok(core::mem::size_of::<usize>() + written) } else { Err(()) }
    }

    pub unsafe fn read_vec<T: Sized + Copy>(&mut self) -> Result<Vec<T>, ()> {
        // 读取长度
        let len: usize = self.read_obj()?;

        let size_bytes = len * core::mem::size_of::<T>();
        if self.available_data() < size_bytes {
            return Err(());
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
            Err(())
        }
    }

    pub unsafe fn write_postcard<T: Serialize>(&mut self, obj: &T) -> Result<usize, ()> {
        let vec = postcard::to_allocvec(obj).map_err(|_| ())?;
        unsafe { self.write_vec(&vec) }
    }

    pub unsafe fn read_postcard<T: DeserializeOwned>(&mut self) -> Result<T, ()> {
        let vec = unsafe { self.read_vec::<u8>() }?;
        postcard::from_bytes(&vec).map_err(|_| ())
    }
}
