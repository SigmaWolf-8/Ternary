#![no_std]
extern crate alloc;
extern crate plenum_std as std;

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;
use std::io::{Cursor, Read, Write, BufRead, BufReader};
use std::string::String;
use std::vec::Vec;

#[test]
fn facade_hashmap_roundtrip() {
    let mut map = HashMap::new();
    map.insert("hello", 42u64);
    map.insert("world", 99);
    assert_eq!(*map.get("hello").unwrap(), 42);
    assert_eq!(map.len(), 2);
}

#[test]
fn facade_mutex_lock_unlock() {
    let m = Mutex::new(alloc::vec![1, 2, 3]);
    {
        let mut guard = m.lock().unwrap();
        guard.push(4);
    }
    let guard = m.lock().unwrap();
    assert_eq!(*guard, alloc::vec![1, 2, 3, 4]);
}

#[test]
fn facade_instant_ordering() {
    let t1 = Instant::now();
    let t2 = Instant::now();
    assert!(t2 >= t1);
    let elapsed = t2 - t1;
    assert!(elapsed.as_nanos() >= 0);
}

#[test]
fn facade_io_cursor_write_read() {
    let mut buf = alloc::vec![0u8; 32];
    {
        let mut cursor = Cursor::new(&mut buf[..]);
        cursor.write_all(b"test").unwrap();
    }
    assert_eq!(&buf[..4], b"test");
}

#[test]
fn facade_bufreader_lines() {
    let data = b"alpha\nbeta\ngamma\n";
    let reader = BufReader::new(Cursor::new(&data[..]));
    let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();
    assert_eq!(lines, alloc::vec!["alpha", "beta", "gamma"]);
}

#[test]
fn facade_mpsc_channel() {
    let (tx, rx) = std::sync::mpsc::channel();
    tx.send(1i32).unwrap();
    tx.send(2).unwrap();
    drop(tx);
    assert_eq!(rx.recv().unwrap(), 1);
    assert_eq!(rx.recv().unwrap(), 2);
    assert!(rx.recv().is_err());
}

#[test]
fn facade_tls_persistence() {
    use core::cell::Cell;
    std::thread::set_current_task_id(500);

    static TLS: std::thread::LocalKey<Cell<i32>> =
        std::thread::LocalKey::new(|| Cell::new(-1));

    TLS.with(|c| {
        assert_eq!(c.get(), -1);
        c.set(100);
    });
    TLS.with(|c| {
        assert_eq!(c.get(), 100);
    });
}

#[test]
fn facade_spawn_join() {
    let handle = std::thread::spawn(|| {
        let mut sum = 0;
        for i in 0..10 {
            sum += i;
        }
        sum
    });
    assert_eq!(handle.join().unwrap(), 45);
}
