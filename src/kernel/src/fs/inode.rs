use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use super::{FsError, FsResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InodeId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InodeType {
    File,
    Directory,
    Symlink,
    Device,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Permissions {
    pub owner_read: bool,
    pub owner_write: bool,
    pub owner_execute: bool,
    pub group_read: bool,
    pub group_write: bool,
    pub group_execute: bool,
    pub other_read: bool,
    pub other_write: bool,
    pub other_execute: bool,
}

impl Permissions {
    pub fn default_file() -> Self {
        Self {
            owner_read: true, owner_write: true, owner_execute: false,
            group_read: true, group_write: false, group_execute: false,
            other_read: true, other_write: false, other_execute: false,
        }
    }

    pub fn default_dir() -> Self {
        Self {
            owner_read: true, owner_write: true, owner_execute: true,
            group_read: true, group_write: false, group_execute: true,
            other_read: true, other_write: false, other_execute: true,
        }
    }

    pub fn read_only() -> Self {
        Self {
            owner_read: true, owner_write: false, owner_execute: false,
            group_read: true, group_write: false, group_execute: false,
            other_read: true, other_write: false, other_execute: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Inode {
    pub id: InodeId,
    pub inode_type: InodeType,
    pub permissions: Permissions,
    pub owner: u32,
    pub group: u32,
    pub size: u64,
    pub link_count: u32,
    pub created_fs: u64,
    pub modified_fs: u64,
    pub data_blocks: Vec<u64>,
}

impl Inode {
    pub fn new(id: InodeId, inode_type: InodeType, permissions: Permissions, owner: u32, group: u32) -> Self {
        Self {
            id,
            inode_type,
            permissions,
            owner,
            group,
            size: 0,
            link_count: 1,
            created_fs: 0,
            modified_fs: 0,
            data_blocks: Vec::new(),
        }
    }

    pub fn is_file(&self) -> bool {
        self.inode_type == InodeType::File
    }

    pub fn is_directory(&self) -> bool {
        self.inode_type == InodeType::Directory
    }

    pub fn increment_links(&mut self) -> FsResult<()> {
        if self.link_count >= 65535 {
            return Err(FsError::LinkLimitExceeded);
        }
        self.link_count += 1;
        Ok(())
    }

    pub fn decrement_links(&mut self) -> u32 {
        self.link_count = self.link_count.saturating_sub(1);
        self.link_count
    }
}

pub struct InodeTable {
    inodes: BTreeMap<u64, Inode>,
    next_id: u64,
    max_inodes: u64,
}

impl InodeTable {
    pub fn new(max_inodes: u64) -> Self {
        Self {
            inodes: BTreeMap::new(),
            next_id: 1,
            max_inodes,
        }
    }

    pub fn allocate(&mut self, inode_type: InodeType, permissions: Permissions, owner: u32, group: u32) -> FsResult<InodeId> {
        if self.inodes.len() as u64 >= self.max_inodes {
            return Err(FsError::NoSpace);
        }
        let id = InodeId(self.next_id);
        self.next_id += 1;
        let inode = Inode::new(id, inode_type, permissions, owner, group);
        self.inodes.insert(id.0, inode);
        Ok(id)
    }

    pub fn free(&mut self, id: InodeId) -> FsResult<()> {
        self.inodes.remove(&id.0).ok_or(FsError::InvalidInode)?;
        Ok(())
    }

    pub fn get(&self, id: InodeId) -> FsResult<&Inode> {
        self.inodes.get(&id.0).ok_or(FsError::InvalidInode)
    }

    pub fn get_mut(&mut self, id: InodeId) -> FsResult<&mut Inode> {
        self.inodes.get_mut(&id.0).ok_or(FsError::InvalidInode)
    }

    pub fn count(&self) -> usize {
        self.inodes.len()
    }

    pub fn available(&self) -> u64 {
        self.max_inodes - self.inodes.len() as u64
    }

    pub fn find_by_type(&self, inode_type: InodeType) -> Vec<InodeId> {
        self.inodes
            .values()
            .filter(|i| i.inode_type == inode_type)
            .map(|i| i.id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate_inode() {
        let mut table = InodeTable::new(100);
        let id = table.allocate(InodeType::File, Permissions::default_file(), 0, 0).unwrap();
        assert_eq!(table.count(), 1);
        let inode = table.get(id).unwrap();
        assert!(inode.is_file());
        assert_eq!(inode.link_count, 1);
    }

    #[test]
    fn test_allocate_directory() {
        let mut table = InodeTable::new(100);
        let id = table.allocate(InodeType::Directory, Permissions::default_dir(), 0, 0).unwrap();
        let inode = table.get(id).unwrap();
        assert!(inode.is_directory());
    }

    #[test]
    fn test_free_inode() {
        let mut table = InodeTable::new(100);
        let id = table.allocate(InodeType::File, Permissions::default_file(), 0, 0).unwrap();
        table.free(id).unwrap();
        assert_eq!(table.count(), 0);
    }

    #[test]
    fn test_free_nonexistent() {
        let mut table = InodeTable::new(100);
        assert_eq!(table.free(InodeId(999)), Err(FsError::InvalidInode));
    }

    #[test]
    fn test_inode_limit() {
        let mut table = InodeTable::new(2);
        table.allocate(InodeType::File, Permissions::default_file(), 0, 0).unwrap();
        table.allocate(InodeType::File, Permissions::default_file(), 0, 0).unwrap();
        assert_eq!(table.allocate(InodeType::File, Permissions::default_file(), 0, 0), Err(FsError::NoSpace));
    }

    #[test]
    fn test_link_count() {
        let mut table = InodeTable::new(100);
        let id = table.allocate(InodeType::File, Permissions::default_file(), 0, 0).unwrap();
        table.get_mut(id).unwrap().increment_links().unwrap();
        assert_eq!(table.get(id).unwrap().link_count, 2);
        table.get_mut(id).unwrap().decrement_links();
        assert_eq!(table.get(id).unwrap().link_count, 1);
    }

    #[test]
    fn test_find_by_type() {
        let mut table = InodeTable::new(100);
        table.allocate(InodeType::File, Permissions::default_file(), 0, 0).unwrap();
        table.allocate(InodeType::Directory, Permissions::default_dir(), 0, 0).unwrap();
        table.allocate(InodeType::File, Permissions::default_file(), 0, 0).unwrap();
        assert_eq!(table.find_by_type(InodeType::File).len(), 2);
        assert_eq!(table.find_by_type(InodeType::Directory).len(), 1);
    }

    #[test]
    fn test_available() {
        let mut table = InodeTable::new(10);
        assert_eq!(table.available(), 10);
        table.allocate(InodeType::File, Permissions::default_file(), 0, 0).unwrap();
        assert_eq!(table.available(), 9);
    }

    #[test]
    fn test_permissions() {
        let p = Permissions::default_file();
        assert!(p.owner_read);
        assert!(p.owner_write);
        assert!(!p.owner_execute);

        let d = Permissions::default_dir();
        assert!(d.owner_execute);

        let r = Permissions::read_only();
        assert!(!r.owner_write);
    }
}
