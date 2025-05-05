# 对于 `risc-v ecall` 指令的整理

## 内核陷入与权限模式

### 模式切换
1. `riscv` 内核包含三个模式: `M-mode`, `S-Mode`, `U-Mode`.
1. `U-Mode` 触发 `Trap` 后, 由 `cpu` 硬件自动进入 `M-Mode`, 运行在 `M-Mode` 的 `rust-sbi` 会将触发 `S-Mode` 陷入.
1. 触发 `Trap` 的方法有两种:  
    - 软件触发: `ecall` 指令, `scause` 最高位为 0 (`Exception`)
    - 硬件触发: 硬件中断, `scause` 最高位为 1 (`Interrupt`)
1. 模式恢复: 触发 `trap` 时, `sstatus` 的 `SIE` 位被保存到 `SPIE` 中, `SIE` 被清零(控制 `M-Mode` 时的中断开关, 清零则无法执行嵌套中断), `SPP` 指示异常发生前的特权模式.


|`csr` 寄存器|`scause`|`stval`|`sstatus`|`sepc`|`sscratch`|
|:---------:|:--------------:|:-------------:|:-----------------:|:-----------:|:---:|
|功能        |触发 `trap` 的原因, 最高位用于区分`Interrupt` 和 `Exception`|触发 `trap` 的附加值, 可以用于传递参数给中断服务函数|保存 `cpu` 运行时的状态信息|保存 `Trap` 触发时的 `pc` 指针, 用于 `Trap` 操作完成后恢复执行流|保存 `S-Mode` 的栈顶指针|

### 执行流
`csr` 的 `stvec` 寄存器保存了 `U-Mode` 陷入到 `S-Mode` 时的行为. 
1. `stvec[MAXLEN:2]`: `BASE` (基地址)  
    `address`: stvec[MAXLEN:2] << 2 为 `trap` 入口基地址.
1. `stvec[1:0]`: `MODE` (模式)  
    - `00`: `stvec::TrapMode::Direct` 非向量中断, 原 `pc` 保存到 `mepc`, `pc` 跳转到 `BASE`   
    - `01`: `stvec::TrapMode::Vectored` 向量中断, 原 `pc` 保存到 `mepc`, `pc` 跳转到 `BASE + 4 * cause`  
    `riscv` 下常用 `direct MODE` 先判断 `Interrupt` 和 `Exception` 再进入对应的服务函数. 如果使用 `Vectored Mode`, 如果有多个同序号的 `Interrupt` 和 `Exception`, 需要在每个中断服务函数中判断.
 

## `U-Mode` 调用 `ecall`.
会触发 用户态 `Trap` 到系统态, `scause/stval/sstatus/sepc` 四个寄存器会被 处理器 自动设置, 其中 `stval` 和 `stcause` 会被传递给服务函数使用. 而 `sstatus` 和 `sepc` 需要等待中断服务完成后, 用于恢复现场, 为了防止服务函数中修改了这两个寄存器, 我们需要在 `Trap` 发生后的入口函数中, 保存这两个寄存器. 此外, 因为 `riscv` 的多个特权模式共用一套寄存器, 因此, `Trap` 的入口函数中也要保存寄存器的值.

程序跳转到 `stvec` 执行, 保存寄存器后, 操作系统执行与用户约定的操作. `rCore` 的实现中, 使用 `x17(a7)` 作为系统调用号, `x10(a0), x11(a1), x12(a2)` 作为参数, `x10(a0)` 作为返回值.
