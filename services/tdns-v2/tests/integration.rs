// TDNS v2.3 — Integration Tests
// Capomastro Holdings Ltd. — Applied Physics Division
//
// End-to-end tests proving the full service pipeline.
// These wire up CRS + FTS + GLB + CON + Bridge + API and run
// complete lifecycle scenarios.
//
// No mocks. No stubs. Real service instances.

#![allow(unused_variables)]

use tdns_v2::addr::{CubeAddr, DIMENSIONS};
use tdns_v2::api::ApiRouter;
use tdns_v2::bridge::{Bridge, Resolution};
use tdns_v2::overlay::{ConNode, LinkState, derive_link_keys};
use tdns_v2::crs::{CrsRegistry, RegistrationResult, VerificationResult};
use tdns_v2::derive::all_rules;
use tdns_v2::fts::{Fts, FtsConfig, FtsEvent, Heartbeat};
use tdns_v2::glb::{Glb, GlbDecision, NodeStatus};
use tdns_v2::routing::NeighborMap;
use tdns_v2::scan::RawValue;
use tdns_v2::schema::{SCHEMA, describe};
use tdns_v2::subcube::SubCube;
use tdns_v2::trit::Trit;
use tdns_v2::wire::{Packet, PacketType};

// ═══════════════════════════════════════════════════════════════════════════
// Test fixtures — raw measurements for known entities
// ═══════════════════════════════════════════════════════════════════════════

fn google_measurements() -> Vec<RawValue> {
    vec![
        RawValue::Pattern("corporate".into()),
        RawValue::Pattern("public".into()),
        RawValue::Numeric(2.0),
        RawValue::Pattern("cloud".into()),
        RawValue::Pattern("website".into()),
        RawValue::Pattern("text".into()),
        RawValue::Pattern("both".into()),
        RawValue::Numeric(4.0),
        RawValue::Numeric(3.0),
        RawValue::Pattern("none".into()),
        RawValue::Numeric(4.0),
        RawValue::Pattern("http".into()),
        RawValue::Numeric(1.0),
        RawValue::Numeric(3.0),
        RawValue::Pattern("current".into()),
        RawValue::Numeric(2.0),
        RawValue::Pattern("accepts".into()),
        RawValue::Numeric(4.0),
        RawValue::Numeric(4.0),
        RawValue::Pattern("free".into()),
        RawValue::Pattern("unicast".into()),
        RawValue::Pattern("through".into()),
        RawValue::Pattern("poll".into()),
        RawValue::Numeric(1.0),
        RawValue::Numeric(5.0),
        RawValue::Numeric(0.0),
        RawValue::Pattern("soc2".into()),
    ]
}

fn pptpro_measurements() -> Vec<RawValue> {
    vec![
        RawValue::Pattern("corporate".into()),
        RawValue::Pattern("public".into()),
        RawValue::Numeric(4.0),
        RawValue::Pattern("cloud".into()),
        RawValue::Pattern("app".into()),
        RawValue::Pattern("live".into()),
        RawValue::Pattern("both".into()),
        RawValue::Numeric(4.0),
        RawValue::Numeric(1.0),
        RawValue::Pattern("password".into()),
        RawValue::Numeric(3.0),
        RawValue::Pattern("websocket".into()),
        RawValue::Numeric(5.0),
        RawValue::Numeric(3.0),
        RawValue::Pattern("live".into()),
        RawValue::Numeric(5.0),
        RawValue::Pattern("no".into()),
        RawValue::Numeric(2.0),
        RawValue::Numeric(2.0),
        RawValue::Pattern("free".into()),
        RawValue::Pattern("multicast".into()),
        RawValue::Pattern("out".into()),
        RawValue::Pattern("push".into()),
        RawValue::Numeric(3.0),
        RawValue::Numeric(5.0),
        RawValue::Numeric(4.0),
        RawValue::Pattern("self-certified".into()),
    ]
}

