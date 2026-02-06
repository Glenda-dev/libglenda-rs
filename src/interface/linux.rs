pub trait LinuxFileSystemService {
    fn sys_getcwd(&self, buf: *mut u8, size: usize) -> isize;
    fn sys_dup(&self, oldfd: usize) -> isize;
    fn sys_dup3(&self, oldfd: usize, newfd: usize, flags: usize) -> isize;
    fn sys_mkdirat(&self, dirfd: usize, path: *const u8, mode: usize) -> isize;
    fn sys_unlinkat(&self, dirfd: usize, path: *const u8, flags: usize) -> isize;
    fn sys_chdir(&self, path: *const u8) -> isize;
    fn sys_openat(&self, dirfd: usize, path: *const u8, flags: usize, mode: usize) -> isize;
    fn sys_close(&self, fd: usize) -> isize;
    fn sys_pipe2(&self, pipefd: *mut i32, flags: usize) -> isize;
    fn sys_getdents64(&self, fd: usize, dirp: *mut u8, count: usize) -> isize;
    fn sys_lseek(&self, fd: usize, offset: isize, whence: usize) -> isize;
    fn sys_read(&self, fd: usize, buf: *mut u8, count: usize) -> isize;
    fn sys_write(&self, fd: usize, buf: *const u8, count: usize) -> isize;
    fn sys_readlinkat(&self, dirfd: usize, path: *const u8, buf: *mut u8, bufsize: usize) -> isize;
    fn sys_newfstatat(
        &self,
        dirfd: usize,
        path: *const u8,
        statbuf: *mut u8,
        flags: usize,
    ) -> isize;
    fn sys_fstat(&self, fd: usize, statbuf: *mut u8) -> isize;
}

pub trait LinuxProcessService {
    fn sys_exit(&self, error_code: usize) -> !;
    fn sys_exit_group(&self, error_code: usize) -> !;
    fn sys_kill(&self, pid: usize, sig: usize) -> isize;
    fn sys_getpid(&self) -> isize;
    fn sys_getppid(&self) -> isize;
    fn sys_getuid(&self) -> isize;
    fn sys_geteuid(&self) -> isize;
    fn sys_getgid(&self) -> isize;
    fn sys_getegid(&self) -> isize;
    fn sys_gettid(&self) -> isize;
    fn sys_clone(
        &self,
        flags: usize,
        stack: usize,
        ptid: *mut u32,
        tls: usize,
        ctid: *mut u32,
    ) -> isize;
    fn sys_execve(
        &self,
        pathname: *const u8,
        argv: *const *const u8,
        envp: *const *const u8,
    ) -> isize;
    fn sys_wait4(&self, pid: isize, wstatus: *mut i32, options: usize, rusage: *mut u8) -> isize;
    fn sys_prlimit64(
        &self,
        pid: usize,
        resource: usize,
        new_limit: *const u8,
        old_limit: *mut u8,
    ) -> isize;
}

pub trait LinuxMemoryService {
    fn sys_brk(&self, brk: usize) -> isize;
    fn sys_munmap(&self, addr: usize, length: usize) -> isize;
    fn sys_mmap(
        &self,
        addr: usize,
        length: usize,
        prot: usize,
        flags: usize,
        fd: usize,
        offset: usize,
    ) -> isize;
    fn sys_mprotect(&self, addr: usize, length: usize, prot: usize) -> isize;
}

pub trait LinuxTimeService {
    fn sys_clock_gettime(&self, clockid: usize, tp: *mut u8) -> isize;
}

pub trait LinuxMiscService {
    fn sys_uname(&self, buf: *mut u8) -> isize;
}
