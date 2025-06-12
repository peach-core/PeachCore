
use crate::{
    fs::{
        File,
        Inode,
    },
    mm::UserBuffer,
    sync::UPIntrFreeCell,
};
use alloc::{
    sync::Arc,
    vec::Vec,
};
use bitflags::*;


pub struct OSInode<I: Inode> {
    readable: bool,
    writable: bool,
    inner: UPIntrFreeCell<OSInodeInner<I>>,
}
struct OSInodeInner<I: Inode> {
    offset: usize,
    inode: Arc<I>,
}

impl<I: Inode> OSInode<I> {
    pub fn new(readable: bool, writable: bool, inode: Arc<I>) -> Self {
        Self {
            readable,
            writable,
            inner: unsafe { UPIntrFreeCell::new(OSInodeInner { offset: 0, inode }) },
        }
    }
    pub fn read_all(&self) -> Vec<u8> {
        let mut inner = self.inner.exclusive_access();
        let mut buf = [0u8; 512];
        let mut v = Vec::new();
        loop {
            let len = inner.inode.read_at(inner.offset, &mut buf);
            if len == 0 {
                break;
            }
            inner.offset += len;
            v.extend_from_slice(&buf[..len]);
        }
        v
    }
}

bitflags! {
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR   = 1 << 1;
        const CREATE = 1 << 9;
        const TRUNC  = 1 << 10;
    }
}

impl OpenFlags {
    pub fn read_write(&self) -> (bool, bool) {
        if self.is_empty() {
            (true, false)
        } else if self.contains(Self::WRONLY) {
            (false, true)
        } else {
            (true, true)
        }
    }
}

impl<I: Inode> File for OSInode<I> {
    fn readable(&self) -> bool {
        self.readable
    }
    fn writable(&self) -> bool {
        self.writable
    }
    fn read(&self, mut buf: UserBuffer) -> usize {
        let mut inner = self.inner.exclusive_access();
        let mut tot = 0;
        for slice in buf.buffers.iter_mut() {
            let n = inner.inode.read_at(inner.offset, slice);
            if n == 0 {
                break;
            }
            inner.offset += n;
            tot += n;
        }
        tot
    }
    fn write(&self, buf: UserBuffer) -> usize {
        let mut inner = self.inner.exclusive_access();
        let mut tot = 0;
        for slice in buf.buffers.iter() {
            let n = inner.inode.write_at(inner.offset, slice);
            assert_eq!(n, slice.len());
            inner.offset += n;
            tot += n;
        }
        tot
    }
}
