# 线程相关调用

- [ ] TODO: 补充 `clone.arg(tls)` 的功能.
## `CLONE` (核心)

### 原型

- 定义于: `kernel/fork.c`, 其中 `riscv` 的寄存器调用约束为:  
```C
int clone(int flags, void *child_stack, pid_t *_Nullable parent_tid,
          pid_t *_Nullable child_tid, void *_Nullable tls);
```

### `arguments`

1. `stack`: 用户栈基地址, 由父线程分配, `clone` 之后会将 `void* stack` 写入新线程的 `sp` 指针.
2. `flags`: 长度为32bits, 低8位为 `SIGNAL`, 子线程退出时会触发父线程的该信号, 用于通知父线程回收资源. \
    高24位 控制 `clone` 的行为, 定义于 `include/uapi/linux/sched.h` 文件中. 下一小结简述这些 `CLONE FLAG` 的行为.
3. `parent_pid`: 指向父进程的一个内存空间, 会将 `clone` 创建的子进程 `tid` 写入该变量中, 便于父进程跟踪子进程.
4. `child_tid`: 指向子进程空间的一个地址, 会将 子进程 `tid` 的值写入该地址, 子进程不需要额外调用 `gettid syscall`.
5. `tls`: 

### `return`
返回值: 
- -1: 发生异常
- \>= 0: 子线程的 tid.


### `CLONE FLAGS`

```C
// include/uapi/linux/sched.h
#define CSIGNAL		            0x000000ff	/* signal mask to be sent at exit */
#define CLONE_VM	            0x00000100	/* set if VM shared between processes */
#define CLONE_FS	            0x00000200	/* set if fs info shared between processes */
#define CLONE_FILES	            0x00000400	/* set if open files shared between processes */
#define CLONE_SIGHAND	        0x00000800	/* set if signal handlers and blocked signals shared */
#define CLONE_PIDFD	            0x00001000	/* set if a pidfd should be placed in parent */
#define CLONE_PTRACE	        0x00002000	/* set if we want to let tracing continue on the child too */
#define CLONE_VFORK	            0x00004000	/* set if the parent wants the child to wake it up on mm_release */
#define CLONE_PARENT	        0x00008000	/* set if we want to have the same parent as the cloner */
#define CLONE_THREAD	        0x00010000	/* Same thread group? */
#define CLONE_NEWNS	            0x00020000	/* New mount namespace group */
#define CLONE_SYSVSEM	        0x00040000	/* share system V SEM_UNDO semantics */
#define CLONE_SETTLS	        0x00080000	/* create a new TLS for the child */
#define CLONE_PARENT_SETTID	    0x00100000	/* set the TID in the parent */
#define CLONE_CHILD_CLEARTID	0x00200000	/* clear the TID in the child */
#define CLONE_DETACHED		    0x00400000	/* Unused, ignored */
#define CLONE_UNTRACED		    0x00800000	/* set if the tracing process can't force CLONE_PTRACE on this clone */
#define CLONE_CHILD_SETTID	    0x01000000	/* set the TID in the child */
#define CLONE_NEWCGROUP		    0x02000000	/* New cgroup namespace */
#define CLONE_NEWUTS		    0x04000000	/* New utsname namespace */
#define CLONE_NEWIPC		    0x08000000	/* New ipc namespace */
#define CLONE_NEWUSER		    0x10000000	/* New user namespace */
#define CLONE_NEWPID		    0x20000000	/* New pid namespace */
#define CLONE_NEWNET		    0x40000000	/* New network namespace */
#define CLONE_IO		        0x80000000	/* Clone io context */
```

### 调用过程
1. 父进程为子进程创建用户栈空间.
2. 分配一个空间给子进程, 用来保存该子进程的 `pid`, 该空间的地址同时也被用作 `futex` 的地址, 在 `pthread_join` 时用于同步父子线程( `clear_child_tid` 属性不为空时, `CLONE_CHILD_CLEARTID` 标志和后续的 `set_tid_address syscall` 会修改该标志位).
3. 可选: 传递需要共享的变量, 子进程退出时的信号量(`flags` 低8位).
4. 根据 `clone` 的返回值区分子进程和父进程, 执行不同操作. 
5. 子进程执行完成后, `exit syscall` 会根据 `clear_child_tid` 属性决定是否唤醒在该地址上等待的线程, 若该地址非空, 则唤醒所有等待线程, 并将该地址中的值(即当前进程的 `tid` 清零, 并将 `tid` 号归还给 `tid_allocator`), 将当前 线程状态 置为 僵尸线程, 清理当前进程的部分资源(所有资源的引用计数减一, 如果该资源未与父进程共享, 则立即释放) 和 该进程的内核栈空间.
6. 父进程调用 `pthread_join` 时检查 `clear_child_tid` 指向地址中的值(该地址由父进程在 `clone` 时分配, 因此父进程一定拥有该地址的所有权, 且即使改地址内的值为 0 父进程也一定知道该地址归属于哪个线程, clone 会返回子线程 `tid`), 如果为 0 则可以回收对应子线程资源(用户栈空间), 否则在该地址上调用 `futex` 等待.

