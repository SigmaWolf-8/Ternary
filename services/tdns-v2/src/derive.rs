// TDNS v2.3 — Derivation Rules (First-Principle Mathematics)
// Capomastro Holdings Ltd. — Applied Physics Division
//
// DERIVATION PRINCIPLE:
//
// Every trit is derived from one universal formula:
//
//   gf3 = min(floor(3k / N), 2)     where k = signals fired, N = total signals
//   trit = gf3 + 1                   lift from GF(3) {0,1,2} to trit {1,2,3}
//
// This is the natural ternary quantization: boundaries at exactly 1/3 and 2/3
// of the signal space. No arbitrary thresholds. No tuning parameters. The math
// determines the boundaries.
//
// Two derivation types:
//   CATEGORICAL — Scanner produces a pattern string. Direct mapping to trit.
//                 No quantization needed. Confidence always High.
//   QUANTITATIVE — Scanner counts binary signals (k out of N). project_to_gf3
//                  maps to trit. Confidence always High because inputs are binary.
//
// 15 categorical rules + 12 quantitative rules = 27 dimensions.
// Confidence is always High. There is no Medium or Low — if the measurement
// can't be determined, the scanner should not produce a value.

use crate::scan::{Confidence, DerivationError, DerivationRule, RawValue};
use crate::trit::Trit;

// ─── Universal Derivation Formula ────────────────────────────────────────────

/// Project a signal count to GF(3), then lift to trit space.
///
/// `gf3 = min(floor(3k / N), 2)`
/// `trit = gf3 + 1`
///
/// This is the ONLY quantitative derivation function in the system.
/// Every numeric dimension uses it. The boundaries between trit values
/// are at exactly N/3 and 2N/3 — derived from the definition of
/// ternary quantization, not from empirical tuning.
///
/// # Panics
/// Panics if N is 0 (no signals defined for this dimension).
pub fn project_to_gf3(k: u32, n: u32) -> Trit {
    assert!(n > 0, "N must be > 0");
    let gf3 = std::cmp::min((3 * k) / n, 2) as u8;
    Trit::from_gf3(gf3).unwrap()
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn expect_numeric(dim: usize, raw: &RawValue) -> Result<f64, DerivationError> {
    match raw {
        RawValue::Numeric(n) => Ok(*n),
        _ => Err(DerivationError::TypeMismatch { dim }),
    }
}

fn expect_pattern(dim: usize, raw: &RawValue) -> Result<&str, DerivationError> {
    match raw {
        RawValue::Pattern(s) => Ok(s.as_str()),
        _ => Err(DerivationError::TypeMismatch { dim }),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// WHO — Trits 1–4
// ═══════════════════════════════════════════════════════════════════════════

// ── Trit 1: What kind? [CATEGORICAL] ────────────────────────────────────

pub struct DeriveEntityKind;
impl DerivationRule for DeriveEntityKind {
    fn dimension(&self) -> usize { 0 }
    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let pattern = expect_pattern(0, raw)?;
        let trit = match pattern.to_lowercase().as_str() {
            "personal" | "individual" | "private" => Trit::V1,
            "corporate" | "company" | "organization" | "business" => Trit::V2,
            "governance" | "government" | "gov" | "mil" | "edu" => Trit::V3,
            _ => Trit::V2,
        };
        Ok((trit, Confidence::High))
    }
}

// ── Trit 2: Who's it for? [CATEGORICAL] ─────────────────────────────────

pub struct DeriveAudience;
impl DerivationRule for DeriveAudience {
    fn dimension(&self) -> usize { 1 }
    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let pattern = expect_pattern(1, raw)?;
        let trit = match pattern.to_lowercase().as_str() {
            "private" | "personal" | "self" => Trit::V1,
            "group" | "team" | "organization" | "internal" => Trit::V2,
            "public" | "everyone" | "open" => Trit::V3,
            _ => Trit::V3,
        };
        Ok((trit, Confidence::High))
    }
}

// ── Trit 3: Who runs it? [QUANTITATIVE, N=5] ───────────────────────────

/// 5 binary signals: about_page, contact_info, legal_entity, physical_address, gov_domain.
/// project_to_gf3 boundaries: k<2 → V1 (Anonymous), k=2-3 → V2 (Known), k≥4 → V3 (Transparent).
pub struct DeriveOperatorTransparency;
impl DerivationRule for DeriveOperatorTransparency {
    fn dimension(&self) -> usize { 2 }
    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let k = expect_numeric(2, raw)? as u32;
        Ok((project_to_gf3(k, 5), Confidence::High))
    }
}

