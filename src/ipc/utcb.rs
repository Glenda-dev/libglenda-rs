use super::{Badge, MsgTag};
use crate::cap::CapPtr;
use crate::mem::UTCB_VA;
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
    pub tail: usize,
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
        if self.tail >= self.head {
            self.tail - self.head
        } else {
            BUFFER_MAX_SIZE - self.head + self.tail
        }
    }

    pub fn available_space(&self) -> usize {
        BUFFER_MAX_SIZE - self.available_data() - 1
    }

    pub fn write(&mut self, data: &[u8]) -> usize {
        let len = core::cmp::min(data.len(), self.available_space());
        for i in 0..len {
            self.ipc_buffer[self.tail] = data[i];
            self.tail = (self.tail + 1) % BUFFER_MAX_SIZE;
        }
        len
    }

    pub fn read(&mut self, data: &mut [u8]) -> usize {
        let len = core::cmp::min(data.len(), self.available_data());
        for i in 0..len {
            data[i] = self.ipc_buffer[self.head];
            self.head = (self.head + 1) % BUFFER_MAX_SIZE;
        }
        len
    }

    pub fn clear(&mut self) {
        self.msg_tag = MsgTag::empty();
        self.mrs_regs = [0; MAX_MRS];
        self.cap_transfer = CapPtr::null();
        self.recv_window = CapPtr::null();
        self.head = 0;
        self.tail = 0;
        for byte in self.ipc_buffer.iter_mut() {
            *byte = 0;
        }
    }

    pub unsafe fn write_obj<T: Sized + Copy>(&mut self, obj: &T) -> Result<usize, ()> {
        let size = core::mem::size_of::<T>();

        // 检查缓冲区空间是否足够
        if self.available_space() < size {
            return Err(());
        }

        // 将结构体指针转换为字节切片
        // 因为 obj 是引用，它的内存是连续的，可以直接转换
        let ptr = obj as *const T as *const u8;
        let slice = unsafe { core::slice::from_raw_parts(ptr, size) };

        // 调用底层的 write，它会处理将连续切片写入到可能回绕(wrap-around)的环形缓冲区的逻辑
        let written = self.write(slice);

        if written == size {
            Ok(written)
        } else {
            // 如果写入字节数不符合预期（理论上 available_space 检查过不会发生）
            Err(())
        }
    }

    /// 从 IPC 缓冲区反序列化读取对象
    ///
    /// # Safety
    /// `T` 必须符合 IPC 安全传输的要求（见 `write_obj`）。
    pub unsafe fn read_obj<T: Sized + Copy>(&mut self) -> Result<T, ()> {
        let size = core::mem::size_of::<T>();

        // 检查缓冲区数据是否足够
        if self.available_data() < size {
            return Err(());
        }

        // 创建未初始化的内存用于接收数据
        let mut obj = MaybeUninit::<T>::uninit();
        let ptr = obj.as_mut_ptr() as *mut u8;

        // 创建指向目标结构体内存的字节切片
        let slice = unsafe { core::slice::from_raw_parts_mut(ptr, size) };

        // 调用底层的 read，从可能回绕的环形缓冲区读取数据填充到我们的连续内存 slice 中
        let read = self.read(slice);

        if read == size { Ok(unsafe { obj.assume_init() }) } else { Err(()) }
    }

    pub unsafe fn write_vec<T: Sized + Copy>(&mut self, data: &[T]) -> Result<usize, ()> {
        let len = data.len();
        let size_bytes = len * core::mem::size_of::<T>();

        if self.available_space() < core::mem::size_of::<usize>() + size_bytes {
            return Err(());
        }

        (unsafe { self.write_obj(&len) })?;

        let ptr = data.as_ptr() as *const u8;
        let slice = unsafe { core::slice::from_raw_parts(ptr, size_bytes) };
        let written = self.write(slice);

        if written == size_bytes { Ok(core::mem::size_of::<usize>() + written) } else { Err(()) }
    }
    pub unsafe fn write_postcard<T: Serialize>(&mut self, obj: &T) -> Result<usize, ()> {
        let vec = postcard::to_allocvec(obj).map_err(|_| ())?;
        unsafe { self.write_vec(&vec) }
    }

    pub unsafe fn read_postcard<T: DeserializeOwned>(&mut self) -> Result<T, ()> {
        let vec = self.read_vec::<u8>()?;
        postcard::from_bytes(&vec).map_err(|_| ())
    }
}
