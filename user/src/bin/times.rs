#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use core::default;
use user_lib::{sleep, times, fork, yield_, Tms};

#[no_mangle]
pub fn main() -> i32 {
    let pid = fork();
    if pid == 0 {
        let mut tms: Tms = Default::default();
        let tms_ptr: *mut Tms = &mut tms;
        times(tms_ptr);
        println!("{},{}",tms.tms_usrtime,tms.tms_systime);
        println!("{},{}",tms.tms_child_usrtime,tms.tms_child_systime);
        sleep(10000);
        yield_();
    }
    else {
        sleep(500);
        let mut tms: Tms = Default::default();
        let tms_ptr: *mut Tms = &mut tms;
        times(tms_ptr);
        println!("{},{}",tms.tms_usrtime,tms.tms_systime);
        println!("{},{}",tms.tms_child_usrtime,tms.tms_child_systime);
    }
    0
}