// ── Trit 4: Who hosts it? [CATEGORICAL] ─────────────────────────────────

pub struct DeriveHostingModel;
impl DerivationRule for DeriveHostingModel {
    fn dimension(&self) -> usize { 3 }
    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let pattern = expect_pattern(3, raw)?;
        let trit = match pattern.to_lowercase().as_str() {
            "self" | "self-hosted" | "residential" | "home" => Trit::V1,
            "provider" | "vps" | "dedicated" | "hosting" | "colocation" => Trit::V2,
            "cloud" | "aws" | "azure" | "gcp" | "cloudflare" | "vercel" | "netlify" => Trit::V3,
            _ => Trit::V2,
        };
        Ok((trit, Confidence::High))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// WHAT — Trits 5–8
// ═══════════════════════════════════════════════════════════════════════════

// ── Trit 5: What is it? [CATEGORICAL] ───────────────────────────────────

pub struct DeriveFormFactor;
impl DerivationRule for DeriveFormFactor {
    fn dimension(&self) -> usize { 4 }
    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let pattern = expect_pattern(4, raw)?;
        let trit = match pattern.to_lowercase().as_str() {
            "website" | "web" | "static" | "html" => Trit::V1,
            "app" | "application" | "api" | "spa" | "webapp" => Trit::V2,
            "device" | "iot" | "mqtt" | "embedded" | "sensor" => Trit::V3,
            _ => Trit::V1,
        };
        Ok((trit, Confidence::High))
    }
}

// ── Trit 6: What's on it? [CATEGORICAL] ─────────────────────────────────

pub struct DeriveContentType;
impl DerivationRule for DeriveContentType {
    fn dimension(&self) -> usize { 5 }
    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let pattern = expect_pattern(5, raw)?;
        let trit = match pattern.to_lowercase().as_str() {
            "text" | "html" | "json" | "xml" | "markdown" => Trit::V1,
            "media" | "image" | "video" | "audio" | "multimedia" => Trit::V2,
            "live" | "stream" | "streaming" | "realtime" | "websocket" => Trit::V3,
            _ => Trit::V1,
        };
        Ok((trit, Confidence::High))
    }
}

// ── Trit 7: Who uses it? [CATEGORICAL] ──────────────────────────────────

pub struct DeriveConsumerType;
impl DerivationRule for DeriveConsumerType {
    fn dimension(&self) -> usize { 6 }
    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let pattern = expect_pattern(6, raw)?;
        let trit = match pattern.to_lowercase().as_str() {
            "people" | "human" | "ui" | "browser" => Trit::V1,
            "software" | "api" | "machine" | "bot" => Trit::V2,
            "both" | "mixed" | "hybrid" => Trit::V3,
            _ => Trit::V3,
        };
        Ok((trit, Confidence::High))
    }
}

// ── Trit 8: Does it think? [QUANTITATIVE, N=5] ─────────────────────────

