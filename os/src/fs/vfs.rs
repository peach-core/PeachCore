use alloc::{
    string::String,
    sync::Arc,
    vec::Vec,
};

pub trait Inode: Send + Sync + 'static {
    fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize;

    fn write_at(&self, offset: usize, buf: &[u8]) -> usize;

    fn clear(&self);

    fn ls(&self) -> Vec<String>;

    fn find(&self, name: &str) -> Option<Arc<Self>>;

    fn create(&self, name: &str) -> Option<Arc<Self>>;
}

pub trait FileSystemTrait: Send + Sync {
    type Inode: Inode + 'static;

    fn get_root_inode() -> Arc<Self::Inode>;
}
