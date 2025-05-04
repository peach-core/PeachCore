use core::ptr::addr_of_mut;

use riscv::register::sstatus::clear_sie;

use crate::{
    config::USER_STACK_SIZE,
    syscall::process::sys_exit,
};

mod task1_main;

pub static mut KERNEL_TASK1_USER_STACK: [u8; USER_STACK_SIZE] = [0; USER_STACK_SIZE];

pub fn get_task1_user_stack_top() -> usize {
    unsafe { addr_of_mut!(KERNEL_TASK1_USER_STACK) as usize + USER_STACK_SIZE }
}

pub fn __start() {
    let exit_code = task1_main::main();
    unsafe {
        clear_sie();
    }
    sys_exit(exit_code);
}
