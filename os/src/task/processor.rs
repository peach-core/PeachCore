use super::{
    __switch,
    fetch_task,
    ProcessControlBlock,
    TaskContext,
    TaskStatus,
    TaskStruct,
};
use crate::{
    sync::UPIntrFreeCell,
    trap::TrapContext,
};
use alloc::sync::Arc;
use core::arch::asm;
use lazy_static::*;

pub struct Processor {
    current: Option<Arc<TaskStruct>>,
    idle_task_ctx: TaskContext,
}

impl Processor {
    pub fn new() -> Self {
        Self {
            current: None,
            idle_task_ctx: TaskContext::zero_init(),
        }
    }
    fn get_idle_task_ctx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_ctx as *mut _
    }
    pub fn take_current(&mut self) -> Option<Arc<TaskStruct>> {
        self.current.take()
    }
    pub fn current(&self) -> Option<Arc<TaskStruct>> {
        self.current.as_ref().map(Arc::clone)
    }
}

lazy_static! {
    pub static ref PROCESSOR: UPIntrFreeCell<Processor> =
        unsafe { UPIntrFreeCell::new(Processor::new()) };
}

pub fn run_tasks() {
    loop {
        let mut processor = PROCESSOR.exclusive_access();
        if let Some(task) = fetch_task() {
            let idle_task_ctx_ptr = processor.get_idle_task_ctx_ptr();
            // access coming task TCB exclusively
            let next_task_ctx_ptr = task.inner.exclusive_session(|task_inner| {
                task_inner.task_status = TaskStatus::Running;
                &task_inner.task_ctx as *const TaskContext
            });
            processor.current = Some(task);
            // release processor manually
            drop(processor);
            unsafe {
                __switch(idle_task_ctx_ptr, next_task_ctx_ptr);
            }
        } else {
            println!("no tasks available in run_tasks");
        }
    }
}

pub fn take_current_task() -> Option<Arc<TaskStruct>> {
    PROCESSOR.exclusive_access().take_current()
}

pub fn current_task() -> Option<Arc<TaskStruct>> {
    PROCESSOR.exclusive_access().current()
}

pub fn current_process() -> Arc<ProcessControlBlock> {
    current_task().unwrap().process.upgrade().unwrap()
}

pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    task.get_user_token()
}

pub fn current_trap_ctx() -> &'static mut TrapContext {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .get_trap_ctx()
}

pub fn current_trap_ctx_user_va() -> usize {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .trap_ctx_user_va()
}

pub fn current_kstack_top() -> usize {
    if let Some(task) = current_task() {
        task.kstack.get_top()
    } else {
        let mut boot_stack_top;
        unsafe { asm!("la {},boot_stack_top",out(reg) boot_stack_top) };
        boot_stack_top
    }
    // current_task().unwrap().kstack.get_top()
}

pub fn schedule(switched_task_ctx_ptr: *mut TaskContext) {
    let idle_task_ctx_ptr =
        PROCESSOR.exclusive_session(|processor| processor.get_idle_task_ctx_ptr());
    unsafe {
        __switch(switched_task_ctx_ptr, idle_task_ctx_ptr);
    }
}
