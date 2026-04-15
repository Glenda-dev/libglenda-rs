use crate::arch::mem::UTCB_VA;
use crate::protocol::hosted::{HostedMessage, HostedReply};
use bincode;
use lazy_static::lazy_static;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::Mutex;

lazy_static! {
    static ref GLOBAL_STREAM: Mutex<Option<UnixStream>> = Mutex::new(None);
}

pub fn init_hosted_ipc(path: &str) -> std::io::Result<()> {
    let stream = UnixStream::connect(path)?;
    // 设置非阻塞可能导致问题，保持默认阻塞。
    let mut global = GLOBAL_STREAM.lock().unwrap();
    *global = Some(stream);
    Ok(())
}

pub unsafe fn syscall(cptr: usize, method: usize) -> usize {
    let mut global = GLOBAL_STREAM.lock().unwrap();
    if let Some(ref mut stream) = *global {
        let msg = HostedMessage::SysInvoke { cptr, method, utcb_ptr: UTCB_VA };

        // 序列化消息
        let bytes = bincode::serialize(&msg).unwrap();
        let len = bytes.len() as u32;

        // 写入长度和消息
        if stream.write_all(&len.to_le_bytes()).is_err() {
            return usize::MAX;
        }
        if stream.write_all(&bytes).is_err() {
            return usize::MAX;
        }
        if stream.flush().is_err() {
            return usize::MAX;
        }

        // 等待响应长度
        let mut len_buf = [0u8; 4];
        if stream.read_exact(&mut len_buf).is_err() {
            return usize::MAX;
        }
        let reply_len = u32::from_le_bytes(len_buf) as usize;

        // 等待响应内容
        let mut reply_buf = vec![0u8; reply_len];
        if stream.read_exact(&mut reply_buf).is_err() {
            return usize::MAX;
        }

        let reply: HostedReply = bincode::deserialize(&reply_buf).unwrap();

        match reply {
            HostedReply::Success { ret } => ret,
            HostedReply::Error(err) => err as usize,
        }
    } else {
        // 如果没有连接到 hutch，直接通过 std 输出报错并返回错误
        // 注意：这里不要递归调用 glenda::println 避免循环依赖，直接使用 eprintln
        eprintln!(
            "[GLENDA-HOSTED] Syscall failed: Not connected to hutch (cptr: 0x{:x}, method: {})",
            cptr, method
        );
        usize::MAX
    }
}

#[inline(always)]
pub unsafe fn syscall_ipc(
    cptr: usize,
    method: usize,
    msgtag: &mut usize,
    badge: &mut usize,
    mrs: &mut [usize; 4],
) -> usize {
    let _ = (msgtag, badge, mrs);
    unsafe { syscall(cptr, method) }
}
