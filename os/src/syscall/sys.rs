extern crate shared_defination;
use core::intrinsics::size_of;

use crate::{
    mm::translated_byte_buffer,
    syscall::user_space::__user,
    task::{
        current_process,
        current_user_token,
    },
};
use alloc::slice;
use shared_defination::sysinfo::{
    self,
    *,
};

pub fn sys_uname(sysinfo_uaddr: __user<*mut Sysinfo>) -> isize {
    if sysinfo_uaddr.inner() as usize == 0 {
        return -1;
    }
    let mut sysinfo_vec = translated_byte_buffer(
        current_user_token(),
        __user::new(sysinfo_uaddr.inner() as usize as *const u8),
        size_of::<Sysinfo>(),
    );

    let mut index = 0usize;
    let sys_info = unsafe {
        slice::from_raw_parts(
            &SYS_INFO as *const Sysinfo as *const u8,
            size_of::<Sysinfo>(),
        )
    };
    for s in sysinfo_vec.iter_mut() {
        for ch in s.iter_mut() {
            *ch = sys_info[index];
            index += 1;
        }
    }
    0
}
