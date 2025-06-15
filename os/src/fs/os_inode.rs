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
    pub struct OpenFlags: i32 {
        const ACCMODE   = 0o3;               // O_ACCMODE
        const RDONLY    = 0o0;               // O_RDONLY
        const WRONLY    = 0o1;               // O_WRONLY
        const RDWR      = 0o2;               // O_RDWR
        const CREAT     = 0o100;             // O_CREAT
        const EXCL      = 0o200;             // O_EXCL
        const NOCTTY    = 0o400;             // O_NOCTTY
        const TRUNC     = 0o1000;            // O_TRUNC
        const APPEND    = 0o2000;            // O_APPEND
        const NONBLOCK  = 0o4000;            // O_NONBLOCK
        const NDELAY    = Self::NONBLOCK.bits; // O_NDELAY = O_NONBLOCK
        const SYNC      = 0o4010000;         // O_SYNC
        const FSYNC     = Self::SYNC.bits;     // O_FSYNC = O_SYNC
        const DSYNC     = 0o10000;           // O_DSYNC
        const ASYNC     = 0o20000;           // O_ASYNC
        const DIRECT    = 0o40000;           // O_DIRECT
        const LARGEFILE = 0o100000;          // O_LARGEFILE
        const NOATIME   = 0o1000000;         // O_NOATIME
        const DIRECTORY = 0o200000;          // O_DIRECTORY
        const PATH      = 0o10000000;        // O_PATH
        const TMPFILE   = 0o20200000;        // O_TMPFILE = O_DIRECTORY|020000000
        const NOFOLLOW  = 0o400000;          // O_NOFOLLOW
        const CLOEXEC   = 0o2000000;         // O_CLOEXEC
    }
}

bitflags! {
    pub struct FileMode: u32 {
        const S_IRWXU = 0o700; // rwx, ---, ---
        const S_IRUSR = 0o400; // r--, ---, ---
        const S_IWUSR = 0o200; // -w-, ---, ---
        const S_IXUSR = 0o100; // --x, ---, ---

        const S_IRWXG = 0o070; // ---, rwx, ---
        const S_IRGRP = 0o040; // ---, r--, ---
        const S_IWGRP = 0o020; // ---, -w-, ---
        const S_IXGRP = 0o010; // ---, --x, ---

        const S_IRWXO = 0o007; // ---, ---, rwx
        const S_IROTH = 0o004; // ---, ---, r--
        const S_IWOTH = 0o002; // ---, ---, -w-
        const S_IXOTH = 0o001; // ---, ---, --x

        const S_ISUID = 0o4000; // Set user ID on execution
        const S_ISGID = 0o2000; // Set group ID on execution
        const S_ISVTX = 0o1000; // Sticky bit

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
