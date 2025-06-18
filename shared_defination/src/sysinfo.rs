pub const OS_NAME: &str = "PeachOS" ;
pub const KERNEL_NAME: &str = "PeachCore" ;
pub const NODE_NAME: &str = "admin" ;
pub const KERNEL_RELEASE: &str = "" ;
pub const KERNEL_VERSION: &str = "PeachOS" ;
pub const MACHINE: &str = "riscv64" ;

pub struct Sysinfo {
    pub os_name: [u8; 65],
    pub node_name: [u8; 65],
    pub kernel_release: [u8; 65],
    pub kernel_version: [u8; 65],
    pub machine: [u8; 65],
    pub kernel_name: [u8; 65],
}