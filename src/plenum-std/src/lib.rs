#![no_std]
#![doc = "PlenumNET std shim — maps Rust's std API surface to kernel primitives."]
#![doc = ""]
#![doc = "## Usage as std replacement"]
#![doc = ""]
#![doc = "Downstream crates that require `std` can be ported by adding:"]
#![doc = "```ignore"]
#![doc = "#![no_std]"]
#![doc = "extern crate plenum_std as std;"]
#![doc = "```"]
#![doc = ""]
#![doc = "The kernel registers backends at boot:"]
#![doc = "- `plenum_std::time::register_clock_source(fn)` — monotonic nanosecond clock"]
#![doc = "- `plenum_std::thread::register_task_spawner(fn)` — kernel task scheduler"]
#![doc = "- `plenum_std::thread::set_current_task_id(id)` — per-context task identity"]

extern crate alloc;

pub mod collections;
pub mod sync;
pub mod thread;
pub mod time;
pub mod io;
pub mod net;
pub mod fs;
pub mod env;
pub mod process;
pub mod panic;

pub mod fmt {
    pub use core::fmt::*;
}

pub mod string {
    pub use alloc::string::*;
}

pub mod vec {
    pub use alloc::vec::*;
}

pub mod boxed {
    pub use alloc::boxed::*;
}

pub mod rc {
    pub use alloc::rc::*;
}

pub mod borrow {
    pub use alloc::borrow::*;
}

pub mod cell {
    pub use core::cell::*;
}

pub mod marker {
    pub use core::marker::*;
}

pub mod ops {
    pub use core::ops::*;
}

pub mod cmp {
    pub use core::cmp::*;
}

pub mod hash {
    pub use core::hash::*;
}

pub mod iter {
    pub use core::iter::*;
}

pub mod convert {
    pub use core::convert::*;
}

pub mod option {
    pub use core::option::*;
}

pub mod result {
    pub use core::result::*;
}

pub mod slice {
    pub use alloc::slice::*;
}

pub mod str {
    pub use alloc::str::*;
}

pub mod num {
    pub use core::num::*;
}

pub mod mem {
    pub use core::mem::*;
}

pub mod ptr {
    pub use core::ptr::*;
}

pub mod clone {
    pub use core::clone::*;
}

pub mod default {
    pub use core::default::*;
}

pub mod any {
    pub use core::any::*;
}

pub mod prelude {
    pub mod v1 {
        pub use alloc::boxed::Box;
        pub use alloc::string::{String, ToString};
        pub use alloc::vec::Vec;
        pub use alloc::format;
        pub use alloc::vec;
        pub use core::prelude::v1::*;
    }