/// 5 binary signals: ml_endpoints, ml_frameworks, personalization, search_ranking, ml_headers.
/// project_to_gf3 boundaries: k<2 → V1 (No), k=2-3 → V2 (Partly), k≥4 → V3 (Yes).
pub struct DeriveIntelligence;
impl DerivationRule for DeriveIntelligence {
    fn dimension(&self) -> usize { 7 }
    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let k = expect_numeric(7, raw)? as u32;
        Ok((project_to_gf3(k, 5), Confidence::High))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// WHERE — Trits 9–12
// ═══════════════════════════════════════════════════════════════════════════

// ── Trit 9: Who can see it? [QUANTITATIVE, N=3] ────────────────────────

/// 3 binary signals: site_responds, no_auth_challenge, serves_public_content.
/// project_to_gf3 boundaries: k=0 → V1 (Private), k=1 → V2 (Group), k=2-3 → V3 (Everyone).
pub struct DeriveVisibility;
impl DerivationRule for DeriveVisibility {
    fn dimension(&self) -> usize { 8 }
    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let k = expect_numeric(8, raw)? as u32;
        Ok((project_to_gf3(k, 3), Confidence::High))
    }
}

// ── Trit 10: Do I need to log in? [CATEGORICAL] ────────────────────────

pub struct DeriveAuthModel;
impl DerivationRule for DeriveAuthModel {
    fn dimension(&self) -> usize { 9 }
    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let pattern = expect_pattern(9, raw)?;
        let trit = match pattern.to_lowercase().as_str() {
            "none" | "open" | "anonymous" => Trit::V1,
            "password" | "basic" | "form" | "login" => Trit::V2,
            "mfa" | "cert" | "biometric" | "id" | "idcheck" | "id_check" => Trit::V3,
            _ => Trit::V1,
        };
        Ok((trit, Confidence::High))
    }
}

// ── Trit 11: How many servers? [QUANTITATIVE, N=6] ─────────────────────

/// 6 binary signals: dns_resolves, dns_multiple_records, dns_many_records,
/// cdn_header_present, cdn_cache_header, proxy_via_header.
/// project_to_gf3 boundaries: k<2 → V1 (One), k=2-3 → V2 (Several), k≥4 → V3 (Many).
pub struct DeriveInfraScale;
impl DerivationRule for DeriveInfraScale {
    fn dimension(&self) -> usize { 10 }
    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let k = expect_numeric(10, raw)? as u32;
        Ok((project_to_gf3(k, 6), Confidence::High))
    }
}

// ── Trit 12: What connection? [CATEGORICAL] ─────────────────────────────

pub struct DeriveConnectionProtocol;
impl DerivationRule for DeriveConnectionProtocol {
    fn dimension(&self) -> usize { 11 }
    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let pattern = expect_pattern(11, raw)?;
        let trit = match pattern.to_lowercase().as_str() {
            "http" | "https" | "http/1" | "http/2" | "http/3" => Trit::V1,
            "websocket" | "ws" | "wss" | "grpc" => Trit::V2,
            "tcp" | "udp" | "mqtt" | "raw" | "custom" => Trit::V3,
            _ => Trit::V1,
        };
        Ok((trit, Confidence::High))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// WHEN — Trits 13–16
// ═══════════════════════════════════════════════════════════════════════════

// ── Trit 13: What era? [QUANTITATIVE, N=6] ──────────────────────────────

/// 6 binary signals for modern protocol stack:
/// alt_svc, permissions_policy, nel, report_to, cross_origin_policy, csp.
/// These headers didn't exist before their era.
/// project_to_gf3 boundaries: k<2 → V1 (Pre-2010), k=2-3 → V2 (2010s), k≥4 → V3 (2020s+).
pub struct DeriveEra;
impl DerivationRule for DeriveEra {
    fn dimension(&self) -> usize { 12 }
    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let k = expect_numeric(12, raw)? as u32;
        Ok((project_to_gf3(k, 6), Confidence::High))
    }
}

// ── Trit 14: When is it available? [QUANTITATIVE, N=3] ──────────────────

/// 3 binary signals: no_maintenance_page, no_business_hours_language, uptime_claim.
/// project_to_gf3 boundaries: k=0 → V1 (Business hours), k=1 → V2 (Extended), k=2-3 → V3 (24/7).
pub struct DeriveAvailability;
impl DerivationRule for DeriveAvailability {
    fn dimension(&self) -> usize { 13 }
    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let k = expect_numeric(13, raw)? as u32;
        Ok((project_to_gf3(k, 3), Confidence::High))
    }
}

