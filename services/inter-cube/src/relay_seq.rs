// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// RELAY SEQUENCING & DEDUP — Task #27, Task 5
//
// Sequence-numbered delivery with:
// - Monotonic seq on every relay envelope
// - lastSeenSeq + topicEpoch on reconnect for gap detection
// - Bitmap resync (hard-capped at 8KB)
// - Global tombstones with golden angle suggestedResyncAfterMs
// - Per-connection resync rate limit (max 3/min)
// - RelaySequenceStore: persistent dedup state with TLSponge-385 integrity
//
// Sponge Context String Registry (this module):
// | Context String                | Usage                      | Module        |
// |-------------------------------|----------------------------|---------------|
// | "PlenumNET-DEDUP-STATE-v1"    | Dedup state integrity hash | relay_seq.rs  |

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use ternary_math::sponge::hash_hex;

use crate::relay_server::golden_angle_delay;

// ═══════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════

/// Maximum bitmap resync payload size (8KB hard cap).
pub const MAX_RESYNC_BITMAP_SIZE: usize = 8_192;

/// Resync rate limit: max 3 per minute per connection.
pub const RESYNC_RATE_LIMIT: usize = 3;

/// Resync rate window: 60 seconds.
pub const RESYNC_RATE_WINDOW_SECS: u64 = 60;

/// Context string for dedup state integrity hash.
const DEDUP_CONTEXT: &str = "PlenumNET-DEDUP-STATE-v1";

/// Version byte for dedup state file format.
const DEDUP_FORMAT_VERSION: u8 = 1;

/// Truncated hash length for integrity (16 bytes = 128 bits).
const INTEGRITY_HASH_LEN: usize = 16;

// ═══════════════════════════════════════════════════════════════════════
// TOMBSTONE
// ═══════════════════════════════════════════════════════════════════════

/// Global tombstone — generated only on global per-client queue eviction.
/// NOT from per-topic backpressure, reauthorization failures, or topic GC.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Tombstone {
    #[serde(rename = "type")]
    pub msg_type: String,
    #[serde(rename = "resyncCount")]
    pub resync_count: u64,
    #[serde(rename = "suggestedResyncAfterMs")]
    pub suggested_resync_after_ms: u64,
    #[serde(rename = "topicSeqs")]
    pub topic_seqs: HashMap<String, TopicSeqSnapshot>,
    #[serde(rename = "gapSizeEstimate")]
    pub gap_size_estimate: u64,
}

/// Per-topic sequence snapshot within a tombstone.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TopicSeqSnapshot {
    pub seq: u64,
    #[serde(rename = "topicEpoch")]
    pub epoch: u64,
}

/// Generate a tombstone for a client whose global queue overflowed.
///
/// `disconnect_index` is the server-global monotonic counter for
/// golden angle stagger (φ_ternary = 182(3−√5)/364).
pub fn generate_tombstone(
    resync_count: u64,
    disconnect_index: u64,
    topic_seqs: HashMap<String, TopicSeqSnapshot>,
    gap_estimate: u64,
) -> Tombstone {
    Tombstone {
        msg_type: "tombstone".to_string(),
        resync_count,
        suggested_resync_after_ms: golden_angle_delay(disconnect_index, 2000, 5000),
        topic_seqs,
        gap_size_estimate: gap_estimate,
    }
}

// ═══════════════════════════════════════════════════════════════════════
// RESYNC RATE LIMITER
// ═══════════════════════════════════════════════════════════════════════

/// Per-connection resync rate limiter (sliding window, max 3/min).
#[derive(Debug)]
pub struct ResyncRateLimiter {
    /// Timestamps of recent resync requests per connection.
    windows: HashMap<String, Vec<Instant>>,
    /// Max attempts per window.
    max_per_window: usize,
    /// Window duration.
    window_duration: std::time::Duration,
}

impl ResyncRateLimiter {
    pub fn new() -> Self {
        ResyncRateLimiter {
            windows: HashMap::new(),
            max_per_window: RESYNC_RATE_LIMIT,
            window_duration: std::time::Duration::from_secs(RESYNC_RATE_WINDOW_SECS),
        }
    }

    /// Check if a resync is allowed for this connection.
    /// Returns true if allowed, false if rate-limited.
    pub fn check_and_record(&mut self, address: &str) -> bool {
        let now = Instant::now();
        let window = self.windows.entry(address.to_string()).or_insert_with(Vec::new);

        // Evict expired entries
        window.retain(|ts| now.duration_since(*ts) < self.window_duration);

        if window.len() >= self.max_per_window {
            false
        } else {
            window.push(now);
            true
        }
    }

