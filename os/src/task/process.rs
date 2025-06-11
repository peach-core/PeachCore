use super::{
    add_task,
    dir_struct::DirStruct,
    id::RecycleAllocator,
    manager::insert_into_pid2process,
    pid_alloc,
    PidHandle,
    SignalFlags,
    TaskStruct,
};
use crate::{
    config::{
        PAGE_SIZE,
        USER_STACK_TOP,
    },
    fs::{
        File,
        OSInode,
        Stdin,
        Stdout,
        ROOT_INODE,
    },
    mm::{
        translated_refmut,
        MemorySet,
        PTEFlags,
        VirtAddr,
        KERNEL_SPACE,
    },
    sync::{
        Condvar,
        Mutex,
        Semaphore,
        UPIntrFreeCell,
        UPIntrRefMut,
    },
    trap::{
        trap_handler,
        TrapContext,
    },
};
use alloc::{
    string::String,
    sync::{
        Arc,
        Weak,
    },
    vec,
    vec::Vec,
};

pub struct ProcessControlBlock {
    // immutable
    pub pid_handle: PidHandle,
    // mutable
    inner: UPIntrFreeCell<ProcessControlBlockInner>,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Privilege {
    User,
    Kernel,
}

#[rustfmt::skip]
pub struct ProcessControlBlockInner {
    // =====================================================
    //                  Process Status
    // =====================================================
    pub is_zombie: bool,                                    // is_zombie process
    pub exit_code: i32,                                     // exit code


    // =====================================================
    //                    Process Tree
    // =====================================================
    pub parent: Option<Weak<ProcessControlBlock>>,          // parent process
    pub children: Vec<Arc<ProcessControlBlock>>,            // children processes array

    pub dir_struct: Arc<DirStruct>,                         // Process Session. Aslo Process Group.


    // =====================================================
    //                      Thread
    // =====================================================
    pub tasks: Vec<Option<Arc<TaskStruct>>>,                // threads in this process group
    pub task_res_allocator: RecycleAllocator,               /* thread allocator: use tid to
                                                             * alloc shared resources in thread
                                                             * group */

    // =====================================================
    //                     Resources
    // =====================================================
    pub memory_set: MemorySet,                              // memory space
    pub fd_table: Vec<Option<Arc<dyn File + Send + Sync>>>, // file description
    pub signals: SignalFlags,                               // signals
    pub mutex_list: Vec<Option<Arc<dyn Mutex>>>,
    pub semaphore_list: Vec<Option<Arc<Semaphore>>>,
    pub condvar_list: Vec<Option<Arc<Condvar>>>,
    pub program_brk_bottom: usize,                          // user heap lowerbound.
    pub current_heap_top: usize,                            // current upperbound os heap.
    pub privilege: Privilege,                               // U-Mode Process or K-Mode Thread.
}

impl ProcessControlBlockInner {
    #[allow(unused)]
    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }

    pub fn alloc_fd(&mut self) -> usize {
        if let Some(fd) = (0..self.fd_table.len()).find(|fd| self.fd_table[*fd].is_none()) {
            fd
        } else {
            self.fd_table.push(None);
            self.fd_table.len() - 1
        }
    }

    pub fn alloc_tid(&mut self) -> usize {
        self.task_res_allocator.alloc()
    }

    pub fn dealloc_tid(&mut self, tid: usize) {
        self.task_res_allocator.dealloc(tid)
    }

    pub fn thread_count(&self) -> usize {
        self.tasks.len()
    }

    pub fn get_task(&self, tid: usize) -> Arc<TaskStruct> {
        self.tasks[tid].as_ref().unwrap().clone()
    }

    pub fn get_cwd(&self) -> String {
        self.dir_struct.getcwd()
    }

    pub fn change_program_brk(&mut self, size: isize) -> Option<usize> {
        let old_brk = self.current_heap_top;
        let new_brk = size as usize;
        
        if size == 0 {
            return Some(old_brk);
        } 
        
        let result =  if new_brk < old_brk {
            self.memory_set.shink_to(
                VirtAddr(self.program_brk_bottom).into(),
                VirtAddr(new_brk as usize).ceil().into(),
            )
        } else {
            self.memory_set.append_to(
                VirtAddr(self.program_brk_bottom).into(),
                VirtAddr(new_brk as usize).ceil().into(),
            )
        };

        if result {
            self.current_heap_top = new_brk as usize;
            Some(old_brk)
        } else {
            None
        }
    }

