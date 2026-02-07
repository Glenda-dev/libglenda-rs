use crate::cap::Endpoint;
use crate::error::Error;
use crate::interface::device::BlockDevice;
use crate::ipc::{MsgFlags, MsgTag, UTCB};
use crate::protocol::device::{BLOCK_PROTO, block};

pub struct BlockClient {
    endpoint: Endpoint,
}

impl BlockClient {
    pub const fn new(endpoint: Endpoint) -> Self {
        Self { endpoint }
    }
}

impl BlockDevice for BlockClient {
    fn capacity(&self) -> u64 {
        let utcb = unsafe { UTCB::get() };
        let tag = MsgTag::new(BLOCK_PROTO, block::GET_CAPACITY, MsgFlags::NONE);

        match self.endpoint.call(tag) {
            Ok(_) => utcb.mrs_regs[0] as u64,
            Err(_) => 0,
        }
    }

    fn block_size(&self) -> u32 {
        let utcb = unsafe { UTCB::get() };
        let tag = MsgTag::new(BLOCK_PROTO, block::GET_BLOCK_SIZE, MsgFlags::NONE);

        match self.endpoint.call(tag) {
            Ok(_) => utcb.mrs_regs[0] as u32,
            Err(_) => 512,
        }
    }

    fn read_blocks(&mut self, sector: u64, buf: &mut [u8]) -> Result<usize, Error> {
        let bs = self.block_size() as usize;
        if bs == 0 {
            return Err(Error::InvalidArgs);
        }

        let mut current_sector = sector;
        let mut total_read = 0;
        let utcb = unsafe { UTCB::get() };
        let max_payload = utcb.ipc_buffer.len();

        // Calculate max blocks per request that fit in buffer
        let blocks_per_req = max_payload / bs;
        if blocks_per_req == 0 {
            return Err(Error::InvalidArgs);
        }

        for chunk in buf.chunks_mut(blocks_per_req * bs) {
            let num_blocks = chunk.len() / bs;
            if num_blocks == 0 {
                break;
            }

            let tag = MsgTag::new(BLOCK_PROTO, block::READ_BLOCKS, MsgFlags::NONE);
            utcb.mrs_regs[0] = current_sector as usize;
            utcb.mrs_regs[1] = num_blocks;

            self.endpoint.call(tag)?;

            let len = utcb.size;
            chunk[..len].copy_from_slice(&utcb.ipc_buffer[..len]);

            total_read += len;
            current_sector += num_blocks as u64;

            if len < chunk.len() {
                break; // Short read
            }
        }

        Ok(total_read)
    }

    fn write_blocks(&mut self, sector: u64, buf: &[u8]) -> Result<usize, Error> {
        let bs = self.block_size() as usize;
        if bs == 0 {
            return Err(Error::InvalidArgs);
        }

        let mut current_sector = sector;
        let mut total_written = 0;
        let utcb = unsafe { UTCB::get() };
        let max_payload = utcb.ipc_buffer.len();

        let blocks_per_req = max_payload / bs;
        if blocks_per_req == 0 {
            return Err(Error::InvalidArgs);
        }

        for chunk in buf.chunks(blocks_per_req * bs) {
            let num_blocks = chunk.len() / bs;
            if num_blocks == 0 {
                break;
            }

            utcb.ipc_buffer[..chunk.len()].copy_from_slice(chunk);

            let tag = MsgTag::new(BLOCK_PROTO, block::WRITE_BLOCKS, MsgFlags::NONE);
            utcb.size = chunk.len();
            utcb.mrs_regs[0] = current_sector as usize;
            utcb.mrs_regs[1] = num_blocks;

            self.endpoint.call(tag)?;

            // Protocol assumes void return on success or error
            total_written += chunk.len();
            current_sector += num_blocks as u64;
        }

        Ok(total_written)
    }

    fn sync(&mut self) -> Result<(), Error> {
        let tag = MsgTag::new(BLOCK_PROTO, block::SYNC, MsgFlags::NONE);
        self.endpoint.call(tag)
    }
}
