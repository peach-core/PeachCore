use crate::{
    sync::UPIntrFreeCell,
    task::{
        TaskStruct,
        block_current_and_run_next,
        current_task,
        wakeup_task,
    },
};
use alloc::{
    collections::VecDeque,
    sync::Arc,
};

pub struct Semaphore {
    pub inner: UPIntrFreeCell<SemaphoreInner>,
}

pub struct SemaphoreInner {
    pub count: isize,
    pub wait_queue: VecDeque<Arc<TaskStruct>>,
}

impl Semaphore {
    pub fn new(res_count: usize) -> Self {
        Self {
            inner: unsafe {
                UPIntrFreeCell::new(SemaphoreInner {
                    count: res_count as isize,
                    wait_queue: VecDeque::new(),
                })
            },
        }
    }

    pub fn up(&self) {
        let mut inner = self.inner.try_exclusive_access().unwrap();
        inner.count += 1;
        if inner.count <= 0
            && let Some(task) = inner.wait_queue.pop_front()
        {
            wakeup_task(task);
        }
    }

    pub fn down(&self) {
        let mut inner = self.inner.try_exclusive_access().unwrap();
        inner.count -= 1;
        if inner.count < 0 {
            inner.wait_queue.push_back(current_task().unwrap());
            drop(inner);
            block_current_and_run_next();
        }
    }
}
