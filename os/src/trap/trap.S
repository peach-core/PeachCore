.altmacro
.macro SAVE_REG n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_REG n
    ld x\n, \n*8(sp)
.endm
.macro SAVE_FP_GEG n
    fsw f\n, ((\n+37)*8)(sp)
.endm
.macro LOAD_FP_GEG n
    flw f\n, ((\n+37)*8)(sp)
.endm

    .section .text.trampoline
    .globl __traps_entry
    .globl __traps_restore
    .globl __traps_entry_k
    .globl __traps_restore_k
    .align 2
__traps_entry:
    csrrw sp, sscratch, sp
    # now sp->*TrapContext in user space, sscratch->user stack
    # save other general purpose registers
    sd x1, 1*8(sp)
    # skip sp(x2), we will save it later
    sd x3, 3*8(sp)
    # skip tp(x4), application does not use it
    # save x5~x31
    .set n, 5
    .rept 27
        SAVE_REG %n
        .set n, n+1
    .endr
    # we can use t0/t1/t2 freely, because they have been saved in TrapContext
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    # read user stack from sscratch and save it in TrapContext
    csrr t2, sscratch
    sd t2, 2*8(sp)

    # Save FPU register files.
    li t1, 0x6000
    and t0, t0, t1
    bne t1, t0, __save_fp_over
    .set n, 0
    .rept 32
        SAVE_FP_GEG %n
    .endr
__save_fp_over:

    # load kernel_satp into t0
    ld t0, 34*8(sp)
    # load trap_handler into t1
    ld t1, 36*8(sp)
    # move to kernel_sp
    ld sp, 35*8(sp)
    # switch to kernel space
    csrw satp, t0
    sfence.vma
    # jump to trap_handler
    jr t1

__traps_restore:
    # a0: *TrapContext in user space(Constant); a1: user space token
    # switch to user space
    csrw satp, a1
    sfence.vma
    csrw sscratch, a0
    mv sp, a0
    # now sp points to TrapContext in user space, start restoring based on it
    # restore sstatus/sepc
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1

    # Load FPU register files.
    li t1, 0x6000
    add t0, t0, t1
    bne t0, t1, __load_fp_over
    .set n, 0
    .rept 32
        LOAD_FP_GEG %n
    .endr
__load_fp_over:

    # restore general purpose registers except x0/sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_REG %n
        .set n, n+1
    .endr
    # back to user stack
    ld sp, 2*8(sp)
    sret

    .equ TrapContextSize, 69
    .align 2
__traps_entry_k:
    addi sp, sp, -(TrapContextSize*8)
    sd x1, 1*8(sp)
    sd x3, 3*8(sp)
    .set n, 5
    .rept 27
        SAVE_REG %n
        .set n, n+1
    .endr
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)

    # Save FPU register files.
    li t1, 0x6000
    and t0, t0, t1
    bne t1, t0, __save_fp_over_k
    .set n, 0
    .rept 32
        SAVE_FP_GEG %n
    .endr
__save_fp_over_k:

    mv a0, sp
    csrr t2, sscratch
    jalr t2

__traps_restore_k:
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1

    # Load FPU register files.
    li t1, 0x6000
    add t0, t0, t1
    bne t0, t1, __load_fp_over_k
    .set n, 0
    .rept 32
        LOAD_FP_GEG %n
    .endr
__load_fp_over_k:

    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_REG %n
        .set n, n+1
    .endr
    addi sp, sp, (TrapContextSize*8)
    sret
