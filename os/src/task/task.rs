use core::mem::size_of;

use super::{
    KernelStack,
    ProcessControlBlock,
    TaskContext,
    id::TaskUserRes,
    kstack_alloc,
};
use crate::{
    mm::PhysPageNum,
    sync::{
        UPIntrFreeCell,
        UPIntrRefMut,
    },
    trap::TrapContext,
};
use alloc::sync::{
    Arc,
    Weak,
};

pub struct TaskStruct {
    // immutable
    pub process: Weak<ProcessControlBlock>,
    pub kstack: KernelStack,
    // mutable
    pub inner: UPIntrFreeCell<TaskControlBlockInner>,
}

impl TaskStruct {
    pub fn inner_exclusive_access(&self) -> UPIntrRefMut<'_, TaskControlBlockInner> {
        self.inner.exclusive_access()
    }

    pub fn get_user_token(&self) -> usize {
        let process = self.process.upgrade().unwrap();
        let inner = process.inner_exclusive_access();
        inner.memory_set.token()
    }
}

pub struct TaskControlBlockInner {
    pub res: Option<TaskUserRes>,
    pub trap_ctx_ppn: PhysPageNum,
    pub task_ctx: TaskContext,
    pub task_status: TaskStatus,
    pub exit_code: Option<i32>,
}

impl TaskControlBlockInner {
    pub fn get_trap_ctx(&self) -> &'static mut TrapContext {
        self.trap_ctx_ppn.get_mut()
    }

    #[allow(unused)]
    fn get_status(&self) -> TaskStatus {
        self.task_status
    }
}

impl TaskStruct {
    pub fn new(
        process: Arc<ProcessControlBlock>, ustack_base: usize, alloc_user_res: bool,
    ) -> Self {
        let res = TaskUserRes::new(Arc::clone(&process), ustack_base, alloc_user_res);
        let trap_ctx_ppn = res.trap_ctx_ppn();
        let kstack = kstack_alloc();
        let kstack_top = kstack.get_top();
        Self {
            process: Arc::downgrade(&process),
            kstack,
            inner: unsafe {
                UPIntrFreeCell::new(TaskControlBlockInner {
                    res: Some(res),
                    trap_ctx_ppn,
                    task_ctx: TaskContext::goto_trap_return(kstack_top),
                    task_status: TaskStatus::Ready,
                    exit_code: None,
                })
            },
        }
    }

    pub fn new_kpthread(process: Arc<ProcessControlBlock>, ustack_base: usize) -> Self {
        let res = TaskUserRes::new_kpthread(Arc::clone(&process), ustack_base);
        let kstack = kstack_alloc();
        let kstack_top = kstack.get_top();
        Self {
            process: Arc::downgrade(&process),
            kstack,
            inner: unsafe {
                UPIntrFreeCell::new(TaskControlBlockInner {
                    res: Some(res),
                    trap_ctx_ppn: 0.into(),
                    // save TrapContext in the top of kernel stack.
                    task_ctx: TaskContext::goto_kpthread_trap_return(
                        kstack_top - size_of::<TrapContext>(),
                    ),
                    task_status: TaskStatus::Ready,
                    exit_code: None,
                })
            },
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Blocked,
}
