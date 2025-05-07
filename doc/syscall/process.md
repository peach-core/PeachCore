# 进程相关系统调用

- 线程部分在此略述

## 相关系统调用

### 调度
```C
#define __NR_waitid 95
#define __NR_wait4 260
#define __NR_clone 220
#define __NR_execve 221
```

### 进程和线程标识符
```C
#define __NR_getpid 172

#define __NR_set_tid_address 96
#define __NR_gettid 178

#define __NR_getppid 173

#define __NR_setpgid 154
#define __NR_getpgid 155

#define __NR_getsid 156
#define __NR_setsid 157
```
1. `pid`: 进程id, 一个进程在操作系统中的唯一标识符, 一个进程同时也是一个线程组, 线程组内的线程共享 `pid`.   \
在 `clone(fork)` 时, 由操作系统分配, 用户无法更改.
2. `tid`: 线程id, 线程组中标识线程的单位, 由 `libc` 的进程库管理; \
    每个进程中保留了一个 `futex` 地址, 用于进程退出的时候进行同步(也可以选用信号量进行同步) `set_tid_address` 用来设置当前线程的 `futex` 地址, 该地址上也保存了 当前进程的 `tid`.
3. `ppid`: 父进程 `pid`.
4. `pgid`: `Process Group ID` 进程组 id, 等于该进程组组长的 `pid`.
5. `sid`: `Session ID`

### 用户和组标识符
``` C
#define __NR_getuid 174
#define __NR_setuid 146

#define __NR_geteuid 175
#define __NR_setreuid 145

#define __NR_setfsuid 151

#define __NR_getresuid 148
#define __NR_setresuid 147

#define __NR_getgid 176
#define __NR_setgid 144

#define __NR_getegid 177
#define __NR_setregid 143

#define __NR_setfsgid 152

#define __NR_getresgid 150
#define __NR_setresgid 149

```
- r: 当前进程原始的
- e: 当前进程生效的
- s: 当前会话的
- fs: 当前文件系统的

1. uid: 用户
2. gid: 用户组

两者排列组合形成了上述大量的系统调用.