    /// Remove a connection's rate limit state (on disconnect).
    pub fn remove(&mut self, address: &str) {
        self.windows.remove(address);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// RELAY SEQUENCE STORE — Persistent dedup state
// ═══════════════════════════════════════════════════════════════════════

/// Per-topic dedup state entry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TopicDedup {
    pub last_processed_seq: u64,
    pub topic_epoch: u64,
}

/// Persistent dedup state for the relay client.
///
/// Separate from the heartbeat SequenceStore in persistence.rs.
/// File format: version_byte(1) || bincode_payload(N) || truncated_hash(16)
///
/// Hash: TLSponge-385(DEDUP_CONTEXT || version_byte || bincode_payload) → first 16 bytes
/// On corruption → fresh start + warning + full resync.
#[derive(Debug)]
pub struct RelaySequenceStore {
    /// Per-topic dedup state.
    topics: HashMap<String, TopicDedup>,
    /// File path (empty = in-memory only).
    file_path: PathBuf,
    /// Dirty flag for lazy flushing.
    dirty: bool,
}

impl RelaySequenceStore {
    /// Create a new store backed by the given file.
    pub fn new(file_path: impl Into<PathBuf>) -> Self {
        RelaySequenceStore {
            topics: HashMap::new(),
            file_path: file_path.into(),
            dirty: false,
        }
    }

    /// Create an in-memory-only store (for testing).
    pub fn in_memory() -> Self {
        RelaySequenceStore {
            topics: HashMap::new(),
            file_path: PathBuf::new(),
            dirty: false,
        }
    }

    /// Load persisted state from disk. Verifies TLSponge-385 integrity hash.
    /// On corruption or missing file → returns 0 (fresh start).
    pub fn load(&mut self) -> usize {
        if self.file_path.as_os_str().is_empty() || !self.file_path.exists() {
            return 0;
        }

        let data = match fs::read(&self.file_path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("[relay-dedup] Failed to read dedup state: {}", e);
                return 0;
            }
        };

        // Minimum: version(1) + hash(16) = 17 bytes
        if data.len() < 1 + INTEGRITY_HASH_LEN {
            eprintln!("[relay-dedup] Dedup state too short ({} bytes), fresh start", data.len());
            return 0;
        }

        let version = data[0];
        if version != DEDUP_FORMAT_VERSION {
            eprintln!("[relay-dedup] Unknown dedup format version {}, fresh start", version);
            return 0;
        }

        let payload_end = data.len() - INTEGRITY_HASH_LEN;
        let payload = &data[1..payload_end];
        let stored_hash = &data[payload_end..];

        // Verify integrity: TLSponge-385(context || version || payload) → first 16 bytes
        let mut hash_input = Vec::with_capacity(DEDUP_CONTEXT.len() + 1 + payload.len());
        hash_input.extend_from_slice(DEDUP_CONTEXT.as_bytes());
        hash_input.push(version);
        hash_input.extend_from_slice(payload);
        let full_hash = hash_hex(&hash_input);

        // Truncate hex hash to 16 bytes (32 hex chars → 16 bytes)
        let expected_hash = hex_to_bytes(&full_hash[..32.min(full_hash.len())]);
        if expected_hash.len() < INTEGRITY_HASH_LEN || stored_hash != &expected_hash[..INTEGRITY_HASH_LEN] {
            eprintln!("[relay-dedup] Integrity hash mismatch, fresh start + full resync");
            self.topics.clear();
            self.dirty = false;
            return 0;
        }

        // Deserialize bincode payload
        match bincode::deserialize::<HashMap<String, TopicDedup>>(payload) {
            Ok(topics) => {
                let count = topics.len();
                self.topics = topics;
                self.dirty = false;
                println!("[relay-dedup] Loaded {} topic dedup entries", count);
                count
            }
            Err(e) => {
                eprintln!("[relay-dedup] Bincode decode failed: {}, fresh start", e);
                self.topics.clear();
                self.dirty = false;
                0
            }
        }
    }

    /// Flush state to disk with atomic write (.tmp + rename) and TLSponge-385 integrity.
    pub fn flush(&mut self) -> Result<bool, std::io::Error> {
        if self.file_path.as_os_str().is_empty() {
            return Ok(false); // in-memory
        }
        if !self.dirty {
            return Ok(false); // nothing changed
        }

        let payload = bincode::serialize(&self.topics)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        // Compute integrity hash
        let mut hash_input = Vec::with_capacity(DEDUP_CONTEXT.len() + 1 + payload.len());
        hash_input.extend_from_slice(DEDUP_CONTEXT.as_bytes());
        hash_input.push(DEDUP_FORMAT_VERSION);
        hash_input.extend_from_slice(&payload);
        let full_hash = hash_hex(&hash_input);
        let hash_bytes = hex_to_bytes(&full_hash[..32.min(full_hash.len())]);

        // Build file: version || payload || hash(16)
        let mut file_data = Vec::with_capacity(1 + payload.len() + INTEGRITY_HASH_LEN);
        file_data.push(DEDUP_FORMAT_VERSION);
        file_data.extend_from_slice(&payload);
        if hash_bytes.len() >= INTEGRITY_HASH_LEN {
            file_data.extend_from_slice(&hash_bytes[..INTEGRITY_HASH_LEN]);
        } else {
            // Pad if hash is short (shouldn't happen)
            file_data.extend_from_slice(&hash_bytes);
            file_data.resize(1 + payload.len() + INTEGRITY_HASH_LEN, 0);
        }

        // Atomic write: .tmp + rename
        let tmp_path = self.file_path.with_extension("dedup.tmp");
        {
            let mut f = fs::File::create(&tmp_path)?;
            f.write_all(&file_data)?;
            f.sync_all()?;
        }
        fs::rename(&tmp_path, &self.file_path)?;
        self.dirty = false;
        Ok(true)
    }

