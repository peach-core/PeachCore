use crate::{
    drivers::block::VirtIOBlock,
    fs::Arc,
};
use easy_fs::{
    EasyFileSystem,
    Inode as EasyInode,
};
use redoxfs::Disk;

use super::{
    BlockDevice,
    FileSystemTrait,
};
use crate::fs::Inode;

pub type FileSystem = EasyFileSystem;

impl Inode for EasyInode {
    fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        easy_fs::Inode::read_at(self, offset, buf)
    }

    fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        easy_fs::Inode::write_at(self, offset, buf)
    }

    fn clear(&self) {
        easy_fs::Inode::clear(self);
    }

    fn create(&self, name: &str) -> Option<Arc<Self>> {
        easy_fs::Inode::create(self, name)
    }

    fn ls(&self) -> alloc::vec::Vec<alloc::string::String> {
        easy_fs::Inode::ls(self)
    }

    fn find(&self, name: &str) -> Option<Arc<Self>> {
        easy_fs::Inode::find(self, name).map(|inode| inode as Arc<Self>)
    }
}

impl easy_fs::BlockDevice for VirtIOBlock {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        BlockDevice::read_block(self, block_id, buf)
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) {
        BlockDevice::write_block(self, block_id, buf)
    }

    fn handle_irq(&self) {
        BlockDevice::handle_irq(self);
    }
}

impl FileSystemTrait for FileSystem {
    type Inode = EasyInode;
    fn get_root_inode() -> Arc<Self::Inode> {
        let block_dev = VirtIOBlock::instance();
        Arc::new(EasyFileSystem::root_inode(&EasyFileSystem::open(block_dev)))
    }
} 
