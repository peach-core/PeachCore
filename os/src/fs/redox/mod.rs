use core::cell::LazyCell;

use alloc::sync::Arc;
use redoxfs::{
    FileSystem,
    TreePtr,
};
use spin::Mutex;
use syscall::Result;

mod inode;

pub use inode::*;

use crate::drivers::block::VirtIOBlock;

use super::BlockDevice;

pub struct CoreDisk<B: BlockDevice>(pub B);

impl<T: BlockDevice> BlockDevice for Arc<T> {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        (**self).read_block(block_id, buf)
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) {
        (**self).write_block(block_id, buf)
    }

    fn handle_irq(&self) {
        (**self).handle_irq()
    }

    fn instance() -> Arc<Self> {
        Arc::new(T::instance())
    }
}

const SECTOR_SIZE: usize = 512; // VIRTIOBlock has 512 bytes per block

impl<B: BlockDevice> redoxfs::Disk for CoreDisk<B> {
    unsafe fn read_at(&mut self, block: u64, buffer: &mut [u8]) -> Result<usize> {
        for (i, chunk) in buffer.chunks_mut(SECTOR_SIZE).enumerate() {
            let blk = (block * 8) as usize + i;
            self.0.read_block(blk, chunk);
        }

        Ok(buffer.len())
    }

    unsafe fn write_at(&mut self, block: u64, buffer: &[u8]) -> Result<usize> {
        for (i, chunk) in buffer.chunks(SECTOR_SIZE).enumerate() {
            let blk = (block * 8) as usize + i;
            self.0.write_block(blk, chunk);
        }

        Ok(buffer.len())
    }

    fn size(&mut self) -> Result<u64> {
        const SIZE: u64 = 32 * 2048 * 512; // 32MiB
        Ok(SIZE)
    }
}

impl super::FileSystemTrait for FileSystem<CoreDisk<Arc<VirtIOBlock>>> {
    type Inode = RefoxInode<CoreDisk<Arc<VirtIOBlock>>>;

    fn get_root_inode() -> alloc::sync::Arc<Self::Inode> {
        fn init_root_inode() -> Arc<RefoxInode<CoreDisk<Arc<VirtIOBlock>>>> {
            let root_inode = RefoxInode::new(
                Arc::new(Mutex::new(
                    FileSystem::open(CoreDisk(VirtIOBlock::instance()), None, None, true)
                        .expect("Failed to open RedoxFS disk"),
                )),
                TreePtr::root(),
            );
            Arc::new(root_inode)
        }

        lazy_static::lazy_static! {
            static ref ROOT_INODE: Arc<RefoxInode<CoreDisk<Arc<VirtIOBlock>>>> =
                init_root_inode();
        }

        ROOT_INODE.clone()
    }
}
