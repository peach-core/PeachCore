use core::str::FromStr;

use crate::{
    fs::OSInode,
    sync::UPIntrFreeCell,
};
use alloc::{
    string::String,
    sync::Arc,
};

// TODO: Perhaps there will be concurrent bugs, multi-fs_struct have same work directory. We need
// some mechanism to make sure they are clone from same Arc<OSInode>.
// Here We have only one root directory. So I don't dispose it.
pub struct DirStruct {
    inner: UPIntrFreeCell<DirStructInner>,
}

pub struct DirStructInner {
    path: Path,
    // pub root: Path,
}

impl DirStruct {
    pub fn new(current_inode: &Arc<OSInode>) -> Self {
        DirStruct {
            inner: unsafe {
                UPIntrFreeCell::new(DirStructInner {
                    // TODO: get pwd from current_inode. Need support by fs.
                    // assert(current_inode.is_dir())
                    // pwd = current_inode.get_name();
                    path: Path {
                        cwd: String::from_str("/").unwrap(),
                        inode: current_inode.clone(),
                    },
                })
            },
        }
    }

    pub fn get_current_inode(&self) -> Arc<OSInode> {
        let inner = self.inner.exclusive_access();
        let os_inode = inner.path.inode.clone();
        drop(inner);
        os_inode
    }

    /// On error, -1 is returned.
    /// Maybe return Result is better?
    /// TODO: only an interface now. To realize it, we need support by fs. `fs.find(path) ->
    /// OSInode`
    pub fn chdir(&self, path: &str) -> isize {
        let mut inner = self.inner.exclusive_access();
        inner.path.cwd = String::from_str(path).unwrap();
        0
    }

    pub fn getcwd(& self) -> String {
        let inner = self.inner.exclusive_access();
        inner.path.cwd.clone()
    }
}

struct Path {
    pub cwd: String,
    pub inode: Arc<OSInode>,
}
