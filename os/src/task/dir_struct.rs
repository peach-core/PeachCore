use core::str::FromStr;

use crate::{
    fs::Inode,
    sync::UPIntrFreeCell,
};
use alloc::{
    string::String,
    sync::Arc,
};

// TODO: Perhaps there will be concurrent bugs, multi-fs_struct have same work directory. We need
// some mechanism to make sure they are clone from same Arc<dyn Inode>.
// Here We have only one root directory. So I don't dispose it.
pub struct DirStruct<I: Inode> {
    inner: UPIntrFreeCell<DirStructInner<I>>,
}

pub struct DirStructInner<I: Inode> {
    pub path: Path<I>,
    // pub root: Path,
}

impl<I: Inode> DirStruct<I> {
    pub fn new(current_inode: &Arc<I>) -> Self {
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

    pub fn get_current_inode(&self) -> Arc<I> {
        let inner = self.inner.exclusive_access();
        let os_inode = inner.path.inode.clone();
        drop(inner);
        os_inode
    }

    /// On error, -1 is returned.
    /// Maybe return Result is better?
    /// TODO: only an interface now. To realize it, we need support by fs. `fs.find(path) ->
    /// dyn Inode`
    pub fn chdir(&self, path: &str) -> isize {
        let mut inner = self.inner.exclusive_access();
        inner.path.cwd = String::from_str(path).unwrap();
        0
    }

    pub fn getcwd(&self) -> String {
        let inner = self.inner.exclusive_access();
        inner.path.cwd.clone()
    }

    pub fn mkdirat(&self, name: &str) -> Option<Arc<I>> {
        let mut inner = self.inner.exclusive_access();
        let inode = inner.path.inode.mkdir(name)?;
        inner.path.cwd.push('/');
        inner.path.cwd.push_str(name);
        Some(inode)
    }

    pub fn rmdirat(&self, name: &str) -> Option<Arc<I>> {
        let mut inner = self.inner.exclusive_access();
        let inode = inner.path.inode.rmdir(name)?;
        if let Some(pos) = inner.path.cwd.rfind('/') {
            inner.path.cwd.truncate(pos);
        }
        Some(inode)
    }
}

struct Path<I: Inode> {
    pub cwd: String,
    pub inode: Arc<I>,
}
