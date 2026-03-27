use crate::io::{self, Error, ErrorKind, Read, Write, Seek, SeekFrom};
use alloc::string::String;
use alloc::vec::Vec;

pub struct File;

impl File {
    pub fn open<P>(_path: P) -> io::Result<File> {
        Err(Error::new(ErrorKind::Unsupported, "fonts embedded, no filesystem"))
    }

    pub fn create<P>(_path: P) -> io::Result<File> {
        Err(Error::new(ErrorKind::Unsupported, "fonts embedded, no filesystem"))
    }

    pub fn metadata(&self) -> io::Result<Metadata> {
        Err(Error::new(ErrorKind::Unsupported, "fonts embedded, no filesystem"))
    }

    pub fn set_len(&self, _size: u64) -> io::Result<()> {
        Err(Error::new(ErrorKind::Unsupported, "fonts embedded, no filesystem"))
    }

    pub fn sync_all(&self) -> io::Result<()> {
        Err(Error::new(ErrorKind::Unsupported, "fonts embedded, no filesystem"))
    }

    pub fn sync_data(&self) -> io::Result<()> {
        Err(Error::new(ErrorKind::Unsupported, "fonts embedded, no filesystem"))
    }

    pub fn try_clone(&self) -> io::Result<File> {
        Err(Error::new(ErrorKind::Unsupported, "fonts embedded, no filesystem"))
    }
}

impl Read for File {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Err(Error::new(ErrorKind::Unsupported, "fonts embedded, no filesystem"))
    }
}

impl Write for File {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        Err(Error::new(ErrorKind::Unsupported, "fonts embedded, no filesystem"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Err(Error::new(ErrorKind::Unsupported, "fonts embedded, no filesystem"))
    }
}

impl Seek for File {
    fn seek(&mut self, _pos: SeekFrom) -> io::Result<u64> {
        Err(Error::new(ErrorKind::Unsupported, "fonts embedded, no filesystem"))
    }
}

pub fn read_to_string<P>(_path: P) -> io::Result<String> {
    Err(Error::new(ErrorKind::Unsupported, "fonts embedded, no filesystem"))
}

pub fn read<P>(_path: P) -> io::Result<Vec<u8>> {
    Err(Error::new(ErrorKind::Unsupported, "fonts embedded, no filesystem"))
}

pub fn write<P, C: AsRef<[u8]>>(_path: P, _contents: C) -> io::Result<()> {
    Err(Error::new(ErrorKind::Unsupported, "fonts embedded, no filesystem"))
}

pub fn remove_file<P>(_path: P) -> io::Result<()> {
    Err(Error::new(ErrorKind::Unsupported, "fonts embedded, no filesystem"))
}

pub fn create_dir<P>(_path: P) -> io::Result<()> {
    Err(Error::new(ErrorKind::Unsupported, "fonts embedded, no filesystem"))
}

pub fn create_dir_all<P>(_path: P) -> io::Result<()> {
    Err(Error::new(ErrorKind::Unsupported, "fonts embedded, no filesystem"))
}

pub fn remove_dir<P>(_path: P) -> io::Result<()> {
    Err(Error::new(ErrorKind::Unsupported, "fonts embedded, no filesystem"))
}

pub fn remove_dir_all<P>(_path: P) -> io::Result<()> {
    Err(Error::new(ErrorKind::Unsupported, "fonts embedded, no filesystem"))
}

pub fn rename<P, Q>(_from: P, _to: Q) -> io::Result<()> {
    Err(Error::new(ErrorKind::Unsupported, "fonts embedded, no filesystem"))
}

pub fn copy<P, Q>(_from: P, _to: Q) -> io::Result<u64> {
    Err(Error::new(ErrorKind::Unsupported, "fonts embedded, no filesystem"))
}

pub fn metadata<P>(_path: P) -> io::Result<Metadata> {
    Err(Error::new(ErrorKind::Unsupported, "fonts embedded, no filesystem"))
}

pub fn canonicalize<P>(_path: P) -> io::Result<alloc::string::String> {
    Err(Error::new(ErrorKind::Unsupported, "fonts embedded, no filesystem"))
}

pub fn read_dir<P>(_path: P) -> io::Result<ReadDir> {
    Err(Error::new(ErrorKind::Unsupported, "fonts embedded, no filesystem"))
}

pub struct Metadata {
    _private: (),
}

impl Metadata {
    pub fn len(&self) -> u64 {
        0
    }

    pub fn is_dir(&self) -> bool {
        false
    }

    pub fn is_file(&self) -> bool {
        false
    }

    pub fn is_symlink(&self) -> bool {
        false
    }
}

pub struct ReadDir;

impl Iterator for ReadDir {
    type Item = io::Result<DirEntry>;
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

pub struct DirEntry {
    _private: (),
}

impl DirEntry {
    pub fn file_name(&self) -> String {
        String::new()
    }

    pub fn metadata(&self) -> io::Result<Metadata> {
        Err(Error::new(ErrorKind::Unsupported, "fonts embedded, no filesystem"))
    }
}

pub struct OpenOptions {
    _read: bool,
    _write: bool,
    _append: bool,
    _truncate: bool,
    _create: bool,
    _create_new: bool,
}

impl OpenOptions {
    pub fn new() -> Self {
        Self {
            _read: false,
            _write: false,
            _append: false,
            _truncate: false,
            _create: false,
            _create_new: false,
        }
    }

    pub fn read(&mut self, read: bool) -> &mut Self {
        self._read = read;
        self
    }

    pub fn write(&mut self, write: bool) -> &mut Self {
        self._write = write;
        self
    }

    pub fn append(&mut self, append: bool) -> &mut Self {
        self._append = append;
        self
    }

    pub fn truncate(&mut self, truncate: bool) -> &mut Self {
        self._truncate = truncate;
        self
    }

    pub fn create(&mut self, create: bool) -> &mut Self {
        self._create = create;
        self
    }

    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        self._create_new = create_new;
        self
    }

    pub fn open<P>(&self, _path: P) -> io::Result<File> {
        Err(Error::new(ErrorKind::Unsupported, "fonts embedded, no filesystem"))
    }
}
