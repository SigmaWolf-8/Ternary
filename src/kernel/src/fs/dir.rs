use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use super::{FsError, FsResult, validate_name};
use super::inode::InodeId;

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub inode_id: InodeId,
}

pub struct Directory {
    pub inode_id: InodeId,
    pub parent_id: InodeId,
    entries: BTreeMap<String, InodeId>,
}

impl Directory {
    pub fn new(inode_id: InodeId, parent_id: InodeId) -> Self {
        Self {
            inode_id,
            parent_id,
            entries: BTreeMap::new(),
        }
    }

    pub fn root(inode_id: InodeId) -> Self {
        Self::new(inode_id, inode_id)
    }

    pub fn add_entry(&mut self, name: String, inode_id: InodeId) -> FsResult<()> {
        validate_name(&name)?;
        if self.entries.contains_key(&name) {
            return Err(FsError::AlreadyExists);
        }
        self.entries.insert(name, inode_id);
        Ok(())
    }

    pub fn remove_entry(&mut self, name: &str) -> FsResult<InodeId> {
        self.entries.remove(name).ok_or(FsError::NotFound)
    }

    pub fn lookup(&self, name: &str) -> FsResult<InodeId> {
        if name == "." {
            return Ok(self.inode_id);
        }
        if name == ".." {
            return Ok(self.parent_id);
        }
        self.entries.get(name).copied().ok_or(FsError::NotFound)
    }

    pub fn list(&self) -> Vec<DirEntry> {
        let mut entries = Vec::new();
        entries.push(DirEntry { name: String::from("."), inode_id: self.inode_id });
        entries.push(DirEntry { name: String::from(".."), inode_id: self.parent_id });
        for (name, &inode_id) in &self.entries {
            entries.push(DirEntry { name: name.clone(), inode_id });
        }
        entries
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.entries.contains_key(name)
    }

    pub fn rename(&mut self, old_name: &str, new_name: String) -> FsResult<()> {
        validate_name(&new_name)?;
        if self.entries.contains_key(&new_name) {
            return Err(FsError::AlreadyExists);
        }
        let inode_id = self.entries.remove(old_name).ok_or(FsError::NotFound)?;
        self.entries.insert(new_name, inode_id);
        Ok(())
    }
}

pub struct DirectoryTable {
    dirs: BTreeMap<u64, Directory>,
}

impl DirectoryTable {
    pub fn new() -> Self {
        Self { dirs: BTreeMap::new() }
    }

    pub fn create_root(&mut self, inode_id: InodeId) {
        self.dirs.insert(inode_id.0, Directory::root(inode_id));
    }

    pub fn create(&mut self, inode_id: InodeId, parent_id: InodeId) -> FsResult<()> {
        if self.dirs.contains_key(&inode_id.0) {
            return Err(FsError::AlreadyExists);
        }
        self.dirs.insert(inode_id.0, Directory::new(inode_id, parent_id));
        Ok(())
    }

    pub fn remove(&mut self, inode_id: InodeId) -> FsResult<()> {
        let dir = self.dirs.get(&inode_id.0).ok_or(FsError::NotFound)?;
        if !dir.is_empty() {
            return Err(FsError::NotEmpty);
        }
        self.dirs.remove(&inode_id.0);
        Ok(())
    }

    pub fn get(&self, inode_id: InodeId) -> FsResult<&Directory> {
        self.dirs.get(&inode_id.0).ok_or(FsError::NotADirectory)
    }

    pub fn get_mut(&mut self, inode_id: InodeId) -> FsResult<&mut Directory> {
        self.dirs.get_mut(&inode_id.0).ok_or(FsError::NotADirectory)
    }

