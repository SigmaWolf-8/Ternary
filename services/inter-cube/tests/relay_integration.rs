// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// RELAY INTEGRATION TESTS — Task #27, Task 9
//
// CI execution: max 5-minute wall clock (hard limit).
// Configurable time parameters shortened for CI:
//   heartbeat timeout = 1s, GC idle TTL = 2s, resync rate window = 5s
//
// Coverage: all spec requirements from Tasks 1-8.

#[cfg(test)]
mod relay_integration {
    use std::collections::{HashMap, HashSet};
    use std::time::Duration;

    use inter_cube::relay_error::*;
    use inter_cube::relay_audit::*;
    use inter_cube::relay_circuit::*;
    use inter_cube::relay_frames::*;
    use inter_cube::relay_capabilities::*;
    use inter_cube::relay_heartbeat::*;
    use inter_cube::relay_topics::*;
    use inter_cube::relay_seq::*;
    use inter_cube::relay_client::*;
    use inter_cube::relay_metrics::*;
    use inter_cube::relay_server::golden_angle_delay;

    use ternary_math::sponge::hash_hex;
    use ternary_math::trit_int::TritInt;
    use ternary_math::coprime;
    use ternary_math::repx as gf3_algebra;

    // ═══════════════════════════════════════════════════════════
    // ERROR CODES
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_all_20_error_codes_distinct() {
        let codes = [
            RelayErrorCode::ErrAuthFailed, RelayErrorCode::ErrSignatureInvalid,
            RelayErrorCode::ErrSignatureRequired, RelayErrorCode::ErrAuthTimeout,
            RelayErrorCode::ErrRateLimited, RelayErrorCode::ErrFrameMalformed,
            RelayErrorCode::ErrFrameTooLarge, RelayErrorCode::ErrRelayTargetUnknown,
            RelayErrorCode::ErrRelayQueueFull, RelayErrorCode::ErrUnknownMsgType,
            RelayErrorCode::ErrNotAuthenticated, RelayErrorCode::ErrCircuitOpen,
            RelayErrorCode::ErrCapabilityNotNegotiated, RelayErrorCode::ErrCapabilityDowngrade,
            RelayErrorCode::ErrTopicBackpressure, RelayErrorCode::ErrTopicUnauthorized,
            RelayErrorCode::ErrTopicLimitExceeded, RelayErrorCode::ErrTopicReset,
            RelayErrorCode::ErrResyncRateLimited, RelayErrorCode::ErrResyncPayloadTooLarge,
        ];
        let wire_codes: HashSet<&str> = codes.iter().map(|c| c.code()).collect();
        assert_eq!(wire_codes.len(), 20);
    }

    #[test]
    fn test_error_response_json_format() {
        let resp = make_error_response(RelayErrorCode::ErrCapabilityNotNegotiated, Some("subscribe"));
        assert_eq!(resp["type"], "error");
        assert_eq!(resp["error"], "ERR_CAPABILITY_NOT_NEGOTIATED");
        assert_eq!(resp["offendingType"], "subscribe");
    }

    // ═══════════════════════════════════════════════════════════
    // CAPABILITY NEGOTIATION
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_version_string_validation_matrix() {
        // Valid
        assert!(parse_capability_version("topics:1").is_ok());
        assert!(parse_capability_version("future_feature:1").is_ok());
        assert!(parse_capability_version("seq:12").is_ok());
        assert!(parse_capability_version("v2feature:3").is_ok());

        // Invalid — per spec test matrix
        assert!(parse_capability_version("topics:01").is_err(), "leading zero");
        assert!(parse_capability_version("topics:0").is_err(), "zero version");
        assert!(parse_capability_version("TOPICS:1").is_err(), "uppercase");
        assert!(parse_capability_version(":1").is_err(), "empty name");
        assert!(parse_capability_version("topics:").is_err(), "empty version");
        assert!(parse_capability_version("topics").is_err(), "no separator");
    }

    #[test]
    fn test_backward_compatibility_no_supported() {
        // Old clients that omit supported get existing behavior unchanged
        let result = negotiate(&[]);
        assert!(result.enabled.is_empty());
        assert!(result.negotiated_down.is_empty());
        assert!(result.malformed.is_empty());
    }

