use crate::sync::mutex::{Mutex, MutexGuard};
use crate::sys::sbrk;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr;

/// 每个内存块的头部信息
#[repr(C)]
pub struct BlockHeader {
    pub size: usize,      // 包含头部的大小，最高位 1 表示已分配，0 表示空闲
    pub prev_size: usize, // 前一个物理块的大小，用于向前合并
}

impl BlockHeader {
    const ALLOC_BIT: usize = 1 << (core::mem::size_of::<usize>() * 8 - 1);

    fn new(size: usize, prev_size: usize, allocated: bool) -> Self {
        let size_val = if allocated { size | Self::ALLOC_BIT } else { size };
        Self { size: size_val, prev_size }
    }

    fn size(&self) -> usize {
        self.size & !Self::ALLOC_BIT
    }

    fn set_size(&mut self, size: usize) {
        let allocated = self.is_allocated();
        self.size = if allocated { size | Self::ALLOC_BIT } else { size };
    }

    fn is_allocated(&self) -> bool {
        (self.size & Self::ALLOC_BIT) != 0
    }

    fn set_allocated(&mut self, allocated: bool) {
        let size = self.size();
        if allocated {
            self.size = size | Self::ALLOC_BIT;
        } else {
            self.size = size;
        }
    }

    pub fn next(&self) -> *mut BlockHeader {
        (self as *const Self as usize + self.size()) as *mut BlockHeader
    }

    pub fn prev(&self) -> *mut BlockHeader {
        (self as *const Self as usize - self.prev_size) as *mut BlockHeader
    }
}

pub struct LinkedListAllocator {
    start_addr: usize,
    end_addr: usize,
}

impl LinkedListAllocator {
    pub const fn new() -> Self {
        Self { start_addr: 0, end_addr: 0 }
    }

    pub fn init(&mut self) {
        // 初始时不管理任何区域，等待 sbrk 扩展
    }

    pub fn header_at(addr: usize) -> &'static mut BlockHeader {
        unsafe { &mut *(addr as *mut BlockHeader) }
    }

    pub fn add_region(&mut self, addr: usize, size: usize) {
        // 如果物理上紧跟在 end_addr 之后，尝试与之前管理的区域块合并
        // 注意：addr 是新申请页的起始地址，self.end_addr 是之前管理区域的结束地址
        if self.start_addr != 0 && addr == self.end_addr {
            // 找到之前区域的最后一个块
            let last_footer_addr = self.end_addr - core::mem::size_of::<usize>();
            let last_size = unsafe { *(last_footer_addr as *const usize) };
            let last_header_addr = self.end_addr - last_size;
            let last_header = Self::header_at(last_header_addr);

            // 如果最后一个块是空闲的，直接扩大它
            if !last_header.is_allocated() {
                let new_total_size = last_size + size;
                last_header.set_size(new_total_size);
                // 更新新块尾部的 footer
                unsafe {
                    let footer_ptr = (addr + size - core::mem::size_of::<usize>()) as *mut usize;
                    footer_ptr.write(new_total_size);
                }
                self.end_addr = addr + size;
                return;
            }
        }

        let header = Self::header_at(addr);
        let prev_size = if self.start_addr == 0 {
            self.start_addr = addr;
            0
        } else {
            // 如果物理上不连续，或者前一个块不可合并，记录到前一个物理边界的距离
            addr - self.end_addr
        };

        *header = BlockHeader::new(size, prev_size, false);
        // 在块尾部也存储一份 size，方便从后向前查找（类似 boundary tag）
        unsafe {
            let footer_ptr = (addr + size - core::mem::size_of::<usize>()) as *mut usize;
            footer_ptr.write(size);
        }
        self.end_addr = addr + size;
    }

    fn find_free_block(&mut self, size: usize, align: usize) -> Option<usize> {
        let mut curr_addr = self.start_addr;
        while curr_addr < self.end_addr {
            let header = Self::header_at(curr_addr);
            let block_size = header.size();

            if !header.is_allocated() {
                let addr =
                    (curr_addr + core::mem::size_of::<BlockHeader>() + align - 1) & !(align - 1);
                let payload_offset = addr - curr_addr;

                if block_size >= payload_offset + size {
                    return Some(curr_addr);
                }
            }
            curr_addr += block_size;
        }
        None
    }

    fn size_align(layout: Layout) -> (usize, usize) {
        let size = layout.size().max(core::mem::size_of::<usize>() * 2);
        let align = layout.align().max(core::mem::align_of::<BlockHeader>());
        // 向上对齐到 BlockHeader 的对齐要求
        let size = (size + core::mem::size_of::<BlockHeader>() - 1)
            & !(core::mem::size_of::<BlockHeader>() - 1);
        (size, align)
    }
}

pub struct LockedLinkedListAllocator(Mutex<LinkedListAllocator>);