    pub fn current_task_mmap(&mut self, addr: usize, len: usize, prot: usize) -> isize {
        bitflags! {
            pub struct MmapProtect: u8{
                const R = 1 << 0;
                const W = 1 << 1;
                const X = 1 << 2;
            }
        }

        let prot_bit = MmapProtect::from_bits(prot as u8).unwrap();
        let mut pte_flags = PTEFlags::empty();
        if self.privilege == Privilege::User {
            pte_flags |= PTEFlags::U;
        }
        if (prot_bit & MmapProtect::R) != MmapProtect::empty() {
            pte_flags |= PTEFlags::R;
        }
        if (prot_bit & MmapProtect::W) != MmapProtect::empty() {
            pte_flags |= PTEFlags::W;
        }
        if (prot_bit & MmapProtect::X) != MmapProtect::empty() {
            pte_flags |= PTEFlags::X;
        }
        assert_eq!(prot >> 3, 0);
        assert_eq!(addr & (PAGE_SIZE - 1), 0);
        assert_eq!(len & (PAGE_SIZE - 1), 0);

        let start_vaddr: VirtAddr = addr.into();
        let end_vaddr: VirtAddr = (addr + len).into();

        self.memory_set.mmap(start_vaddr, end_vaddr, pte_flags)
    }

    pub fn current_task_munmap(&mut self, addr: usize) -> isize {
        assert_eq!(addr & (PAGE_SIZE - 1), 0);
        self.memory_set.munmap(addr.into())
    }
}

