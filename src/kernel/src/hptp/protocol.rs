use alloc::collections::BTreeMap;
use alloc::string::String;
use super::{HptpError, HptpResult};
use crate::timing::FS_PER_NS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HptpPrecisionLevel {
    Millisecond,
    Microsecond,
    Nanosecond,
    Picosecond,
    Femtosecond,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HptpClockSource {
    Local,
    Gpsdo,
    AtomicRubidium,
    AtomicCesium,
    OpticalLattice,
    ChipScale,
    NetworkPeer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HptpMessageType {
    SyncRequest,
    SyncResponse,
    FollowUp,
    DelayRequest,
    DelayResponse,
    Announce,
    Management,
}

#[derive(Debug, Clone)]
pub struct HptpTimestamp {
    pub femtoseconds: u128,
    pub source: HptpClockSource,
    pub precision: HptpPrecisionLevel,
    pub sequence: u64,
}

impl HptpTimestamp {
    pub fn new(femtoseconds: u128, source: HptpClockSource, precision: HptpPrecisionLevel, sequence: u64) -> Self {
        Self { femtoseconds, source, precision, sequence }
    }

    pub fn from_nanoseconds(ns: u64, source: HptpClockSource, sequence: u64) -> Self {
        Self {
            femtoseconds: ns as u128 * FS_PER_NS,
            source,
            precision: HptpPrecisionLevel::Nanosecond,
            sequence,
        }
    }

    pub fn elapsed_since(&self, other: &HptpTimestamp) -> i128 {
        self.femtoseconds as i128 - other.femtoseconds as i128
    }

    pub fn is_after(&self, other: &HptpTimestamp) -> bool {
        self.femtoseconds > other.femtoseconds
    }
}

#[derive(Debug, Clone)]
pub struct HptpSyncMessage {
    pub message_type: HptpMessageType,
    pub timestamp: HptpTimestamp,
    pub sender_id: u64,
    pub receiver_id: u64,
    pub round_trip_delay_fs: u128,
    pub correction_fs: i128,
}

#[derive(Debug, Clone)]
pub struct HptpPeer {
    pub peer_id: u64,
    pub address: String,
    pub clock_source: HptpClockSource,
    pub precision: HptpPrecisionLevel,
    pub offset_fs: i128,
    pub delay_fs: u128,
    pub last_sync_seq: u64,
    pub sync_count: u64,
    pub is_master: bool,
    pub stratum: u8,
}

pub struct HptpSession {
    pub local_id: u64,
    pub local_clock_source: HptpClockSource,
    pub precision: HptpPrecisionLevel,
    pub peers: BTreeMap<u64, HptpPeer>,
    pub current_sequence: u64,
    pub is_master: bool,
    pub stratum: u8,
    pub max_drift_fs: i128,
    pub sync_interval_ns: u64,
}

impl HptpSession {
    pub fn new(local_id: u64, clock_source: HptpClockSource, precision: HptpPrecisionLevel) -> Self {
        Self {
            local_id,
            local_clock_source: clock_source,
            precision,
            peers: BTreeMap::new(),
            current_sequence: 0,
            is_master: false,
            stratum: 15,
            max_drift_fs: 100,
            sync_interval_ns: 1_000_000_000,
        }
    }

    pub fn add_peer(&mut self, peer: HptpPeer) {
        self.peers.insert(peer.peer_id, peer);
    }

    pub fn remove_peer(&mut self, peer_id: u64) {
        self.peers.remove(&peer_id);
    }

    pub fn get_peer(&self, peer_id: u64) -> Option<&HptpPeer> {
        self.peers.get(&peer_id)
    }

    pub fn initiate_sync(&mut self, peer_id: u64, local_timestamp_fs: u128) -> HptpResult<HptpSyncMessage> {
        if !self.peers.contains_key(&peer_id) {
            return Err(HptpError::PeerUnreachable(alloc::format!("peer {} not found", peer_id)));
        }
        self.current_sequence += 1;
        let timestamp = HptpTimestamp::new(
            local_timestamp_fs,
            self.local_clock_source,
            self.precision,
            self.current_sequence,
        );
        Ok(HptpSyncMessage {
            message_type: HptpMessageType::SyncRequest,
            timestamp,
            sender_id: self.local_id,
            receiver_id: peer_id,
            round_trip_delay_fs: 0,
            correction_fs: 0,
        })
    }

    pub fn process_sync_response(&mut self, response: &HptpSyncMessage, local_receive_fs: u128) -> HptpResult<i128> {
        let peer_id = response.sender_id;
        let peer = self.peers.get_mut(&peer_id)
            .ok_or_else(|| HptpError::PeerUnreachable(alloc::format!("peer {} not found", peer_id)))?;

        let t2 = response.timestamp.femtoseconds as i128;
        let t1 = response.correction_fs;
        let t3 = response.round_trip_delay_fs as i128;
        let t4 = local_receive_fs as i128;

        let offset = (t2 - t1 - t4 + t3) / 2;
        peer.offset_fs = offset;
        peer.last_sync_seq = response.timestamp.sequence;
        peer.sync_count += 1;

        Ok(offset)
    }

    pub fn send_delay_request(&mut self, peer_id: u64, timestamp_fs: u128) -> HptpResult<HptpSyncMessage> {
        if !self.peers.contains_key(&peer_id) {
            return Err(HptpError::PeerUnreachable(alloc::format!("peer {} not found", peer_id)));
        }
        self.current_sequence += 1;
        let timestamp = HptpTimestamp::new(
            timestamp_fs,
            self.local_clock_source,
            self.precision,
            self.current_sequence,
        );
        Ok(HptpSyncMessage {
            message_type: HptpMessageType::DelayRequest,
            timestamp,
            sender_id: self.local_id,
            receiver_id: peer_id,
            round_trip_delay_fs: 0,
            correction_fs: 0,
        })
    }

    pub fn process_delay_response(&mut self, response: &HptpSyncMessage, local_receive_fs: u128) -> HptpResult<u128> {
        let peer_id = response.sender_id;
        let peer = self.peers.get_mut(&peer_id)
            .ok_or_else(|| HptpError::PeerUnreachable(alloc::format!("peer {} not found", peer_id)))?;

        let remote_ts = response.timestamp.femtoseconds;
        let delay = if local_receive_fs > remote_ts {
            (local_receive_fs - remote_ts) / 2
        } else {
            (remote_ts - local_receive_fs) / 2
        };
        peer.delay_fs = delay;

        Ok(delay)
    }

    pub fn best_master_clock(&self) -> Option<u64> {
        self.peers.values()
            .filter(|p| p.is_master)
            .min_by(|a, b| {
                a.stratum.cmp(&b.stratum)
                    .then(b.precision.cmp(&a.precision))
            })
            .map(|p| p.peer_id)
    }

    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    pub fn current_offset(&self, peer_id: u64) -> Option<i128> {
        self.peers.get(&peer_id).map(|p| p.offset_fs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_creation() {
        let ts = HptpTimestamp::new(1000, HptpClockSource::Local, HptpPrecisionLevel::Femtosecond, 1);
        assert_eq!(ts.femtoseconds, 1000);
        assert_eq!(ts.source, HptpClockSource::Local);
        assert_eq!(ts.precision, HptpPrecisionLevel::Femtosecond);
        assert_eq!(ts.sequence, 1);
    }

    #[test]
    fn test_timestamp_from_nanoseconds() {
        let ts = HptpTimestamp::from_nanoseconds(1, HptpClockSource::Gpsdo, 1);
        assert_eq!(ts.femtoseconds, FS_PER_NS);
        assert_eq!(ts.precision, HptpPrecisionLevel::Nanosecond);
    }

    #[test]
    fn test_timestamp_from_nanoseconds_large() {
        let ts = HptpTimestamp::from_nanoseconds(1000, HptpClockSource::AtomicCesium, 5);
        assert_eq!(ts.femtoseconds, 1000 * FS_PER_NS);
    }

    #[test]
    fn test_timestamp_elapsed_since_positive() {
        let t1 = HptpTimestamp::new(1000, HptpClockSource::Local, HptpPrecisionLevel::Femtosecond, 1);
        let t2 = HptpTimestamp::new(3000, HptpClockSource::Local, HptpPrecisionLevel::Femtosecond, 2);
        assert_eq!(t2.elapsed_since(&t1), 2000);
    }

    #[test]
    fn test_timestamp_elapsed_since_negative() {
        let t1 = HptpTimestamp::new(3000, HptpClockSource::Local, HptpPrecisionLevel::Femtosecond, 1);
        let t2 = HptpTimestamp::new(1000, HptpClockSource::Local, HptpPrecisionLevel::Femtosecond, 2);
        assert_eq!(t2.elapsed_since(&t1), -2000);
    }

    #[test]
    fn test_timestamp_is_after() {
        let t1 = HptpTimestamp::new(1000, HptpClockSource::Local, HptpPrecisionLevel::Femtosecond, 1);
        let t2 = HptpTimestamp::new(2000, HptpClockSource::Local, HptpPrecisionLevel::Femtosecond, 2);
        assert!(t2.is_after(&t1));
        assert!(!t1.is_after(&t2));
        assert!(!t1.is_after(&t1));
    }

    #[test]
    fn test_session_creation() {
        let session = HptpSession::new(1, HptpClockSource::AtomicCesium, HptpPrecisionLevel::Femtosecond);
        assert_eq!(session.local_id, 1);
        assert_eq!(session.peer_count(), 0);
        assert_eq!(session.stratum, 15);
        assert_eq!(session.max_drift_fs, 100);
        assert_eq!(session.sync_interval_ns, 1_000_000_000);
    }

    #[test]
    fn test_peer_add_and_get() {
        let mut session = HptpSession::new(1, HptpClockSource::Local, HptpPrecisionLevel::Nanosecond);
        let peer = HptpPeer {
            peer_id: 10,
            address: String::from("192.168.1.1"),
            clock_source: HptpClockSource::AtomicRubidium,
            precision: HptpPrecisionLevel::Picosecond,
            offset_fs: 0,
            delay_fs: 0,
            last_sync_seq: 0,
            sync_count: 0,
            is_master: true,
            stratum: 1,
        };
        session.add_peer(peer);
        assert_eq!(session.peer_count(), 1);
        let p = session.get_peer(10).unwrap();
        assert_eq!(p.peer_id, 10);
        assert_eq!(p.stratum, 1);
    }

    #[test]
    fn test_peer_remove() {
        let mut session = HptpSession::new(1, HptpClockSource::Local, HptpPrecisionLevel::Nanosecond);
        let peer = HptpPeer {
            peer_id: 10,
            address: String::from("10.0.0.1"),
            clock_source: HptpClockSource::Gpsdo,
            precision: HptpPrecisionLevel::Microsecond,
            offset_fs: 0,
            delay_fs: 0,
            last_sync_seq: 0,
            sync_count: 0,
            is_master: false,
            stratum: 5,
        };
        session.add_peer(peer);
        assert_eq!(session.peer_count(), 1);
        session.remove_peer(10);
        assert_eq!(session.peer_count(), 0);
        assert!(session.get_peer(10).is_none());
    }

    #[test]
    fn test_initiate_sync() {
        let mut session = HptpSession::new(1, HptpClockSource::AtomicCesium, HptpPrecisionLevel::Femtosecond);
        let peer = HptpPeer {
            peer_id: 2,
            address: String::from("10.0.0.2"),
            clock_source: HptpClockSource::AtomicRubidium,
            precision: HptpPrecisionLevel::Picosecond,
            offset_fs: 0,
            delay_fs: 0,
            last_sync_seq: 0,
            sync_count: 0,
            is_master: true,
            stratum: 1,
        };
        session.add_peer(peer);
        let msg = session.initiate_sync(2, 5000).unwrap();
        assert_eq!(msg.message_type, HptpMessageType::SyncRequest);
        assert_eq!(msg.sender_id, 1);
        assert_eq!(msg.receiver_id, 2);
        assert_eq!(msg.timestamp.femtoseconds, 5000);
        assert_eq!(session.current_sequence, 1);
    }

    #[test]
    fn test_initiate_sync_unknown_peer() {
        let mut session = HptpSession::new(1, HptpClockSource::Local, HptpPrecisionLevel::Nanosecond);
        assert!(session.initiate_sync(99, 1000).is_err());
    }

    #[test]
    fn test_process_sync_response() {
        let mut session = HptpSession::new(1, HptpClockSource::AtomicCesium, HptpPrecisionLevel::Femtosecond);
        let peer = HptpPeer {
            peer_id: 2,
            address: String::from("10.0.0.2"),
            clock_source: HptpClockSource::AtomicRubidium,
            precision: HptpPrecisionLevel::Picosecond,
            offset_fs: 0,
            delay_fs: 0,
            last_sync_seq: 0,
            sync_count: 0,
            is_master: true,
            stratum: 1,
        };
        session.add_peer(peer);

        let response = HptpSyncMessage {
            message_type: HptpMessageType::SyncResponse,
            timestamp: HptpTimestamp::new(1100, HptpClockSource::AtomicRubidium, HptpPrecisionLevel::Picosecond, 1),
            sender_id: 2,
            receiver_id: 1,
            round_trip_delay_fs: 1200,
            correction_fs: 1000,
        };

        let offset = session.process_sync_response(&response, 1300).unwrap();
        let expected = (1100 - 1000 - 1300 + 1200) / 2;
        assert_eq!(offset, expected);
        assert_eq!(session.get_peer(2).unwrap().sync_count, 1);
    }

    #[test]
    fn test_process_sync_response_unknown_peer() {
        let mut session = HptpSession::new(1, HptpClockSource::Local, HptpPrecisionLevel::Nanosecond);
        let response = HptpSyncMessage {
            message_type: HptpMessageType::SyncResponse,
            timestamp: HptpTimestamp::new(100, HptpClockSource::Gpsdo, HptpPrecisionLevel::Nanosecond, 1),
            sender_id: 99,
            receiver_id: 1,
            round_trip_delay_fs: 0,
            correction_fs: 0,
        };
        assert!(session.process_sync_response(&response, 200).is_err());
    }

    #[test]
    fn test_send_delay_request() {
        let mut session = HptpSession::new(1, HptpClockSource::Local, HptpPrecisionLevel::Nanosecond);
        let peer = HptpPeer {
            peer_id: 3,
            address: String::from("10.0.0.3"),
            clock_source: HptpClockSource::ChipScale,
            precision: HptpPrecisionLevel::Microsecond,
            offset_fs: 0,
            delay_fs: 0,
            last_sync_seq: 0,
            sync_count: 0,
            is_master: false,
            stratum: 4,
        };
        session.add_peer(peer);
        let msg = session.send_delay_request(3, 7000).unwrap();
        assert_eq!(msg.message_type, HptpMessageType::DelayRequest);
        assert_eq!(msg.timestamp.femtoseconds, 7000);
    }

    #[test]
    fn test_process_delay_response() {
        let mut session = HptpSession::new(1, HptpClockSource::Local, HptpPrecisionLevel::Nanosecond);
        let peer = HptpPeer {
            peer_id: 3,
            address: String::from("10.0.0.3"),
            clock_source: HptpClockSource::ChipScale,
            precision: HptpPrecisionLevel::Microsecond,
            offset_fs: 0,
            delay_fs: 0,
            last_sync_seq: 0,
            sync_count: 0,
            is_master: false,
            stratum: 4,
        };
        session.add_peer(peer);

        let response = HptpSyncMessage {
            message_type: HptpMessageType::DelayResponse,
            timestamp: HptpTimestamp::new(8000, HptpClockSource::ChipScale, HptpPrecisionLevel::Microsecond, 1),
            sender_id: 3,
            receiver_id: 1,
            round_trip_delay_fs: 0,
            correction_fs: 0,
        };

        let delay = session.process_delay_response(&response, 10000).unwrap();
        assert_eq!(delay, (10000 - 8000) / 2);
        assert_eq!(session.get_peer(3).unwrap().delay_fs, delay);
    }

    #[test]
    fn test_best_master_clock_single() {
        let mut session = HptpSession::new(1, HptpClockSource::Local, HptpPrecisionLevel::Nanosecond);
        let peer = HptpPeer {
            peer_id: 10,
            address: String::from("10.0.0.10"),
            clock_source: HptpClockSource::AtomicCesium,
            precision: HptpPrecisionLevel::Femtosecond,
            offset_fs: 0,
            delay_fs: 0,
            last_sync_seq: 0,
            sync_count: 0,
            is_master: true,
            stratum: 1,
        };
        session.add_peer(peer);
        assert_eq!(session.best_master_clock(), Some(10));
    }

    #[test]
    fn test_best_master_clock_stratum_ordering() {
        let mut session = HptpSession::new(1, HptpClockSource::Local, HptpPrecisionLevel::Nanosecond);
        let peer_a = HptpPeer {
            peer_id: 10,
            address: String::from("10.0.0.10"),
            clock_source: HptpClockSource::AtomicRubidium,
            precision: HptpPrecisionLevel::Picosecond,
            offset_fs: 0,
            delay_fs: 0,
            last_sync_seq: 0,
            sync_count: 0,
            is_master: true,
            stratum: 3,
        };
        let peer_b = HptpPeer {
            peer_id: 20,
            address: String::from("10.0.0.20"),
            clock_source: HptpClockSource::AtomicCesium,
            precision: HptpPrecisionLevel::Femtosecond,
            offset_fs: 0,
            delay_fs: 0,
            last_sync_seq: 0,
            sync_count: 0,
            is_master: true,
            stratum: 1,
        };
        session.add_peer(peer_a);
        session.add_peer(peer_b);
        assert_eq!(session.best_master_clock(), Some(20));
    }

    #[test]
    fn test_best_master_clock_no_masters() {
        let mut session = HptpSession::new(1, HptpClockSource::Local, HptpPrecisionLevel::Nanosecond);
        let peer = HptpPeer {
            peer_id: 10,
            address: String::from("10.0.0.10"),
            clock_source: HptpClockSource::Gpsdo,
            precision: HptpPrecisionLevel::Nanosecond,
            offset_fs: 0,
            delay_fs: 0,
            last_sync_seq: 0,
            sync_count: 0,
            is_master: false,
            stratum: 5,
        };
        session.add_peer(peer);
        assert_eq!(session.best_master_clock(), None);
    }

    #[test]
    fn test_current_offset() {
        let mut session = HptpSession::new(1, HptpClockSource::Local, HptpPrecisionLevel::Nanosecond);
        let peer = HptpPeer {
            peer_id: 5,
            address: String::from("10.0.0.5"),
            clock_source: HptpClockSource::Gpsdo,
            precision: HptpPrecisionLevel::Nanosecond,
            offset_fs: -42,
            delay_fs: 0,
            last_sync_seq: 0,
            sync_count: 0,
            is_master: false,
            stratum: 3,
        };
        session.add_peer(peer);
        assert_eq!(session.current_offset(5), Some(-42));
        assert_eq!(session.current_offset(99), None);
    }

    #[test]
    fn test_precision_level_ordering() {
        assert!(HptpPrecisionLevel::Femtosecond > HptpPrecisionLevel::Picosecond);
        assert!(HptpPrecisionLevel::Picosecond > HptpPrecisionLevel::Nanosecond);
        assert!(HptpPrecisionLevel::Nanosecond > HptpPrecisionLevel::Microsecond);
        assert!(HptpPrecisionLevel::Microsecond > HptpPrecisionLevel::Millisecond);
    }

    #[test]
    fn test_sequence_increments() {
        let mut session = HptpSession::new(1, HptpClockSource::Local, HptpPrecisionLevel::Nanosecond);
        let peer = HptpPeer {
            peer_id: 2,
            address: String::from("10.0.0.2"),
            clock_source: HptpClockSource::Gpsdo,
            precision: HptpPrecisionLevel::Nanosecond,
            offset_fs: 0,
            delay_fs: 0,
            last_sync_seq: 0,
            sync_count: 0,
            is_master: false,
            stratum: 5,
        };
        session.add_peer(peer);
        session.initiate_sync(2, 100).unwrap();
        assert_eq!(session.current_sequence, 1);
        session.initiate_sync(2, 200).unwrap();
        assert_eq!(session.current_sequence, 2);
        session.send_delay_request(2, 300).unwrap();
        assert_eq!(session.current_sequence, 3);
    }
}
