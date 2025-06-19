mod context;
mod dir_struct;
mod fd_table;
mod id;
mod manager;
mod process;
mod processor;
mod signal;
mod switch;
#[allow(clippy::module_inception)]
mod task;
pub mod wait_queue;

use core::intrinsics::atomic_store_release;

use self::id::TaskUserRes;
use crate::{
    fs::{
        open_file,
        OpenFlags,
    },
    mm::translated_refmut,
    sbi::shutdown,
    sync::sys_futex,
    syscall::user_space::__user,
};
use alloc::{
    sync::Arc,
    vec::Vec,
};
use lazy_static::*;
use log::trace;
use manager::fetch_task;
pub use process::ProcessControlBlock;
use shared_defination::{
    clone::flag::{
        CLONE_CHILD_CLEARTID,
        CLONE_VM,
    },
    syscall_nr::call::{
        CLONE,
        FUTEX,
    },
};
use switch::__switch;

pub use context::TaskContext;
pub use id::{
    kstack_alloc,
    pid_alloc,
    KernelStack,
    PidHandle,
    IDLE_PID,
};
pub use manager::{
    add_task,
    pid2process,
    remove_from_pid2process,
    wakeup_task,
};
pub use processor::{
    current_kstack_top,
    current_process,
    current_task,
    current_trap_ctx,
    current_trap_ctx_user_va,
    current_user_token,
    get_processor_slice_time,
    run_tasks,
    schedule,
    take_current_task,
};
pub use signal::SignalFlags;
pub use task::{
    TaskStatus,
    TaskStruct,
};

pub fn suspend_current_and_run_next() {
    // There must be an application running.
    let task = take_current_task().unwrap();

    let process = task.process.upgrade().unwrap();
    process.accumulate_systime(get_processor_slice_time());

    // ---- access current TCB exclusively
    let mut task_inner = task.inner_exclusive_access();
    let task_ctx_ptr = &mut task_inner.task_ctx as *mut TaskContext;
    // Change status to Ready
    task_inner.task_status = TaskStatus::Ready;
    drop(task_inner);
    // ---- release current TCB

    // push back to ready queue.
    add_task(task);
    // jump to scheduling cycle
    schedule(task_ctx_ptr);
}

/// This function must be followed by a schedule
pub fn block_current_task() -> *mut TaskContext {
    let task = take_current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();
    task_inner.task_status = TaskStatus::Blocked;
    &mut task_inner.task_ctx as *mut TaskContext
}

pub fn block_current_and_run_next() {
    let process = current_process();
    process.accumulate_systime(get_processor_slice_time());
    drop(process);
    let task_ctx_ptr = block_current_task();
    schedule(task_ctx_ptr);
}

/// Exit the current 'Running' task and run the next task in task list.
pub fn exit_current_and_run_next(exit_code: i32) {
    let task = current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();
    let process = task.process.upgrade().unwrap();
    let tid = task_inner.res.as_ref().unwrap().tid;

    if task.clone_flags & CLONE_CHILD_CLEARTID != 0 {
        const FUTEX_WAKE: isize = 1;
        let uaddr = __user::new(task.ctid_ptr as *mut u32);
        let addr = translated_refmut(current_user_token(), uaddr);
        unsafe { atomic_store_release(addr, 0) };
        sys_futex(uaddr, FUTEX_WAKE, 1, 0, __user::new(0 as *mut u32), 0);
    }

    let mut is_thread = false;
    if task.clone_flags & CLONE_VM != 0 {
        is_thread = true;
    }

    // record exit code
    task_inner.exit_code = Some(exit_code & (0xff));
    task_inner.res = None;
    // here we do not remove the thread since we are still using the kstack
    // it will be deallocated when sys_waittid is called
    drop(task_inner);
    let task = take_current_task().unwrap();
    drop(task);

    if is_thread {
        process.inner_exclusive_access().tasks[tid] = None;
    }

    // however, if this is the main thread of current process
    // the process should terminate at once
    if tid == 0 {
        let pid = process.getpid();
        if pid == IDLE_PID {
            println!(
                "[kernel] Idle process exit with exit_code {} ...",
                exit_code
            );
            if exit_code != 0 {
                //crate::sbi::shutdown(255); //255 == -1 for err hint
                shutdown(true);
            } else {
                //crate::sbi::shutdown(0); //0 for success hint
                shutdown(false);
            }
        }
        remove_from_pid2process(pid);
        let mut process_inner = process.inner_exclusive_access();
        // mark this process as a zombie process
        process_inner.is_zombie = true;
        // record exit code of main process
        process_inner.exit_code = exit_code << 8;

        {
            // move all child processes under init process
            let mut initproc_inner = INITPROC.inner_exclusive_access();
            for child in process_inner.children.iter() {
                child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
                initproc_inner.children.push(child.clone());
            }
        }

        // deallocate user res (including tid/trap_ctx/ustack) of all threads
        // it has to be done before we dealloc the whole memory_set
        // otherwise they will be deallocated twice
        let mut recycle_res = Vec::<TaskUserRes>::new();
        for task in process_inner.tasks.iter().filter(|t| t.is_some()) {
            let task = task.as_ref().unwrap();
            let mut task_inner = task.inner_exclusive_access();
            if let Some(res) = task_inner.res.take() {
                recycle_res.push(res);
            }
        }
        // dealloc_tid and dealloc_user_res require access to PCB inner, so we
        // need to collect those user res first, then release process_inner
        // for now to avoid deadlock/double borrow problem.
        drop(process_inner);
        recycle_res.clear();

        let mut process_inner = process.inner_exclusive_access();
        process_inner.children.clear();
        // deallocate other data in user space i.e. program code/data section
        process_inner.memory_set.recycle_data_pages();
        // drop file descriptors
        process_inner.fd_table.clear();
        // Remove all tasks except for the main thread itself.
        // This is because we are still using the kstack under the TCB
        // of the main thread. This TCB, including its kstack, will be
        // deallocated when the process is reaped via waitpid.
        while process_inner.tasks.len() > 1 {
            process_inner.tasks.pop();
        }
    }
    // process.accumulate_systime(get_processor_slice_time());
    drop(process);
    // we do not have to save task context
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut _);
}

lazy_static! {
    pub static ref INITPROC: Arc<ProcessControlBlock> = {
        let inode = open_file("initproc", OpenFlags::RDONLY).unwrap();
        let v = inode.read_all();
        ProcessControlBlock::new(v.as_slice())
    };
}

pub fn add_initproc() {
    let _initproc = INITPROC.clone();
}

pub fn check_signals_of_current() -> Option<(i32, &'static str)> {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    process_inner.signals.check_error()
}

pub fn current_add_signal(signal: SignalFlags) {
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    process_inner.signals |= signal;
}

/*********************************************** */
lazy_static! {
    pub static ref KTHREAD_PROC: Arc<ProcessControlBlock> = {
        ProcessControlBlock::new_kpthread(
            crate::kpthread_test::__start as usize,
            crate::kpthread_test::get_task1_user_stack_top(),
        )
    };
}

#[allow(unused)]
pub fn add_kpthread() {
    let kpthread = KTHREAD_PROC.clone();
    trace!("kpthread pid: {}", kpthread.pid_handle.0);
}
