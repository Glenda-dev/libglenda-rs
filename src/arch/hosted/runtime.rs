use crate::arch::hosted::mem::UTCB_VA;
use crate::arch::hosted::syscall::init_hosted_ipc;
use crate::ipc::ThreadControlBlock;
use std::env;

extern "C" fn siginfo_handler(
    sig: libc::c_int,
    info: *mut libc::siginfo_t,
    _ucontext: *mut libc::c_void,
) {
    let addr = unsafe { if !info.is_null() { (*info).si_addr() } else { core::ptr::null_mut() } };
    println!(
        "--- Glenda Hosted Runtime Error ---\nReceived signal: {}\nFaulting address: {:?}",
        sig, addr
    );
    std::process::exit(128 + sig as i32);
}

pub fn crt0_init() {
    unsafe {
        let mut sa: libc::sigaction = core::mem::zeroed();
        sa.sa_sigaction = siginfo_handler as *const () as usize;
        sa.sa_flags = libc::SA_SIGINFO;
        libc::sigaction(libc::SIGSEGV, &sa, core::ptr::null_mut());
        libc::sigaction(libc::SIGILL, &sa, core::ptr::null_mut());
        libc::sigaction(libc::SIGBUS, &sa, core::ptr::null_mut());
        libc::sigaction(libc::SIGFPE, &sa, core::ptr::null_mut());
    }

    let socket_path = env::var("GLENDA_HUTCH_SOCK").unwrap_or("/tmp/glenda_hutch.sock".to_string());
    init_hosted_ipc(&socket_path).expect("Failed to connect to hutch");

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
                UTCB_VA,
                std::io::Error::last_os_error()
            );
        }
    }

    unsafe {
        static mut MAIN_TCB: ThreadControlBlock = ThreadControlBlock::new();
        MAIN_TCB.self_ptr = core::ptr::addr_of_mut!(MAIN_TCB) as usize;
        MAIN_TCB.tid = 0;
        crate::arch::thread::set_thread_pointer(core::ptr::addr_of_mut!(MAIN_TCB) as usize);
    }
}

pub unsafe fn panic_break() {
    #[cfg(target_os = "linux")]
    unsafe {
        libc::raise(libc::SIGTRAP);
    }
}

pub fn backtrace() {
    println!("--- HOSTED BACKTRACE (Use GDB/LLDB to debug) ---");
}

pub const ARCH: &str = std::env::consts::ARCH;
