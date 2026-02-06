use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use super::{FsError, FsResult};
use super::inode::InodeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MountId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsType {
    TernaryFs,
    RamFs,
    DevFs,
    ProcFs,
    SysFs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MountFlags {
    pub read_only: bool,
    pub no_exec: bool,
    pub no_dev: bool,
    pub synchronous: bool,
}

impl MountFlags {
    pub fn default_rw() -> Self {
        Self { read_only: false, no_exec: false, no_dev: false, synchronous: false }
    }

    pub fn default_ro() -> Self {
        Self { read_only: true, no_exec: false, no_dev: false, synchronous: false }
    }
}

#[derive(Debug, Clone)]
pub struct MountPoint {
    pub id: MountId,
    pub fs_type: FsType,
    pub path: String,
    pub root_inode: InodeId,
    pub flags: MountFlags,
}

pub struct MountTable {
    mounts: BTreeMap<u32, MountPoint>,
    path_index: BTreeMap<String, MountId>,
    next_id: u32,
    max_mounts: usize,
}

impl MountTable {
    pub fn new(max_mounts: usize) -> Self {
        Self {
            mounts: BTreeMap::new(),
            path_index: BTreeMap::new(),
            next_id: 1,
            max_mounts,
        }
    }

    pub fn mount(&mut self, path: String, fs_type: FsType, root_inode: InodeId, flags: MountFlags) -> FsResult<MountId> {
        if self.mounts.len() >= self.max_mounts {
            return Err(FsError::CapacityExceeded);
        }
        if self.path_index.contains_key(&path) {
            return Err(FsError::AlreadyMounted);
        }

        let id = MountId(self.next_id);
        self.next_id += 1;

        let mount = MountPoint { id, fs_type, path: path.clone(), root_inode, flags };
        self.mounts.insert(id.0, mount);
        self.path_index.insert(path, id);
        Ok(id)
    }

    pub fn unmount(&mut self, id: MountId) -> FsResult<()> {
        let mount = self.mounts.remove(&id.0).ok_or(FsError::NotMounted)?;
        self.path_index.remove(&mount.path);
        Ok(())
    }

    pub fn unmount_path(&mut self, path: &str) -> FsResult<()> {
        let id = self.path_index.get(path).copied().ok_or(FsError::NotMounted)?;
        self.unmount(id)
    }

    pub fn get(&self, id: MountId) -> FsResult<&MountPoint> {
        self.mounts.get(&id.0).ok_or(FsError::NotMounted)
    }

    pub fn find_by_path(&self, path: &str) -> Option<&MountPoint> {
        self.path_index
            .get(path)
            .and_then(|id| self.mounts.get(&id.0))
    }

    pub fn find_mount_for_path(&self, path: &str) -> Option<&MountPoint> {
        let mut best: Option<&MountPoint> = None;
        let mut best_len = 0;
        for mount in self.mounts.values() {
            if path.starts_with(&mount.path) && mount.path.len() > best_len {
                best = Some(mount);
                best_len = mount.path.len();
            }
        }
        best
    }

    pub fn count(&self) -> usize {
        self.mounts.len()
    }

    pub fn list(&self) -> Vec<&MountPoint> {
        self.mounts.values().collect()
    }

    pub fn mounts_by_type(&self, fs_type: FsType) -> Vec<&MountPoint> {
        self.mounts.values().filter(|m| m.fs_type == fs_type).collect()
    }

    pub fn is_read_only(&self, path: &str) -> bool {
        self.find_mount_for_path(path).map_or(false, |m| m.flags.read_only)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mount() {
        let mut table = MountTable::new(16);
        let id = table.mount(String::from("/"), FsType::TernaryFs, InodeId(1), MountFlags::default_rw()).unwrap();
        assert_eq!(table.count(), 1);
        let mount = table.get(id).unwrap();
        assert_eq!(mount.fs_type, FsType::TernaryFs);
        assert_eq!(mount.path, "/");
    }

    #[test]
    fn test_mount_duplicate_path() {
        let mut table = MountTable::new(16);
        table.mount(String::from("/"), FsType::TernaryFs, InodeId(1), MountFlags::default_rw()).unwrap();
        assert_eq!(
            table.mount(String::from("/"), FsType::RamFs, InodeId(2), MountFlags::default_rw()),
            Err(FsError::AlreadyMounted)
        );
    }

    #[test]
    fn test_unmount() {
        let mut table = MountTable::new(16);
        let id = table.mount(String::from("/"), FsType::TernaryFs, InodeId(1), MountFlags::default_rw()).unwrap();
        table.unmount(id).unwrap();
        assert_eq!(table.count(), 0);
    }

    #[test]
    fn test_unmount_path() {
        let mut table = MountTable::new(16);
        table.mount(String::from("/mnt"), FsType::RamFs, InodeId(2), MountFlags::default_rw()).unwrap();
        table.unmount_path("/mnt").unwrap();
        assert_eq!(table.count(), 0);
    }

    #[test]
    fn test_find_by_path() {
        let mut table = MountTable::new(16);
        table.mount(String::from("/"), FsType::TernaryFs, InodeId(1), MountFlags::default_rw()).unwrap();
        assert!(table.find_by_path("/").is_some());
        assert!(table.find_by_path("/mnt").is_none());
    }

    #[test]
    fn test_find_mount_for_path() {
        let mut table = MountTable::new(16);
        table.mount(String::from("/"), FsType::TernaryFs, InodeId(1), MountFlags::default_rw()).unwrap();
        table.mount(String::from("/home"), FsType::RamFs, InodeId(2), MountFlags::default_rw()).unwrap();

        let m = table.find_mount_for_path("/home/user/file.txt").unwrap();
        assert_eq!(m.path, "/home");

        let m2 = table.find_mount_for_path("/etc/config").unwrap();
        assert_eq!(m2.path, "/");
    }

    #[test]
    fn test_mount_capacity() {
        let mut table = MountTable::new(2);
        table.mount(String::from("/"), FsType::TernaryFs, InodeId(1), MountFlags::default_rw()).unwrap();
        table.mount(String::from("/mnt"), FsType::RamFs, InodeId(2), MountFlags::default_rw()).unwrap();
        assert_eq!(
            table.mount(String::from("/tmp"), FsType::RamFs, InodeId(3), MountFlags::default_rw()),
            Err(FsError::CapacityExceeded)
        );
    }

    #[test]
    fn test_mounts_by_type() {
        let mut table = MountTable::new(16);
        table.mount(String::from("/"), FsType::TernaryFs, InodeId(1), MountFlags::default_rw()).unwrap();
        table.mount(String::from("/dev"), FsType::DevFs, InodeId(2), MountFlags::default_ro()).unwrap();
        table.mount(String::from("/proc"), FsType::ProcFs, InodeId(3), MountFlags::default_ro()).unwrap();
        assert_eq!(table.mounts_by_type(FsType::TernaryFs).len(), 1);
        assert_eq!(table.mounts_by_type(FsType::DevFs).len(), 1);
    }

    #[test]
    fn test_read_only() {
        let mut table = MountTable::new(16);
        table.mount(String::from("/"), FsType::TernaryFs, InodeId(1), MountFlags::default_rw()).unwrap();
        table.mount(String::from("/rom"), FsType::TernaryFs, InodeId(2), MountFlags::default_ro()).unwrap();
        assert!(!table.is_read_only("/etc/file"));
        assert!(table.is_read_only("/rom/data"));
    }

    #[test]
    fn test_mount_flags() {
        let rw = MountFlags::default_rw();
        assert!(!rw.read_only);
        let ro = MountFlags::default_ro();
        assert!(ro.read_only);
    }

    #[test]
    fn test_list_mounts() {
        let mut table = MountTable::new(16);
        table.mount(String::from("/"), FsType::TernaryFs, InodeId(1), MountFlags::default_rw()).unwrap();
        table.mount(String::from("/mnt"), FsType::RamFs, InodeId(2), MountFlags::default_rw()).unwrap();
        assert_eq!(table.list().len(), 2);
    }
}
