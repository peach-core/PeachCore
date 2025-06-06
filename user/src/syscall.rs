extern crate shared_defination;
use shared_defination::syscall_nr::call;

bitflags! {
    pub struct MapProtect: u8{
        const R = 1 << 0;
        const W = 1 << 1;
        const X = 1 << 2;
    }
}

pub struct TimeVal {
    pub sec: u64, // 自 Unix 纪元起的秒数
    #[allow(dead_code)]
    pub usec: u64, // 微秒数
}

fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") id
        );
    }
    ret
}

pub fn sys_dup(fd: usize) -> isize {
    syscall(call::DUP3, [fd, 0, 0])
}

pub fn sys_connect(dest: u32, sport: u16, dport: u16) -> isize {
    syscall(
        call::CONNECT,
        [dest as usize, sport as usize, dport as usize],
    )
}

// just listen for tcp connections now
pub fn sys_listen(sport: u16) -> isize {
    syscall(call::LISTEN, [sport as usize, 0, 0])
}

pub fn sys_accept(socket_fd: usize) -> isize {
    syscall(call::ACCEPT, [socket_fd, 0, 0])
}

pub fn sys_open(path: &str, flags: u32) -> isize {
    syscall(call::OPENAT, [path.as_ptr() as usize, flags as usize, 0])
}

pub fn sys_close(fd: usize) -> isize {
    syscall(call::CLOSE, [fd, 0, 0])
}

pub fn sys_pipe(pipe: &mut [usize]) -> isize {
    syscall(call::PIPE2, [pipe.as_mut_ptr() as usize, 0, 0])
}

pub fn sys_read(fd: usize, buffer: &mut [u8]) -> isize {
    syscall(call::READ, [fd, buffer.as_mut_ptr() as usize, buffer.len()])
}

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(call::WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}

pub fn sys_exit(exit_code: i32) -> ! {
    syscall(call::EXIT, [exit_code as usize, 0, 0]);
    panic!("sys_exit never returns!");
}

pub fn sys_sleep(time: &TimeVal) -> isize {
    syscall(call::NANOSLEEP, [time as *const TimeVal as usize, 0, 0])
}

pub fn sys_yield() -> isize {
    syscall(call::SCHED_YIELD, [0, 0, 0])
}

pub fn sys_kill(pid: usize, signal: i32) -> isize {
    syscall(call::KILL, [pid, signal as usize, 0])
}

pub fn sys_get_time() -> isize {
    let mut time: TimeVal = TimeVal { sec: 0, usec: 0 };
    syscall(
        call::GETTIMEOFDAY,
        [&mut time as *mut TimeVal as usize, 0, 0],
    );
    return (time.sec as isize) * 1000;
}

pub fn sys_getpid() -> isize {
    syscall(call::GETPID, [0, 0, 0])
}

pub fn sys_fork() -> isize {
    syscall(call::CLONE, [0, 0, 0])
}

pub fn sys_exec(path: &str, args: &[*const u8]) -> isize {
    syscall(
        call::EXECVE,
        [path.as_ptr() as usize, args.as_ptr() as usize, 0],
    )
}

pub fn sys_waitpid(pid: isize, exit_code: *mut i32) -> isize {
    syscall(call::WAIT4, [pid as usize, exit_code as usize, 0])
}

pub fn sys_thread_create(entry: usize, arg: usize) -> isize {
    syscall(call::THREAD_CREATE, [entry, arg, 0])
}

pub fn sys_gettid() -> isize {
    syscall(call::GETTID, [0; 3])
}

pub fn sys_waittid(tid: usize) -> isize {
    syscall(call::WAITID, [tid, 0, 0])
}

pub fn sys_mutex_create(blocking: bool) -> isize {
    syscall(call::MUTEX_CREATE, [blocking as usize, 0, 0])
}

pub fn sys_mutex_lock(id: usize) -> isize {
    syscall(call::MUTEX_LOCK, [id, 0, 0])
}

pub fn sys_mutex_unlock(id: usize) -> isize {
    syscall(call::MUTEX_UNLOCK, [id, 0, 0])
}

pub fn sys_futex(
    uaddr: usize,
    futex_op: usize,
    val: usize,
    _val2: usize,
    _uaddr2: usize,
    _val3: usize,
) -> isize {
    syscall(call::FUTEX, [uaddr, futex_op, val])
}

pub fn sys_semaphore_create(res_count: usize) -> isize {
    syscall(call::SEMAPHORE_CREATE, [res_count, 0, 0])
}

pub fn sys_semaphore_up(sem_id: usize) -> isize {
    syscall(call::SEMAPHORE_UP, [sem_id, 0, 0])
}

pub fn sys_semaphore_down(sem_id: usize) -> isize {
    syscall(call::SEMAPHORE_DOWN, [sem_id, 0, 0])
}

pub fn sys_condvar_create() -> isize {
    syscall(call::CONDVAR_CREATE, [0, 0, 0])
}

pub fn sys_condvar_signal(condvar_id: usize) -> isize {
    syscall(call::CONDVAR_SIGNAL, [condvar_id, 0, 0])
}

pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> isize {
    syscall(call::CONDVAR_WAIT, [condvar_id, mutex_id, 0])
}

pub fn sys_framebuffer() -> isize {
    syscall(call::FRAMEBUFFER, [0, 0, 0])
}

pub fn sys_framebuffer_flush() -> isize {
    syscall(call::FRAMEBUFFER_FLUSH, [0, 0, 0])
}

pub fn sys_event_get() -> isize {
    syscall(call::EVENT_GET, [0, 0, 0])
}

pub fn sys_key_pressed() -> isize {
    syscall(call::KEY_PRESSED, [0, 0, 0])
}

pub fn sys_sbrk(size: isize) -> isize {
    syscall(call::BRK, [size as usize, 0, 0])
}

pub fn sys_mmap(addr: usize, len: usize, prot: MapProtect) -> isize {
    syscall(call::MMAP, [addr, len, prot.bits() as usize])
}

pub fn sys_munmap(addr: usize) -> isize {
    syscall(call::MUNMAP, [addr, 0, 0])
}
