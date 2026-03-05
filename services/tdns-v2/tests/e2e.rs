// TDNS v2.3 — End-to-End Integration Tests
// Capomastro Holdings Ltd. — Applied Physics Division
//
// Proves the full pipeline composes correctly across all modules:
//   scanner → derive → crs → fts → glb → con → bridge → wire → api
//
// No mocks. No stubs. Real derivation, real routing, real key derivation,
// real wire encoding. The only thing missing is live network I/O
// (scanner probes use test fixtures for deterministic assertions).

use tdns_v2::addr::{CubeAddr, DIMENSIONS};
use tdns_v2::api::ApiRouter;
use tdns_v2::bridge::{Bridge, Resolution, is_plm_name};
use tdns_v2::overlay::{ConNode, derive_link_keys};
use tdns_v2::crs::{CrsRegistry, RegistrationResult, VerificationResult};
use tdns_v2::fts::{Fts, FtsConfig, FtsEvent, Heartbeat, HealthState};
use tdns_v2::glb::{Glb, GlbDecision};
use tdns_v2::routing::NeighborMap;
use tdns_v2::scan::RawValue;
use tdns_v2::schema::describe;
use tdns_v2::subcube::SubCube;
use tdns_v2::trit::Trit;
use tdns_v2::wire::{Packet, PacketType, parse_heartbeat_payload, flags};

fn google_measurements() -> Vec<RawValue> {
    vec![
        RawValue::Pattern("corporate".into()),
        RawValue::Pattern("public".into()),
        RawValue::Numeric(0.5),
        RawValue::Pattern("cloud".into()),
        RawValue::Pattern("website".into()),
        RawValue::Pattern("text".into()),
        RawValue::Pattern("both".into()),
        RawValue::Numeric(0.9),
        RawValue::Numeric(200.0),
        RawValue::Pattern("none".into()),
        RawValue::Numeric(100.0),
        RawValue::Pattern("http".into()),
        RawValue::Numeric(1998.0),
        RawValue::Numeric(99.99),
        RawValue::Pattern("current".into()),
        RawValue::Numeric(200.0),
        RawValue::Pattern("accepts".into()),
        RawValue::Numeric(20.0),
        RawValue::Numeric(4.0),
        RawValue::Pattern("free".into()),
        RawValue::Pattern("unicast".into()),
        RawValue::Pattern("through".into()),
        RawValue::Pattern("poll".into()),
        RawValue::Numeric(3600.0),
        RawValue::Numeric(0.95),
        RawValue::Numeric(30.0),
        RawValue::Pattern("soc2".into()),
    ]
}

fn pptpro_measurements() -> Vec<RawValue> {
    vec![
        RawValue::Pattern("corporate".into()),
        RawValue::Pattern("public".into()),
        RawValue::Numeric(0.9),
        RawValue::Pattern("cloud".into()),
        RawValue::Pattern("app".into()),
        RawValue::Pattern("live".into()),
        RawValue::Pattern("both".into()),
        RawValue::Numeric(0.95),
        RawValue::Numeric(401.0),
        RawValue::Pattern("password".into()),
        RawValue::Numeric(3.0),
        RawValue::Pattern("websocket".into()),
        RawValue::Numeric(2024.0),
        RawValue::Numeric(99.99),
        RawValue::Pattern("live".into()),
        RawValue::Numeric(5.0),
        RawValue::Pattern("no".into()),
        RawValue::Numeric(3.0),
        RawValue::Numeric(2.0),
        RawValue::Pattern("free".into()),
        RawValue::Pattern("multicast".into()),
        RawValue::Pattern("out".into()),
        RawValue::Pattern("push".into()),
        RawValue::Numeric(999999.0),
        RawValue::Numeric(0.95),
        RawValue::Numeric(0.0),
        RawValue::Pattern("self-certified".into()),
    ]
}

fn blog_measurements() -> Vec<RawValue> {
    vec![
        RawValue::Pattern("personal".into()),
        RawValue::Pattern("public".into()),
        RawValue::Numeric(0.1),
        RawValue::Pattern("provider".into()),
        RawValue::Pattern("website".into()),
        RawValue::Pattern("text".into()),
        RawValue::Pattern("people".into()),
        RawValue::Numeric(0.0),
        RawValue::Numeric(200.0),
        RawValue::Pattern("none".into()),
        RawValue::Numeric(1.0),
        RawValue::Pattern("http".into()),
        RawValue::Numeric(2015.0),
        RawValue::Numeric(99.0),
        RawValue::Pattern("historical".into()),
        RawValue::Numeric(800.0),
        RawValue::Pattern("no".into()),
        RawValue::Numeric(0.0),
        RawValue::Numeric(0.0),
        RawValue::Pattern("free".into()),
        RawValue::Pattern("unicast".into()),
        RawValue::Pattern("out".into()),
        RawValue::Pattern("poll".into()),
        RawValue::Numeric(0.0),
        RawValue::Numeric(0.15),
        RawValue::Numeric(5.0),
        RawValue::Pattern("no".into()),
    ]
}

