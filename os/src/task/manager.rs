use super::{
    ProcessControlBlock,
    TaskStatus,
    TaskStruct,
};
use crate::sync::UPIntrFreeCell;
use alloc::{
    collections::{
        BTreeMap,
        VecDeque,
    },
    sync::Arc,
};
use lazy_static::*;

pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskStruct>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    pub fn add(&mut self, task: Arc<TaskStruct>) {
        self.ready_queue.push_back(task);
    }
    pub fn fetch(&mut self) -> Option<Arc<TaskStruct>> {
        self.ready_queue.pop_front()
    }
}

lazy_static! {
    pub static ref TASK_MANAGER: UPIntrFreeCell<TaskManager> =
        unsafe { UPIntrFreeCell::new(TaskManager::new()) };
    pub static ref PID2PCB: UPIntrFreeCell<BTreeMap<usize, Arc<ProcessControlBlock>>> =
        unsafe { UPIntrFreeCell::new(BTreeMap::new()) };
}

pub fn add_task(task: Arc<TaskStruct>) {
    TASK_MANAGER.try_exclusive_access().unwrap().add(task);
}

pub fn wakeup_task(task: Arc<TaskStruct>) {
    let mut task_inner = task.inner_exclusive_access();
    task_inner.task_status = TaskStatus::Ready;
    drop(task_inner);
    add_task(task);
}

pub fn fetch_task() -> Option<Arc<TaskStruct>> {
    TASK_MANAGER.try_exclusive_access().unwrap().fetch()
}

pub fn pid2process(pid: usize) -> Option<Arc<ProcessControlBlock>> {
    let map = PID2PCB.try_exclusive_access().unwrap();
    map.get(&pid).map(Arc::clone)
}

pub fn insert_into_pid2process(pid: usize, process: Arc<ProcessControlBlock>) {
    PID2PCB.try_exclusive_access().unwrap().insert(pid, process);
}

pub fn remove_from_pid2process(pid: usize) {
    let mut map = PID2PCB.try_exclusive_access().unwrap();
    if map.remove(&pid).is_none() {
        panic!("cannot find pid {} in pid2task!", pid);
    }
}
