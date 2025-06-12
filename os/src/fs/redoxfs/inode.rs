use redoxfs::{
    Disk,
    FileSystem,
    Node,
    TreePtr,
};

use crate::fs::Inode;

pub struct RefoxInode<D>
where
    D: Disk,
{
    fs: FileSystem<D>,
    node_ptr: TreePtr<Node>,
}

impl<D> Inode for RefoxInode<D> where D: Disk {
    fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        let current_time = redox_syscall::time::get_time();
        self.fs.tx(|tx| {
            tx.read_node(self.node_ptr, offset as u64, buf, atime, atime_nsec)
        }).expect("Failed to read node")
    }
}
