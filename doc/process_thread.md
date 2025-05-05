# 进程与线程  

## 进程  
进程作为分配资源的基本单位, 所有属于同一个进程的线程共享 一个内存空间.  
进程的唯一标识符是 `pid`, 用于在系统中确定 一个进程(线程组).
```rust
pub struct ProcessControlBlock {
    // immutable
    pub pid: PidHandle,
    // mutable
    inner: UPIntrFreeCell<ProcessControlBlockInner>,
}

pub struct ProcessControlBlockInner {
    pub is_zombie: bool,                                    // is_zombie process
    pub memory_set: MemorySet,                              // memory space
    pub parent: Option<Weak<ProcessControlBlock>>,          // parent process
    pub children: Vec<Arc<ProcessControlBlock>>,            // children processes array
    pub exit_code: i32,                                     // exit code
    pub fd_table: Vec<Option<Arc<dyn File + Send + Sync>>>, // file description
    pub signals: SignalFlags,                               // signals
    pub tasks: Vec<Option<Arc<TaskControlBlock>>>,          // threads in this process group
    pub task_res_allocator: RecycleAllocator,               /* thread allocator: use tid to
                                                             * alloc shared resources in thread
                                                             * group */
    pub mutex_list: Vec<Option<Arc<dyn Mutex>>>,            
    pub semaphore_list: Vec<Option<Arc<Semaphore>>>,        
    pub condvar_list: Vec<Option<Arc<Condvar>>>,    
}
```

## 线程 

### 资源
线程是最小可独立调度的单位, 线程会独立/共享地占有线程组(进程)的资源, 其中栈空间为独立占有. 每个线程控制块中包含指向进程组的指针.  
线程在内核空间中占用的资源: 自己的内核栈空间(RAII), `task_context`, 

`tid` 用来标识线程, 用户空间的资源可以通过 `tid` 类来互斥分配, 由 `TaskUserRes` 管理, 包含:
1. tid(线程号, 用于表示一个线程组中的不同线程), 每个线程组中, 每个线程有自己的用户栈, 有自己的 `trap_ctx_ppn`(一个在用户内存空间, 有着 S 权限的临时空间, 从用户态切换回内核态时, 由于寄存器不足, 无法同时将栈和内存空间都切换, 因此需要先切换栈, 后切换内存空间. 我们将栈临时切换到一个中间态, 它在用户空间, 但是允许 S-Mode 读写, 我们将内核栈地址, 内核 satp_token 和 中断上下文信息均保存在这个中间页上, 用于辅助完成跳转). 每个线程共享 .data 等其他内存信息.
2. 用户栈, 如前所述, 用户栈为每个线程独占, 由 可以由 `tid` 计算偏移得出每个用户的栈空间.
3. `trap_ctx_ppn`, 每个线程独占一个 `trap` 上下文. 该空间在用户空间中, 由 `alloc_user_res` 根据 `tid` 通过 `trap_ctx_bottom_from_tid` 计算得出映射到用户空间页表的虚拟地址空间, 通过 `process` 中的页表进行实际内存分配, 保证互斥. 将该 `trap_ctx` 的虚拟地址所对应 物理页保存在 `TaskControlBlockInner::trap_ctx_ppn` 中, 方便初始化 `trap_ctx`(第一次执行程序时, 需要操作系统再创建线程时提供 `trap_ctx`)

### 状态记录 
用于表示该线程的当前状态信息, `task_status` 和 `exit_code`.

```rust
pub struct TaskControlBlock {
    // immutable
    pub process: Weak<ProcessControlBlock>,
    pub kstack: KernelStack,
    // mutable
    pub inner: UPIntrFreeCell<TaskControlBlockInner>,
}

pub struct TaskControlBlockInner {
    pub res: Option<TaskUserRes>,
    pub trap_ctx_ppn: PhysPageNum,
    pub task_ctx: TaskContext,
    pub task_status: TaskStatus,
    pub exit_code: Option<i32>,
}

pub struct TaskUserRes {
    pub tid: usize,
    pub ustack_base: usize,
    pub process: Weak<ProcessControlBlock>,
    pub is_kpthread: bool,
}
```