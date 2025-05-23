mod fs;
mod gui;
mod input;
mod net;
pub mod process;
mod sync;
mod thread;
pub mod user_space;

use fs::*;
use gui::*;
use input::*;
use net::*;
use process::*;
use sync::*;
use thread::*;
#[allow(unused)]
extern crate syscall_nr;
use syscall_nr::call;
use user_space::__user;

pub fn syscall(syscall_id: usize, args: [usize; 6]) -> isize {
    match syscall_id {
        call::DUP3 => sys_dup(args[0]),
        call::CONNECT => sys_connect(args[0] as _, args[1] as _, args[2] as _),
        call::LISTEN => sys_listen(args[0] as _),
        call::ACCEPT => sys_accept(args[0] as _),
        call::OPENAT => sys_open(__user::new(args[0] as *const u8), args[1] as u32),
        call::CLOSE => sys_close(args[0]),
        call::PIPE2 => sys_pipe(__user::new(args[0] as *mut usize)),
        call::READ => sys_read(args[0], __user::new(args[1] as *const u8), args[2]),
        call::WRITE => sys_write(args[0], __user::new(args[1] as *const u8), args[2]),
        call::EXIT => sys_exit(args[0] as i32),
        call::NANOSLEEP => sys_sleep(args[0]),
        call::SCHED_YIELD => sys_yield(),
        call::KILL => sys_kill(args[0], args[1] as u32),
        call::GETTIMEOFDAY => sys_get_time(),
        call::GETPID => sys_getpid(),
        call::CLONE => sys_fork(),
        call::EXECVE => sys_exec(
            __user::new(args[0] as *const u8),
            __user::new(args[1] as *const usize),
        ),
        call::WAIT4 => sys_waitpid(args[0] as isize, __user::new(args[1] as *mut i32)),
        call::THREAD_CREATE => sys_thread_create(args[0], args[1]),
        call::GETTID => sys_gettid(),
        call::WAITID => sys_waittid(args[0]) as isize,
        call::MUTEX_CREATE => sys_mutex_create(args[0] == 1),
        call::MUTEX_LOCK => sys_mutex_lock(args[0]),
        call::MUTEX_UNLOCK => sys_mutex_unlock(args[0]),
        call::SEMAPHORE_CREATE => sys_semaphore_create(args[0]),
        call::SEMAPHORE_UP => sys_semaphore_up(args[0]),
        call::SEMAPHORE_DOWN => sys_semaphore_down(args[0]),
        call::CONDVAR_CREATE => sys_condvar_create(),
        call::CONDVAR_SIGNAL => sys_condvar_signal(args[0]),
        call::CONDVAR_WAIT => sys_condvar_wait(args[0], args[1]),
        call::FRAMEBUFFER => sys_framebuffer(),
        call::FRAMEBUFFER_FLUSH => sys_framebuffer_flush(),
        call::EVENT_GET => sys_event_get(),
        call::KEY_PRESSED => sys_key_pressed(),

        call::GETCWD => sys_getcwd(__user::new(args[0] as *const u8), args[1]),
        // TODO, only interface here.
        call::CHDIR => sys_chdir(__user::new(args[0] as *const u8)),
        call::FCHDIR => sys_fchdir(args[0]),
        call::MKDIRAT => sys_mkdirat(args[0] as isize, __user::new(args[1] as *const u8), args[2]),
        call::UNLINKAT => sys_unlinkat(args[0] as isize, __user::new(args[1] as *const u8)),
        call::SYMLINKAT => sys_symlinkat(
            __user::new(args[0] as *const u8),
            args[1] as isize,
            __user::new(args[2] as *const u8),
        ),
        call::LINKAT => sys_linkat(
            args[0] as isize,
            __user::new(args[1] as *const u8),
            args[2] as isize,
            __user::new(args[3] as *const u8),
            args[4] as isize,
        ),

        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
