#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(core_intrinsics)]
#![feature(alloc_error_handler)]

//use crate::drivers::{GPU_DEVICE, KEYBOARD_DEVICE, MOUSE_DEVICE, INPUT_CONDVAR};
use crate::drivers::{
    GPU_DEVICE,
    KEYBOARD_DEVICE,
    MOUSE_DEVICE,
};
extern crate alloc;

#[macro_use]
extern crate bitflags;

use log::*;

#[path = "boards/qemu.rs"]
mod board;

#[macro_use]
mod console;
mod config;
mod drivers;
mod fpu;
mod fs;
mod kpthread_test;
mod lang_items;
mod logging;
mod mm;
mod net;
mod sbi;
mod sync;
mod syscall;
mod task;
mod timer;
mod string;
mod trap;

pub use kpthread_test::__start;
use trap::kpthread_trap_return;

use crate::drivers::chardev::{
    CharDevice,
    UART,
};

core::arch::global_asm!(include_str!("entry.asm"));

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

use lazy_static::*;
use sync::UPIntrFreeCell;

lazy_static! {
    pub static ref DEV_NON_BLOCKING_ACCESS: UPIntrFreeCell<bool> =
        unsafe { UPIntrFreeCell::new(false) };
}

#[allow(dead_code)]
fn debug_log() {
    info!(
        "kpthread_trap_return = {:#X}",
        kpthread_trap_return as usize
    );
    info!(
        "kpthread_test::__start = {:#X}",
        kpthread_test::__start as usize
    );
    info!(
        "kpthread_test::get_task1_user_stack_top() = {:#X}",
        kpthread_test::get_task1_user_stack_top()
    );
}

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    logging::init();
    fpu::fpu_enable();
    mm::init();
    UART.init();
    info!("KERN: init gpu");
    let _gpu = GPU_DEVICE.clone();
    info!("KERN: init keyboard");
    let _keyboard = KEYBOARD_DEVICE.clone();
    info!("KERN: init mouse");
    let _mouse = MOUSE_DEVICE.clone();
    info!("KERN: init trap");
    trap::init();
    trap::enable_timer_interrupt();
    board::device_init();
    fs::list_apps();
    // debug_log();
    task::add_kpthread();
    task::add_initproc();
    *DEV_NON_BLOCKING_ACCESS.exclusive_access() = true;
    timer::set_next_trigger();
    task::run_tasks();
    panic!("Unreachable in rust_main!");
}
