#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{sleep, times, fork, yield_};

#[no_mangle]
pub fn main() -> i32 {
    let pid = fork();
    if pid == 0 {
        let mut tms: [usize; 4] = [0; 4];
        let tms_ptr: *mut [usize; 4] = &mut tms;
        times(tms_ptr);
        println!("{},{}",tms[0],tms[1]);
        println!("{},{}",tms[2],tms[3]);
        sleep(10000);
        yield_();
    }
    else {
        sleep(500);
        let mut tms: [usize; 4] = [0; 4];
        let tms_ptr: *mut [usize; 4] = &mut tms;
        times(tms_ptr);
        println!("{},{}",tms[0],tms[1]);
        println!("{},{}",tms[2],tms[3]);
    }
    0
}