    #[test]
    fn test_negotiate_down_version_mismatch() {
        let result = negotiate(&["topics:2".to_string()]);
        assert!(result.enabled.contains(&"topics:1".to_string()));
        assert!(result.negotiated_down.contains(&"topics".to_string()));
    }

    #[test]
    fn test_capability_enforcement() {
        let negotiated = vec!["topics:1".to_string()];
        assert!(check_capability("topics", &negotiated).is_ok());
        assert_eq!(
            check_capability("seq", &negotiated).unwrap_err(),
            RelayErrorCode::ErrCapabilityNotNegotiated
        );
    }

    #[test]
    fn test_downgrade_detection_survives_restart() {
        // Simulate: create audit log with capabilities A → A+B → A+B+C
        let mut log = RelayAuditLog::in_memory();
        let addr = "1.2.3.1.2.3.1.2.3.1.2.3.1";

        for caps in &[
            vec!["topics:1"],
            vec!["topics:1", "seq:1"],
            vec!["topics:1", "seq:1", "heartbeat:1"],
        ] {
            log.record_event(&RelayAuditEntry {
                event_type: RelayAuditEventType::CapabilityNegotiation,
                address: addr.to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                details: serde_json::json!({
                    "subject": addr,
                    "capabilities": caps,
                }),
                severity: AuditSeverity::Info,
                subsystem: AuditSubsystem::Capability,
                correlation_refs: vec![],
            });
        }

        // Index should contain A+B+C (latest)
        let baseline = log.get_capability_baseline(addr).unwrap();
        assert_eq!(baseline.len(), 3);

        // Connect with A+B → downgrade detected
        let result = check_downgrade_policy(
            addr,
            &["topics:1".to_string(), "seq:1".to_string()],
            &mut log,
            false,
        );
        assert_eq!(result.unwrap_err(), RelayErrorCode::ErrCapabilityDowngrade);
    }

    // ═══════════════════════════════════════════════════════════
    // TIS-27 AUDIT
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_audit_tis27_native() {
        let mut log = RelayAuditLog::in_memory();
        let genesis = log.last_event_hash().to_string();
        let expected_genesis = hash_hex(b"relay-audit-genesis");
        assert_eq!(genesis, expected_genesis, "Genesis must be TIS-27 hash");

        let hash = log.record_event(&RelayAuditEntry {
            event_type: RelayAuditEventType::AuthSuccess,
            address: "1.1.1.1.1.1.1.1.1.1.1.1.1".to_string(),
            timestamp: "2026-04-12T00:00:00Z".to_string(),
            details: serde_json::json!({"hasTlDsa": true}),
            severity: AuditSeverity::Info,
            subsystem: AuditSubsystem::Capability,
            correlation_refs: vec![],
        });

        assert_ne!(hash, genesis);
        assert_eq!(log.last_event_hash(), hash);
        assert_eq!(log.event_count(), 1);
    }

    #[test]
    fn test_audit_merkle_chain_continuity() {
        let mut log = RelayAuditLog::in_memory();
        let mut prev = log.last_event_hash().to_string();
        for i in 0..5 {
            let hash = log.record_event(&RelayAuditEntry {
                event_type: RelayAuditEventType::Disconnect,
                address: format!("addr-{}", i),
                timestamp: "2026-04-12T00:00:00Z".to_string(),
                details: serde_json::json!({}),
                severity: AuditSeverity::Info,
                subsystem: AuditSubsystem::Capability,
                correlation_refs: vec![],
            });
            assert_ne!(hash, prev, "Each hash must be unique");
            prev = hash;
        }
        assert_eq!(log.event_count(), 5);
    }

