use core::sync::atomic::{
    AtomicU8,
    Ordering,
};

use super::{
    TaskStatus,
    TaskStruct,
};
use alloc::{
    collections::vec_deque::VecDeque,
    sync::Arc,
};

const FREE: u8 = 0;
const LOCK: u8 = 1;

pub struct WaitQueue {
    queue: VecDeque<Arc<TaskStruct>>,
    lock: AtomicU8, // 0 for free, 1 for lock
}

impl WaitQueue {
    pub fn new() -> Self {
        return WaitQueue {
            queue: VecDeque::new(),
            lock: AtomicU8::new(FREE),
        };
    }

    fn take(&mut self) -> Option<Arc<TaskStruct>> {
        return self.queue.pop_front();
    }

    fn add(&mut self, task: Arc<TaskStruct>) {
        let mut task_inner = task.inner_exclusive_access();
        task_inner.task_status = TaskStatus::Blocked;
        drop(task_inner);
        self.queue.push_back(task);
    }

    #[allow(unused)]
    pub fn len(&self) -> usize {
        return self.queue.len();
    }

    pub fn lock(&mut self) -> WaitQueueLockGuard {
        loop {
            let lock = self.lock.swap(LOCK, Ordering::AcqRel);
            if lock == FREE {
                break;
            }
        }
        return WaitQueueLockGuard {
            q: self as *mut Self,
        };
    }

    fn unlock(&mut self) {
        self.lock.store(FREE, Ordering::Release);
    }
}

pub struct WaitQueueLockGuard {
    q: *mut WaitQueue,
}

impl Drop for WaitQueueLockGuard {
    fn drop(&mut self) {
        unsafe {
            (self.q.as_mut()).unwrap().unlock();
        }
    }
}

impl WaitQueueLockGuard {
    fn get_mut<'a>(&mut self) -> &'a mut WaitQueue {
        unsafe { self.q.as_mut().unwrap() }
    }

    #[allow(unused)]
    fn get_ref<'a>(&self) -> &'a WaitQueue {
        unsafe { self.q.as_ref().unwrap() }
    }

    pub fn take(&mut self) -> Option<Arc<TaskStruct>> {
        self.get_mut().take()
    }

    pub fn add(&mut self, task: Arc<TaskStruct>) {
        self.get_mut().add(task)
    }
}
