extern crate shared_defination;
use shared_defination::sysinfo::*;
use crate::task::current_process;
use crate::string::{self, str_to_u8}; 
pub fn sys_uname(sysinfo_ptr: usize) -> isize {
    let sysinfo_phy_ptr = (
        current_process()
        .inner_exclusive_access()
        .memory_set
        .translate_va(sysinfo_ptr)
    ) as *mut Sysinfo;
    if sysinfo_phy_ptr as usize != 0 {
        unsafe {
            str_to_u8(KERNEL_NAME, &mut (*sysinfo_phy_ptr).kernel_name);
            str_to_u8(KERNEL_RELEASE, &mut (*sysinfo_phy_ptr).kernel_release);
            str_to_u8(KERNEL_VERSION, &mut (*sysinfo_phy_ptr).kernel_version);
            str_to_u8(OS_NAME, &mut (*sysinfo_phy_ptr).os_name);
            str_to_u8(MACHINE, &mut (*sysinfo_phy_ptr).machine);
            str_to_u8(NODE_NAME, &mut (*sysinfo_phy_ptr).node_name);
        }
        0
    }
    else {-1}
}