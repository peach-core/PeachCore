#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::vec;
use core::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
use user_lib::{exit, futex, thread_create, waittid};

static mut LOCK: AtomicU8 = AtomicU8::new(0);
static mut COUNT: isize = 0;

const FUTEX_WAIT: usize = 0;
const FUTEX_WAKE: usize = 1;

pub fn futex_wait(uaddr: usize, expected: u32) -> isize {
    futex(uaddr, FUTEX_WAIT, expected as usize)
}

pub fn futex_wake(uaddr: usize, count: usize) -> isize {
    futex(uaddr, FUTEX_WAKE, count)
}

pub fn futex_mutex_lock(m: &mut AtomicU8) {
    let mut c: u8 = 0;

    c = m.swap(1, Ordering::AcqRel);
    if c == 0 {
        return;
    } else {
        m.store(c, Ordering::Release);
    }

    loop {
        c = m.swap(2, Ordering::AcqRel);
        if c == 0 {
            return; // got the lock
        }

        // wait while lock != 0
        futex_wait(m.as_ptr() as usize, 2);
    }
}

pub fn futex_mutex_unlock(m: &mut AtomicU8) {
    assert_ne!(m.load(Ordering::Acquire), 0);
    if m.fetch_sub(1, Ordering::AcqRel) != 1 {
        m.store(0, Ordering::Release);
        futex_wake(m.as_ptr() as usize, 1);
    }
}

pub fn thread_a() -> ! {
    for _ in 0..10000 {
        unsafe {
            futex_mutex_lock(&mut LOCK);
            COUNT += 2;
            futex_mutex_unlock(&mut LOCK);
        }
    }
    exit(1)
}

pub fn thread_b() -> ! {
    for _ in 0..10000 {
        unsafe {
            futex_mutex_lock(&mut LOCK);
            COUNT += 1;
            futex_mutex_unlock(&mut LOCK);
        }
    }
    exit(2)
}

pub fn thread_c() -> ! {
    for _ in 0..10000 {
        unsafe {
            futex_mutex_lock(&mut LOCK);
            COUNT -= 1;
            futex_mutex_unlock(&mut LOCK);
        }
    }
    exit(3)
}

#[no_mangle]
pub fn main() -> i32 {
    let v = vec![
        thread_create(thread_a as usize, 0),
        thread_create(thread_b as usize, 0),
        thread_create(thread_c as usize, 0),
    ];
    for tid in v.iter() {
        let exit_code = waittid(*tid as usize);
        println!("thread#{} exited with code {}", tid, exit_code);
    }
    unsafe {
        println!("main thread exited. COUNT = {}", COUNT);
    }
    0
}
