use core::{clone, ops::{Index, IndexMut}};

use alloc::{
    collections::{
        btree_map::BTreeMap,
        btree_set::BTreeSet,
        vec_deque::VecDeque,
    },
    sync::Arc,
};

use crate::fs::File;

/// if do dealloc_fd(oldfd) will set table[oldfd] as [`None`], and insert oldfd into recycle.
/// Only if a new_fd is not in recycle, we should insert it into table.
/// alloc_fd can assigned newfd. So fd assigned manually will be insert into manually_used
/// [`BTreeSet`].
pub struct FdTable {
    recycle: BTreeSet<usize>,
    table: BTreeMap<usize, Option<Arc<dyn File + Send + Sync>>>,
    manually_used: BTreeSet<usize>,
    upper_bound: usize,
}

impl FdTable {
    pub fn new() -> Self {
        Self {
            recycle: BTreeSet::new(),
            table: BTreeMap::new(),
            manually_used: BTreeSet::new(),
            upper_bound: 0,
        }
    }

    pub fn clone(&self) -> Self {
        let mut new = Self {
            recycle: self.recycle.clone(),
            table: BTreeMap::new(),
            manually_used: self.manually_used.clone(),
            upper_bound: self.upper_bound,
        };

        for (fd, val) in self.table.iter() {
            if let Some(file) = val {
                new.table.insert(*fd, Some(file.clone()));
            } else {
                new.table.insert(*fd, None);
            }
        }

        new
    }

    /// alloc an free fd, set new_fd == -1 for any fd, or return fd.
    /// IF new_fd is not free, panic. ONLY set new_fd for sys_dup3.
    pub fn alloc_fd(&mut self, newfd: isize) -> Option<usize> {
        if newfd == -1 {
            if !self.recycle.is_empty() {
                self.recycle.pop_first()
            } else {
                // skip used fd if assigned by newfd.
                while self.manually_used.contains(&self.upper_bound) {
                    self.upper_bound += 1;
                }

                self.table.insert(self.upper_bound, None);
                self.upper_bound += 1;
                Some(self.upper_bound - 1)
            }
        } else {
            let newfd: usize = newfd as usize;
            if self.table.contains_key(&newfd) && !self.recycle.contains(&newfd) {
                // panic!("alloc_fd({}), has been occupied", newfd)
                return None;
            }

            if !self.recycle.remove(&newfd) {
                self.table.insert(newfd, None);
                self.manually_used.insert(newfd);
            }
            Some(newfd)
        }
    }

    pub fn dealloc_fd(&mut self, oldfd: usize) {
        if let Some(fd) = self.table.get_mut(&oldfd) {
            *fd = None;
            self.recycle.insert(oldfd);
        } else {
            panic!("dealloc({}) does not be occupied.", oldfd);
        }
    }

    pub fn get(&self, index: usize) -> &Option<Arc<dyn File + Send + Sync>> {
        self.table.get(&index).unwrap()
    }

    pub fn get_mut(&mut self, index: usize) -> &mut Option<Arc<dyn File + Send + Sync>> {
        self.table.get_mut(&index).unwrap()
    }

    pub fn len(&self) -> usize {
        return self.table.len();
    }

    pub fn clear(&mut self) {
        self.manually_used.clear();
        self.recycle.clear();
        self.table.clear();
        self.upper_bound = 0;
    }

    pub fn contains(&self, fd: usize) -> bool {
        if let Some(f) = self.table.get(&fd) {
            f.is_some()
        }
        else {
            false
        }
    }
}

impl Index<usize> for FdTable {
    type Output = Option<Arc<dyn File + Send + Sync>>;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
    }
}

impl IndexMut<usize> for FdTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index)
    }
}