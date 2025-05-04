use log::{info, trace};

use crate::{println, timer::get_time, syscall::process::sys_yield};

const LOOP_SIZE: usize = 5;

pub fn main() -> i32 {
    for i in 0..LOOP_SIZE {
        let mut val = 1;
        for i in 1..i * 1e6 as usize {
            val += i;
        }
        info!(
            "Hello, World! from kernel task. val = {} [{}/{}]",
            val, i, LOOP_SIZE
        );
    }
    let mut time = get_time();
    loop {
        while get_time() < time {
            sys_yield();
        }
        time = get_time() + 10_000_000;
        trace!("[kthread] time = {}", time);
    }
    #[allow(unreachable_code)]
    0
}