#[test]
fn e2e_register_resolve_route() {
    let mut crs = CrsRegistry::new();
    let now = 1_000_000_000u64;

    let g_result = crs.register(
        "google.plm".into(), "plm".into(),
        vec![0x01], google_measurements(), now, None,
    );
    let g_addr = match g_result {
        RegistrationResult::Ok { address, .. } => address,
        other => panic!("Google registration failed: {:?}", other),
    };

    let p_result = crs.register(
        "pptpro.capomastro.plm".into(), "capomastro.plm".into(),
        vec![0x02], pptpro_measurements(), now, Some(100),
    );
    let p_addr = match p_result {
        RegistrationResult::Ok { address, .. } => address,
        other => panic!("PPTPro registration failed: {:?}", other),
    };

    let b_result = crs.register(
        "nonnas-cucina.plm".into(), "plm".into(),
        vec![0x03], blog_measurements(), now, None,
    );
    let _b_addr = match b_result {
        RegistrationResult::Ok { address, .. } => address,
        other => panic!("Blog registration failed: {:?}", other),
    };

    assert_eq!(crs.entity_count(), 3);

    let g_trn = crs.resolve("google.plm").unwrap();
    assert_eq!(g_trn.address, g_addr);

    let p_trn = crs.resolve("pptpro.capomastro.plm").unwrap();
    assert!(p_trn.is_hptp_mandatory());
    assert!(p_trn.is_hptp_synced());

    assert_eq!(crs.reverse_lookup(&g_addr).unwrap(), "google.plm");
    assert_eq!(crs.reverse_lookup(&p_addr).unwrap(), "pptpro.capomastro.plm");

    let g_map = crs.neighbor_map(&g_addr).unwrap().clone();
    let glb = Glb::new(g_addr, g_map);

    let decision = glb.route(&p_addr, now);
    match decision {
        GlbDecision::Forward { remaining_hops, .. } => {
            let expected_dist = g_addr.distance(&p_addr);
            assert!(remaining_hops <= expected_dist);
        }
        GlbDecision::NoRoute { .. } => {
        }
        other => panic!("unexpected GLB decision: {:?}", other),
    }

    assert_eq!(glb.route(&g_addr, now), GlbDecision::DeliverLocal);

    let desc = describe(&p_addr);
    assert_eq!(desc.len(), 27);
    let (_, _, label, _) = &desc[14];
    assert_eq!(*label, "Live");
}

#[test]
fn e2e_fts_heartbeat_to_glb_enforcement() {
    let mut crs = CrsRegistry::new();
    let now = 1_000_000_000u64;

    let p_result = crs.register(
        "pptpro.capomastro.plm".into(), "capomastro.plm".into(),
        vec![0x02], pptpro_measurements(), now, Some(100),
    );
    let p_addr = match p_result {
        RegistrationResult::Ok { address, .. } => address,
        other => panic!("failed: {:?}", other),
    };
    assert!(p_addr.is_hptp_mandatory());

    crs.register(
        "google.plm".into(), "plm".into(),
        vec![0x01], google_measurements(), now, None,
    );
    let g_addr = crs.resolve_addr("google.plm").unwrap();

    let mut fts = Fts::new();

    let events = fts.process_heartbeat(&Heartbeat::new(p_addr, now, 100, 1));
    assert!(events.iter().any(|e| matches!(e, FtsEvent::NodeAlive { .. })));
    assert!(events.iter().any(|e| matches!(e, FtsEvent::HptpUpdate { mandatory: true, .. })));

    let events = fts.process_heartbeat(&Heartbeat::new(g_addr, now, 0, 1));
    assert!(events.iter().any(|e| matches!(e, FtsEvent::NodeAlive { .. })));
    assert!(!events.iter().any(|e| matches!(e, FtsEvent::HptpUpdate { .. })));

    let map = crs.neighbor_map(&g_addr).unwrap_or(&NeighborMap::new(g_addr)).clone();
    let mut glb = Glb::new(g_addr, map);

    glb.report_hptp_offset(p_addr, 100, now);
    assert!(glb.check_hptp(&p_addr, now).is_none());

    glb.report_hptp_offset(p_addr, 5_000, now + 1000);

    match glb.check_hptp(&p_addr, now + 1000) {
        Some(GlbDecision::HptpDropped { target, .. }) => {
            assert_eq!(target, p_addr);
        }
        other => panic!("expected HptpDropped, got {:?}", other),
    }

    assert!(glb.check_hptp(&g_addr, now + 1000).is_none());
}

