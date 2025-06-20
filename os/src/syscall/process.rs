use crate::{
    board::CLOCK_FREQ,
    fs::{
        open_file,
        OpenFlags,
    },
    mm::{
        translated_byte_buffer,
        translated_ref,
        translated_refmut,
        translated_str,
        UserBuffer,
    },
    syscall::thread::sys_gettid,
    task::{
        add_task,
        current_process,
        current_task,
        current_user_token,
        exit_current_and_run_next,
        pid2process,
        suspend_current_and_run_next,
        ProcessControlBlock,
        SignalFlags,
        TaskStruct,
    },
    timer::get_time_ms,
};
use alloc::{
    collections::VecDeque,
    string::String,
    sync::Arc,
    vec::Vec,
};
use shared_defination::{
    clone::{
        self,
        flag::CLONE_PARENT_SETTID,
    },
    times::Tms,
};

use super::{
    user_space::__user,
    TimeVal,
};

pub fn sys_exit(exit_code: i32) -> ! {
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_get_time(ts: __user<*mut TimeVal>, _tz: i32) -> isize {
    let ptr = ts.inner() as *mut u64;
    let sec = translated_refmut(current_user_token(), __user::new(ptr));
    let usec = translated_refmut(current_user_token(), __user::new(unsafe { ptr.add(8) }));
    let t = get_time_ms();
    *sec = (t / 1000) as u64;
    *usec = 0;
    0
}

pub fn sys_getpid() -> isize {
    current_task().unwrap().process.upgrade().unwrap().getpid() as isize
}

pub fn sys_getcwd(buf: __user<*const u8>, buf_len: usize) -> isize {
    let process = current_process();
    let cwd = process.getcwd();
    if buf_len <= cwd.len() {
        return 0;
    }

    let user_buf = UserBuffer::new(translated_byte_buffer(current_user_token(), buf, buf_len));

    for (index, ch) in user_buf.into_iter().enumerate() {
        if index == cwd.len() {
            unsafe { ch.write('\0' as u8) };
            return buf.inner() as isize;
        }
        unsafe { ch.write(cwd.as_bytes()[index]) };
    }

    return 0;
}

pub fn sys_chdir(path: __user<*const u8>) -> isize {
    let new = translated_str(current_user_token(), path);
    current_process().chdir(new.as_str())
}

pub fn sys_fchdir(fd: usize) -> isize {
    current_process().fchdir(fd)
}

// TODO
pub fn sys_mkdirat(_dfd: isize, name: __user<*const u8>, _mode: usize) -> isize {
    let new = translated_str(current_user_token(), name);
    current_process().mkdirat(new.as_str())
}

pub fn sys_unlinkat(_dfd: isize, name: __user<*const u8>) -> isize {
    let new = translated_str(current_user_token(), name);
    current_process().unlinkat(new.as_str())
}

#[allow(unused)]
pub fn sys_symlinkat(
    _oldname: __user<*const u8>, _newdfd: isize, _newname: __user<*const u8>,
) -> isize {
    // TODO
    0
}

#[allow(unused)]
pub fn sys_linkat(
    _olddfd: isize, _oldname: __user<*const u8>, _newdfd: isize, _newname: __user<*const u8>,
    _flags: isize,
) -> isize {
    //TODO
    0
}

pub fn sys_clone(
    flags: usize, stack: usize, ptid: __user<*mut u32>, _tls: __user<*mut usize>,
    ctid: __user<*mut u32>,
) -> isize {
    // fork
    if flags & clone::flag::CLONE_VM == 0 {
        return sys_fork(flags, stack);
    }

    let task = current_task().unwrap();
    let process = task.process.upgrade().unwrap();
    // create a new thread
    let new_task = Arc::new(TaskStruct::new(
        Arc::clone(&process),
        flags,
        ctid,
        task.inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .ustack_base,
        true, // TODO: user stack was allocate by parent thread.
    ));

    let new_task_inner = new_task.inner_exclusive_access();
    let new_task_res = new_task_inner.res.as_ref().unwrap();
    let new_task_tid = new_task_res.tid;
    let mut process_inner = process.inner_exclusive_access();
    // add new thread to current process
    let tasks = &mut process_inner.tasks;
    while tasks.len() < new_task_tid + 1 {
        tasks.push(None);
    }
    tasks[new_task_tid] = Some(Arc::clone(&new_task));
    let new_task_trap_ctx = new_task_inner.get_trap_ctx();
    let task_inner = task.inner_exclusive_access();
    *new_task_trap_ctx = *task_inner.get_trap_ctx();
    (*new_task_trap_ctx).x[2] = stack;
    (*new_task_trap_ctx).x[10] = 0;

    drop(process_inner);
    if flags & CLONE_PARENT_SETTID != 0 {
        if ptid.inner() as usize != 0usize {
            let ptid_addr = translated_refmut(current_user_token(), ptid);
            *ptid_addr = new_task_tid as u32;
        } else {
            return -1;
        }
    }

    // add new task to scheduler
    add_task(Arc::clone(&new_task));

    new_task_tid as isize
}

pub fn sys_fork(_flags: usize, stack: usize) -> isize {
    let current_process = current_process();
    let new_process = current_process.fork();
    let new_pid = new_process.getpid();
    // modify trap context of new_task, because it returns immediately after switching
    let new_process_inner = new_process.inner_exclusive_access();
    let task = new_process_inner.tasks[0].as_ref().unwrap();
    let trap_ctx = task.inner_exclusive_access().get_trap_ctx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_ctx.x[10] = 0;

    if stack != 0 {
        trap_ctx.x[2] = stack;
    }

    new_pid as isize
}

pub fn sys_exec(path: __user<*const u8>, mut args: __user<*const usize>) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);
    let mut args_vec: Vec<String> = Vec::new();
    loop {
        let arg_str_ptr = *translated_ref(token, args);
        if arg_str_ptr == 0 {
            break;
        }
        args_vec.push(translated_str(token, (arg_str_ptr as *const u8).into()));
        unsafe {
            args = __user::new(args.inner().add(1));
        }
    }
    if let Some(app_inode) = open_file(path.as_str(), OpenFlags::RDONLY) {
        let all_data = app_inode.read_all();
        let process = current_process();
        let argc = args_vec.len();
        process.exec(all_data.as_slice(), args_vec);
        // return argc because cx.x[10] will be covered with it later
        argc as isize
    } else {
        -1
    }
}