impl LockedLinkedListAllocator {
    pub const fn new() -> Self {
        Self(Mutex::new(LinkedListAllocator::new()))
    }

    pub fn lock(&self) -> MutexGuard<'_, LinkedListAllocator> {
        self.0.lock()
    }
}

unsafe impl GlobalAlloc for LockedLinkedListAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let (size, align) = LinkedListAllocator::size_align(layout);
        let mut allocator = self.0.lock();

        if let Some(header_addr) = allocator.find_free_block(size, align) {
            let header = LinkedListAllocator::header_at(header_addr);
            let total_size = header.size();

            // 尝试分割块 (Split)
            // 剩下的空间至少要能容纳一个新的 Header + 两个 usize 的 Payload (即 size_align 的最小值)
            let min_block_size =
                core::mem::size_of::<BlockHeader>() + core::mem::size_of::<usize>() * 2;
            let needed_full_size = core::mem::size_of::<BlockHeader>() + size;

            if total_size >= needed_full_size + min_block_size {
                let remaining_size = total_size - needed_full_size;
                header.set_size(needed_full_size);
                header.set_allocated(true);

                // 更新当块的 footer
                unsafe {
                    let footer_ptr = (header_addr + needed_full_size
                        - core::mem::size_of::<usize>())
                        as *mut usize;
                    footer_ptr.write(needed_full_size);
                }

                // 创建并更新新块 (Remaining block)
                let next_header_addr = header_addr + needed_full_size;
                let next_header = LinkedListAllocator::header_at(next_header_addr);
                *next_header = BlockHeader::new(remaining_size, needed_full_size, false);
                unsafe {
                    let next_footer_ptr = (next_header_addr + remaining_size
                        - core::mem::size_of::<usize>())
                        as *mut usize;
                    next_footer_ptr.write(remaining_size);
                }

                // 如果新块后面还有块，更新其 prev_size
                let next_next_addr = next_header_addr + remaining_size;
                if next_next_addr < allocator.end_addr {
                    let next_next_header = LinkedListAllocator::header_at(next_next_addr);
                    next_next_header.prev_size = remaining_size;
                }
            } else {
                header.set_allocated(true);
            }

            let payload_addr = header_addr + core::mem::size_of::<BlockHeader>();
            payload_addr as *mut u8
        } else {
            // 通过 sbrk 扩展堆
            let request_size = (size + core::mem::size_of::<BlockHeader>() + 4095) & !4095;
            match sbrk(request_size as isize) {
                Ok(new_heap_start) => {
                    allocator.add_region(new_heap_start, request_size);
                    drop(allocator);
                    unsafe { self.alloc(layout) }
                }
                Err(_) => ptr::null_mut(),
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        let header_addr = ptr as usize - core::mem::size_of::<BlockHeader>();
        let allocator = self.0.lock();
        let header = LinkedListAllocator::header_at(header_addr);
        header.set_allocated(false);

        let curr_header_addr = header_addr;
        let mut curr_size = header.size();

        // 1. 向后合并
        let next_addr = curr_header_addr + curr_size;
        if next_addr < allocator.end_addr {
            let next_header = LinkedListAllocator::header_at(next_addr);
            if !next_header.is_allocated() {
                curr_size += next_header.size();
                // 物理合并后，更新当前块的大小
                header.set_size(curr_size);
                // 更新合并后大块的 footer
                unsafe {
                    let footer_ptr = (curr_header_addr + curr_size - core::mem::size_of::<usize>())
                        as *mut usize;
                    footer_ptr.write(curr_size);
                }
                // 更新再后一个块的 prev_size
                let next_next_addr = curr_header_addr + curr_size;
                if next_next_addr < allocator.end_addr {
                    let next_next_header = LinkedListAllocator::header_at(next_next_addr);
                    next_next_header.prev_size = curr_size;
                }
            }
        }

        // 2. 向前合并
        if header.prev_size > 0 {
            let prev_addr = curr_header_addr - header.prev_size;
            let prev_header = LinkedListAllocator::header_at(prev_addr);
            if !prev_header.is_allocated() {
                let total_merged_size = prev_header.size() + curr_size;
                prev_header.set_size(total_merged_size);
                // 更新合并后大块的 footer
                unsafe {
                    let footer_ptr = (prev_addr + total_merged_size - core::mem::size_of::<usize>())
                        as *mut usize;
                    footer_ptr.write(total_merged_size);
                }
                // 更新后一个块的 prev_size
                let next_after_merged = prev_addr + total_merged_size;
                if next_after_merged < allocator.end_addr {
                    let next_header = LinkedListAllocator::header_at(next_after_merged);
                    next_header.prev_size = total_merged_size;
                }
            }
        }
    }
}

impl LockedLinkedListAllocator {
    pub fn add_free_region(&self, addr: usize, size: usize) {
        self.lock().add_region(addr, size);
    }
    pub fn init(&self) {}
}
