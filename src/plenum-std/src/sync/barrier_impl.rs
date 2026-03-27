use core::sync::atomic::{AtomicUsize, Ordering};

pub struct Barrier {
    count: usize,
    arrived: AtomicUsize,
    generation: AtomicUsize,
}

impl Barrier {
    pub const fn new(n: usize) -> Self {
        Self {
            count: n,
            arrived: AtomicUsize::new(0),
            generation: AtomicUsize::new(0),
        }
    }

    pub fn wait(&self) -> BarrierWaitResult {
        let gen = self.generation.load(Ordering::Relaxed);
        let arrived = self.arrived.fetch_add(1, Ordering::AcqRel) + 1;

        if arrived == self.count {
            self.arrived.store(0, Ordering::Release);
            self.generation.fetch_add(1, Ordering::Release);
            BarrierWaitResult { is_leader: true }
        } else {
            while self.generation.load(Ordering::Acquire) == gen {
                core::hint::spin_loop();
            }
            BarrierWaitResult { is_leader: false }
        }
    }
}

pub struct BarrierWaitResult {
    is_leader: bool,
}

impl BarrierWaitResult {
    pub fn is_leader(&self) -> bool {
        self.is_leader
    }
}