// ── Trit 15: What kind of data? [CATEGORICAL] ──────────────────────────

pub struct DeriveDataFreshness;
impl DerivationRule for DeriveDataFreshness {
    fn dimension(&self) -> usize { 14 }
    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let pattern = expect_pattern(14, raw)?;
        let trit = match pattern.to_lowercase().as_str() {
            "historical" | "archive" | "static" | "old" => Trit::V1,
            "current" | "recent" | "updated" | "periodic" => Trit::V2,
            "live" | "streaming" | "realtime" | "continuous" => Trit::V3,
            _ => Trit::V2,
        };
        Ok((trit, Confidence::High))
    }
}

// ── Trit 16: Is it real-time? [QUANTITATIVE, N=6] ──────────────────────

/// 6 binary timeliness signals:
/// dynamic_content, short_cache, websocket, sse_eventsource, grpc, streaming_content_type.
/// First 2 indicate dynamism (not batch). Last 4 indicate real-time protocols.
/// project_to_gf3 boundaries: k<2 → V1 (Batch), k=2-3 → V2 (Near-time), k≥4 → V3 (Real-time).
pub struct DeriveLatencyProfile;
impl DerivationRule for DeriveLatencyProfile {
    fn dimension(&self) -> usize { 15 }
    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let k = expect_numeric(15, raw)? as u32;
        Ok((project_to_gf3(k, 6), Confidence::High))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// WHY — Trits 17–20
// ═══════════════════════════════════════════════════════════════════════════

// ── Trit 17: Does it handle money? [CATEGORICAL] ────────────────────────

pub struct DerivePaymentModel;
impl DerivationRule for DerivePaymentModel {
    fn dimension(&self) -> usize { 16 }
    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let pattern = expect_pattern(16, raw)?;
        let trit = match pattern.to_lowercase().as_str() {
            "no" | "none" | "free" => Trit::V1,
            "accepts" | "stripe" | "paypal" | "checkout" | "merchant" => Trit::V2,
            "processes" | "bank" | "exchange" | "processor" | "fintech" => Trit::V3,
            _ => Trit::V1,
        };
        Ok((trit, Confidence::High))
    }
}

// ── Trit 18: Does it want my data? [QUANTITATIVE, N=5] ─────────────────

/// 5 binary signals: input_fields_present, signup_form, tracking_scripts,
/// cookie_consent_complex, data_sharing_scripts.
/// project_to_gf3 boundaries: k<2 → V1 (No), k=2-3 → V2 (Some), k≥4 → V3 (Lots).
pub struct DeriveDataAppetite;
impl DerivationRule for DeriveDataAppetite {
    fn dimension(&self) -> usize { 17 }
    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let k = expect_numeric(17, raw)? as u32;
        Ok((project_to_gf3(k, 5), Confidence::High))
    }
}

// ── Trit 19: Does it have policies? [QUANTITATIVE, N=5] ────────────────

/// 5 binary signals: privacy_page, terms_page, cookie_policy_link, gdpr_text, accessibility_text.
/// project_to_gf3 boundaries: k<2 → V1 (No), k=2-3 → V2 (Basic), k≥4 → V3 (Detailed).
pub struct DerivePolicyPresence;
impl DerivationRule for DerivePolicyPresence {
    fn dimension(&self) -> usize { 18 }
    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let k = expect_numeric(18, raw)? as u32;
        Ok((project_to_gf3(k, 5), Confidence::High))
    }
}

// ── Trit 20: Does it cost money? [CATEGORICAL] ─────────────────────────

