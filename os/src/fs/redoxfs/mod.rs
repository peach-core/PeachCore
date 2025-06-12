use redoxfs::FileSystem;
use redox_syscall::Result;

mod inode;

pub use inode::*;

use super::BlockDevice;

struct CoreDisk<B: BlockDevice>(pub B);

impl<B: BlockDevice> redoxfs::Disk for CoreDisk<B> {
    unsafe fn read_at(&mut self, block: u64, buffer: &mut [u8]) -> Result<usize> {
        self.0.read_block(block as usize, buffer);

        Ok(buffer.len())
    }

    unsafe fn write_at(&mut self, block: u64, buffer: &[u8]) -> Result<usize> {
        self.0.write_block(block as usize, buffer);

        Ok(buffer.len())
    }

    fn size(&mut self) -> Result<u64> {
        const SIZE : u64 = 512; // VIRTIOBlock has 512 bytes per block
        Ok(SIZE)
    }
}

impl<D> super::FileSystemTrait for FileSystem<D>
where
    D: BlockDevice + 'static,{
    type Inode = RefoxInode;

    fn root_inode(&self) -> alloc::sync::Arc<Self::Inode> {}
}
