#![no_std]
#![no_main]

const THREAD_STACK_SIZE: usize = 2048;
const CLONE_THREAD_FLAG: usize = CLONE_VM
    | CLONE_FS
    | CLONE_FILES
    | CLONE_SIGHAND
    | CLONE_THREAD
    | CLONE_SYSVSEM
    | CLONE_CHILD_CLEARTID
    | CLONE_PARENT_SETTID;

use core::{
    str,
    sync::atomic::{AtomicU32, Ordering},
    u32,
};

use shared_defination::clone::flag::*;
use user_lib::{clone, futex, gettid};

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

#[repr(C)]
struct ThreadType {
    ptid: u32,
    ctid: AtomicU32,
    args: *mut u8,
    ret: Option<i32>,
    stack: [u8; THREAD_STACK_SIZE],
}

impl ThreadType {
    pub fn new(args: *mut u8) -> Self {
        Self {
            ptid: u32::MAX,
            ctid: AtomicU32::new(u32::MAX),
            args,
            ret: None,
            stack: [0; THREAD_STACK_SIZE],
        }
    }
}

struct ArgType<'a> {
    s: &'a str,
}

fn child_thread(ptr: *mut u8) {
    let thread = unsafe { (ptr as *mut ThreadType).as_mut().unwrap() };
    let args = unsafe { (thread.args as usize as *mut ArgType).as_mut().unwrap() };

    let tid = gettid() as usize;
    let mut t = 1;
    for i in 0..(6 - tid) {
        t *= 2;
    }
    println!("{}  [thread{}].", args.s, tid);
    for i in 0..t * 2 {
        let mut num = tid;
        for j in 0..5000000 {
            num = (num * (i + j + tid)) % (1e9 as usize + 7);
        }
        println!("{}  [thread{}] num = {}.", args.s, tid, num);
    }

    thread.ret = Some(tid as i32);
}

fn create_thread(thread: &mut ThreadType) -> u32 {
    thread.ptid = clone(
        child_thread,
        unsafe { thread.stack.as_mut_ptr().add(THREAD_STACK_SIZE) },
        CLONE_THREAD_FLAG,
        (thread) as *mut _ as *mut u8,
        thread.ctid.as_ptr(),
        0 as *mut u8,
        thread.ctid.as_ptr(),
    ) as u32;
    println!(
        "child thread crate. tid = {}, ctid = {}",
        thread.ptid,
        thread.ctid.load(Ordering::Acquire),
    );

    thread.ptid
}

fn wait_thread(thread: &ThreadType) -> i32 {
    loop {
        let c = thread.ctid.swap(thread.ptid, Ordering::AcqRel);
        if c == 0 {
            break; // got the lock
        }

        // wait while lock != 0
        futex_wait(thread.ctid.as_ptr() as usize, 1);
    }

    println!(
        "\x1b[36m[main] child thread exit. tid = {}, ret = {}",
        thread.ptid,
        thread.ret.unwrap()
    );
    thread.ret.unwrap()
}

#[no_mangle]
pub fn main(_argc: usize, _argv: &[&str]) -> i32 {
    let mut arg1 = ArgType { s: "\x1b[31m" };
    let mut arg2 = ArgType { s: "\x1b[32m" };
    let mut arg3 = ArgType { s: "\x1b[33m" };
    let mut arg4 = ArgType { s: "\x1b[34m" };
    let mut arg5 = ArgType { s: "\x1b[35m" };
    let mut thread1 = ThreadType::new((&mut arg1) as *mut _ as *mut u8);
    let mut thread2 = ThreadType::new((&mut arg2) as *mut _ as *mut u8);
    let mut thread3 = ThreadType::new((&mut arg3) as *mut _ as *mut u8);
    let mut thread4 = ThreadType::new((&mut arg4) as *mut _ as *mut u8);
    let mut thread5 = ThreadType::new((&mut arg5) as *mut _ as *mut u8);

    create_thread(&mut thread1);
    create_thread(&mut thread2);
    create_thread(&mut thread3);
    create_thread(&mut thread4);
    create_thread(&mut thread5);

    // wake up.
    wait_thread(&thread1);
    wait_thread(&thread2);
    wait_thread(&thread3);
    wait_thread(&thread4);
    wait_thread(&thread5);

    println!("\x1b[37m[main] exit.");

    0
}
