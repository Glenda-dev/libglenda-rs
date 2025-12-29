mod syscall;
pub use syscall::PosixSyscall;

use crate::cap::CapPtr;
use crate::ipc::MsgTag;
use crate::ipc::utcb;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PosixRequest {
    pub syscall_id: PosixSyscall,
    pub args: [usize; 5],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PosixResponse {
    pub error: i32, // 0 为成功，负数为 errno
    pub ret: usize, // 返回值
}

impl PosixResponse {
    pub fn success(ret: usize) -> Self {
        Self { error: 0, ret }
    }

    pub fn error(errno: i32) -> Self {
        Self { error: errno, ret: 0 }
    }
}

/// POSIX 系统调用处理函数类型
/// badge: 调用者的身份标识（由内核注入）
/// req: 请求参数
pub type PosixHandler = fn(badge: usize, req: &PosixRequest) -> PosixResponse;

/// POSIX 服务分发器
pub struct PosixDispatcher {
    handlers: [Option<PosixHandler>; 256],
}

impl PosixDispatcher {
    pub const fn new() -> Self {
        Self { handlers: [None; 256] }
    }

    /// 注册一个系统调用处理函数
    pub fn register(&mut self, id: PosixSyscall, handler: PosixHandler) {
        self.handlers[id as usize] = Some(handler);
    }

    /// 分发并执行系统调用
    pub fn dispatch(&self, badge: usize, req: &PosixRequest) -> PosixResponse {
        let id = req.syscall_id as usize;
        if id < 256 {
            if let Some(handler) = self.handlers[id] {
                return handler(badge, req);
            }
        }
        PosixResponse::error(-38) // ENOSYS: Function not implemented
    }
}

pub extern "C" fn posix_call(posixd_cap: CapPtr, request: &PosixRequest) -> PosixResponse {
    let utcb = utcb::get();
    let msg_tag = MsgTag::new(request.syscall_id as usize, 5);
    let args = &request.args;

    let res = posixd_cap.ipc_call(msg_tag, args);

    if res == 0 {
        let err = utcb.mrs[0] as isize;
        if err >= 0 {
            PosixResponse::success(utcb.mrs[1])
        } else {
            PosixResponse::error(err as i32)
        }
    } else {
        PosixResponse::error(0) // IPC 调用失败
    }
}
