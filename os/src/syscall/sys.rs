extern crate shared_defination;
use shared_defination::sysinfo::*;
use crate::task::current_process;

pub fn sys_uname(sysinfo_ptr: usize) -> isize {
    let sysinfo_phy = (current_process().inner_exclusive_access().memory_set.translate_va(sysinfo_ptr)) as *mut Sysinfo;
    if sysinfo_phy as usize != 0 {
        unsafe {
            (*sysinfo_phy).kernel_name = KERNEL_NAME;
            (*sysinfo_phy).kernel_release = KERNEL_RELEASE;
            (*sysinfo_phy).kernel_version = KERNEL_VERSION;
            (*sysinfo_phy).os_name = OS_NAME;
            (*sysinfo_phy).machine = MACHINE;
            (*sysinfo_phy).node_name = NODE_NAME;
        }
        0
    }
    else {-1}
}