#[test]
fn e2e_con_tunnel_wire_packet() {
    let mut crs = CrsRegistry::new();
    let now = 1_000_000_000u64;

    crs.register(
        "google.plm".into(), "plm".into(),
        vec![0x01], google_measurements(), now, None,
    );
    crs.register(
        "pptpro.capomastro.plm".into(), "capomastro.plm".into(),
        vec![0x02], pptpro_measurements(), now, Some(100),
    );

    let g_addr = crs.resolve_addr("google.plm").unwrap();
    let p_addr = crs.resolve_addr("pptpro.capomastro.plm").unwrap();

    let secret = b"fabric-shared-secret-for-test";
    let mut con_g = ConNode::new(g_addr, secret.to_vec());
    let _con_p = ConNode::new(p_addr, secret.to_vec());

    let g_map = crs.neighbor_map(&g_addr).unwrap().clone();
    con_g.establish_from_neighbor_map(&g_map, now);

    let (_out_gp, _in_gp) = derive_link_keys(&g_addr, &p_addr, secret);
    let g_out = con_g.outbound_key(&p_addr);

    if let Some(key) = g_out {
        let (rederived_out, _) = derive_link_keys(&g_addr, &p_addr, secret);
        assert_eq!(key.as_bytes(), rederived_out.as_bytes());
    }

    let payload = b"CRS scan results for google.plm".to_vec();
    let pkt = Packet::data(g_addr, p_addr, payload.clone(), now);

    assert_eq!(pkt.version, 0x23);
    assert_eq!(pkt.packet_type, PacketType::Data);
    assert!(pkt.is_point());
    assert!(!pkt.is_multicast());
    assert!(pkt.requires_hptp());
    assert!(pkt.verify_integrity());

    let wire = pkt.to_wire();
    let decoded = Packet::from_wire(&wire).unwrap();
    assert_eq!(decoded.source, g_addr);
    assert_eq!(decoded.destination, p_addr);
    assert_eq!(decoded.payload, payload);
    assert_eq!(decoded.timestamp_ns, now);

    let hb_pkt = Packet::heartbeat(p_addr, -200, 42, now);
    assert!(hb_pkt.flags & flags::HPTP_MANDATORY != 0);

    let hb_wire = hb_pkt.to_wire();
    let hb_decoded = Packet::from_wire(&hb_wire).unwrap();
    let (offset, seq) = parse_heartbeat_payload(&hb_decoded.payload).unwrap();
    assert_eq!(offset, -200);
    assert_eq!(seq, 42);

    let mut bad_wire = wire.clone();
    bad_wire[36] ^= 0xFF;
    assert!(Packet::from_wire(&bad_wire).is_err());
}

#[test]
fn e2e_bridge_resolution() {
    let mut crs = CrsRegistry::new();
    let now = 1_000_000_000u64;

    crs.register(
        "google.plm".into(), "plm".into(),
        vec![0x01], google_measurements(), now, None,
    );
    crs.register(
        "pptpro.capomastro.plm".into(), "capomastro.plm".into(),
        vec![0x02], pptpro_measurements(), now, Some(100),
    );

    let g_addr = crs.resolve_addr("google.plm").unwrap();
    let crs_shared = std::sync::Arc::new(std::sync::RwLock::new(crs));
    let mut bridge = Bridge::new(crs_shared.clone());

    match bridge.resolve("google.plm", now) {
        Resolution::Tdns { name, address, hptp_mandatory, .. } => {
            assert_eq!(name, "google.plm");
            assert!(!hptp_mandatory);
            assert_eq!(address, g_addr);
        }
        other => panic!("expected Tdns, got {:?}", other),
    }

    match bridge.resolve("pptpro.capomastro.plm", now) {
        Resolution::Tdns { hptp_mandatory, .. } => {
            assert!(hptp_mandatory);
        }
        other => panic!("expected Tdns, got {:?}", other),
    }

    assert!(is_plm_name("google.plm"));
    assert!(!is_plm_name("google.com"));

    match bridge.resolve("nonexistent.plm", now) {
        Resolution::Failed { .. } => {}
        other => panic!("expected Failed, got {:?}", other),
    }

    assert_eq!(bridge.tdns_count(), 2);
    assert_eq!(bridge.failed_count(), 1);
}