    /// Update dedup state for a topic.
    pub fn update(&mut self, topic: &str, seq: u64, epoch: u64) {
        let entry = self.topics.entry(topic.to_string()).or_insert(TopicDedup {
            last_processed_seq: 0,
            topic_epoch: epoch,
        });

        // Only advance forward (monotonic)
        if epoch != entry.topic_epoch {
            // Epoch changed — reset sequence
            entry.topic_epoch = epoch;
            entry.last_processed_seq = seq;
            self.dirty = true;
        } else if seq > entry.last_processed_seq {
            entry.last_processed_seq = seq;
            self.dirty = true;
        }
    }

    /// Check if a message is a duplicate (already processed).
    pub fn is_duplicate(&self, topic: &str, seq: u64, epoch: u64) -> bool {
        if let Some(entry) = self.topics.get(topic) {
            if epoch == entry.topic_epoch {
                seq <= entry.last_processed_seq
            } else {
                false // Different epoch — not a duplicate
            }
        } else {
            false // No entry — never seen
        }
    }

    /// Reset dedup state for a topic (on topic_reset from server).
    pub fn reset_topic(&mut self, topic: &str, new_epoch: u64) {
        self.topics.insert(topic.to_string(), TopicDedup {
            last_processed_seq: 0,
            topic_epoch: new_epoch,
        });
        self.dirty = true;
    }

    /// Remove a topic from dedup state (on topic_revoked).
    pub fn remove_topic(&mut self, topic: &str) {
        if self.topics.remove(topic).is_some() {
            self.dirty = true;
        }
    }

    /// Apply tombstone topicSeqs snapshot.
    /// Topics present in snapshot: reset to snapshot values.
    /// Topics absent from snapshot: client's persisted seq is authoritative.
    pub fn apply_tombstone(&mut self, topic_seqs: &HashMap<String, TopicSeqSnapshot>) {
        for (topic, snap) in topic_seqs {
            self.topics.insert(topic.clone(), TopicDedup {
                last_processed_seq: snap.seq,
                topic_epoch: snap.epoch,
            });
        }
        self.dirty = true;
    }

    /// Get the last processed seq for a topic (for lastSeenSeq on reconnect).
    pub fn get(&self, topic: &str) -> Option<&TopicDedup> {
        self.topics.get(topic)
    }

    /// Get all topic dedup state (for resync requests).
    pub fn all_topics(&self) -> &HashMap<String, TopicDedup> {
        &self.topics
    }

    pub fn is_dirty(&self) -> bool { self.dirty }
    pub fn topic_count(&self) -> usize { self.topics.len() }
}

