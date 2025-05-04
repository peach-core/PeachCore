
.altmacro
.macro SAVE_REG n
    sd x\n, \n*8(sp)
.endm

.macro LOAD_REG n
    ld x\n, \n*8(sp)
.endm

.macro SAVE_FP_GEG n
    fsw f\n, ((\n+34)*8)(sp)
.endm

.macro LOAD_FP_GEG n
    flw f\n, ((\n+34)*8)(sp)
.endm

.equ TrapContextSize, 69

    .globl __kpthread_traps_entry
    .globl __kpthread_traps_restore
    .align 2
__kpthread_traps_entry:
    csrrw sp, sscratch, sp
    addi sp, sp, -(8*TrapContextSize)

    sd x1, 1*8(sp)
    # store current sp - sizeof (TrapContext)
    sd sp, 2*8(sp)
    sd x3, 3*8(sp)

    .set n, 5
    .rept 27
        SAVE_REG %n
        .set n, n + 1
    .endr

    // save sstatus, sepc.
    csrr t0, sstatus
    csrr t1, sepc
    csrr t2, sscratch
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
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

    mv a0, sp
    call kpthread_trap_handler

__kpthread_traps_restore:
    mv sp, a0

    # load sstatus, sepc.
    ld t0, 32*8(sp)
    ori t0, t0, 0x0010 # set spie bit, enable kernel thread interupt.
    ld t1, 33*8(sp)
    ld t2, 2*8(sp)

    csrw sstatus, t0
    csrw sepc, t1
    csrw sscratch, t2

    # Load FPU register files.
    li t1, 0x6000
    add t0, t0, t1
    bne t0, t1, __load_fp_over
    .set n, 0
    .rept 32
        LOAD_FP_GEG %n
    .endr
__load_fp_over:
    
    ld x1, 1*8(sp)
    ld x3, 1*8(sp)

    .set n, 5
    .rept 27
        LOAD_REG %n
        .set n, n + 1
    .endr

    addi sp, sp, 8*TrapContextSize
    csrrw sp, sscratch, sp
    sret