    pub mod rust_2021 {
        pub use super::v1::*;
        pub use core::prelude::rust_2021::*;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashmap_roundtrip() {
        use collections::HashMap;
        let mut map = HashMap::new();
        map.insert("key1", 42);
        map.insert("key2", 99);
        assert_eq!(map.get("key1"), Some(&42));
        assert_eq!(map.get("key2"), Some(&99));
        assert_eq!(map.len(), 2);
        map.remove("key1");
        assert_eq!(map.get("key1"), None);
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_hashset_basic() {
        use collections::HashSet;
        let mut set = HashSet::new();
        set.insert(1);
        set.insert(2);
        set.insert(1);
        assert_eq!(set.len(), 2);
        assert!(set.contains(&1));
        assert!(set.contains(&2));
    }

    #[test]
    fn test_btreemap_ordered() {
        use collections::BTreeMap;
        let mut map = BTreeMap::new();
        map.insert(3, "c");
        map.insert(1, "a");
        map.insert(2, "b");
        let keys: alloc::vec::Vec<_> = map.keys().collect();
        assert_eq!(keys, alloc::vec![&1, &2, &3]);
    }

    #[test]
    fn test_instant_monotonic() {
        let t1 = time::Instant::now();
        let t2 = time::Instant::now();
        let t3 = time::Instant::now();
        assert!(t2 >= t1);
        assert!(t3 >= t2);
        let elapsed = t3 - t1;
        assert!(elapsed.as_nanos() > 0);
    }

    #[test]
    fn test_instant_elapsed_nonzero() {
        let t1 = time::Instant::now();
        let _t2 = time::Instant::now();
        let elapsed = t1.elapsed();
        assert!(elapsed.as_nanos() > 0);
    }

    #[test]
    fn test_system_time_ordering() {
        let st1 = time::SystemTime::now();
        let st2 = time::SystemTime::now();
        let diff = st2.duration_since(st1).unwrap();
        assert!(diff.as_nanos() > 0);
    }

    #[test]
    fn test_tls_persistence() {
        use core::cell::Cell;

        thread::set_current_task_id(100);

        static TLS_COUNTER: thread::LocalKey<Cell<u32>> =
            thread::LocalKey::new(|| Cell::new(0));

        TLS_COUNTER.with(|c| {
            assert_eq!(c.get(), 0);
            c.set(42);
        });

        TLS_COUNTER.with(|c| {
            assert_eq!(c.get(), 42);
        });

        TLS_COUNTER.with(|c| {
            c.set(c.get() + 1);
        });
        TLS_COUNTER.with(|c| {
            assert_eq!(c.get(), 43);
        });
    }

    #[test]
    fn test_tls_task_isolation() {
        use core::cell::Cell;

        static TLS_VAL: thread::LocalKey<Cell<u32>> =
            thread::LocalKey::new(|| Cell::new(0));

        thread::set_current_task_id(200);
        TLS_VAL.with(|c| c.set(10));

        thread::set_current_task_id(201);
        TLS_VAL.with(|c| {
            assert_eq!(c.get(), 0);
            c.set(20);
        });

        thread::set_current_task_id(200);
        TLS_VAL.with(|c| {
            assert_eq!(c.get(), 10);
        });

        thread::set_current_task_id(201);
        TLS_VAL.with(|c| {
            assert_eq!(c.get(), 20);
        });
    }

    #[test]
    fn test_spawn_and_join() {
        let handle = thread::spawn(|| 42);
        assert!(handle.is_finished());
        let result = handle.join().unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_spawn_captures_state() {
        let data = alloc::vec![1, 2, 3];
        let handle = thread::spawn(move || {
            data.iter().sum::<i32>()
        });
        assert_eq!(handle.join().unwrap(), 6);
    }

    #[test]
    fn test_mpsc_send_recv() {
        let (tx, rx) = sync::mpsc::channel();
        tx.send(1).unwrap();
        tx.send(2).unwrap();
        tx.send(3).unwrap();
        assert_eq!(rx.recv().unwrap(), 1);
        assert_eq!(rx.recv().unwrap(), 2);
        assert_eq!(rx.recv().unwrap(), 3);
    }

    #[test]
    fn test_mpsc_disconnect_on_sender_drop() {
        let (tx, rx) = sync::mpsc::channel::<i32>();
        tx.send(42).unwrap();
        drop(tx);
        assert_eq!(rx.recv().unwrap(), 42);
        assert!(rx.recv().is_err());
    }

    #[test]
    fn test_mpsc_clone_senders() {
        let (tx, rx) = sync::mpsc::channel();
        let tx2 = tx.clone();
        tx.send(1).unwrap();
        tx2.send(2).unwrap();
        drop(tx);
        assert!(rx.recv().is_ok());
        assert!(rx.recv().is_ok());
        tx2.send(3).unwrap();
        assert_eq!(rx.recv().unwrap(), 3);
        drop(tx2);
        assert!(rx.recv().is_err());
    }

    #[test]
    fn test_mutex_basic() {
        let m = sync::Mutex::new(0);
        {
            let mut guard = m.lock().unwrap();
            *guard = 42;
        }
        assert_eq!(*m.lock().unwrap(), 42);
    }

    #[test]
    fn test_rwlock_basic() {
        let lock = sync::RwLock::new(5);
        assert_eq!(*lock.read().unwrap(), 5);
        {
            let mut w = lock.write().unwrap();
            *w = 10;
        }
        assert_eq!(*lock.read().unwrap(), 10);
    }

    #[test]
    fn test_once_call_once() {
        static ONCE: sync::Once = sync::Once::new();
        let mut called = 0u32;
        ONCE.call_once(|| {
            called += 1;
        });
        assert_eq!(called, 1);
    }

    #[test]
    fn test_io_cursor_roundtrip() {
        use io::{Cursor, Read, Write};
        let mut buf = alloc::vec![0u8; 64];
        let mut cursor = Cursor::new(&mut buf[..]);
        let data = b"hello world";
        cursor.write_all(data).unwrap();

        let mut cursor2 = Cursor::new(&buf[..data.len()]);
        let mut output = alloc::vec![0u8; data.len()];
        cursor2.read_exact(&mut output).unwrap();
        assert_eq!(&output, data);
    }

    #[test]
    fn test_bufreader_read_line() {
        use io::BufRead;
        let data = b"line1\nline2\nline3\n";
        let reader = io::BufReader::new(io::Cursor::new(&data[..]));
        let lines: alloc::vec::Vec<_> = reader.lines().map(|l| l.unwrap()).collect();
        assert_eq!(lines, alloc::vec!["line1", "line2", "line3"]);
    }

    #[test]
    fn test_net_stub_returns_error() {
        let result = net::TcpStream::connect("127.0.0.1:80");
        assert!(result.is_err());
    }

    #[test]
    fn test_fs_stub_returns_error() {
        let result = fs::read_to_string("/etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn test_env_stub() {
        assert!(env::var("HOME").is_err());
        let args: alloc::vec::Vec<_> = env::args().collect();
        assert_eq!(args.len(), 1);
        assert_eq!(args[0], "plenumnet-kernel");
    }

    #[test]
    fn test_clock_source_registration() {
        use core::sync::atomic::{AtomicU64, Ordering};
        static FAKE_CLOCK: AtomicU64 = AtomicU64::new(1_000_000);

        fn fake_clock() -> u64 {
            FAKE_CLOCK.fetch_add(1_000_000, Ordering::Relaxed)
        }

        time::register_clock_source(fake_clock);

        let t1 = time::Instant::now();
        let t2 = time::Instant::now();
        let elapsed = t2 - t1;
        assert_eq!(elapsed.as_millis(), 1);

        time::register_clock_source(fake_clock);
    }
}
