use super::{sys_sbrk, sys_mmap, sys_munmap};
pub use super::MapProtect;

pub fn sbrk(size: isize) -> isize {
    sys_sbrk(size)
}

pub fn mmap(addr: usize, len: usize, prot: MapProtect) -> isize {
    sys_mmap(addr, len, prot)
}

pub fn munmap(addr: usize) -> isize {
    sys_munmap(addr)
}