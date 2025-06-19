const OS_NAME: &str = "PeachOS";
const NODE_NAME: &str = "admin";
const KERNEL_RELEASE: &str = "";
const KERNEL_VERSION: &str = "PeachOS";
const MACHINE: &str = "riscv64";
const KERNEL_NAME: &str = "PeachCore";

#[repr(C)]
pub struct Sysinfo {
    pub os_name: [u8; 10],
    pub node_name: [u8; 10],
    pub kernel_release: [u8; 10],
    pub kernel_version: [u8; 10],
    pub machine: [u8; 10],
    pub kernel_name: [u8; 10],
}

const fn str_to_u8_array_10(s: &str) -> [u8; 10] {
    let bytes = s.as_bytes();
    let mut buf = [0u8; 10];
    let mut i = 0;
    while i < bytes.len() && i < 10 {
        buf[i] = bytes[i];
        i += 1;
    }
    buf
}

pub const SYS_INFO: Sysinfo = Sysinfo {
    os_name: str_to_u8_array_10(OS_NAME),
    node_name: str_to_u8_array_10(NODE_NAME),
    kernel_release: str_to_u8_array_10(KERNEL_RELEASE),
    kernel_version: str_to_u8_array_10(KERNEL_VERSION),
    machine: str_to_u8_array_10(MACHINE),
    kernel_name: str_to_u8_array_10(KERNEL_NAME),
};
