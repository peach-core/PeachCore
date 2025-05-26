#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{mmap, munmap, sbrk, MapProtect};

pub const MMAP_SPACE_LOWER_BOUND: usize = 0x0002_0000_0000;

#[no_mangle]
fn main() -> i32 {
    let brk_bottom = sbrk(0);
    println!("brk_bottom = {:#x}", brk_bottom);
    sbrk(128);
    let mut brk_top = sbrk(0);
    println!("current brk top = {:#x}", brk_top);

    let ptr = brk_bottom as usize as *mut usize;
    for i in 1..11 {
        unsafe {
            let cur = ptr.add(i);
            println!("cur = {:#x}, write {}", cur as usize, i * i);
            cur.write(i * i);
        }
    }

    for i in (1..11).rev() {
        unsafe {
            print!("{}, ", ptr.add(i).read());
        }
    }

    sbrk(4096 * 6);
    brk_top = sbrk(-(4096 * 6 + 128));
    println!("do sbrk(4096 * 6) ...\tcurrent brk top = {:#x}", brk_top);
    brk_top = sbrk(0);
    println!("release memory...\tcurrent brk top = {:#x}", brk_top);

    println!("task01: brk_test OK.");

    println!("Test mmap syscall");
    let mmap_addr = MMAP_SPACE_LOWER_BOUND;
    assert_eq!(mmap(mmap_addr, 4096, MapProtect::R | MapProtect::W), 0);
    let mmap_ptr = mmap_addr as *mut usize;
    for i in 0..100 {
        unsafe {
            mmap_ptr.add(i).write(i * i);
        }
    }
    assert_eq!(munmap(mmap_addr), 0);
    println!("Pass mmap syscall test");

    0
}
