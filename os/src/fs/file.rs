use alloc::sync::Arc;

use super::{
    FileSystemTrait,
    OSInode,
    OpenFlags,
    SysFileSystem,
};
use crate::{fs::Inode, mm::UserBuffer};

pub trait File: Send + Sync {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
}

pub fn open_file(
    name: &str, flags: OpenFlags,
) -> Option<Arc<OSInode<<SysFileSystem as FileSystemTrait>::Inode>>> {
    let (r, w) = flags.read_write();
    let root = SysFileSystem::get_root_inode();
    if flags.contains(OpenFlags::CREAT) {
        let inode = root.find(name).or_else(|| root.create(name))?;
        if flags.contains(OpenFlags::TRUNC) {
            inode.clear()
        }
        Some(Arc::new(OSInode::new(r, w, inode)))
    } else {
        let inode = root.find(name)?;
        log::info!("open_file: found file {}", name);
        if flags.contains(OpenFlags::TRUNC) {
            log::info!("open_file: truncating file {}", name);
            inode.clear();
            log::info!("open_file: file {} truncated", name);
        }
        Some(Arc::new(OSInode::new(r, w, inode)))
    }
}

pub fn list_apps() {
    println!("/**** APPS ****");
    for app in SysFileSystem::get_root_inode().ls() {
        println!("{}", app);
    }
    println!("**************/")
}
