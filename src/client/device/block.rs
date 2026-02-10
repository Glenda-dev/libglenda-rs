use crate::cap::Endpoint;
use crate::error::Error;
use crate::interface::device::BlockDevice;
use crate::ipc::{MsgFlags, MsgTag, UTCB};
use crate::protocol::device::{BLOCK_PROTO, block};
use crate::set_mrs;

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
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        let tag = MsgTag::new(BLOCK_PROTO, block::GET_CAPACITY, MsgFlags::NONE);
        utcb.set_msg_tag(tag);
        match self.endpoint.call(&mut utcb) {
            Ok(_) => utcb.get_mr(0) as u64,
            Err(_) => 0,
        }
    }

    fn block_size(&self) -> u32 {
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        let tag = MsgTag::new(BLOCK_PROTO, block::GET_BLOCK_SIZE, MsgFlags::NONE);
        utcb.set_msg_tag(tag);
        match self.endpoint.call(&mut utcb) {
            Ok(_) => utcb.get_mr(0) as u32,
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
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        let max_payload = utcb.ipc_buffer().len();

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
            set_mrs!(utcb, current_sector as usize, num_blocks);
            utcb.set_msg_tag(tag);

            self.endpoint.call(&mut utcb)?;

            let len = utcb.get_size();
            chunk[..len].copy_from_slice(&utcb.ipc_buffer()[..len]);

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
        let utcb = unsafe { UTCB::new() };

        let max_payload = utcb.ipc_buffer().len();

        let blocks_per_req = max_payload / bs;
        if blocks_per_req == 0 {
            return Err(Error::InvalidArgs);
        }

        for chunk in buf.chunks(blocks_per_req * bs) {
            let num_blocks = chunk.len() / bs;
            if num_blocks == 0 {
                break;
            }

            utcb.clear();
            utcb.ipc_buffer()[..chunk.len()].copy_from_slice(chunk);

            let tag = MsgTag::new(BLOCK_PROTO, block::WRITE_BLOCKS, MsgFlags::NONE);
            utcb.set_size(chunk.len());
            set_mrs!(utcb, current_sector as usize, num_blocks);
            utcb.set_msg_tag(tag);

            self.endpoint.call(utcb)?;

            // Protocol assumes void return on success or error
            total_written += chunk.len();
            current_sector += num_blocks as u64;
        }

        Ok(total_written)
    }

    fn sync(&mut self) -> Result<(), Error> {
        let tag = MsgTag::new(BLOCK_PROTO, block::SYNC, MsgFlags::NONE);
        let mut utcb = unsafe { UTCB::new() };
        utcb.clear();
        utcb.set_msg_tag(tag);
        self.endpoint.call(&mut utcb)
    }
}