#[test]
fn e2e_drift_redirect_pipeline() {
    let mut crs = CrsRegistry::new();
    let now = 1_000_000_000u64;

    crs.register(
        "google.plm".into(), "plm".into(),
        vec![0x01], google_measurements(), now, None,
    );
    let old_addr = crs.resolve_addr("google.plm").unwrap();

    let mut changed = google_measurements();
    changed[19] = RawValue::Pattern("subscription".into());

    let later = now + 1_000_000_000;
    let result = crs.rescan("google.plm", changed, later);

    let drift_log_len = crs.drift_log().len();

    match result {
        Some(VerificationResult::Drifted { old_address, new_address, changed_dims, .. }) => {
            assert_eq!(old_address, old_addr);
            assert_ne!(old_address, new_address);
            assert!(changed_dims.contains(&19));

            let new_addr = crs.resolve_addr("google.plm").unwrap();
            assert_eq!(new_addr, new_address);

            let redirect = crs.check_redirect(&old_address, later);
            assert_eq!(redirect, Some(new_address));

            let map = NeighborMap::new(old_addr);
            let mut glb = Glb::new(old_addr, map);
            glb.set_redirect(old_address, new_address, later + 86_400_000_000_000);

            match glb.route(&old_address, later) {
                GlbDecision::Redirect { old_addr: o, new_addr: n } => {
                    assert_eq!(o, old_address);
                    assert_eq!(n, new_address);
                }
                other => panic!("expected Redirect, got {:?}", other),
            }

            let crs_shared = std::sync::Arc::new(std::sync::RwLock::new(crs));
            let mut bridge = Bridge::new(crs_shared.clone());
            match bridge.resolve("google.plm", later) {
                Resolution::Tdns { address, .. } => {
                    assert_eq!(address, new_address);
                }
                other => panic!("expected Tdns with new address, got {:?}", other),
            }
        }
        other => panic!("expected Drifted, got {:?}", other),
    }

    assert_eq!(drift_log_len, 1);
}

#[test]
fn e2e_fts_death_to_glb_reroute() {
    let config = FtsConfig {
        heartbeat_interval_ns: 1_000,
        failure_threshold: 2,
        suspect_threshold: 1,
        ..Default::default()
    };
    let mut fts = Fts::with_config(config);

    let mut crs = CrsRegistry::new();
    let now = 1_000_000_000u64;

    crs.register("google.plm".into(), "plm".into(), vec![0x01], google_measurements(), now, None);
    crs.register("pptpro.capomastro.plm".into(), "capomastro.plm".into(), vec![0x02], pptpro_measurements(), now, Some(50));
    crs.register("nonnas-cucina.plm".into(), "plm".into(), vec![0x03], blog_measurements(), now, None);

    let g_addr = crs.resolve_addr("google.plm").unwrap();
    let p_addr = crs.resolve_addr("pptpro.capomastro.plm").unwrap();
    let b_addr = crs.resolve_addr("nonnas-cucina.plm").unwrap();

    fts.process_heartbeat(&Heartbeat::new(g_addr, now, 0, 1));
    fts.process_heartbeat(&Heartbeat::new(p_addr, now, 50, 1));
    fts.process_heartbeat(&Heartbeat::new(b_addr, now, 0, 1));
    assert_eq!(fts.alive_set().len(), 3);

    fts.process_heartbeat(&Heartbeat::new(g_addr, now + 1100, 0, 2));
    fts.process_heartbeat(&Heartbeat::new(b_addr, now + 1100, 0, 2));
    fts.check(now + 1100);

    assert_eq!(fts.node_state(&p_addr), Some(&HealthState::Suspect));

    fts.check(now + 2200);
    assert_eq!(fts.node_state(&p_addr), Some(&HealthState::Dead));

    let diffs = g_addr.differing_dims(&p_addr);
    let mut map = NeighborMap::new(g_addr);
    for &dim in &diffs {
        map.set(dim, p_addr.trit(dim), p_addr);
    }
    let diffs_gb = g_addr.differing_dims(&b_addr);
    for &dim in &diffs_gb {
        if !diffs.contains(&dim) || map.get(dim, b_addr.trit(dim)).is_none() {
            map.set(dim, b_addr.trit(dim), b_addr);
        }
    }

    let mut glb = Glb::new(g_addr, map);
    glb.mark_dead(p_addr);

    match glb.forward_point(&p_addr, now + 2200) {
        GlbDecision::Forward { next_hop, .. } => {
            assert_ne!(next_hop, p_addr, "should not route to dead node");
        }
        GlbDecision::NoRoute { .. } => {
        }
        other => panic!("expected Forward or NoRoute, got {:?}", other),
    }

    let events = fts.process_heartbeat(&Heartbeat::new(p_addr, now + 3000, 50, 2));
    assert!(events.contains(&FtsEvent::NodeRecovered { addr: p_addr }));
    glb.mark_alive(p_addr);
}