impl ProcessControlBlock {
    pub fn inner_exclusive_access(&self) -> UPIntrRefMut<'_, ProcessControlBlockInner> {
        self.inner.exclusive_access()
    }

    pub fn new(elf_data: &[u8]) -> Arc<Self> {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, program_brk, entry_point) = MemorySet::from_elf(elf_data);
        let ustack_base = USER_STACK_TOP;
        // allocate a pid
        let pid_handle = pid_alloc();
        let root_os_inode = Arc::new(OSInode::new(true, true, ROOT_INODE.clone()));

        let process = Arc::new(Self {
            pid_handle,
            inner: unsafe {
                UPIntrFreeCell::new(ProcessControlBlockInner {
                    is_zombie: false,
                    memory_set,
                    parent: None,
                    children: Vec::new(),
                    exit_code: 0,
                    fd_table: vec![
                        // 0 -> stdin
                        Some(Arc::new(Stdin)),
                        // 1 -> stdout
                        Some(Arc::new(Stdout)),
                        // 2 -> stderr
                        Some(Arc::new(Stdout)),
                    ],
                    dir_struct: Arc::new(DirStruct::new(&root_os_inode)),
                    signals: SignalFlags::empty(),
                    tasks: Vec::new(),
                    task_res_allocator: RecycleAllocator::new(),
                    mutex_list: Vec::new(),
                    semaphore_list: Vec::new(),
                    condvar_list: Vec::new(),
                    program_brk_bottom: program_brk,
                    current_heap_top: program_brk,
                    privilege: Privilege::User,
                })
            },
        });
        // create a main thread, we should allocate ustack and trap_ctx here
        let task = Arc::new(TaskStruct::new(Arc::clone(&process), ustack_base, true));
        // prepare trap_ctx of main thread
        let task_inner = task.inner_exclusive_access();
        let trap_ctx = task_inner.get_trap_ctx();
        let ustack_top = task_inner.res.as_ref().unwrap().ustack_top();
        let kstack_top = task.kstack.get_top();
        drop(task_inner);
        *trap_ctx = TrapContext::app_init_context(
            entry_point,
            ustack_top,
            KERNEL_SPACE.exclusive_access().token(),
            kstack_top,
            trap_handler as usize,
        );
        // add main thread to the process
        let mut process_inner = process.inner_exclusive_access();
        process_inner.tasks.push(Some(Arc::clone(&task)));
        drop(process_inner);
        insert_into_pid2process(process.getpid(), Arc::clone(&process));
        // add main thread to scheduler
        add_task(task);
        process
    }

    pub fn new_kpthread(kernel_thread_entry: usize, user_stack_upper_bound: usize) -> Arc<Self> {
        // allocate a pid
        let pid_handle = pid_alloc();
        let ustack_base = user_stack_upper_bound;
        let root_os_inode = Arc::new(OSInode::new(true, true, ROOT_INODE.clone()));

        let process = Arc::new(Self {
            pid_handle,
            inner: unsafe {
                UPIntrFreeCell::new(ProcessControlBlockInner {
                    is_zombie: false,
                    memory_set: MemorySet::new_bare(),
                    parent: None,
                    children: Vec::new(),
                    exit_code: 0,
                    fd_table: vec![
                        // 0 -> stdin
                        Some(Arc::new(Stdin)),
                        // 1 -> stdout
                        Some(Arc::new(Stdout)),
                        // 2 -> stderr
                        Some(Arc::new(Stdout)),
                    ],
                    dir_struct: Arc::new(DirStruct::new(&root_os_inode)),
                    signals: SignalFlags::empty(),
                    tasks: Vec::new(),
                    task_res_allocator: RecycleAllocator::new(),
                    mutex_list: Vec::new(),
                    semaphore_list: Vec::new(),
                    condvar_list: Vec::new(),
                    program_brk_bottom: 0,
                    current_heap_top: 0,
                    privilege: Privilege::Kernel,
                })
            },
        });
        // create a main thread, we should allocate ustack and trap_ctx here
        let task = Arc::new(TaskStruct::new_kpthread(Arc::clone(&process), ustack_base));
        // prepare trap_ctx of main thread
        let task_inner = task.inner_exclusive_access();
        let ustack_top = task_inner.res.as_ref().unwrap().ustack_top();
        let kstack_top = task.kstack.get_top();
        drop(task_inner);
        task.kstack.push_on_top(TrapContext::kpthread_init_context(
            kernel_thread_entry,
            ustack_top,
            KERNEL_SPACE.exclusive_access().token(),
            kstack_top,
            trap_handler as usize,
        ));
        // add main thread to the process
        let mut process_inner = process.inner_exclusive_access();
        process_inner.tasks.push(Some(Arc::clone(&task)));
        drop(process_inner);
        insert_into_pid2process(process.getpid(), Arc::clone(&process));
        // add main thread to scheduler
        add_task(task);
        process
    }

    /// Only support processes with a single thread.
    pub fn exec(self: &Arc<Self>, elf_data: &[u8], args: Vec<String>) {
        assert_eq!(self.inner_exclusive_access().thread_count(), 1);
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, program_brk, entry_point) = MemorySet::from_elf(elf_data);
        let new_token = memory_set.token();

        // substitute memory_set
        self.inner_exclusive_access().memory_set = memory_set;
        self.inner_exclusive_access().program_brk_bottom = program_brk;
        self.inner_exclusive_access().current_heap_top = program_brk;

        // then we alloc user resource for main thread again
        // since memory_set has been changed
        let task = self.inner_exclusive_access().get_task(0);
        let mut task_inner = task.inner_exclusive_access();
        task_inner.res.as_mut().unwrap().ustack_base = USER_STACK_TOP;
        task_inner.res.as_mut().unwrap().alloc_user_res();
        task_inner.trap_ctx_ppn = task_inner.res.as_mut().unwrap().trap_ctx_ppn();
        // push arguments on user stack
        let mut user_sp = task_inner.res.as_mut().unwrap().ustack_top();

        user_sp -= (args.len() + 1) * core::mem::size_of::<usize>();
        let argv_base = user_sp;
        let mut argv: Vec<_> = (0..=args.len())
            .map(|arg| {
                translated_refmut(
                    new_token,
                    ((argv_base + arg * core::mem::size_of::<usize>()) as *mut usize).into(),
                )
            })
            .collect();
        *argv[args.len()] = 0;
        for i in 0..args.len() {
            user_sp -= args[i].len() + 1;
            *argv[i] = user_sp;
            let mut p = user_sp;
            for c in args[i].as_bytes() {
                *translated_refmut(new_token, (p as *mut u8).into()) = *c;
                p += 1;
            }
            *translated_refmut(new_token, (p as *mut u8).into()) = 0;
        }
        // make the user_sp aligned to 8B for k210 platform
        user_sp -= user_sp % core::mem::size_of::<usize>();
        // initialize trap_ctx
        let mut trap_ctx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            task.kstack.get_top(),
            trap_handler as usize,
        );
        trap_ctx.x[10] = args.len();
        trap_ctx.x[11] = argv_base;
        *task_inner.get_trap_ctx() = trap_ctx;
    }

    /// Only support processes with a single thread.
    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        let mut parent = self.inner_exclusive_access();
        assert_eq!(parent.thread_count(), 1);
        // clone parent's memory_set completely including trampoline/ustacks/trap_ctxs
        let memory_set = MemorySet::from_existed_user(&parent.memory_set);
        // alloc a pid
        let pid = pid_alloc();
        // copy fd table
        let mut new_fd_table: Vec<Option<Arc<dyn File + Send + Sync>>> = Vec::new();
        for fd in parent.fd_table.iter() {
            if let Some(file) = fd {
                new_fd_table.push(Some(file.clone()));
            } else {
                new_fd_table.push(None);
            }
        }
        // copy dir_struct.
        let dir = DirStruct::new(&parent.dir_struct.get_current_inode());

        // create child process pcb
        let child = Arc::new(Self {
            pid_handle: pid,
            inner: unsafe {
                UPIntrFreeCell::new(ProcessControlBlockInner {
                    is_zombie: false,
                    memory_set,
                    parent: Some(Arc::downgrade(self)),
                    children: Vec::new(),
                    exit_code: 0,
                    fd_table: new_fd_table,
                    dir_struct: Arc::new(dir),
                    signals: SignalFlags::empty(),
                    tasks: Vec::new(),
                    task_res_allocator: RecycleAllocator::new(),
                    mutex_list: Vec::new(),
                    semaphore_list: Vec::new(),
                    condvar_list: Vec::new(),
                    program_brk_bottom: parent.program_brk_bottom,
                    current_heap_top: parent.current_heap_top,
                    privilege: Privilege::User,
                })
            },
        });
        // add child
        parent.children.push(Arc::clone(&child));
        // create main thread of child process
        let task = Arc::new(TaskStruct::new(
            Arc::clone(&child),
            parent
                .get_task(0)
                .inner_exclusive_access()
                .res
                .as_ref()
                .unwrap()
                .ustack_base(),
            // here we do not allocate trap_ctx or ustack again
            // but mention that we allocate a new kstack here
            false,
        ));
        // attach task to child process
        let mut child_inner = child.inner_exclusive_access();
        child_inner.tasks.push(Some(Arc::clone(&task)));
        drop(child_inner);
        // modify kstack_top in trap_ctx of this thread
        let task_inner = task.inner_exclusive_access();
        let trap_ctx = task_inner.get_trap_ctx();
        trap_ctx.kernel_sp = task.kstack.get_top();
        drop(task_inner);
        insert_into_pid2process(child.getpid(), Arc::clone(&child));
        // add this thread to scheduler
        add_task(task);
        child
    }

    pub fn getpid(&self) -> usize {
        self.pid_handle.0
    }

    pub fn chdir(&self, path: &str) -> isize {
        self.inner.exclusive_session(|inner| {
            inner.dir_struct.chdir(path);
        });
        0
    }

    pub fn getcwd(&self) -> String {
        self.inner.exclusive_session(|inner| inner.get_cwd())
    }

    // TODO
    pub fn fchdir(&self, _fd: usize) -> isize {
        0
    }

    // TODO
    pub fn mkdirat(&self, _path: &str) -> isize {
        0
    }

    // TODO
    pub fn unlinkat(&self, _path: &str) -> isize {
        0
    }

    // memory syscall.
    pub fn change_program_brk(&self, size: isize) -> Option<usize> {
        let mut inner = self.inner_exclusive_access();
        inner.change_program_brk(size)
    }
    pub fn current_task_mmap(&self, addr: usize, len: usize, prot: usize) -> isize {
        let mut inner = self.inner_exclusive_access();
        inner.current_task_mmap(addr, len, prot)
    }
    pub fn current_task_munmap(&self, addr: usize) -> isize {
        let mut inner = self.inner_exclusive_access();
        inner.current_task_munmap(addr)
    }
}
