mod context;

use crate::{
    config::TRAMPOLINE,
    syscall::syscall,
    task::{
        check_signals_of_current, current_add_signal, current_process, current_task, current_trap_cx, current_trap_cx_user_va, current_user_token, exit_current_and_run_next, suspend_current_and_run_next, SignalFlags
    },
    timer::{
        check_timer,
        set_next_trigger,
    },
};
use core::arch::asm;
use riscv::register::{
    mtvec::TrapMode,
    scause::{
        self,
        Exception,
        Interrupt,
        Trap,
    },
    sie,
    sscratch,
    sstatus,
    stval,
    stvec,
};

pub fn init() {
    set_kernel_trap_entry();
}

fn set_kernel_trap_entry() {
    extern "C" {
        fn __traps_entry();
        fn __traps_entry_k();
    }
    let __traps_entry_k_va = __traps_entry_k as usize - __traps_entry as usize + TRAMPOLINE;
    unsafe {
        stvec::write(__traps_entry_k_va, TrapMode::Direct);
        sscratch::write(trap_from_kernel as usize);
    }
}

fn set_user_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE as usize, TrapMode::Direct);
    }
}

fn set_kpthread_trap_entry() {
    unsafe {
        extern "C" {
            fn __kpthread_traps_entry();
        }
        stvec::write(__kpthread_traps_entry as usize, TrapMode::Direct);
    }
}

pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

fn enable_supervisor_interrupt() {
    unsafe {
        sstatus::set_sie();
    }
}

fn disable_supervisor_interrupt() {
    unsafe {
        sstatus::clear_sie();
    }
}

#[no_mangle]
pub fn trap_handler() -> ! {
    set_kernel_trap_entry();
    let scause = scause::read();
    let stval = stval::read();
    // println!("into {:?}", scause.cause());
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            // jump to next instruction anyway
            let mut cx = current_trap_cx();
            cx.sepc += 4;

            enable_supervisor_interrupt();

            // get system call return value
            let result = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]);
            // cx is changed during sys_exec, so we have to call it again
            cx = current_trap_cx();
            cx.x[10] = result as usize;
        }
        Trap::Exception(Exception::StoreFault)
        | Trap::Exception(Exception::StorePageFault)
        | Trap::Exception(Exception::InstructionFault)
        | Trap::Exception(Exception::InstructionPageFault)
        | Trap::Exception(Exception::LoadFault)
        | Trap::Exception(Exception::LoadPageFault) => {
            /*
            println!(
                "[kernel] {:?} in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.",
                scause.cause(),
                stval,
                current_trap_cx().sepc,
            );
            */
            current_add_signal(SignalFlags::SIGSEGV);
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            current_add_signal(SignalFlags::SIGILL);
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            check_timer();
            suspend_current_and_run_next();
        }
        Trap::Interrupt(Interrupt::SupervisorExternal) => {
            crate::board::irq_handler();
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    // check signals
    if let Some((errno, msg)) = check_signals_of_current() {
        println!("[kernel] {}", msg);
        exit_current_and_run_next(errno);
    }
    trap_return();
}

#[no_mangle]
pub fn trap_return() -> ! {
    disable_supervisor_interrupt();
    set_user_trap_entry();
    let trap_cx_user_va = current_trap_cx_user_va();
    let user_satp = current_user_token();
    extern "C" {
        fn __traps_entry();
        fn __traps_restore();
    }
    let restore_va = __traps_restore as usize - __traps_entry as usize + TRAMPOLINE;
    //println!("before return");
    unsafe {
        asm!(
            "fence.i",
            "jr {restore_va}",
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_user_va,
            in("a1") user_satp,
            options(noreturn)
        );
    }
}

#[no_mangle]
pub fn trap_from_kernel(_trap_cx: &TrapContext) {
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Interrupt(Interrupt::SupervisorExternal) => {
            crate::board::irq_handler();
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            check_timer();
            // do not schedule now
        }
        _ => {
            panic!(
                "pid: {} Unsupported trap from kernel: {:?}, stval = {:#x}!",
                current_process().getpid(),
                scause.cause(),
                stval
            );
        }
    }
}


/*********************************************************/
/*                   for kernel pthread                  */
/*********************************************************/

/// kpthread will go here after interrupt.
pub fn kpthread_trap_return(ctx: &mut TrapContext) -> ! {
    set_kpthread_trap_entry();
    extern "C" {
        fn __kpthread_traps_restore();
    }
    let restore = __kpthread_traps_restore as usize;

    unsafe {
        asm!(
            "jr {kernel_restore_va}",   // jump to new addr of __kernel_trap_restore asm function
            kernel_restore_va = in(reg) restore,
            in("a0") ctx,
            options(noreturn),
        );
    }
}

/// Kernel thread interrupt. kernel_trap.S will call this function.
#[no_mangle]
pub fn kpthread_trap_handler(ctx: &mut TrapContext) -> ! {
    let scause = scause::read(); // get trap cause
    let stval = stval::read(); // get extra value
    match scause.cause() {
        Trap::Interrupt(Interrupt::SupervisorExternal) => {
            crate::board::irq_handler();
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            suspend_current_and_run_next();
        }
        _ => {
            panic!(
                "pid: {}, Unsupported trap {:?}, stval = {:#x}!",
                current_process().getpid(),
                scause.cause(),
                stval
            );
        }
    }

    kpthread_trap_return(ctx)
}

pub use context::TrapContext;
