use alloc::collections::VecDeque;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Arc::new(ChannelInner {
        queue: super::mutex_impl::Mutex::new(VecDeque::new()),
        closed: AtomicBool::new(false),
        sender_count: AtomicUsize::new(1),
    });
    (
        Sender {
            inner: inner.clone(),
        },
        Receiver { inner },
    )
}

struct ChannelInner<T> {
    queue: super::mutex_impl::Mutex<VecDeque<T>>,
    closed: AtomicBool,
    sender_count: AtomicUsize,
}

pub struct Sender<T> {
    inner: Arc<ChannelInner<T>>,
}

impl<T> Sender<T> {
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        if self.inner.closed.load(Ordering::Acquire) {
            return Err(SendError(value));
        }
        if let Ok(mut queue) = self.inner.queue.lock() {
            queue.push_back(value);
            Ok(())
        } else {
            Err(SendError(value))
        }
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        self.inner.sender_count.fetch_add(1, Ordering::Relaxed);
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        if self.inner.sender_count.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.inner.closed.store(true, Ordering::Release);
        }
    }
}

pub struct Receiver<T> {
    inner: Arc<ChannelInner<T>>,
}

impl<T> Receiver<T> {
    pub fn recv(&self) -> Result<T, RecvError> {
        loop {
            if let Ok(mut queue) = self.inner.queue.lock() {
                if let Some(value) = queue.pop_front() {
                    return Ok(value);
                }
                if self.inner.closed.load(Ordering::Acquire) {
                    return Err(RecvError);
                }
            }
            core::hint::spin_loop();
        }
    }

    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        if let Ok(mut queue) = self.inner.queue.lock() {
            if let Some(value) = queue.pop_front() {
                return Ok(value);
            }
            if self.inner.closed.load(Ordering::Acquire) {
                return Err(TryRecvError::Disconnected);
            }
        }
        Err(TryRecvError::Empty)
    }

    pub fn iter(&self) -> RecvIter<'_, T> {
        RecvIter { receiver: self }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.inner.closed.store(true, Ordering::Release);
    }
}

pub struct RecvIter<'a, T> {
    receiver: &'a Receiver<T>,
}

impl<'a, T> Iterator for RecvIter<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.receiver.recv().ok()
    }
}

#[derive(Debug)]
pub struct SendError<T>(pub T);

impl<T> core::fmt::Display for SendError<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "sending on a closed channel")
    }
}

#[derive(Debug)]
pub struct RecvError;

impl core::fmt::Display for RecvError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "receiving on a closed channel")
    }
}

#[derive(Debug)]
pub enum TryRecvError {
    Empty,
    Disconnected,
}

impl core::fmt::Display for TryRecvError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TryRecvError::Empty => write!(f, "receiving on an empty channel"),
            TryRecvError::Disconnected => write!(f, "receiving on a closed channel"),
        }
    }
}