pub struct DeriveCostModel;
impl DerivationRule for DeriveCostModel {
    fn dimension(&self) -> usize { 19 }
    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let pattern = expect_pattern(19, raw)?;
        let trit = match pattern.to_lowercase().as_str() {
            "free" | "open" | "gratis" => Trit::V1,
            "payperuse" | "pay-per-use" | "metered" | "usage" => Trit::V2,
            "subscription" | "recurring" | "monthly" | "annual" | "saas" => Trit::V3,
            _ => Trit::V1,
        };
        Ok((trit, Confidence::High))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// HOW — Trits 21–24
// ═══════════════════════════════════════════════════════════════════════════

// ── Trit 21: Who gets it? [CATEGORICAL] ─────────────────────────────────

pub struct DeriveDeliveryModel;
impl DerivationRule for DeriveDeliveryModel {
    fn dimension(&self) -> usize { 20 }
    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let pattern = expect_pattern(20, raw)?;
        let trit = match pattern.to_lowercase().as_str() {
            "unicast" | "direct" | "one" => Trit::V1,
            "multicast" | "group" | "fanout" | "broadcast" => Trit::V2,
            "anycast" | "cdn" | "edge" | "closest" | "geo" => Trit::V3,
            _ => Trit::V1,
        };
        Ok((trit, Confidence::High))
    }
}

// ── Trit 22: Which way does data go? [CATEGORICAL] ─────────────────────

pub struct DeriveDataFlow;
impl DerivationRule for DeriveDataFlow {
    fn dimension(&self) -> usize { 21 }
    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let pattern = expect_pattern(21, raw)?;
        let trit = match pattern.to_lowercase().as_str() {
            "out" | "publish" | "serve" | "emit" | "source" => Trit::V1,
            "through" | "proxy" | "relay" | "pass" | "cdn" => Trit::V2,
            "in" | "ingest" | "collect" | "receive" | "sink" | "log" => Trit::V3,
            _ => Trit::V1,
        };
        Ok((trit, Confidence::High))
    }
}

// ── Trit 23: How do I get updates? [CATEGORICAL] ───────────────────────

pub struct DeriveUpdateModel;
impl DerivationRule for DeriveUpdateModel {
    fn dimension(&self) -> usize { 22 }
    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let pattern = expect_pattern(22, raw)?;
        let trit = match pattern.to_lowercase().as_str() {
            "poll" | "request" | "pull" | "manual" => Trit::V1,
            "subscribe" | "rss" | "atom" | "feed" | "email" => Trit::V2,
            "push" | "websocket" | "sse" | "notification" | "realtime" => Trit::V3,
            _ => Trit::V1,
        };
        Ok((trit, Confidence::High))
    }
}

// ── Trit 24: Does it remember me? [QUANTITATIVE, N=3] ──────────────────

/// 3 binary signals: has_any_cookie, has_persistent_cookie, has_long_lived_cookie.
/// project_to_gf3 boundaries: k=0 → V1 (No), k=1 → V2 (For a bit), k=2-3 → V3 (Always).
pub struct DeriveStatePersistence;
impl DerivationRule for DeriveStatePersistence {
    fn dimension(&self) -> usize { 23 }
    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let k = expect_numeric(23, raw)? as u32;
        Ok((project_to_gf3(k, 3), Confidence::High))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PEACE — Trits 25–27
// ═══════════════════════════════════════════════════════════════════════════

// ── Trit 25: Is it encrypted? [QUANTITATIVE, N=6] ──────────────────────

/// 6 binary signals: tls_present, hsts_header, csp_header,
/// security_txt, x_content_type_options, x_frame_options.
/// project_to_gf3 boundaries: k<2 → V1 (No), k=2-3 → V2 (Basic TLS), k≥4 → V3 (Full TLS).
/// NOTE: Measures entity-level encryption, not CON fabric transport (§2.6).
pub struct DeriveEncryption;
impl DerivationRule for DeriveEncryption {
    fn dimension(&self) -> usize { 24 }
    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let k = expect_numeric(24, raw)? as u32;
        Ok((project_to_gf3(k, 6), Confidence::High))
    }
}