/// Convert hex string to bytes.
fn hex_to_bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .filter_map(|i| {
            if i + 2 <= hex.len() {
                u8::from_str_radix(&hex[i..i + 2], 16).ok()
            } else {
                None
            }
        })
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── Tombstone ───────────────────────────────────────────────

    #[test]
    fn test_tombstone_golden_angle() {
        let t = generate_tombstone(1, 0, HashMap::new(), 42);
        assert_eq!(t.msg_type, "tombstone");
        assert_eq!(t.resync_count, 1);
        assert!(t.suggested_resync_after_ms >= 2000);
        assert!(t.suggested_resync_after_ms < 7000);
        assert_eq!(t.gap_size_estimate, 42);
    }

    #[test]
    fn test_tombstone_stagger_spread() {
        let delays: Vec<u64> = (0..50).map(|i| {
            generate_tombstone(1, i, HashMap::new(), 0).suggested_resync_after_ms
        }).collect();
        let unique: std::collections::HashSet<u64> = delays.iter().cloned().collect();
        assert!(unique.len() > 40, "Golden angle should produce mostly unique delays");
    }

    // ── Resync rate limiter ─────────────────────────────────────

    #[test]
    fn test_rate_limiter_allows_up_to_max() {
        let mut rl = ResyncRateLimiter::new();
        assert!(rl.check_and_record("addr"));
        assert!(rl.check_and_record("addr"));
        assert!(rl.check_and_record("addr"));
        assert!(!rl.check_and_record("addr")); // 4th should fail
    }

    #[test]
    fn test_rate_limiter_per_connection() {
        let mut rl = ResyncRateLimiter::new();
        for _ in 0..3 { rl.check_and_record("a"); }
        // "b" should still have its own window
        assert!(rl.check_and_record("b"));
    }

    // ── RelaySequenceStore ──────────────────────────────────────

    #[test]
    fn test_dedup_basic() {
        let mut store = RelaySequenceStore::in_memory();
        store.update("topic-a", 5, 1000);
        assert!(!store.is_duplicate("topic-a", 6, 1000)); // seq 6 > 5
        assert!(store.is_duplicate("topic-a", 5, 1000));  // seq 5 = 5
        assert!(store.is_duplicate("topic-a", 3, 1000));  // seq 3 < 5
    }

    #[test]
    fn test_dedup_epoch_change() {
        let mut store = RelaySequenceStore::in_memory();
        store.update("topic-a", 50, 1000);
        // Same seq but different epoch → not duplicate
        assert!(!store.is_duplicate("topic-a", 50, 2000));
    }

    #[test]
    fn test_dedup_monotonic() {
        let mut store = RelaySequenceStore::in_memory();
        store.update("topic-a", 10, 1000);
        store.update("topic-a", 5, 1000); // lower seq, same epoch — ignored
        assert_eq!(store.get("topic-a").unwrap().last_processed_seq, 10);
    }

    #[test]
    fn test_reset_topic() {
        let mut store = RelaySequenceStore::in_memory();
        store.update("data", 50, 1000);
        store.reset_topic("data", 2000);
        assert_eq!(store.get("data").unwrap().last_processed_seq, 0);
        assert_eq!(store.get("data").unwrap().topic_epoch, 2000);
    }

    #[test]
    fn test_remove_topic() {
        let mut store = RelaySequenceStore::in_memory();
        store.update("data", 10, 1000);
        store.remove_topic("data");
        assert!(store.get("data").is_none());
    }

    #[test]
    fn test_apply_tombstone() {
        let mut store = RelaySequenceStore::in_memory();
        store.update("a", 10, 100);
        store.update("b", 20, 200);
        store.update("c", 30, 300);

        let mut snapshot = HashMap::new();
        snapshot.insert("a".to_string(), TopicSeqSnapshot { seq: 5, epoch: 100 });
        // "b" absent from snapshot — client's persisted seq is authoritative
        snapshot.insert("c".to_string(), TopicSeqSnapshot { seq: 0, epoch: 400 });

        store.apply_tombstone(&snapshot);

        assert_eq!(store.get("a").unwrap().last_processed_seq, 5);
        assert_eq!(store.get("b").unwrap().last_processed_seq, 20); // unchanged
        assert_eq!(store.get("c").unwrap().topic_epoch, 400); // new epoch
    }

    #[test]
    fn test_persistence_roundtrip() {
        let dir = std::env::temp_dir().join("plenum_dedup_test");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("test-dedup.state");
        let _ = fs::remove_file(&path);

        // Write
        {
            let mut store = RelaySequenceStore::new(path.clone());
            store.update("topic-a", 42, 1000);
            store.update("topic-b", 99, 2000);
            store.flush().unwrap();
        }

        // Read back
        {
            let mut store = RelaySequenceStore::new(path.clone());
            let loaded = store.load();
            assert_eq!(loaded, 2);
            assert_eq!(store.get("topic-a").unwrap().last_processed_seq, 42);
            assert_eq!(store.get("topic-b").unwrap().topic_epoch, 2000);
        }

        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn test_persistence_corruption_detection() {
        let dir = std::env::temp_dir().join("plenum_dedup_corrupt_test");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("test-corrupt.state");
        let _ = fs::remove_file(&path);

        // Write valid state
        {
            let mut store = RelaySequenceStore::new(path.clone());
            store.update("topic-a", 42, 1000);
            store.flush().unwrap();
        }

        // Corrupt the file
        {
            let mut data = fs::read(&path).unwrap();
            if data.len() > 5 {
                data[3] ^= 0xFF; // flip a byte
            }
            fs::write(&path, data).unwrap();
        }

        // Load should detect corruption → fresh start
        {
            let mut store = RelaySequenceStore::new(path.clone());
            let loaded = store.load();
            assert_eq!(loaded, 0, "Corruption must trigger fresh start");
            assert_eq!(store.topic_count(), 0);
        }

        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn test_not_dirty_skips_flush() {
        let mut store = RelaySequenceStore::in_memory();
        assert!(!store.is_dirty());
        assert_eq!(store.flush().unwrap(), false);
    }
}