#[test]
fn e2e_subcube_multicast_wire() {
    let g_addr = CubeAddr::from_category_string(
        "WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313"
    ).unwrap();

    let mut mask = [false; DIMENSIONS];
    mask[15] = true;
    let p_addr = CubeAddr::from_category_string(
        "WO:2333 WA:2333 WR:2222 WN:3333 WY:1221 HO:2133 PE:332"
    ).unwrap();
    let sc = SubCube::new(p_addr, mask);

    assert!(sc.contains(&p_addr));
    assert!(!sc.contains(&g_addr));

    let pkt = Packet::multicast(g_addr, &sc, b"alert: real-time nodes".to_vec(), 1_000_000);
    assert!(pkt.is_multicast());
    assert!(!pkt.is_point());

    let wire = pkt.to_wire();
    let decoded = Packet::from_wire(&wire).unwrap();
    assert_eq!(decoded.packet_type, PacketType::Multicast);
    assert_eq!(decoded.payload, b"alert: real-time nodes");

    let n1 = g_addr.with_trit(0, Trit::V1);
    let n2 = g_addr.with_trit(1, Trit::V1);
    let mut map = NeighborMap::new(g_addr);
    map.set(0, Trit::V1, n1);
    map.set(1, Trit::V1, n2);

    let glb = Glb::new(g_addr, map);
    match glb.forward_subcube(&sc, None, 0) {
        GlbDecision::Multicast { next_hops } => {
            assert!(!next_hops.is_empty());
        }
        GlbDecision::DeliverLocal => {
        }
        other => panic!("expected Multicast or DeliverLocal, got {:?}", other),
    }
}

#[test]
fn e2e_api_router_cycle() {
    let mut router = ApiRouter::new();

    let health = router.handle_health();
    assert_eq!(health.version, "2.3.0");
    assert_eq!(health.entities, 0);

    let desc = router.handle_describe(
        "WO:2333 WA:2333 WR:2222 WN:3333 WY:1221 HO:2133 PE:332"
    ).unwrap();
    assert!(desc.hptp_mandatory);
    assert_eq!(desc.dimensions[0].label, "Corporate");
    assert_eq!(desc.dimensions[4].label, "App");

    let fts = router.handle_fts_status();
    assert_eq!(fts.alive, 0);

    let con = router.handle_con_metrics();
    assert_eq!(con.active_tunnels, 0);

    let local = CubeAddr::from_category_string(
        "WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313"
    ).unwrap();
    router.init_glb(local);

    let route = router.handle_route(
        tdns_v2::api::RouteRequest {
            source: "WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313".into(),
            destination: "WO:2333 WA:2333 WR:2222 WN:3333 WY:1221 HO:2133 PE:332".into(),
        },
        0,
    ).unwrap();
    assert!(route.distance > 0);
    assert!(!route.differing_dims.is_empty());
}

#[test]
fn e2e_con_key_rotation() {
    let g_addr = CubeAddr::from_category_string(
        "WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313"
    ).unwrap();
    let p_addr = CubeAddr::from_category_string(
        "WO:2333 WA:2333 WR:2222 WN:3333 WY:1221 HO:2133 PE:332"
    ).unwrap();

    let secret = b"production-secret";
    let mut con = ConNode::new(g_addr, secret.to_vec());

    con.establish_tunnel(p_addr, 0, Trit::V3, 1000);
    let key_before = con.outbound_key(&p_addr).unwrap().fingerprint();

    con.rekey_all(2000);
    let key_after = con.outbound_key(&p_addr).unwrap().fingerprint();

    assert_ne!(key_before, key_after);
    assert_eq!(con.key_epoch(), 1);

    con.rekey_all(3000);
    let key_after2 = con.outbound_key(&p_addr).unwrap().fingerprint();
    assert_ne!(key_after, key_after2);
    assert_eq!(con.key_epoch(), 2);
}