// ── Trit 26: How many trackers? [QUANTITATIVE, N=5, INVERTED] ──────────

/// 5 binary CLEAN signals: no_analytics, no_social_trackers, no_ad_trackers,
/// no_session_replay, no_crm_trackers.
/// Each signal fires when a tracker CATEGORY is ABSENT.
/// More clean signals = fewer trackers = higher trit (better trust).
/// project_to_gf3 boundaries: k<2 → V1 (Many), k=2-3 → V2 (Few), k≥4 → V3 (None).
/// The inversion is structural — the probe counts absences, not presences.
pub struct DeriveTrackerCount;
impl DerivationRule for DeriveTrackerCount {
    fn dimension(&self) -> usize { 25 }
    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let k = expect_numeric(25, raw)? as u32;
        Ok((project_to_gf3(k, 5), Confidence::High))
    }
}

// ── Trit 27: Has it been audited? [CATEGORICAL] ────────────────────────

pub struct DeriveAuditStatus;
impl DerivationRule for DeriveAuditStatus {
    fn dimension(&self) -> usize { 26 }
    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let pattern = expect_pattern(26, raw)?;
        let trit = match pattern.to_lowercase().as_str() {
            "no" | "none" | "unknown" => Trit::V1,
            "self" | "self-certified" | "claimed" | "badge" => Trit::V2,
            "audited" | "soc2" | "iso27001" | "verified" | "certified" => Trit::V3,
            _ => Trit::V1,
        };
        Ok((trit, Confidence::High))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Registry
// ═══════════════════════════════════════════════════════════════════════════

/// All 27 derivation rules in dimension order.
pub fn all_rules() -> Vec<Box<dyn DerivationRule>> {
    vec![
        Box::new(DeriveEntityKind),              // 1  WHO  CATEGORICAL
        Box::new(DeriveAudience),                // 2  WHO  CATEGORICAL
        Box::new(DeriveOperatorTransparency),    // 3  WHO  QUANTITATIVE N=5
        Box::new(DeriveHostingModel),            // 4  WHO  CATEGORICAL
        Box::new(DeriveFormFactor),              // 5  WHAT CATEGORICAL
        Box::new(DeriveContentType),             // 6  WHAT CATEGORICAL
        Box::new(DeriveConsumerType),            // 7  WHAT CATEGORICAL
        Box::new(DeriveIntelligence),            // 8  WHAT QUANTITATIVE N=5
        Box::new(DeriveVisibility),              // 9  WHERE QUANTITATIVE N=3
        Box::new(DeriveAuthModel),               // 10 WHERE CATEGORICAL
        Box::new(DeriveInfraScale),              // 11 WHERE QUANTITATIVE N=6
        Box::new(DeriveConnectionProtocol),      // 12 WHERE CATEGORICAL
        Box::new(DeriveEra),                     // 13 WHEN QUANTITATIVE N=6
        Box::new(DeriveAvailability),            // 14 WHEN QUANTITATIVE N=3
        Box::new(DeriveDataFreshness),           // 15 WHEN CATEGORICAL
        Box::new(DeriveLatencyProfile),          // 16 WHEN QUANTITATIVE N=6
        Box::new(DerivePaymentModel),            // 17 WHY  CATEGORICAL
        Box::new(DeriveDataAppetite),            // 18 WHY  QUANTITATIVE N=5
        Box::new(DerivePolicyPresence),          // 19 WHY  QUANTITATIVE N=5
        Box::new(DeriveCostModel),               // 20 WHY  CATEGORICAL
        Box::new(DeriveDeliveryModel),           // 21 HOW  CATEGORICAL
        Box::new(DeriveDataFlow),                // 22 HOW  CATEGORICAL
        Box::new(DeriveUpdateModel),             // 23 HOW  CATEGORICAL
        Box::new(DeriveStatePersistence),        // 24 HOW  QUANTITATIVE N=3
        Box::new(DeriveEncryption),              // 25 PEACE QUANTITATIVE N=6
        Box::new(DeriveTrackerCount),            // 26 PEACE QUANTITATIVE N=5
        Box::new(DeriveAuditStatus),             // 27 PEACE CATEGORICAL
    ]
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Universal Formula Tests ──────────────────────────────────────

    #[test]
    fn gf3_projection_n5() {
        assert_eq!(project_to_gf3(0, 5), Trit::V1);
        assert_eq!(project_to_gf3(1, 5), Trit::V1);
        assert_eq!(project_to_gf3(2, 5), Trit::V2);
        assert_eq!(project_to_gf3(3, 5), Trit::V2);
        assert_eq!(project_to_gf3(4, 5), Trit::V3);
        assert_eq!(project_to_gf3(5, 5), Trit::V3);
    }

    #[test]
    fn gf3_projection_n6() {
        assert_eq!(project_to_gf3(0, 6), Trit::V1);
        assert_eq!(project_to_gf3(1, 6), Trit::V1);
        assert_eq!(project_to_gf3(2, 6), Trit::V2);
        assert_eq!(project_to_gf3(3, 6), Trit::V2);
        assert_eq!(project_to_gf3(4, 6), Trit::V3);
        assert_eq!(project_to_gf3(5, 6), Trit::V3);
        assert_eq!(project_to_gf3(6, 6), Trit::V3);
    }

    #[test]
    fn gf3_projection_n3() {
        assert_eq!(project_to_gf3(0, 3), Trit::V1);
        assert_eq!(project_to_gf3(1, 3), Trit::V2);
        assert_eq!(project_to_gf3(2, 3), Trit::V3);
        assert_eq!(project_to_gf3(3, 3), Trit::V3);
    }

    #[test]
    fn gf3_k_exceeds_n() {
        assert_eq!(project_to_gf3(10, 5), Trit::V3);
        assert_eq!(project_to_gf3(100, 6), Trit::V3);
    }

    // ── Rule Coverage ────────────────────────────────────────────────

    #[test]
    fn all_rules_cover_all_dims() {
        let rules = all_rules();
        assert_eq!(rules.len(), 27);
        for (i, rule) in rules.iter().enumerate() {
            assert_eq!(rule.dimension(), i);
        }
    }

    // ── Categorical Rule Tests ───────────────────────────────────────

    #[test]
    fn entity_kind_categorical() {
        let rule = DeriveEntityKind;
        let (t, c) = rule.derive(&RawValue::Pattern("government".into())).unwrap();
        assert_eq!(t, Trit::V3);
        assert_eq!(c, Confidence::High);

        let (t, c) = rule.derive(&RawValue::Pattern("personal".into())).unwrap();
        assert_eq!(t, Trit::V1);
        assert_eq!(c, Confidence::High);
    }

    #[test]
    fn type_mismatch_errors() {
        let rule = DeriveEntityKind;
        assert!(rule.derive(&RawValue::Numeric(42.0)).is_err());
    }

    // ── Quantitative Rule Tests ─────────────────────────────────────

    #[test]
    fn transparency_signals() {
        let rule = DeriveOperatorTransparency;
        let (t, c) = rule.derive(&RawValue::Numeric(0.0)).unwrap();
        assert_eq!(t, Trit::V1);
        assert_eq!(c, Confidence::High);

        let (t, _) = rule.derive(&RawValue::Numeric(2.0)).unwrap();
        assert_eq!(t, Trit::V2);

        let (t, _) = rule.derive(&RawValue::Numeric(5.0)).unwrap();
        assert_eq!(t, Trit::V3);
    }

    #[test]
    fn intelligence_signals() {
        let rule = DeriveIntelligence;
        let (t, _) = rule.derive(&RawValue::Numeric(0.0)).unwrap();
        assert_eq!(t, Trit::V1);

        let (t, _) = rule.derive(&RawValue::Numeric(2.0)).unwrap();
        assert_eq!(t, Trit::V2);

        let (t, _) = rule.derive(&RawValue::Numeric(4.0)).unwrap();
        assert_eq!(t, Trit::V3);
    }

    #[test]
    fn visibility_signals() {
        let rule = DeriveVisibility;
        let (t, _) = rule.derive(&RawValue::Numeric(0.0)).unwrap();
        assert_eq!(t, Trit::V1);

        let (t, _) = rule.derive(&RawValue::Numeric(1.0)).unwrap();
        assert_eq!(t, Trit::V2);

        let (t, _) = rule.derive(&RawValue::Numeric(3.0)).unwrap();
        assert_eq!(t, Trit::V3);
    }

    #[test]
    fn encryption_signals() {
        let rule = DeriveEncryption;
        let (t, c) = rule.derive(&RawValue::Numeric(0.0)).unwrap();
        assert_eq!(t, Trit::V1);
        assert_eq!(c, Confidence::High);

        let (t, _) = rule.derive(&RawValue::Numeric(2.0)).unwrap();
        assert_eq!(t, Trit::V2);

        let (t, _) = rule.derive(&RawValue::Numeric(5.0)).unwrap();
        assert_eq!(t, Trit::V3);
    }

    #[test]
    fn latency_signals() {
        let rule = DeriveLatencyProfile;
        let (t, c) = rule.derive(&RawValue::Numeric(0.0)).unwrap();
        assert_eq!(t, Trit::V1);
        assert_eq!(c, Confidence::High);

        let (t, _) = rule.derive(&RawValue::Numeric(3.0)).unwrap();
        assert_eq!(t, Trit::V2);

        let (t, _) = rule.derive(&RawValue::Numeric(5.0)).unwrap();
        assert_eq!(t, Trit::V3);
    }

    #[test]
    fn tracker_clean_signals() {
        let rule = DeriveTrackerCount;
        let (t, _) = rule.derive(&RawValue::Numeric(0.0)).unwrap();
        assert_eq!(t, Trit::V1);

        let (t, _) = rule.derive(&RawValue::Numeric(3.0)).unwrap();
        assert_eq!(t, Trit::V2);

        let (t, _) = rule.derive(&RawValue::Numeric(5.0)).unwrap();
        assert_eq!(t, Trit::V3);
    }

    #[test]
    fn state_persistence_signals() {
        let rule = DeriveStatePersistence;
        let (t, _) = rule.derive(&RawValue::Numeric(0.0)).unwrap();
        assert_eq!(t, Trit::V1);

        let (t, _) = rule.derive(&RawValue::Numeric(1.0)).unwrap();
        assert_eq!(t, Trit::V2);

        let (t, _) = rule.derive(&RawValue::Numeric(3.0)).unwrap();
        assert_eq!(t, Trit::V3);
    }

    #[test]
    fn era_signals() {
        let rule = DeriveEra;
        let (t, _) = rule.derive(&RawValue::Numeric(0.0)).unwrap();
        assert_eq!(t, Trit::V1);

        let (t, _) = rule.derive(&RawValue::Numeric(3.0)).unwrap();
        assert_eq!(t, Trit::V2);

        let (t, _) = rule.derive(&RawValue::Numeric(5.0)).unwrap();
        assert_eq!(t, Trit::V3);
    }

    #[test]
    fn all_quantitative_always_high_confidence() {
        let rules = all_rules();
        for k in 0..=10u32 {
            for rule in &rules {
                if let Ok((_, conf)) = rule.derive(&RawValue::Numeric(k as f64)) {
                    assert_eq!(conf, Confidence::High,
                        "dim {} with k={} should be High confidence", rule.dimension(), k);
                }
                if let Ok((_, conf)) = rule.derive(&RawValue::Pattern("test".into())) {
                    assert_eq!(conf, Confidence::High,
                        "dim {} with pattern should be High confidence", rule.dimension());
                }
            }
        }
    }
}