    pub fn count(&self) -> usize {
        self.dirs.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directory_new() {
        let dir = Directory::new(InodeId(1), InodeId(0));
        assert!(dir.is_empty());
        assert_eq!(dir.entry_count(), 0);
    }

    #[test]
    fn test_root_directory() {
        let dir = Directory::root(InodeId(1));
        assert_eq!(dir.parent_id, InodeId(1));
    }

    #[test]
    fn test_add_entry() {
        let mut dir = Directory::new(InodeId(1), InodeId(0));
        dir.add_entry(String::from("file.txt"), InodeId(2)).unwrap();
        assert_eq!(dir.entry_count(), 1);
        assert!(dir.contains("file.txt"));
    }

    #[test]
    fn test_add_duplicate() {
        let mut dir = Directory::new(InodeId(1), InodeId(0));
        dir.add_entry(String::from("file.txt"), InodeId(2)).unwrap();
        assert_eq!(dir.add_entry(String::from("file.txt"), InodeId(3)), Err(FsError::AlreadyExists));
    }

    #[test]
    fn test_remove_entry() {
        let mut dir = Directory::new(InodeId(1), InodeId(0));
        dir.add_entry(String::from("file.txt"), InodeId(2)).unwrap();
        let id = dir.remove_entry("file.txt").unwrap();
        assert_eq!(id, InodeId(2));
        assert!(dir.is_empty());
    }

    #[test]
    fn test_lookup() {
        let mut dir = Directory::new(InodeId(1), InodeId(0));
        dir.add_entry(String::from("sub"), InodeId(5)).unwrap();
        assert_eq!(dir.lookup("sub"), Ok(InodeId(5)));
        assert_eq!(dir.lookup("."), Ok(InodeId(1)));
        assert_eq!(dir.lookup(".."), Ok(InodeId(0)));
        assert_eq!(dir.lookup("nope"), Err(FsError::NotFound));
    }

    #[test]
    fn test_list() {
        let mut dir = Directory::new(InodeId(1), InodeId(0));
        dir.add_entry(String::from("a"), InodeId(2)).unwrap();
        dir.add_entry(String::from("b"), InodeId(3)).unwrap();
        let entries = dir.list();
        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0].name, ".");
        assert_eq!(entries[1].name, "..");
    }

    #[test]
    fn test_rename() {
        let mut dir = Directory::new(InodeId(1), InodeId(0));
        dir.add_entry(String::from("old"), InodeId(2)).unwrap();
        dir.rename("old", String::from("new")).unwrap();
        assert!(dir.contains("new"));
        assert!(!dir.contains("old"));
    }

    #[test]
    fn test_rename_conflict() {
        let mut dir = Directory::new(InodeId(1), InodeId(0));
        dir.add_entry(String::from("a"), InodeId(2)).unwrap();
        dir.add_entry(String::from("b"), InodeId(3)).unwrap();
        assert_eq!(dir.rename("a", String::from("b")), Err(FsError::AlreadyExists));
    }

    #[test]
    fn test_directory_table() {
        let mut table = DirectoryTable::new();
        table.create_root(InodeId(1));
        table.create(InodeId(2), InodeId(1)).unwrap();
        assert_eq!(table.count(), 2);

        table.get_mut(InodeId(1)).unwrap().add_entry(String::from("sub"), InodeId(2)).unwrap();

        assert_eq!(table.get(InodeId(1)).unwrap().entry_count(), 1);
    }

    #[test]
    fn test_remove_nonempty_dir() {
        let mut table = DirectoryTable::new();
        table.create_root(InodeId(1));
        table.get_mut(InodeId(1)).unwrap().add_entry(String::from("x"), InodeId(99)).unwrap();
        assert_eq!(table.remove(InodeId(1)), Err(FsError::NotEmpty));
    }

    #[test]
    fn test_invalid_names() {
        let mut dir = Directory::new(InodeId(1), InodeId(0));
        assert!(dir.add_entry(String::from(""), InodeId(2)).is_err());
        assert!(dir.add_entry(String::from("."), InodeId(2)).is_err());
        assert!(dir.add_entry(String::from("a/b"), InodeId(2)).is_err());
    }
}
