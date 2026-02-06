use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use super::{FsError, FsResult};
use super::inode::InodeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FileDescriptor(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenMode {
    Read,
    Write,
    ReadWrite,
    Append,
}

#[derive(Debug, Clone)]
pub struct OpenFile {
    pub fd: FileDescriptor,
    pub inode_id: InodeId,
    pub mode: OpenMode,
    pub offset: u64,
}

pub struct FileData {
    pub inode_id: InodeId,
    pub data: Vec<i8>,
}

impl FileData {
    pub fn new(inode_id: InodeId) -> Self {
        Self {
            inode_id,
            data: Vec::new(),
        }
    }

    pub fn read(&self, offset: u64, buf: &mut [i8]) -> FsResult<usize> {
        let offset = offset as usize;
        if offset >= self.data.len() {
            return Ok(0);
        }
        let available = self.data.len() - offset;
        let count = core::cmp::min(buf.len(), available);
        buf[..count].copy_from_slice(&self.data[offset..offset + count]);
        Ok(count)
    }

    pub fn write(&mut self, offset: u64, data: &[i8]) -> FsResult<usize> {
        let offset = offset as usize;
        let end = offset + data.len();
        if end > self.data.len() {
            self.data.resize(end, 0);
        }
        self.data[offset..end].copy_from_slice(data);
        Ok(data.len())
    }

    pub fn truncate(&mut self, size: u64) {
        self.data.truncate(size as usize);
    }

    pub fn size(&self) -> u64 {
        self.data.len() as u64
    }
}

pub struct FileTable {
    files: BTreeMap<u64, FileData>,
    open_files: BTreeMap<u32, OpenFile>,
    next_fd: u32,
}

impl FileTable {
    pub fn new() -> Self {
        Self {
            files: BTreeMap::new(),
            open_files: BTreeMap::new(),
            next_fd: 3,
        }
    }

    pub fn create_file(&mut self, inode_id: InodeId) {
        self.files.entry(inode_id.0).or_insert_with(|| FileData::new(inode_id));
    }

    pub fn delete_file(&mut self, inode_id: InodeId) -> FsResult<()> {
        let has_open = self.open_files.values().any(|f| f.inode_id == inode_id);
        if has_open {
            return Err(FsError::PermissionDenied);
        }
        self.files.remove(&inode_id.0).ok_or(FsError::NotFound)?;
        Ok(())
    }

    pub fn open(&mut self, inode_id: InodeId, mode: OpenMode) -> FsResult<FileDescriptor> {
        if !self.files.contains_key(&inode_id.0) {
            return Err(FsError::NotFound);
        }
        let fd = FileDescriptor(self.next_fd);
        self.next_fd += 1;

        let offset = if mode == OpenMode::Append {
            self.files.get(&inode_id.0).map_or(0, |f| f.size())
        } else {
            0
        };

        self.open_files.insert(fd.0, OpenFile { fd, inode_id, mode, offset });
        Ok(fd)
    }

    pub fn close(&mut self, fd: FileDescriptor) -> FsResult<()> {
        self.open_files.remove(&fd.0).ok_or(FsError::NotFound)?;
        Ok(())
    }

    pub fn read(&mut self, fd: FileDescriptor, buf: &mut [i8]) -> FsResult<usize> {
        let open = self.open_files.get(&fd.0).ok_or(FsError::NotFound)?;
        if open.mode == OpenMode::Write {
            return Err(FsError::PermissionDenied);
        }
        let inode_id = open.inode_id;
        let offset = open.offset;

        let file = self.files.get(&inode_id.0).ok_or(FsError::NotFound)?;
        let count = file.read(offset, buf)?;

        let open = self.open_files.get_mut(&fd.0).unwrap();
        open.offset += count as u64;
        Ok(count)
    }

    pub fn write(&mut self, fd: FileDescriptor, data: &[i8]) -> FsResult<usize> {
        let open = self.open_files.get(&fd.0).ok_or(FsError::NotFound)?;
        if open.mode == OpenMode::Read {
            return Err(FsError::PermissionDenied);
        }
        let inode_id = open.inode_id;
        let offset = open.offset;

        let file = self.files.get_mut(&inode_id.0).ok_or(FsError::NotFound)?;
        let count = file.write(offset, data)?;

        let open = self.open_files.get_mut(&fd.0).unwrap();
        open.offset += count as u64;
        Ok(count)
    }

    pub fn seek(&mut self, fd: FileDescriptor, offset: u64) -> FsResult<()> {
        let open = self.open_files.get_mut(&fd.0).ok_or(FsError::NotFound)?;
        open.offset = offset;
        Ok(())
    }

    pub fn file_size(&self, inode_id: InodeId) -> FsResult<u64> {
        self.files.get(&inode_id.0).map(|f| f.size()).ok_or(FsError::NotFound)
    }

    pub fn truncate(&mut self, inode_id: InodeId, size: u64) -> FsResult<()> {
        let file = self.files.get_mut(&inode_id.0).ok_or(FsError::NotFound)?;
        file.truncate(size);
        Ok(())
    }

    pub fn open_count(&self) -> usize {
        self.open_files.len()
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    pub fn get_open(&self, fd: FileDescriptor) -> FsResult<&OpenFile> {
        self.open_files.get(&fd.0).ok_or(FsError::NotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_open() {
        let mut ft = FileTable::new();
        let inode = InodeId(1);
        ft.create_file(inode);
        let fd = ft.open(inode, OpenMode::ReadWrite).unwrap();
        assert_eq!(ft.open_count(), 1);
        ft.close(fd).unwrap();
        assert_eq!(ft.open_count(), 0);
    }

    #[test]
    fn test_read_write() {
        let mut ft = FileTable::new();
        let inode = InodeId(1);
        ft.create_file(inode);
        let fd = ft.open(inode, OpenMode::ReadWrite).unwrap();

        ft.write(fd, &[1i8, 0, -1, 1]).unwrap();
        ft.seek(fd, 0).unwrap();

        let mut buf = [0i8; 4];
        let n = ft.read(fd, &mut buf).unwrap();
        assert_eq!(n, 4);
        assert_eq!(buf, [1, 0, -1, 1]);
    }

    #[test]
    fn test_read_mode_no_write() {
        let mut ft = FileTable::new();
        let inode = InodeId(1);
        ft.create_file(inode);
        let fd = ft.open(inode, OpenMode::Read).unwrap();
        assert_eq!(ft.write(fd, &[0i8]), Err(FsError::PermissionDenied));
    }

    #[test]
    fn test_write_mode_no_read() {
        let mut ft = FileTable::new();
        let inode = InodeId(1);
        ft.create_file(inode);
        let fd = ft.open(inode, OpenMode::Write).unwrap();
        let mut buf = [0i8; 1];
        assert_eq!(ft.read(fd, &mut buf), Err(FsError::PermissionDenied));
    }

    #[test]
    fn test_append_mode() {
        let mut ft = FileTable::new();
        let inode = InodeId(1);
        ft.create_file(inode);

        let fd1 = ft.open(inode, OpenMode::Write).unwrap();
        ft.write(fd1, &[1i8, 2, 3]).unwrap();
        ft.close(fd1).unwrap();

        let fd2 = ft.open(inode, OpenMode::Append).unwrap();
        ft.write(fd2, &[4i8, 5]).unwrap();
        ft.close(fd2).unwrap();

        assert_eq!(ft.file_size(inode).unwrap(), 5);
    }

    #[test]
    fn test_seek() {
        let mut ft = FileTable::new();
        let inode = InodeId(1);
        ft.create_file(inode);
        let fd = ft.open(inode, OpenMode::ReadWrite).unwrap();
        ft.write(fd, &[1i8, 2, 3, 4, 5]).unwrap();
        ft.seek(fd, 2).unwrap();
        let mut buf = [0i8; 2];
        ft.read(fd, &mut buf).unwrap();
        assert_eq!(buf, [3, 4]);
    }

    #[test]
    fn test_truncate() {
        let mut ft = FileTable::new();
        let inode = InodeId(1);
        ft.create_file(inode);
        let fd = ft.open(inode, OpenMode::Write).unwrap();
        ft.write(fd, &[1i8; 100]).unwrap();
        ft.close(fd).unwrap();
        ft.truncate(inode, 10).unwrap();
        assert_eq!(ft.file_size(inode).unwrap(), 10);
    }

    #[test]
    fn test_read_past_end() {
        let mut ft = FileTable::new();
        let inode = InodeId(1);
        ft.create_file(inode);
        let fd = ft.open(inode, OpenMode::ReadWrite).unwrap();
        ft.write(fd, &[1i8, 2]).unwrap();
        ft.seek(fd, 10).unwrap();
        let mut buf = [0i8; 5];
        let n = ft.read(fd, &mut buf).unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn test_delete_open_file() {
        let mut ft = FileTable::new();
        let inode = InodeId(1);
        ft.create_file(inode);
        let _fd = ft.open(inode, OpenMode::Read).unwrap();
        assert_eq!(ft.delete_file(inode), Err(FsError::PermissionDenied));
    }

    #[test]
    fn test_delete_closed_file() {
        let mut ft = FileTable::new();
        let inode = InodeId(1);
        ft.create_file(inode);
        ft.delete_file(inode).unwrap();
        assert_eq!(ft.file_count(), 0);
    }

    #[test]
    fn test_open_nonexistent() {
        let mut ft = FileTable::new();
        assert_eq!(ft.open(InodeId(999), OpenMode::Read), Err(FsError::NotFound));
    }

    #[test]
    fn test_multiple_fds() {
        let mut ft = FileTable::new();
        let inode = InodeId(1);
        ft.create_file(inode);
        let fd1 = ft.open(inode, OpenMode::Read).unwrap();
        let fd2 = ft.open(inode, OpenMode::Read).unwrap();
        assert_ne!(fd1, fd2);
        assert_eq!(ft.open_count(), 2);
    }
}