pub fn sys_wait4(pid: isize, exit_code_ptr: __user<*mut i32>) -> isize {
    let mut ret = waitpid(pid, exit_code_ptr);
    while ret == -2 {
        sys_yield();
        ret = waitpid(pid, exit_code_ptr);
    }
    ret
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
fn waitpid(pid: isize, exit_code_ptr: __user<*mut i32>) -> isize {
    let process = current_process();
    // find a child process

    let mut inner = process.inner_exclusive_access();
    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
        // ---- release current PCB
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        // ++++ temporarily access child PCB exclusively
        p.inner_exclusive_access().is_zombie && (pid == -1 || pid as usize == p.getpid())
        // ++++ release child PCB
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);

        {
            let sys_timme = child.get_systime();
            let csys_timme = child.get_chlid_systime();
            inner.accumulate_systime(sys_timme + csys_timme);
        }
        {
            // let usr_timme = child.get_usrtime();
            // let cusr_timme = child.get_child_usrtime();
            // process.accumulate_usrtime(usr_timme + cusr_timme);
        }
        // confirm that child will be deallocated after being removed from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily access child PCB exclusively
        let exit_code = child.inner_exclusive_access().exit_code;
        // ++++ release child PCB
        if exit_code_ptr.inner() as usize != 0 {
            *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        }
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB automatically
}

pub fn sys_kill(pid: usize, signal: u32) -> isize {
    if let Some(process) = pid2process(pid) {
        if let Some(flag) = SignalFlags::from_bits(signal) {
            process.inner_exclusive_access().signals |= flag;
            0
        } else {
            -1
        }
    } else {
        -1
    }
}

pub fn sys_times(times_uaddr: __user<*mut Tms>) -> isize {
    if times_uaddr.inner() as usize == 0 {
        return -1;
    }
    let times_uaddr = times_uaddr.inner() as usize;

    let process = current_process();
    let mut tms_usrtime = translated_refmut(
        current_user_token(),
        __user::from((times_uaddr + 0x00) as *mut usize),
    );
    let mut tms_systime = translated_refmut(
        current_user_token(),
        __user::from((times_uaddr + 0x08) as *mut usize),
    );
    let mut tms_child_usrtime = translated_refmut(
        current_user_token(),
        __user::from((times_uaddr + 0x10) as *mut usize),
    );
    let mut tms_child_systime = translated_refmut(
        current_user_token(),
        __user::from((times_uaddr + 0x18) as *mut usize),
    );

    let process = current_process();

    {
        let times = process.get_times();
        *tms_usrtime = times.tms_usrtime;
        *tms_systime = times.tms_systime;
        *tms_child_usrtime = times.tms_child_usrtime;
        *tms_child_systime = times.tms_child_systime;
        drop(times);
    }

    let inner = process.inner_exclusive_access();
    inner.children.iter().for_each(|child| {
        let child_inner = child.inner_exclusive_access();
        if child_inner.is_zombie {
            let times = &child_inner.times;
            *tms_child_usrtime += times.tms_usrtime + times.tms_child_usrtime;
            *tms_child_systime += times.tms_systime + times.tms_child_systime;
        }
    });

    *tms_usrtime /= CLOCK_FREQ;
    *tms_systime /= CLOCK_FREQ;
    *tms_child_usrtime /= CLOCK_FREQ;
    *tms_child_systime /= CLOCK_FREQ;

    // for child in inner.children.iter() {
    //     let child_inner = child.inner_exclusive_access();
    //     if child_inner.is_zombie {
    //         let times = &child_inner.times;
    //         *tms_child_usrtime += times.tms_usrtime + times.tms_child_usrtime;
    //         *tms_child_systime += times.tms_systime + times.tms_child_systime;
    //     }
    // }

    0
}
