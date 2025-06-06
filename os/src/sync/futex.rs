use core::intrinsics::atomic_load_acquire;

use shared_defination::error::EAGAIN;

use crate::{
    mm::translated_refmut,
    syscall::user_space::__user,
    task::{
        add_task,
        block_current_and_run_next,
        current_process,
        current_task,
        current_user_token,
        wait_queue::WaitQueue,
        TaskStatus,
    },
};

/// TODO: Change enum to num_enum(num_enum crate) for safe FutexOp::from(isize).
#[repr(isize)]
#[allow(non_camel_case_types, unused)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum FutexOp {
    FUTEX_WAIT = 0,
    FUTEX_WAKE = 1,
    FUTEX_FD = 2,
    FUTEX_REQUEUE = 3,
    FUTEX_CMP_REQUEUE = 4,
    FUTEX_WAKE_OP = 5,
    FUTEX_LOCK_PI = 6,
    FUTEX_UNLOCK_PI = 7,
    FUTEX_TRYLOCK_PI = 8,
    FUTEX_WAIT_BITSET = 9,
    FUTEX_WAKE_BITSET = 10,
    FUTEX_WAIT_REQUEUE_PI = 11,
    FUTEX_CMP_REQUEUE_PI = 12,
    FUTEX_LOCK_PI2 = 13,
    FUTEX_OP_SIZE = 14,
    // FUTEX_PRIVATE_FLAG = 128,
    // FUTEX_CLOCK_REALTIME = 256,
    //  FUTEX_CMD_MASK=		~(FUTEX_PRIVATE_FLAG | FUTEX_CLOCK_REALTIME),
    //  FUTEX_WAIT_PRIVATE=	(FUTEX_WAIT | FUTEX_PRIVATE_FLAG),
    //  FUTEX_WAKE_PRIVATE=	(FUTEX_WAKE | FUTEX_PRIVATE_FLAG),
    //  FUTEX_REQUEUE_PRIVATE=	(FUTEX_REQUEUE | FUTEX_PRIVATE_FLAG),
    //  FUTEX_CMP_REQUEUE_PRIVATE= (FUTEX_CMP_REQUEUE | FUTEX_PRIVATE_FLAG),
    //  FUTEX_WAKE_OP_PRIVATE=	(FUTEX_WAKE_OP | FUTEX_PRIVATE_FLAG),
    //  FUTEX_LOCK_PI_PRIVATE=	(FUTEX_LOCK_PI | FUTEX_PRIVATE_FLAG),
    //  FUTEX_LOCK_PI2_PRIVATE=	(FUTEX_LOCK_PI2 | FUTEX_PRIVATE_FLAG),
    //  FUTEX_UNLOCK_PI_PRIVATE=	(FUTEX_UNLOCK_PI | FUTEX_PRIVATE_FLAG),
    //  FUTEX_TRYLOCK_PI_PRIVATE= (FUTEX_TRYLOCK_PI | FUTEX_PRIVATE_FLAG),
    //  FUTEX_WAIT_BITSET_PRIVATE=	(FUTEX_WAIT_BITSET | FUTEX_PRIVATE_FLAG),
    //  FUTEX_WAKE_BITSET_PRIVATE=	(FUTEX_WAKE_BITSET | FUTEX_PRIVATE_FLAG),
    //  FUTEX_WAIT_REQUEUE_PI_PRIVATE=	(FUTEX_WAIT_REQUEUE_PI | \
    // 					 FUTEX_PRIVATE_FLAG),
    //  FUTEX_CMP_REQUEUE_PI_PRIVATE=	(FUTEX_CMP_REQUEUE_PI | \
    // 					 FUTEX_PRIVATE_FLAG),
}

impl From<isize> for FutexOp {
    fn from(value: isize) -> Self {
        if value >= FutexOp::FUTEX_OP_SIZE as isize {
            panic!("FutexOp::from({}), out-of-bounds", value);
        }
        unsafe { core::mem::transmute::<isize, FutexOp>(value) }
    }
}

pub fn sys_futex(
    uaddr: __user<*mut u32>, futex_op: isize, val: u32,
    _val2: u32, /* or const struct timespec *timeout */
    _uaddr2: __user<*mut u32>, _val3: u32,
) -> isize {
    match FutexOp::from(futex_op) {
        FutexOp::FUTEX_WAIT => sys_futex_wait(uaddr, futex_op, val /* , timeout */),
        FutexOp::FUTEX_WAKE => sys_futex_wake(uaddr, futex_op, val),
        _ => -1,
    }
}

pub fn sys_futex_wait(
    uaddr: __user<*mut u32>, futex_op: isize, expected: u32, /* , timeout: &timespec */
) -> isize {
    assert_eq!(futex_op, FutexOp::FUTEX_WAIT as isize);

    /* get and lock futex wait list. */
    let key = uaddr.inner() as usize;
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let futex_table = &mut process_inner.futex_table;
    if !futex_table.contains_key(&key) {
        futex_table.insert(key, WaitQueue::new());
    }
    let wait_queue = futex_table.get_mut(&key).unwrap();
    let mut lock_guard = wait_queue.lock();

    /* check *uaddr and expected. if not eq, return EAGAIN right away. else block current task
     * and add to futex wait list. */
    let satp = current_user_token();
    let addr = translated_refmut(satp, uaddr);

    let val = unsafe { atomic_load_acquire(addr) };
    if val != expected {
        return -(EAGAIN as isize);
    }

    /* add current task into wait queue and set status as BLOCKED. */
    lock_guard.add(current_task().unwrap());

    drop(lock_guard);
    block_current_and_run_next();

    0
}

pub fn sys_futex_wake(uaddr: __user<*mut u32>, futex_op: isize, num_threads: u32) -> isize {
    assert_eq!(futex_op, FutexOp::FUTEX_WAKE as isize);

    /* get and lock futex wait list. */
    let key = uaddr.inner() as usize;
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let futex_table = &mut process_inner.futex_table;
    if !futex_table.contains_key(&key) {
        futex_table.insert(key, WaitQueue::new());
    }
    let wait_queue = futex_table.get_mut(&key).unwrap();
    let mut lock_guard = wait_queue.lock();

    /* wake up num_threads in current futex wait list. */
    let mut count = 0;
    while count < num_threads {
        if let Some(task) = lock_guard.take() {
            let mut task_inner = task.inner_exclusive_access();
            task_inner.task_status = TaskStatus::Ready;
            drop(task_inner);
            add_task(task);
            count += 1;
        } else {
            break;
        }
    }

    count as isize
}
