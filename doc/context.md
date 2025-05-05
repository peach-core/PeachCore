# 上下文切换  

## 任务切换  

### `__switch`
操作系统在对两个 `task` 进行切换的时候, 需要保存上一个 `task` 的寄存器, 栈顶指针, 和 `pc` 地址. 该切换过程发生在 `__switch(TaskContext*, TaskContext*)` 汇编函数中

``` RISC-V Assembly
.altmacro
.macro SAVE_REG n
    sd s\n, (\n+2)*8(a0)
.endm

.macro LOAD_REG n
    ld s\n, (\n+2)*8(a1)
.endm

.section .text
.globl __switch
__switch:
    .set n, 0
    .rept 12
        SAVE_REG %n
        .set n, n + 1
    .endr

    sd sp, 8(a0)
    sd ra, 0(a0)

    .set n, 0
    .rept 12
        LOAD_REG %n
        .set n, n + 1
    .endr

    # load next task, ra was next task's pc addr, and sp was next task's kernel stack.
    ld sp, 8(a1)
    ld ra, 0(a1)
    ret
```

第一部分将当前上下文保存到当前任务的 `TaskContext` 中, 接着将下一个任务的上下文加载到cpu上, 最后执行 `ret`. 
- 注意我们没有保存 `pc` 指针, 而是通过保存和加载 `ra` 返回值地址, 配合上 `ret` 语句恢复执行流.

### `TaskStruct`
`TaskStruct` 中保存了当前任务的完整信息, 通过这些信息, 可以控制该任务的执行与挂起(调度块). 所有任务的调度块都保存在内核调度模块 `TaskManager` 中, 其中还包含 `current_task` 用于指示当前任务的编号.
``` rust
pub struct TaskStruct {
    pub status: TaskStatus,
    pub privilege: Privilege,
    pub context: TaskContext,
}
```

1. `status`: 当前任务的状态
2. `privilege`: 当前任务的特权等级, 用户任务/内核任务
3. `context`: 当前任务的上下文(用于保存上下文)

### `TaskContext`
``` rust
pub struct TaskContext {
    // store ra which used as return address.
    ra: usize,
    // store the current top address of stack.
    kernel_sp: usize,
    // saved generate registers.
    s_reg: [usize; 12],
}
```
用于保存每个任务在内核态的上下文, 因为任务切换时, 所有进程都应该处在内核态.
1.  `ra` 保存接下来要执行的指令地址, 如果该 `task` 首次被载入, 则该地址为 `__trap_store` 的地址, 直接返回用户代码, 若被 `__switch` 函数挂起, 则该地址为 `__switch` 执行后的返回地址(`run_next_task` 执行结束的位置).
2. `kernel_sp` 保存内核栈地址, 内核栈顶保存了用户态的上下文(`Trap_Context`), 用于 `__trap_store` 时恢复用户程序状态, 任务首次加载时, 内核栈顶会被预先装入用户态的初始上下文.
3. `s_reg` 用于保存内核态的寄存器, 由于内核态切换后, 立刻会执行 `ret` 操作回到上一个函数, 因此不需要保存 `s0-s11` 的寄存器.

### `Trap_Context`  
保存中断时, 用户态的上下文, 用于在中断结束后, 恢复用户程序.
```rust
pub struct TrapContext {
    pub x: [usize; 32],   // reg[0..31].
    pub sstatus: Sstatus, // CSR sstatus reg.
    pub sepc: usize,      // CSR sepc reg.
    pub fp : [usize; 32], // FP register[0..31].
}
```
1. `x` 由于中断可能发生在用户程序的任何位置, 因此 `s0-s11` 也必须保存, 否则用户程序不能保证 `s0-s11` 是否被篡改.
2. `sstatus` 保存用户态的 `sstatus`, 由于可能有嵌套陷入发生, 因此需要额外保存, 防止被覆盖.
3. `sepc` 保存用户态的 `pc` 地址, 恢复执行流.
4. `fp` 浮点寄存器的保存区, 用于保存 `f0-f31` 寄存器.

## 内核任务
内核任务在载入时不需要使用 `sret` 返回用户态.

为了保证内核态也能正常响应中断, 我们还需要