fn blog_measurements() -> Vec<RawValue> {
    vec![
        RawValue::Pattern("personal".into()),
        RawValue::Pattern("public".into()),
        RawValue::Numeric(0.0),
        RawValue::Pattern("provider".into()),
        RawValue::Pattern("website".into()),
        RawValue::Pattern("text".into()),
        RawValue::Pattern("people".into()),
        RawValue::Numeric(0.0),
        RawValue::Numeric(3.0),
        RawValue::Pattern("none".into()),
        RawValue::Numeric(1.0),
        RawValue::Pattern("http".into()),
        RawValue::Numeric(3.0),
        RawValue::Numeric(3.0),
        RawValue::Pattern("historical".into()),
        RawValue::Numeric(0.0),
        RawValue::Pattern("no".into()),
        RawValue::Numeric(0.0),
        RawValue::Numeric(0.0),
        RawValue::Pattern("free".into()),
        RawValue::Pattern("unicast".into()),
        RawValue::Pattern("out".into()),
        RawValue::Pattern("poll".into()),
        RawValue::Numeric(0.0),
        RawValue::Numeric(2.0),
        RawValue::Numeric(1.0),
        RawValue::Pattern("no".into()),
    ]
}

// ═══════════════════════════════════════════════════════════════════════════
// SCENARIO 1: Full Registration Lifecycle
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn full_registration_lifecycle() {
    let mut crs = CrsRegistry::new();
    let now = 1_000_000_000u64;

    let result = crs.register(
        "google.plm".into(),
        "plm".into(),
        vec![0xDE, 0xAD],
        google_measurements(),
        now,
        None,
    );
    let g_addr = match result {
        RegistrationResult::Ok { address, .. } => address,
        other => panic!("google registration failed: {:?}", other),
    };
    assert!(!g_addr.is_hptp_mandatory());

    let result = crs.register(
        "pptpro.capomastro.plm".into(),
        "capomastro.plm".into(),
        vec![0xBE, 0xEF],
        pptpro_measurements(),
        now,
        Some(500),
    );
    let p_addr = match result {
        RegistrationResult::Ok { address, .. } => address,
        other => panic!("pptpro registration failed: {:?}", other),
    };
    assert!(p_addr.is_hptp_mandatory());

    let result = crs.register(
        "nonnas-cucina.plm".into(),
        "plm".into(),
        vec![0xCA, 0xFE],
        blog_measurements(),
        now,
        None,
    );
    let b_addr = match result {
        RegistrationResult::Ok { address, .. } => address,
        other => panic!("blog registration failed: {:?}", other),
    };

    assert_eq!(crs.entity_count(), 3);
    assert!(crs.resolve("google.plm").is_some());
    assert!(crs.resolve("pptpro.capomastro.plm").is_some());
    assert!(crs.resolve("nonnas-cucina.plm").is_some());

    assert!(g_addr.distance(&p_addr) > 0);
    assert!(g_addr.distance(&b_addr) > 0);
    assert!(p_addr.distance(&b_addr) > 0);

    assert_eq!(crs.hptp_mandatory_count(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// SCENARIO 2: HPTP Enforcement Pipeline (CRS → FTS → GLB)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn hptp_enforcement_pipeline() {
    let mut crs = CrsRegistry::new();
    let now = 1_000_000_000u64;

    crs.register(
        "pptpro.capomastro.plm".into(),
        "capomastro.plm".into(),
        vec![0xBE, 0xEF],
        pptpro_measurements(),
        now,
        Some(100),
    );
    let p_addr = crs.resolve_addr("pptpro.capomastro.plm").unwrap();

    let mut fts = Fts::new();
    let hb = Heartbeat::new(p_addr, now, 100, 1);
    let events = fts.process_heartbeat(&hb);

    assert!(events.iter().any(|e| matches!(e, FtsEvent::NodeAlive { .. })));
    assert!(events.iter().any(|e| matches!(e, FtsEvent::HptpUpdate { mandatory: true, .. })));

    let local = CubeAddr::from_values(&[1; 27]).unwrap();
    let mut map = NeighborMap::new(local);
    let diffs = local.differing_dims(&p_addr);
    if !diffs.is_empty() {
        map.set(diffs[0], p_addr.trit(diffs[0]), p_addr);
    }
    let mut glb = Glb::new(local, map);

    assert!(glb.check_hptp(&p_addr, now).is_none());

    let hb_bad = Heartbeat::new(p_addr, now + 1000, 5_000, 2);
    fts.process_heartbeat(&hb_bad);

    glb.report_hptp_offset(p_addr, 5_000, now + 1000);

    match glb.check_hptp(&p_addr, now + 1001) {
        Some(GlbDecision::HptpDropped { target, .. }) => {
            assert_eq!(target, p_addr);
        }
        other => panic!("expected HptpDropped, got {:?}", other),
    }

    let hb_good = Heartbeat::new(p_addr, now + 2000, 200, 3);
    fts.process_heartbeat(&hb_good);
    glb.report_hptp_offset(p_addr, 200, now + 2000);

    match glb.node_status(&p_addr, now + 2001) {
        NodeStatus::HptpHolddown { .. } => {}
        other => panic!("expected HptpHolddown, got {:?}", other),
    }

    let after = now + 2000 + 5_000_000_001;
    glb.report_hptp_offset(p_addr, 100, after);
    match glb.node_status(&p_addr, after) {
        NodeStatus::Healthy => {}
        other => panic!("expected Healthy, got {:?}", other),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SCENARIO 3: Property Drift → Re-derivation → Redirect
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn property_drift_full_cycle() {
    let mut crs = CrsRegistry::new();
    let now = 1_000_000_000u64;

    crs.register(
        "google.plm".into(),
        "plm".into(),
        vec![0xDE, 0xAD],
        google_measurements(),
        now,
        None,
    );
    let old_addr = crs.resolve_addr("google.plm").unwrap();

    let mut changed = google_measurements();
    changed[19] = RawValue::Pattern("subscription".into());

    let later = now + 1_000_000_000;
    let result = crs.rescan("google.plm", changed.clone(), later).unwrap();

    match result {
        VerificationResult::Drifted {
            old_address,
            new_address,
            changed_dims,
            ..
        } => {
            assert_eq!(old_address, old_addr);
            assert_ne!(old_address, new_address);
            assert!(changed_dims.contains(&19));

            let resolved = crs.resolve_addr("google.plm").unwrap();
            assert_eq!(resolved, new_address);

            let redirect = crs.check_redirect(&old_addr, later + 100);
            assert_eq!(redirect, Some(new_address));

            let map = NeighborMap::new(old_addr);
            let glb = Glb::new(old_addr, map);
            match glb.forward_point(&old_addr, later + 100) {
                GlbDecision::DeliverLocal => {}
                other => panic!("expected DeliverLocal at old addr, got {:?}", other),
            }
        }
        other => panic!("expected Drifted, got {:?}", other),
    }

    assert_eq!(crs.drift_log().len(), 1);

    let result2 = crs.verify("google.plm", changed, later + 2000);
    match result2 {
        VerificationResult::Verified { .. } => {}
        other => panic!("expected Verified, got {:?}", other),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SCENARIO 4: CON Tunnel Mesh + Key Derivation
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn con_tunnel_mesh() {
    let mut crs = CrsRegistry::new();
    let now = 1_000_000_000u64;
    let secret = b"fabric-shared-secret".to_vec();

    crs.register("google.plm".into(), "plm".into(), vec![1], google_measurements(), now, None);
    crs.register("pptpro.capomastro.plm".into(), "capomastro.plm".into(), vec![2], pptpro_measurements(), now, Some(100));
    crs.register("nonnas-cucina.plm".into(), "plm".into(), vec![3], blog_measurements(), now, None);

    let g = crs.resolve_addr("google.plm").unwrap();
    let p = crs.resolve_addr("pptpro.capomastro.plm").unwrap();
    let b = crs.resolve_addr("nonnas-cucina.plm").unwrap();

    let g_map = crs.neighbor_map(&g).unwrap().clone();
    let mut con_g = ConNode::new(g, secret.clone());
    con_g.establish_from_neighbor_map(&g_map, now);

    assert!(con_g.total_count() >= 2, "Google should have tunnels to at least PPTPro and Blog");

    if let (Some(key_p), Some(key_b)) = (con_g.outbound_key(&p), con_g.outbound_key(&b)) {
        assert_ne!(key_p, key_b, "different neighbors must have different keys");
    }

    let (out_gp, in_gp) = derive_link_keys(&g, &p, &secret);
    let (out_pg, in_pg) = derive_link_keys(&p, &g, &secret);
    assert_eq!(out_gp, out_pg, "canonical key pair must be identical regardless of caller order");
    assert_eq!(in_gp, in_pg);

    con_g.record_sent(&p, 4096);
    con_g.record_received(&p, 2048);
    let (sent, recv) = con_g.total_bytes();
    assert_eq!(sent, 4096);
    assert_eq!(recv, 2048);

    let old_key = con_g.outbound_key(&p).unwrap().clone();
    con_g.rekey_all(now + 1_000_000);
    let new_key = con_g.outbound_key(&p).unwrap();
    assert_ne!(&old_key, new_key);
}

// ═══════════════════════════════════════════════════════════════════════════
// SCENARIO 5: Bridge Resolution (TDNS vs Legacy)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn bridge_tdns_vs_legacy() {
    let mut crs = CrsRegistry::new();
    let now = 1_000_000_000u64;

    crs.register("google.plm".into(), "plm".into(), vec![1], google_measurements(), now, None);
    let g_addr = crs.resolve_addr("google.plm").unwrap();

    let crs_shared = std::sync::Arc::new(std::sync::RwLock::new(crs));
    let mut bridge = Bridge::new(crs_shared);

    match bridge.resolve("google.plm", now + 100) {
        Resolution::Tdns { name, address, .. } => {
            assert_eq!(name, "google.plm");
            assert_eq!(address, g_addr);
        }
        other => panic!("expected Tdns, got {:?}", other),
    }

    let legacy = bridge.resolve("github.com", now + 200);
    match &legacy {
        Resolution::Legacy { name, .. } => assert_eq!(name, "github.com"),
        Resolution::Failed { .. } => {}
        other => panic!("expected Legacy or Failed, got {:?}", other),
    }

    match bridge.resolve("nonexistent.plm", now + 300) {
        Resolution::Failed { name, .. } => assert_eq!(name, "nonexistent.plm"),
        other => panic!("expected Failed, got {:?}", other),
    }

    assert_eq!(bridge.tdns_count(), 1);
    assert!(bridge.failed_count() >= 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// SCENARIO 6: Multi-Node Routing Through Sparse Cube
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn sparse_cube_routing() {
    let mut crs = CrsRegistry::new();
    let now = 1_000_000_000u64;

    crs.register("google.plm".into(), "plm".into(), vec![1], google_measurements(), now, None);
    crs.register("pptpro.capomastro.plm".into(), "capomastro.plm".into(), vec![2], pptpro_measurements(), now, Some(100));
    crs.register("nonnas-cucina.plm".into(), "plm".into(), vec![3], blog_measurements(), now, None);

    let g = crs.resolve_addr("google.plm").unwrap();
    let p = crs.resolve_addr("pptpro.capomastro.plm").unwrap();
    let b = crs.resolve_addr("nonnas-cucina.plm").unwrap();

    let g_map = crs.neighbor_map(&g).unwrap().clone();
    let p_map = crs.neighbor_map(&p).unwrap().clone();

    let glb_g = Glb::new(g, g_map);
    match glb_g.forward_point(&p, now) {
        GlbDecision::Forward { next_hop, remaining_hops, .. } => {
            assert!(remaining_hops < g.distance(&p), "should make progress");
            assert!(next_hop.distance(&p) < g.distance(&p));
        }
        GlbDecision::DeliverLocal => panic!("Google is not PPTPro"),
        GlbDecision::NoRoute { reason } => {
            eprintln!("sparse routing gap: {}", reason);
        }
        other => panic!("unexpected: {:?}", other),
    }

    let density = crs.dimension_density();
    assert_eq!(density.len(), 27);
    for dim in &density {
        let total: usize = dim.iter().sum();
        assert_eq!(total, 3, "each dimension should account for all 3 entities");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SCENARIO 7: FTS Failure Detection → GLB Dead Set → CON Tunnel Down
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn failure_detection_pipeline() {
    let mut crs = CrsRegistry::new();
    let now = 1_000_000_000u64;

    crs.register("google.plm".into(), "plm".into(), vec![1], google_measurements(), now, None);
    crs.register("pptpro.capomastro.plm".into(), "capomastro.plm".into(), vec![2], pptpro_measurements(), now, Some(100));

    let g = crs.resolve_addr("google.plm").unwrap();
    let p = crs.resolve_addr("pptpro.capomastro.plm").unwrap();

    let config = FtsConfig {
        heartbeat_interval_ns: 1000,
        failure_threshold: 2,
        suspect_threshold: 1,
        ..Default::default()
    };
    let mut fts = Fts::with_config(config);
    fts.process_heartbeat(&Heartbeat::new(g, now, 0, 1));
    fts.process_heartbeat(&Heartbeat::new(p, now, 50, 1));

    let g_map = crs.neighbor_map(&g).unwrap().clone();
    let mut glb = Glb::new(g, g_map);

    let mut con = ConNode::new(g, b"secret".to_vec());
    let diffs = g.differing_dims(&p);
    if !diffs.is_empty() {
        con.establish_tunnel(p, diffs[0], p.trit(diffs[0]), now);
    }
    assert_eq!(con.active_count(), 1);

    fts.process_heartbeat(&Heartbeat::new(g, now + 1100, 0, 2));
    let events = fts.check(now + 1100);

    assert!(events.iter().any(|e| matches!(e, FtsEvent::NodeSuspect { addr, .. } if *addr == p)));

    fts.process_heartbeat(&Heartbeat::new(g, now + 2200, 0, 3));
    let events = fts.check(now + 2200);
    assert!(events.iter().any(|e| matches!(e, FtsEvent::NodeDead { addr, .. } if *addr == p)));

    glb.mark_dead(p);
    assert_eq!(glb.dead_count(), 1);

    con.mark_down(&p);
    assert_eq!(con.active_count(), 0);
    assert_eq!(con.link(&p).unwrap().state, LinkState::Down);

    let events = fts.process_heartbeat(&Heartbeat::new(p, now + 3000, 30, 2));
    assert!(events.contains(&FtsEvent::NodeRecovered { addr: p }));

    glb.mark_alive(p);
    assert_eq!(glb.dead_count(), 0);

    con.mark_active(&p);
    assert_eq!(con.active_count(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// SCENARIO 8: Sub-cube Multicast Through GLB
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn subcube_multicast_delivery() {
    let mut crs = CrsRegistry::new();
    let now = 1_000_000_000u64;

    crs.register("google.plm".into(), "plm".into(), vec![1], google_measurements(), now, None);
    crs.register("pptpro.capomastro.plm".into(), "capomastro.plm".into(), vec![2], pptpro_measurements(), now, Some(100));
    crs.register("nonnas-cucina.plm".into(), "plm".into(), vec![3], blog_measurements(), now, None);

    let g = crs.resolve_addr("google.plm").unwrap();

    let g_map = crs.neighbor_map(&g).unwrap().clone();
    let glb = Glb::new(g, g_map);

    let sc = SubCube::wildcard();
    match glb.forward_subcube(&sc, None, now) {
        GlbDecision::Multicast { next_hops } => {
            assert!(!next_hops.is_empty(), "wildcard should fan out");
        }
        GlbDecision::DeliverLocal => {
        }
        other => panic!("expected Multicast or DeliverLocal, got {:?}", other),
    }

    let mut mask = [false; DIMENSIONS];
    mask[14] = true;
    mask[15] = true;
    let p = crs.resolve_addr("pptpro.capomastro.plm").unwrap();
    let sc_hptp = SubCube::new(p, mask);

    assert!(!sc_hptp.contains(&g));
    assert!(sc_hptp.contains(&p));
}

// ═══════════════════════════════════════════════════════════════════════════
// SCENARIO 9: Wire Protocol Round-trip
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn wire_protocol_roundtrip() {
    let g = CubeAddr::from_category_string("WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313").unwrap();
    let p = CubeAddr::from_category_string("WO:2333 WA:2333 WR:2222 WN:3333 WY:1221 HO:2133 PE:332").unwrap();

    let payload = b"hello plenum";
    let packet = Packet::data(g, p, payload.to_vec(), 1_000_000_000);

    assert_eq!(packet.packet_type, PacketType::Data);
    assert_eq!(packet.source, g);
    assert_eq!(packet.payload, payload);

    let wire = packet.to_wire();
    let decoded = Packet::from_wire(&wire).expect("wire roundtrip failed");

    assert_eq!(decoded.source, packet.source);
    assert_eq!(decoded.payload, packet.payload);
    assert_eq!(decoded.packet_type, packet.packet_type);
    assert!(decoded.verify_integrity(), "BLAKE3 integrity check failed");
}

// ═══════════════════════════════════════════════════════════════════════════
// SCENARIO 10: API Router End-to-End
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn api_router_lifecycle() {
    let mut router = ApiRouter::new();
    let now = 1_000_000_000u64;

    let health = router.handle_health();
    assert_eq!(health.status, "ok");
    assert_eq!(health.version, "2.3.3");
    assert_eq!(health.entities, 0);

    let desc = router.handle_describe("WO:2333 WA:2333 WR:2222 WN:3333 WY:1221 HO:2133 PE:332").unwrap();
    assert_eq!(desc.dimensions.len(), 27);
    assert!(desc.hptp_mandatory);
    assert_eq!(desc.dimensions[0].label, "Corporate");
    assert_eq!(desc.dimensions[4].label, "App");
    assert_eq!(desc.dimensions[5].label, "Live");

    assert!(router.handle_resolve("nonexistent.plm").is_err());

    let fts = router.handle_fts_status();
    assert_eq!(fts.alive, 0);

    let con = router.handle_con_metrics();
    assert_eq!(con.active_tunnels, 0);

    let g = CubeAddr::from_category_string("WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313").unwrap();
    router.init_glb(g);

    let route = router.handle_route(
        tdns_v2::api::RouteRequest {
            source: "WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313".into(),
            destination: "WO:2333 WA:2333 WR:2222 WN:3333 WY:1221 HO:2133 PE:332".into(),
        },
        now,
    ).unwrap();
    assert!(route.distance > 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// SCENARIO 11: Schema ↔ Derivation ↔ Address Consistency
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn schema_derivation_consistency() {
    let rules = all_rules();
    assert_eq!(rules.len(), 27);
    assert_eq!(SCHEMA.len(), 27);

    for (i, rule) in rules.iter().enumerate() {
        assert_eq!(
            rule.dimension(), i,
            "rule {} dimension mismatch: expected {}, got {}",
            i, i, rule.dimension()
        );
        assert_eq!(
            SCHEMA[i].number, i + 1,
            "schema {} number mismatch",
            i
        );
    }

    let measurements = google_measurements();
    let mut trits = [Trit::V1; 27];
    for (i, (rule, raw)) in rules.iter().zip(measurements.iter()).enumerate() {
        let (trit, _) = rule.derive(raw).unwrap();
        trits[i] = trit;
    }
    let addr = CubeAddr::new(trits);

    let description = describe(&addr);
    assert_eq!(description.len(), 27);

    assert_eq!(description[0].2, "Corporate");
    assert_eq!(description[24].2, "Full TLS");
}

// ═══════════════════════════════════════════════════════════════════════════
// SCENARIO 12: Address Space Guarantees
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn address_space_guarantees() {
    let space: u64 = 3u64.pow(27);
    assert_eq!(space, 7_625_597_484_987);

    let all_ones = CubeAddr::from_values(&[1; 27]).unwrap();
    let all_threes = CubeAddr::from_values(&[3; 27]).unwrap();
    assert_eq!(all_ones.distance(&all_threes), 27);

    let wire = all_ones.to_wire();
    assert_eq!(wire.len(), 7);

    let display = all_ones.to_category_string();
    assert_eq!(display.split_whitespace().count(), 7);

    let canonical = all_ones.to_canonical_string();
    assert_eq!(canonical.split('.').count(), 9);

    let sc = SubCube::wildcard();
    assert!(sc.contains(&all_ones));
    assert!(sc.contains(&all_threes));
    assert_eq!(sc.size(), space);
}