## set_tid_address

### 原型
```C
    /*
    >> /usr/include/sys/types.h
        typedef __pid_t pid_t;
    >> /usr/include/bits/types.h
        typedef __PID_T_TYPE __pid_t;
    >> /usr/include/bits/typesizes.h
        #define __PID_T_TYPE __S32_TYPE
    >> /usr/include/bits/types.h
        #define __S32_TYPE int
    */
    pid_t set_tid_address(int __user* tidptr);
```

### `arguments` 
tidptr: 要被写入当前进程 `tid` 的用户空间地址指针. 后续线程 `exit` 时, 会唤醒在改地址等待的线程.  

非常底层的系统调用, 一般由线程库管理, 与 `brk` 类似, 不建议用户自行使用.

### `return`
当前进程的 `tid` 值.

## POSIX pthread lib

### 对 `clone` 的约束

- 由于 `clone syscall` 过于强大, 而 `POSIX线程` 的要求相对严格, 因此有的 `clone flag` 在 pthread_create 时有固定设置, 不允许修改, 以保证 `clone` 的行为符合 `POSIX` 规范.

#### 必须被设置的 `flag` (核心标志)

|Flag|功能|
|:-:|:-:|
|`CLONE_VM`         | ​共享虚拟内存空间​(线程共享同一进程的地址空间).|
|`CLONE_FS`         | 共享文件系统信息​(包括当前工作目录 `cwd`、根目录等 `fs_struct`).|
|`CLONE_FILES`      | ​共享文件描述符表​(线程共享打开的文件句柄).|
|`CLONE_SIGHAND`    | ​共享信号处理程序​(线程共享相同的信号处理函数表).|
|`CLONE_THREAD`     | ​线程组标识​(将新线程加入同一线程组，共享 `TGID，符合` `POSIX` 要求).|
|`CLONE_SYSVSEM`    | ​共享 `System V` 信号量​(确保线程间信号量操作一致).|

#### 不允许被设置的 `flag`

|​Flag​|​原因​|
|:-:|:-:|
|`CLONE_NEWNS`  | 创建新挂载命名空间(线程需共享挂载点，隔离会导致行为异常)|
|`CLONE_NEWPID` | 创建新 `PID` 命名空间(线程需共享同一进程 `ID` 空间)|
|`CLONE_NEWNET` | 创建新网络命名空间(线程需共享网络栈)|

#### 可选的 `flag`

|​Flag​|作用​|
|:-:|:-:|
|`CLONE_SETTLS`         |​设置线程本地存储(`TLS`)​(用于 pthread_key_create 等 TLS 操作)|
|`CLONE_PARENT_SETTID​`  |将线程 ID 写入父进程指定的地址​(用于返回线程 TID)|
|`CLONE_CHILD_CL`       |EARTID​线程退出时清除子线程 ID​(用于线程同步和资源回收)|
|`CLONE_DETACHED`       |​创建分离线程​(已废弃，现代 glibc 改用 pthread_detach 控制)|

### `TLS` 机制

#### `目的`

为每个线程实现一个线程本地的全局变量(不在线程栈上). 做到线程间隔离.

#### 实现方式: 硬件与编译器配合

在 elf 文件中有两个特殊的段, `.tbss` 和 `.tdata`, 用于保存线程本地变量(每个线程在创建时可选是否为自己新建一份线程本地变量, 或是与父进程共享 `thread_local` 变量), `CLONE_SETTLS` 标记可以为子进程新建一份变量.  

在线程拷贝时(`clone`), `kernel` 会将 `.tbss` 和 `.tdata` 进行一次拷贝, 并将新拷贝的 `.tbss` 和 `.tdata` 的起始地址写入 `tp` 专用寄存器, 代码在访问 `.tbss` 和 `.tdata` 数据时会通过 `tp` 寄存器的偏移地址进行访问.

|架构      |寄存器	   |用于 TLS base|
|:-------:|:-------: |:----------:|
|x86      |	fs	     |fs:[offset]|
|x86_64   |	fs	     |fs:[offset]|
|ARM      |	TPIDRURW |系统寄存器|
|RISC-V   |	tp (x4)	 |Thread Pointer|

不难发现, `TLS` 机制主要由编译器和硬件实现与保证.

