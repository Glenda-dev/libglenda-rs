use crate::arch::hosted::mem::UTCB_VA;
use crate::arch::hosted::syscall::init_hosted_ipc;
use std::env;
use crate::ipc::ThreadControlBlock;

pub fn crt0_init() {
    let socket_path = env::var("GLENDA_HUTCH_SOCK").unwrap_or("/tmp/glenda_hutch.sock".to_string());
    init_hosted_ipc(&socket_path).expect("Failed to connect to hutch");

    // 映射 UTCB 区域。在 Hosted 模式下，我们将该地址固定，以便 hutch 识别。
    unsafe {
        let ret = libc::mmap(
            UTCB_VA as *mut libc::c_void,
            4096,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_FIXED,
            -1,
            0,
        );
        if ret == libc::MAP_FAILED {
            panic!(
                "Failed to mmap fixed UTCB at 0x{:x}. Error: {}",
                UTCB_VA, std::io::Error::last_os_error()
            );
        }
    }

    // 初始化主线程的 TCB 和 TLS 环境
    unsafe {
        static mut MAIN_TCB: ThreadControlBlock = ThreadControlBlock::new();
        MAIN_TCB.self_ptr = core::ptr::addr_of_mut!(MAIN_TCB) as usize;
        MAIN_TCB.tid = 0;
        crate::arch::thread::set_thread_pointer(core::ptr::addr_of_mut!(MAIN_TCB) as usize);
    }
}

pub unsafe fn panic_break() {
    #[cfg(target_os = "linux")]
    unsafe { libc::raise(libc::SIGTRAP); }
}

pub fn backtrace() {
    println!("--- HOSTED BACKTRACE (Use GDB/LLDB to debug) ---");
}
