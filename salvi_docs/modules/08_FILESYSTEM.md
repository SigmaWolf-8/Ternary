# Module Guide: Filesystem

**Module:** `salvi_fs`  
**Status:** Complete (P1.5-011 to P1.5-015)  
**Tests:** ~56 tests

---

## Overview

The Filesystem module provides a complete hierarchical file storage system optimized for ternary data. It implements inode-based file management, directory operations with efficient lookup, and a flexible mount system supporting multiple filesystem types.

### Key Features

- **Inode Management** — Ternary-native file metadata
- **Directory Operations** — B-tree indexed directories
- **File Operations** — Mode enforcement and access control
- **Mount System** — VFS layer supporting multiple filesystems
- **Journaling** — Crash-consistent metadata updates

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                 Virtual Filesystem (VFS)                     │
├─────────────────────────────────────────────────────────────┤
│   open() read() write() mkdir() unlink() mount()           │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │  TernFS  │  │   Ext4   │  │   FAT    │  │  TmpFS   │   │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘   │
├─────────────────────────────────────────────────────────────┤
│                    Inode Cache Layer                         │
├─────────────────────────────────────────────────────────────┤
│                Buffer Cache (I/O Subsystem)                  │
└─────────────────────────────────────────────────────────────┘
```

---

## Inode Management

### Inode Structure

```rust
use salvi_fs::inode::{Inode, InodeType, InodeId};

pub struct Inode {
    pub id: InodeId,
    pub inode_type: InodeType,
    pub uid: UserId,
    pub gid: GroupId,
    pub mode: FileMode,
    pub size: u64,
    pub blocks: u64,
    pub atime: Timestamp,
    pub mtime: Timestamp,
    pub ctime: Timestamp,
    pub crtime: Timestamp,
    pub link_count: u32,
    pub data_blocks: BlockPointers,
    pub xattrs: XattrBlock,
}

pub enum InodeType {
    RegularFile,
    Directory,
    Symlink,
    CharDevice(DeviceId),
    BlockDevice(DeviceId),
    Fifo,
    Socket,
}
```

### Block Pointers (Ternary-Optimized)

```rust
pub struct BlockPointers {
    pub direct: [BlockId; 12],
    pub indirect: BlockId,
    pub double_indirect: BlockId,
    pub triple_indirect: BlockId,
}

// With ternary block IDs, each pointer level holds 3^9 = 19,683 pointers
// Max file size with triple indirect: ~150 PB (at 4KB blocks)
```

### Inode Operations

```rust
use salvi_fs::inode::{InodeCache, InodeOps};

let cache = InodeCache::instance();

let inode = cache.get(filesystem, inode_id)?;

let new_inode = cache.create(filesystem, InodeType::RegularFile)?;
new_inode.uid = current_uid();
new_inode.gid = current_gid();
new_inode.mode = FileMode::from_bits(0o644);

inode.mtime = Timestamp::now();
cache.write(inode)?;
cache.delete(inode)?;
```

---

## Directory Operations

### Directory Entry Structure

```rust
use salvi_fs::dir::{DirEntry, DirEntryType};

pub struct DirEntry {
    pub inode: InodeId,
    pub entry_type: DirEntryType,
    pub name: FileName,
    pub name_hash: u32,
}

pub struct Directory {
    pub inode: Inode,
    pub btree: BTree<FileName, DirEntry>,
}
```

### Directory Operations

```rust
use salvi_fs::dir::{Dir, DirOps};

let dir = Dir::open("/home/user")?;

for entry in dir.entries()? {
    println!("{}: {:?}", entry.name, entry.entry_type);
}

let entry = dir.lookup("myfile.txt")?;
dir.mkdir("newdir", FileMode::from_bits(0o755))?;
dir.remove("oldfile.txt")?;
dir.rename("old_name", "new_name")?;
```

### Path Resolution

```rust
use salvi_fs::path::{PathResolver, ResolveFlags};

let resolver = PathResolver::new();

let inode = resolver.resolve("/home/user/file.txt")?;
let inode = resolver.resolve_with_flags(
    "/home/user/link",
    ResolveFlags::FOLLOW_SYMLINKS
)?;
let inode = resolver.resolve_at(dir_fd, "relative/path")?;
```

---

## File Operations

### Opening Files

```rust
use salvi_fs::file::{File, OpenFlags, FileMode};

let file = File::open("/path/to/file", OpenFlags::READ)?;

let file = File::open(
    "/path/to/new_file",
    OpenFlags::CREATE | OpenFlags::WRITE | OpenFlags::TRUNCATE,
)?;
```

### Reading and Writing

```rust
let mut buffer = vec![0u8; 4096];
let bytes_read = file.read(&mut buffer)?;
let bytes_read = file.read_at(&mut buffer, offset)?;

let bytes_written = file.write(&data)?;
let bytes_written = file.write_at(&data, offset)?;
```

### File Locking

```rust
use salvi_fs::lock::{FileLock, LockType};

let lock = file.lock(LockType::Exclusive)?;
drop(lock);

let lock = file.lock(LockType::Shared)?;

match file.try_lock(LockType::Exclusive) {
    Ok(lock) => { /* Got lock */ },
    Err(LockError::WouldBlock) => { /* Lock held by another */ },
}
```

---

## Mount System

### Mounting Filesystems

```rust
use salvi_fs::mount::{Mount, MountFlags};

Mount::mount(
    "/dev/sda1",
    "/mnt/data",
    "ternfs",
    MountFlags::empty(),
    None,
)?;

Mount::mount(
    "/dev/sda2",
    "/mnt/backup",
    "ext4",
    MountFlags::READ_ONLY | MountFlags::NO_EXEC,
    Some("noatime"),
)?;

Mount::unmount("/mnt/data", UnmountFlags::empty())?;
```

### Filesystem Types

```rust
use salvi_fs::fstype::{FsType, FsTypeRegistry};

let registry = FsTypeRegistry::instance();

registry.register(FsType {
    name: "ternfs",
    mount: ternfs_mount,
    unmount: ternfs_unmount,
    flags: FsFlags::REQUIRES_DEV,
})?;
```

---

## TernFS (Native Ternary Filesystem)

### Features

| Feature | Description |
|---------|-------------|
| Block Size | 6 trytes (ternary-aligned) |
| Max File Size | ~150 PB |
| Max Path Length | 729 trytes (3^6) |
| Directory Index | B-tree with ternary keys |
| Journaling | Metadata journaling |
| Compression | Optional ternary compression |

### Creating TernFS

```rust
use salvi_fs::ternfs::TernFs;

TernFs::format("/dev/sda1", FormatOptions {
    block_size: 4096,
    inode_ratio: 16384,
    journal_size: 128 * 1024 * 1024,
    label: "ternary_root",
})?;
```

---

## Journaling

### Transaction API

```rust
use salvi_fs::journal::{Journal, Transaction};

let journal = Journal::instance();

let txn = journal.begin()?;
txn.write_inode(inode)?;
txn.write_block(block_id, data)?;
txn.update_directory(dir_inode, entry)?;
txn.commit()?;
```

### Recovery

```rust
if journal.needs_recovery()? {
    let recovered = journal.recover()?;
    println!("Recovered {} transactions", recovered);
}
```

---

## Best Practices

### 1. Always Close Files
Use RAII (drop) to ensure files are closed and buffers flushed.

### 2. Use Journaling for Metadata
Enable journaling to prevent filesystem corruption on crashes.

### 3. Check Return Values
All file operations can fail - handle errors appropriately.

### 4. Use Path Resolution Safely
Always validate paths and handle symlink loops.

---

*Così sia.*
