#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering};

use shared_defination::clone::flag::*;
use user_lib::{clone, futex, sleep, yield_};

#[macro_use]
extern crate user_lib;
extern crate alloc;

const FUTEX_WAIT: usize = 0;
const FUTEX_WAKE: usize = 1;

pub fn futex_wait(uaddr: usize, expected: u32) -> isize {
    futex(uaddr, FUTEX_WAIT, expected as usize)
}

pub fn futex_wake(uaddr: usize, count: usize) -> isize {
    futex(uaddr, FUTEX_WAKE, count)
}

pub fn child_thread(ptr: *mut u8) {
    println!("[thread] ptr is :{}", ptr as usize);   
}

#[no_mangle]
pub fn main(argc: usize, argv: &[&str]) -> i32 {
    let mut stack = [0; 1024];
    let flag = CLONE_VM | CLONE_FS | CLONE_FILES | CLONE_SIGHAND | CLONE_THREAD |
                CLONE_SYSVSEM | CLONE_CHILD_SETTID | CLONE_CHILD_CLEARTID |
                CLONE_PARENT_SETTID;
    let mut ctid = AtomicU32::new(1);
    clone(child_thread, stack.as_mut_ptr(), flag , 114514 as *mut u8, 0 as *mut u32, 0 as *mut u8, ctid.as_ptr());
    loop {
        let c = ctid.swap(1, Ordering::AcqRel);
        if c == 0 {
            break; // got the lock
        }
        
        // wait while lock != 0
        futex_wait(ctid.as_ptr() as usize, 1);
    }

    0
}
