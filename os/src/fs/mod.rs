pub mod easyfs;
pub mod file;
pub mod os_inode;
pub mod pipe;
// pub mod redoxfs;
pub mod stdio;
pub mod vfs;

pub use vfs::FileSystemTrait;

use alloc::sync::Arc;
pub use file::File;
pub use pipe::make_pipe;
pub use stdio::{
    Stdin,
    Stdout,
};

pub use os_inode::*;

pub use vfs::Inode;

use crate::drivers::block::VirtIOBlock;
pub use file::{
    list_apps,
    open_file,
};

pub type SysFileSystem = easyfs::FileSystem;
pub type SysInode = easy_fs::Inode;
pub type SysBlockDevice = VirtIOBlock;

pub trait BlockDevice: Send + Sync {
    fn instance() -> Arc<Self>
    where
        Self: Sized;

    fn read_block(&self, block_id: usize, buf: &mut [u8]);
    fn write_block(&self, block_id: usize, buf: &[u8]);
    fn handle_irq(&self);
}
