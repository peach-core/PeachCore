use alloc::sync::Arc;
use redoxfs::{
    Disk,
    FileSystem,
    Node,
    TreePtr,
};
use spin::Mutex;

use crate::{fs::Inode, timer};

pub struct RefoxInode<D>
where
    D: Disk,
{
    fs: Arc<Mutex<FileSystem<D>>>,
    node_ptr: TreePtr<Node>,
}

impl<D> RefoxInode<D>
where
    D: Disk + Send + 'static,
{
    pub fn new(fs: Arc<Mutex<FileSystem<D>>>, node_ptr: TreePtr<Node>) -> Self {
        Self { fs, node_ptr }
    }

    pub fn node_ptr(&self) -> TreePtr<Node> {
        self.node_ptr
    }
}

impl<D> Inode for RefoxInode<D> where D: Disk + Send + 'static {
    fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        let current_time = timer::get_time_ms();
        self.fs.lock().tx(|tx| {
            tx.read_node(self.node_ptr, offset as u64, buf, current_time as u64, (current_time % 1_000_000) as u32)
        }).expect("Failed to read node")
    }

    fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        let current_time = timer::get_time_ms();
        self.fs.lock().tx(|tx| {
            tx.write_node(self.node_ptr, offset as u64, buf, current_time as u64, (current_time % 1_000_000) as u32)
        }).expect("Failed to write node")
    }

    fn clear(&self) {
        let current_time = timer::get_time_ms();
        self.fs.lock().tx(|tx| {
            tx.truncate_node(self.node_ptr, 0, current_time as u64, (current_time % 1_000_000) as u32)
        }).expect("Failed to clear node");
    }

    fn create(&self, name: &str) -> Option<Arc<Self>> {
        let current_time = timer::get_time_ms();
        let mode = 0o644; // Default file mode, can be adjusted as needed
        self.fs.lock().tx(|tx| {
            tx.create_node(self.node_ptr, name, mode, current_time as u64, (current_time % 1_000_000) as u32)
        }).ok().map(|node| {
            Arc::new(RefoxInode {
                fs: self.fs.clone(),
                node_ptr: node.ptr(),
            })
        })
    }

    fn find(&self, name: &str) -> Option<Arc<Self>> {
        self.fs.lock().tx(|tx| {
            tx.find_node(self.node_ptr, name)
        }).ok().map(|node| {
            Arc::new(RefoxInode {
                fs: self.fs.clone(),
                node_ptr: node.ptr(),
            })
        })
    }

    fn ls(&self) -> alloc::vec::Vec<alloc::string::String> {
        self.fs.lock().tx(|tx| {
            let mut entries = alloc::vec::Vec::new();
            tx.child_nodes(self.node_ptr, &mut entries)?;
            let names = entries.into_iter()
                .filter_map(|e| e.name().map(|s| alloc::string::String::from(s)))
                .collect();
            Ok(names)
        }).expect("Failed to list nodes")
    }
}
