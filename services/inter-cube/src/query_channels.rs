// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
//! Daemon-owned logical query channels. No service-slot ports are allocated.
//! Shared by the daemon and Yoda; advertisements are not generation evidence.
use std::{collections::HashMap, sync::{Arc, Mutex}, time::{Duration, Instant}};
use serde::{Deserialize, Serialize};

pub const PROTOCOL: u8 = 1;
pub const CHANNELS_PER_NODE: u8 = 27;
const MAX_HISTORY: usize = 4096;
const REPLAY_WINDOW: Duration = Duration::from_secs(86400);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub protocol: u8,
    pub node: u8,
    pub channel: u8,
    /// Changes every daemon process start; old advertisements cannot authorize work.
    pub epoch: String,
}

impl Binding {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.protocol != PROTOCOL { return Err("unsupported query-channel protocol"); }
        if !(1..=3).contains(&self.node) { return Err("node outside Array3"); }
        if !(1..=CHANNELS_PER_NODE).contains(&self.channel) { return Err("channel outside node"); }
        if self.epoch.len() != 32 || !self.epoch.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Err("invalid daemon epoch");
        }
        Ok(())
    }
    pub fn matches_response(&self, response: &Self) -> bool {
        self.validate().is_ok() && response.validate().is_ok() && self == response
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelSnapshot {
    pub binding: Binding,
    pub busy: bool,
    pub completed: u64,
    pub failed: u64,
}

struct Channel { active: bool, completed: u64, failed: u64 }
struct State {
    channels: Vec<Channel>,
    seen: HashMap<String, Instant>,
    active: usize,
}

/// Real per-channel ownership plus a shared model-capacity limit.
/// Busy is a rejection before acceptance, never an implicit second dispatch.
pub struct Pool {
    node: u8,
    epoch: String,
    capacity: usize,
    state: Mutex<State>,
}

impl Pool {
    pub fn new(node: u8, epoch: String, capacity: usize) -> Result<Arc<Self>, &'static str> {
        Binding { protocol: PROTOCOL, node, channel: 1, epoch: epoch.clone() }.validate()?;
        if !(1..=27).contains(&capacity) { return Err("model concurrency must be 1..27"); }
        Ok(Arc::new(Self {
            node, epoch, capacity,
            state: Mutex::new(State {
                channels: (0..27).map(|_| Channel { active: false, completed: 0, failed: 0 }).collect(),
                seen: HashMap::new(), active: 0,
            }),
        }))
    }

    pub fn capacity(&self) -> usize { self.capacity }

    pub fn snapshot(&self) -> Result<Vec<ChannelSnapshot>, &'static str> {
        let s = self.state.lock().map_err(|_| "query pool unavailable")?;
        Ok(s.channels.iter().enumerate().map(|(i,c)| ChannelSnapshot {
            binding: Binding { protocol: PROTOCOL, node: self.node, channel: i as u8 + 1, epoch: self.epoch.clone() },
            busy: c.active, completed: c.completed, failed: c.failed,
        }).collect())
    }

    pub fn admit(self: &Arc<Self>, binding: &Binding, request_id: &str) -> Result<Permit, &'static str> {
        binding.validate()?;
        if binding.node != self.node || binding.epoch != self.epoch { return Err("stale or wrong-node binding"); }
        if request_id.is_empty() || request_id.len() > 128
            || !request_id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-') {
            return Err("invalid request identity");
        }
        let mut s = self.state.lock().map_err(|_| "query pool unavailable")?;
        s.seen.retain(|_, at| at.elapsed() < REPLAY_WINDOW);
        if s.seen.contains_key(request_id) { return Err("request already accepted; do not redispatch"); }
        if s.seen.len() >= MAX_HISTORY { return Err("replay ledger full; admission blocked"); }
        let index = binding.channel as usize - 1;
        if s.channels[index].active { return Err("channel busy; not accepted"); }
        if s.active >= self.capacity { return Err("model capacity busy; not accepted"); }
        s.seen.insert(request_id.to_owned(), Instant::now());
        s.channels[index].active = true;
        s.active += 1;
        Ok(Permit { pool: self.clone(), index, succeeded: false })
    }
}

/// Retained across the actual model HTTP request. Dropping on error/cancellation
/// releases capacity and records failure, never a completed generation.
pub struct Permit { pool: Arc<Pool>, index: usize, succeeded: bool }
impl Permit {
    pub fn complete(&mut self, nonempty_answer: &str) -> Result<(), &'static str> {
        if nonempty_answer.trim().is_empty() { return Err("empty model answer"); }
        self.succeeded = true;
        Ok(())
    }
}
impl Drop for Permit {
    fn drop(&mut self) {
        if let Ok(mut s) = self.pool.state.lock() {
            let c = &mut s.channels[self.index];
            c.active = false;
            if self.succeeded { c.completed += 1; } else { c.failed += 1; }
            s.active -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn pool(capacity: usize) -> Arc<Pool> { Pool::new(2, "a".repeat(32), capacity).unwrap() }
    #[test] fn all_bindings_are_distinct_and_admissible() {
        for node in 1..=3 {
            let p = Pool::new(node, "a".repeat(32), 1).unwrap();
            let bindings = p.snapshot().unwrap();
            assert_eq!(bindings.len(), 27);
            for (i,c) in bindings.iter().enumerate() {
                assert_eq!(c.binding.channel as usize, i+1);
                let mut lease=p.admit(&c.binding, &format!("request-{node}-{i}")).unwrap();
                lease.complete("test fixture, not model-generation evidence").unwrap();
            }
            assert!(p.snapshot().unwrap().iter().all(|c| c.completed == 1 && !c.busy));
        }
    }
    #[test] fn exclusivity_capacity_replay_and_cancellation() {
        let p=pool(1); let b=p.snapshot().unwrap();
        let lease=p.admit(&b[0].binding,"one").unwrap();
        assert!(p.admit(&b[0].binding,"two").is_err());
        assert!(p.admit(&b[1].binding,"two").is_err());
        drop(lease);
        assert!(p.admit(&b[0].binding,"one").is_err());
        assert!(p.admit(&b[1].binding,"two").is_ok());
        assert_eq!(p.snapshot().unwrap()[0].failed,1);
    }
    #[test] fn reject_wrong_epoch_node_channel_protocol_and_empty_output() {
        let p=pool(1); let b=p.snapshot().unwrap()[0].binding.clone();
        for wrong in [
            Binding { epoch:"b".repeat(32),..b.clone() },
            Binding { node:3,..b.clone() },
            Binding { channel:0,..b.clone() },
            Binding { channel:28,..b.clone() },
            Binding { protocol:2,..b.clone() },
        ] {
            assert!(p.admit(&wrong,"request").is_err());
            assert!(!b.matches_response(&wrong));
        }
        let mut lease=p.admit(&b,"request").unwrap();
        assert!(lease.complete(" \n").is_err()); drop(lease);
        assert_eq!(p.snapshot().unwrap()[0].completed,0);
    }
}