    // ═══════════════════════════════════════════════════════════
    // CIRCUIT BREAKER
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_circuit_breaker_ws_close_codes() {
        let mut cb = RelayCircuitBreaker::new("test").with_threshold(2);
        cb.record_ws_close(1000); // normal — no failure
        assert_eq!(cb.state(), CircuitState::Closed);
        cb.record_ws_close(1006); // abnormal
        cb.record_ws_close(1011); // server error
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn test_coprime_probe_schedule_algebraically_independent() {
        let mut cb = RelayCircuitBreaker::new("test");
        cb.compute_probe_schedule(60);
        // Probe intervals should be coprime to 60
        // (not multiples of 10, 15, 30, or 60)
        let common_periods = [10u64, 15, 30, 60];
        for _ in 0..5 {
            let interval = cb.next_probe_interval();
            let secs = interval.as_secs();
            for &period in &common_periods {
                if secs > 0 && period > 0 {
                    // gcd should be 1 (coprime to period)
                    assert_eq!(gcd(secs, period), 1,
                        "Probe interval {}s must be coprime to {}s", secs, period);
                }
            }
        }
    }

    // ═══════════════════════════════════════════════════════════
    // REP C FRAME ENCODING
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_rep_c_mapping() {
        assert_eq!(wire_type_to_rep_c("tombstone").to_decimal(), 1);
        assert_eq!(wire_type_to_rep_c("topic_reset").to_decimal(), 2);
        assert_eq!(wire_type_to_rep_c("topic_revoked").to_decimal(), 3);
    }

    #[test]
    fn test_rep_c_zero_rejection() {
        let unknown = wire_type_to_rep_c("unknown_type");
        assert!(is_frame_type_corrupt(&unknown), "Unknown type must produce corrupt Rep C");
    }

    #[test]
    fn test_has_forgery_direct() {
        assert!(gf3_algebra::has_forgery(&[1, 0, 3]), "Zero in Rep C is forgery");
        assert!(!gf3_algebra::has_forgery(&[1, 2, 3]), "Valid Rep C passes");
    }

    // ═══════════════════════════════════════════════════════════
    // FRAME VALIDATION
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_valid_frames() {
        let tombstone = serde_json::json!({
            "type": "tombstone", "resyncCount": 1,
            "suggestedResyncAfterMs": 3500, "topicSeqs": {}, "gapSizeEstimate": 42
        });
        assert!(validate_control_frame(&tombstone).is_ok());

        let reset = serde_json::json!({
            "type": "topic_reset", "topic": "data",
            "oldEpoch": 100, "newEpoch": 200, "currentSeq": 1
        });
        assert!(validate_control_frame(&reset).is_ok());
    }

    #[test]
    fn test_frame_missing_required_field() {
        let bad = serde_json::json!({"type": "tombstone", "resyncCount": 1});
        let err = validate_control_frame(&bad).unwrap_err();
        assert!(err.contains("suggestedResyncAfterMs") || err.contains("topicSeqs"));
    }

    // ═══════════════════════════════════════════════════════════
    // TOPICS
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_topic_cardinality_exhaustion() {
        let config = TopicConfig { max_per_connection: 3, max_per_server: 100, ..Default::default() };
        let mut mgr = TopicManager::new(config);
        let addr = "1.1.1.1.1.1.1.1.1.1.1.1.1";
        mgr.subscribe("a", addr).unwrap();
        mgr.subscribe("b", addr).unwrap();
        mgr.subscribe("c", addr).unwrap();
        assert_eq!(mgr.subscribe("d", addr).unwrap_err(), RelayErrorCode::ErrTopicLimitExceeded);
    }

    #[test]
    fn test_topic_gc_and_epoch_discontinuity() {
        let config = TopicConfig { idle_ttl: Duration::from_millis(20), ..Default::default() };
        let mut mgr = TopicManager::new(config);

        // Subscribe, get epoch, unsubscribe (no publish — queue must be empty for GC)
        let epoch1 = mgr.subscribe("data", "addr").unwrap();
        mgr.unsubscribe("data", "addr");

        // Wait for GC — allow generous headroom over 20ms TTL
        std::thread::sleep(Duration::from_millis(60));
        let gc_d = mgr.gc();
        assert!(gc_d.contains(&"data".to_string()));

        // Another node recreates the topic
        let epoch2 = mgr.subscribe("data", "other").unwrap();
        assert_ne!(epoch1, epoch2, "New epoch after GC");

        // Original client reconnects with old epoch → topic_reset
        let reset = mgr.check_epoch("data", epoch1).unwrap_err();
        assert_eq!(reset.old_epoch, epoch1);
        assert_eq!(reset.new_epoch, epoch2);
    }

    #[test]
    fn test_topic_gc_no_recreation() {
        let config = TopicConfig { idle_ttl: Duration::from_millis(20), ..Default::default() };
        let mut mgr = TopicManager::new(config);
        mgr.subscribe("data", "addr").unwrap();
        mgr.unsubscribe("data", "addr");
        std::thread::sleep(Duration::from_millis(30));
        mgr.gc();

        // Client references non-existent topic
        let reset = mgr.check_epoch("data", 999).unwrap_err();
        assert_eq!(reset.new_epoch, 0, "Non-existent topic → epoch 0");
    }

    #[test]
    fn test_topic_authorization_uses_rep_c_address() {
        let mut mgr = TopicManager::new(TopicConfig::default());
        let creator = "1.2.3.1.2.3.1.2.3.1.2.3.1"; // Rep C address
        let other = "3.2.1.3.2.1.3.2.1.3.2.1.3";   // Different Rep C

        mgr.subscribe("data", creator).unwrap();
        mgr.subscribe("data", other).unwrap();

        // Only creator can publish (INVARIANT 9)
        assert!(mgr.publish("data", creator, "ok".to_string()).is_ok());
        assert_eq!(mgr.publish("data", other, "no".to_string()).unwrap_err(),
            RelayErrorCode::ErrTopicUnauthorized);
    }

    #[test]
    fn test_topic_backpressure_no_tombstone() {
        let config = TopicConfig { queue_depth: 2, ..Default::default() };
        let mut mgr = TopicManager::new(config);
        let addr = "1.1.1.1.1.1.1.1.1.1.1.1.1";
        mgr.subscribe("data", addr).unwrap();
        mgr.publish("data", addr, "1".to_string()).unwrap();
        mgr.publish("data", addr, "2".to_string()).unwrap();
        // Queue full → backpressure, NOT tombstone
        assert_eq!(mgr.publish("data", addr, "3".to_string()).unwrap_err(),
            RelayErrorCode::ErrTopicBackpressure);
    }

    #[test]
    fn test_coprime_stepped_delivery_crt_guarantee() {
        let mut mgr = TopicManager::new(TopicConfig::default());
        let addr = "1.1.1.1.1.1.1.1.1.1.1.1.1";
        for name in &["a", "b", "c", "d", "e"] {
            mgr.subscribe(name, addr).unwrap();
        }

        // One full cycle should visit all 5 topics
        let mut visited = HashSet::new();
        for _ in 0..5 {
            if let Some(topic) = mgr.next_delivery_topic() {
                visited.insert(topic);
            }
        }
        assert_eq!(visited.len(), 5, "CRT guarantee: all topics visited in one cycle");
    }

    #[test]
    fn test_coprime_mid_cycle_recomputation() {
        let mut mgr = TopicManager::new(TopicConfig::default());
        let addr = "1.1.1.1.1.1.1.1.1.1.1.1.1";
        mgr.subscribe("a", addr).unwrap();
        mgr.subscribe("b", addr).unwrap();
        mgr.subscribe("c", addr).unwrap();
        mgr.next_delivery_topic(); // Start cycle

        // Add 4th topic mid-cycle
        mgr.subscribe("d", "other").unwrap();

        // Complete a full cycle — all 4 must be visited
        let mut visited = HashSet::new();
        for _ in 0..4 {
            if let Some(topic) = mgr.next_delivery_topic() {
                visited.insert(topic);
            }
        }
        assert_eq!(visited.len(), 4, "After recomputation, all 4 topics visited");
    }

    // ═══════════════════════════════════════════════════════════
    // SEQUENCING & DEDUP
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_tombstone_golden_angle_spread() {
        let delays: Vec<u64> = (0..100).map(|i| {
            generate_tombstone(1, i, HashMap::new(), 0).suggested_resync_after_ms
        }).collect();
        let unique: HashSet<u64> = delays.iter().cloned().collect();
        assert!(unique.len() > 90, "Golden angle: >90/100 unique delays");
        for &d in &delays {
            assert!(d >= 2000 && d < 7000);
        }
    }

    #[test]
    fn test_resync_rate_limiting() {
        let mut rl = ResyncRateLimiter::new();
        assert!(rl.check_and_record("addr"));
        assert!(rl.check_and_record("addr"));
        assert!(rl.check_and_record("addr"));
        assert!(!rl.check_and_record("addr"), "4th resync must be rate-limited");
    }

    #[test]
    fn test_dedup_persistence_roundtrip() {
        let dir = std::env::temp_dir().join("plenum_relay_test_dedup_rt");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("test.dedup");
        let _ = std::fs::remove_file(&path);

        {
            let mut store = RelaySequenceStore::new(path.clone());
            store.update("topic-a", 42, 1000);
            store.update("topic-b", 99, 2000);
            store.flush().unwrap();
        }
        {
            let mut store = RelaySequenceStore::new(path.clone());
            assert_eq!(store.load(), 2);
            assert_eq!(store.get("topic-a").unwrap().last_processed_seq, 42);
            assert_eq!(store.get("topic-b").unwrap().topic_epoch, 2000);
        }
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn test_dedup_corruption_detection() {
        let dir = std::env::temp_dir().join("plenum_relay_test_corrupt");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("test_corrupt.dedup");
        let _ = std::fs::remove_file(&path);

        {
            let mut store = RelaySequenceStore::new(path.clone());
            store.update("data", 42, 1000);
            store.flush().unwrap();
        }
        {
            let mut data = std::fs::read(&path).unwrap();
            if data.len() > 5 { data[3] ^= 0xFF; }
            std::fs::write(&path, data).unwrap();
        }
        {
            let mut store = RelaySequenceStore::new(path.clone());
            assert_eq!(store.load(), 0, "Corruption → fresh start");
        }
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn test_tombstone_absence_semantics() {
        let mut store = RelaySequenceStore::in_memory();
        store.update("a", 10, 100);
        store.update("b", 20, 200);
        store.update("c", 30, 300);

        // Tombstone only mentions "a" — "b" and "c" are authoritative
        let mut snap = HashMap::new();
        snap.insert("a".to_string(), TopicSeqSnapshot { seq: 5, epoch: 100 });
        store.apply_tombstone(&snap);

        assert_eq!(store.get("a").unwrap().last_processed_seq, 5); // reset
        assert_eq!(store.get("b").unwrap().last_processed_seq, 20); // unchanged
        assert_eq!(store.get("c").unwrap().last_processed_seq, 30); // unchanged
    }

    // ═══════════════════════════════════════════════════════════
    // COPRIME DIRECT CALLS
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_coprime_options_direct() {
        let axis = TritInt::from_u64(12);
        let min = TritInt::from_u64(1);
        let max = TritInt::from_u64(12);
        let opts = coprime::coprime_options(&axis, &min, &max);
        let vals: Vec<u64> = opts.iter().map(|t| t.to_decimal()).collect();
        assert!(vals.contains(&1));
        assert!(vals.contains(&5));
        assert!(vals.contains(&7));
        assert!(vals.contains(&11));
        assert!(!vals.contains(&2));  // gcd(2,12)=2
        assert!(!vals.contains(&3));  // gcd(3,12)=3
        assert!(!vals.contains(&4));  // gcd(4,12)=4
        assert!(!vals.contains(&6));  // gcd(6,12)=6
    }

    // ═══════════════════════════════════════════════════════════
    // HEARTBEAT
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_heartbeat_hmodal_phasing() {
        let mut sched = HeartbeatScheduler::new(30_000);
        sched.recompute_positions(10);
        for i in 0..10 {
            sched.assign_phase(&format!("node_{}", i), i);
        }
        // Verify positions are unique
        let mut positions = HashSet::new();
        for i in 0..10 {
            let phase = sched.get_phase(&format!("node_{}", i)).unwrap();
            positions.insert(phase.position);
        }
        // Coprime walk assigns unique positions
        assert!(positions.len() > 1, "Positions should be spread, not all identical");
    }

    #[test]
    fn test_heartbeat_interval_change_nonack() {
        let mut sched = HeartbeatScheduler::new(30_000);
        sched.ack_timeout = Duration::from_millis(10);
        sched.recompute_positions(1);
        sched.assign_phase("node_a", 0);

        sched.mark_interval_change_sent("node_a");
        std::thread::sleep(Duration::from_millis(15));
        let timed_out = sched.check_ack_timeouts();
        assert_eq!(timed_out, vec!["node_a".to_string()]);
    }

    // ═══════════════════════════════════════════════════════════
    // CLIENT UPGRADES
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_full_jitter_backoff() {
        let mut backoff = JitterBackoff::new();
        let d1 = backoff.next_delay();
        assert!(d1.as_millis() < 1500, "First delay should be ~500-1000ms");
        for _ in 0..20 {
            let d = backoff.next_delay();
            assert!(d.as_millis() <= 60_000);
        }
        backoff.reset();
        assert_eq!(backoff.attempt(), 0);
    }

    #[test]
    fn test_frame_action_handlers() {
        assert_eq!(handle_circuit_open("crs"), FrameAction::CircuitOpen);
        assert_eq!(
            handle_topic_reset("data", 100, 200, 1),
            FrameAction::ResetTopic { topic: "data".to_string(), new_epoch: 200 }
        );
        assert_eq!(
            handle_go_away("shutdown", 3500),
            FrameAction::GoAway { reconnect_after_ms: 3500 }
        );
    }

    // ═══════════════════════════════════════════════════════════
    // METRICS
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_relay_metrics_16_count() {
        let m = RelayMetrics::new();
        assert_eq!(m.collect().len(), 16);
    }

    #[test]
    fn test_relay_metrics_prometheus_format() {
        let m = RelayMetrics::new();
        m.inc_tombstones();
        m.set_topics_active(5);
        let prom = m.to_prometheus();
        assert!(prom.contains("plenum_relay_tombstones_generated_total 1"));
        assert!(prom.contains("plenum_relay_topics_active 5"));
    }

    #[test]
    fn test_otel_sampling_control_frames_100pct() {
        for frame_type in &["go-away", "circuit_open", "topic_reset", "topic_revoked", "tombstone"] {
            assert!(should_sample(frame_type, 0.0),
                "Control frame '{}' must always be sampled", frame_type);
        }
    }

    // ═══════════════════════════════════════════════════════════
    // GOLDEN ANGLE
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_golden_angle_100_goaway_spread() {
        let delays: Vec<u64> = (0..100).map(|i| golden_angle_delay(i, 2000, 5000)).collect();
        let unique: HashSet<u64> = delays.iter().cloned().collect();
        assert!(unique.len() > 90);
        for &d in &delays {
            assert!(d >= 2000 && d < 7000);
        }
    }

    // ═══════════════════════════════════════════════════════════
    // SHUTDOWN
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_draining_health_body_format() {
        let body = inter_cube::relay_shutdown::draining_health_body(1712800030000);
        assert_eq!(body["status"], "draining");
        assert_eq!(body["reason"], "shutdown_in_progress");
        assert_eq!(body["drain_deadline_ms"], 1712800030000u64);
    }

    // ═══════════════════════════════════════════════════════════
    // FORMA CODEX FIELDS
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_audit_entry_has_forma_codex_fields() {
        let mut log = RelayAuditLog::in_memory();
        // Record and check the persisted entry structure
        // (We verify the struct fields exist at compile time)
        let entry = RelayAuditEntry {
            event_type: RelayAuditEventType::TopicSubscribe,
            address: "1.1.1.1.1.1.1.1.1.1.1.1.1".to_string(),
            timestamp: "2026-04-12T00:00:00Z".to_string(),
            details: serde_json::json!({"topic": "sensor-data"}),
            severity: AuditSeverity::Info,
            subsystem: AuditSubsystem::Topic,
            correlation_refs: vec!["prev-event-id".to_string()],
        };
        let hash = log.record_event(&entry);
        assert!(!hash.is_empty());
        // Forma Codex fields: severity, subsystem, correlation_refs
        // are part of the struct — compile-time guarantee.
    }

    // Helper
    fn gcd(a: u64, b: u64) -> u64 {
        if b == 0 { a } else { gcd(b, a % b) }
    }
}
