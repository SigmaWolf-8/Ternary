// TDNS v2.3 — Derivation Rules
// Capomastro Holdings Ltd. — Applied Physics Division
//
// 27 dimension-specific derivation rules. Each converts a raw CRS
// scan measurement into a trit value. This is the zero-human-input
// engine — every trit is derived from observable, testable signals.
//
// Each rule implements the DerivationRule trait from scan.rs.

use crate::scan::{Confidence, DerivationError, DerivationRule, RawValue};
use crate::trit::Trit;

// ─── Helper ──────────────────────────────────────────────────────────────────

fn expect_numeric(dim: usize, raw: &RawValue) -> Result<f64, DerivationError> {
    match raw {
        RawValue::Numeric(n) => Ok(*n),
        _ => Err(DerivationError::TypeMismatch { dim }),
    }
}

#[allow(dead_code)]
fn expect_text(dim: usize, raw: &RawValue) -> Result<&str, DerivationError> {
    match raw {
        RawValue::Text(s) => Ok(s.as_str()),
        _ => Err(DerivationError::TypeMismatch { dim }),
    }
}

#[allow(dead_code)]
fn expect_bool(dim: usize, raw: &RawValue) -> Result<bool, DerivationError> {
    match raw {
        RawValue::Boolean(b) => Ok(*b),
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

pub struct DeriveEntityKind;

impl DerivationRule for DeriveEntityKind {
    fn dimension(&self) -> usize { 0 }

    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let pattern = expect_pattern(0, raw)?;
        match pattern.to_lowercase().as_str() {
            "personal" | "individual" | "private" => Ok((Trit::V1, Confidence::High)),
            "corporate" | "company" | "organization" | "business" => Ok((Trit::V2, Confidence::High)),
            "governance" | "government" | "gov" | "mil" | "edu" => Ok((Trit::V3, Confidence::High)),
            _ => Ok((Trit::V2, Confidence::Low)),
        }
    }
}

pub struct DeriveAudience;

impl DerivationRule for DeriveAudience {
    fn dimension(&self) -> usize { 1 }

    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let pattern = expect_pattern(1, raw)?;
        match pattern.to_lowercase().as_str() {
            "private" | "personal" | "self" => Ok((Trit::V1, Confidence::High)),
            "group" | "team" | "organization" | "internal" => Ok((Trit::V2, Confidence::High)),
            "public" | "everyone" | "open" => Ok((Trit::V3, Confidence::High)),
            _ => Ok((Trit::V3, Confidence::Medium)),
        }
    }
}

pub struct DeriveOperatorTransparency;

impl DerivationRule for DeriveOperatorTransparency {
    fn dimension(&self) -> usize { 2 }

    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let score = expect_numeric(2, raw)?;
        let trit = if score < 0.34 {
            Trit::V1
        } else if score < 0.67 {
            Trit::V2
        } else {
            Trit::V3
        };
        let confidence = if score < 0.1 || score > 0.9 {
            Confidence::High
        } else if score < 0.25 || score > 0.75 {
            Confidence::Medium
        } else {
            Confidence::Low
        };
        Ok((trit, confidence))
    }
}

pub struct DeriveHostingModel;

impl DerivationRule for DeriveHostingModel {
    fn dimension(&self) -> usize { 3 }

    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let pattern = expect_pattern(3, raw)?;
        match pattern.to_lowercase().as_str() {
            "self" | "self-hosted" | "residential" | "home" => Ok((Trit::V1, Confidence::High)),
            "provider" | "vps" | "dedicated" | "hosting" | "colocation" => Ok((Trit::V2, Confidence::High)),
            "cloud" | "aws" | "azure" | "gcp" | "cloudflare" | "vercel" | "netlify" => Ok((Trit::V3, Confidence::High)),
            _ => Ok((Trit::V2, Confidence::Low)),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// WHAT — Trits 5–8
// ═══════════════════════════════════════════════════════════════════════════

pub struct DeriveFormFactor;

impl DerivationRule for DeriveFormFactor {
    fn dimension(&self) -> usize { 4 }

    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let pattern = expect_pattern(4, raw)?;
        match pattern.to_lowercase().as_str() {
            "website" | "web" | "static" | "html" => Ok((Trit::V1, Confidence::High)),
            "app" | "application" | "api" | "spa" | "webapp" => Ok((Trit::V2, Confidence::High)),
            "device" | "iot" | "mqtt" | "embedded" | "sensor" => Ok((Trit::V3, Confidence::High)),
            _ => Ok((Trit::V1, Confidence::Low)),
        }
    }
}

pub struct DeriveContentType;

impl DerivationRule for DeriveContentType {
    fn dimension(&self) -> usize { 5 }

    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let pattern = expect_pattern(5, raw)?;
        match pattern.to_lowercase().as_str() {
            "text" | "html" | "json" | "xml" | "markdown" => Ok((Trit::V1, Confidence::High)),
            "media" | "image" | "video" | "audio" | "multimedia" => Ok((Trit::V2, Confidence::High)),
            "live" | "stream" | "streaming" | "realtime" | "websocket" => Ok((Trit::V3, Confidence::High)),
            _ => Ok((Trit::V1, Confidence::Medium)),
        }
    }
}

pub struct DeriveConsumerType;

impl DerivationRule for DeriveConsumerType {
    fn dimension(&self) -> usize { 6 }

    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let pattern = expect_pattern(6, raw)?;
        match pattern.to_lowercase().as_str() {
            "people" | "human" | "ui" | "browser" => Ok((Trit::V1, Confidence::High)),
            "software" | "api" | "machine" | "bot" => Ok((Trit::V2, Confidence::High)),
            "both" | "mixed" | "hybrid" => Ok((Trit::V3, Confidence::High)),
            _ => Ok((Trit::V3, Confidence::Low)),
        }
    }
}

pub struct DeriveIntelligence;

impl DerivationRule for DeriveIntelligence {
    fn dimension(&self) -> usize { 7 }

    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let score = expect_numeric(7, raw)?;
        let trit = if score < 0.34 {
            Trit::V1
        } else if score < 0.67 {
            Trit::V2
        } else {
            Trit::V3
        };
        let confidence = if score < 0.1 || score > 0.9 {
            Confidence::High
        } else {
            Confidence::Medium
        };
        Ok((trit, confidence))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// WHERE — Trits 9–12
// ═══════════════════════════════════════════════════════════════════════════

pub struct DeriveVisibility;

impl DerivationRule for DeriveVisibility {
    fn dimension(&self) -> usize { 8 }

    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let status = expect_numeric(8, raw)? as u16;
        match status {
            200..=299 => Ok((Trit::V3, Confidence::High)),
            401 | 403 => Ok((Trit::V2, Confidence::High)),
            0 => Ok((Trit::V1, Confidence::High)),
            _ => Ok((Trit::V2, Confidence::Medium)),
        }
    }
}

pub struct DeriveAuthModel;

impl DerivationRule for DeriveAuthModel {
    fn dimension(&self) -> usize { 9 }

    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let pattern = expect_pattern(9, raw)?;
        match pattern.to_lowercase().as_str() {
            "none" | "open" | "anonymous" => Ok((Trit::V1, Confidence::High)),
            "password" | "basic" | "form" | "login" => Ok((Trit::V2, Confidence::High)),
            "mfa" | "cert" | "biometric" | "id" | "idcheck" | "id_check" => Ok((Trit::V3, Confidence::High)),
            _ => Ok((Trit::V1, Confidence::Low)),
        }
    }
}

pub struct DeriveInfraScale;

impl DerivationRule for DeriveInfraScale {
    fn dimension(&self) -> usize { 10 }

    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let count = expect_numeric(10, raw)? as u32;
        let trit = match count {
            0..=1 => Trit::V1,
            2..=5 => Trit::V2,
            _ => Trit::V3,
        };
        Ok((trit, Confidence::High))
    }
}

pub struct DeriveConnectionProtocol;

impl DerivationRule for DeriveConnectionProtocol {
    fn dimension(&self) -> usize { 11 }

    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let pattern = expect_pattern(11, raw)?;
        match pattern.to_lowercase().as_str() {
            "http" | "https" | "http/1" | "http/2" | "http/3" => Ok((Trit::V1, Confidence::High)),
            "websocket" | "ws" | "wss" | "grpc" => Ok((Trit::V2, Confidence::High)),
            "tcp" | "udp" | "mqtt" | "raw" | "custom" => Ok((Trit::V3, Confidence::High)),
            _ => Ok((Trit::V1, Confidence::Low)),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// WHEN — Trits 13–16
// ═══════════════════════════════════════════════════════════════════════════

pub struct DeriveEra;

impl DerivationRule for DeriveEra {
    fn dimension(&self) -> usize { 12 }

    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let year = expect_numeric(12, raw)? as u32;
        let trit = match year {
            0..=2009 => Trit::V1,
            2010..=2019 => Trit::V2,
            _ => Trit::V3,
        };
        Ok((trit, Confidence::High))
    }
}

pub struct DeriveAvailability;

impl DerivationRule for DeriveAvailability {
    fn dimension(&self) -> usize { 13 }

    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let uptime_pct = expect_numeric(13, raw)?;
        let trit = if uptime_pct < 50.0 {
            Trit::V1
        } else if uptime_pct < 95.0 {
            Trit::V2
        } else {
            Trit::V3
        };
        let confidence = if uptime_pct < 30.0 || uptime_pct > 99.0 {
            Confidence::High
        } else {
            Confidence::Medium
        };
        Ok((trit, confidence))
    }
}

pub struct DeriveDataFreshness;

impl DerivationRule for DeriveDataFreshness {
    fn dimension(&self) -> usize { 14 }

    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let pattern = expect_pattern(14, raw)?;
        match pattern.to_lowercase().as_str() {
            "historical" | "archive" | "static" | "old" => Ok((Trit::V1, Confidence::High)),
            "current" | "recent" | "updated" | "periodic" => Ok((Trit::V2, Confidence::High)),
            "live" | "streaming" | "realtime" | "continuous" => Ok((Trit::V3, Confidence::High)),
            _ => Ok((Trit::V2, Confidence::Medium)),
        }
    }
}

pub struct DeriveLatencyProfile;

impl DerivationRule for DeriveLatencyProfile {
    fn dimension(&self) -> usize { 15 }

    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let latency_ms = expect_numeric(15, raw)?;
        let trit = if latency_ms > 5000.0 {
            Trit::V1
        } else if latency_ms >= 100.0 {
            Trit::V2
        } else {
            Trit::V3
        };
        let confidence = if latency_ms > 10000.0 || latency_ms < 10.0 {
            Confidence::High
        } else {
            Confidence::Medium
        };
        Ok((trit, confidence))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// WHY — Trits 17–20
// ═══════════════════════════════════════════════════════════════════════════

pub struct DerivePaymentModel;

impl DerivationRule for DerivePaymentModel {
    fn dimension(&self) -> usize { 16 }

    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let pattern = expect_pattern(16, raw)?;
        match pattern.to_lowercase().as_str() {
            "no" | "none" | "free" => Ok((Trit::V1, Confidence::High)),
            "accepts" | "stripe" | "paypal" | "checkout" | "merchant" => Ok((Trit::V2, Confidence::High)),
            "processes" | "bank" | "exchange" | "processor" | "fintech" => Ok((Trit::V3, Confidence::High)),
            _ => Ok((Trit::V1, Confidence::Medium)),
        }
    }
}

pub struct DeriveDataAppetite;

impl DerivationRule for DeriveDataAppetite {
    fn dimension(&self) -> usize { 17 }

    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let score = expect_numeric(17, raw)?;
        let trit = if score < 1.0 {
            Trit::V1
        } else if score < 6.0 {
            Trit::V2
        } else {
            Trit::V3
        };
        Ok((trit, Confidence::High))
    }
}

pub struct DerivePolicyPresence;

impl DerivationRule for DerivePolicyPresence {
    fn dimension(&self) -> usize { 18 }

    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let count = expect_numeric(18, raw)? as u32;
        let trit = match count {
            0 => Trit::V1,
            1..=2 => Trit::V2,
            _ => Trit::V3,
        };
        Ok((trit, Confidence::High))
    }
}

pub struct DeriveCostModel;

impl DerivationRule for DeriveCostModel {
    fn dimension(&self) -> usize { 19 }

    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let pattern = expect_pattern(19, raw)?;
        match pattern.to_lowercase().as_str() {
            "free" | "open" | "gratis" => Ok((Trit::V1, Confidence::High)),
            "payperuse" | "pay-per-use" | "metered" | "usage" => Ok((Trit::V2, Confidence::High)),
            "subscription" | "recurring" | "monthly" | "annual" | "saas" => Ok((Trit::V3, Confidence::High)),
            _ => Ok((Trit::V1, Confidence::Low)),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// HOW — Trits 21–24
// ═══════════════════════════════════════════════════════════════════════════

pub struct DeriveDeliveryModel;

impl DerivationRule for DeriveDeliveryModel {
    fn dimension(&self) -> usize { 20 }

    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let pattern = expect_pattern(20, raw)?;
        match pattern.to_lowercase().as_str() {
            "unicast" | "direct" | "one" => Ok((Trit::V1, Confidence::High)),
            "multicast" | "group" | "fanout" | "broadcast" => Ok((Trit::V2, Confidence::High)),
            "anycast" | "cdn" | "edge" | "closest" | "geo" => Ok((Trit::V3, Confidence::High)),
            _ => Ok((Trit::V1, Confidence::Medium)),
        }
    }
}

pub struct DeriveDataFlow;

impl DerivationRule for DeriveDataFlow {
    fn dimension(&self) -> usize { 21 }

    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let pattern = expect_pattern(21, raw)?;
        match pattern.to_lowercase().as_str() {
            "out" | "publish" | "serve" | "emit" | "source" => Ok((Trit::V1, Confidence::High)),
            "through" | "proxy" | "relay" | "pass" | "cdn" => Ok((Trit::V2, Confidence::High)),
            "in" | "ingest" | "collect" | "receive" | "sink" | "log" => Ok((Trit::V3, Confidence::High)),
            _ => Ok((Trit::V1, Confidence::Medium)),
        }
    }
}

pub struct DeriveUpdateModel;

impl DerivationRule for DeriveUpdateModel {
    fn dimension(&self) -> usize { 22 }

    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let pattern = expect_pattern(22, raw)?;
        match pattern.to_lowercase().as_str() {
            "poll" | "request" | "pull" | "manual" => Ok((Trit::V1, Confidence::High)),
            "subscribe" | "rss" | "atom" | "feed" | "email" => Ok((Trit::V2, Confidence::High)),
            "push" | "websocket" | "sse" | "notification" | "realtime" => Ok((Trit::V3, Confidence::High)),
            _ => Ok((Trit::V1, Confidence::Low)),
        }
    }
}

pub struct DeriveStatePersistence;

impl DerivationRule for DeriveStatePersistence {
    fn dimension(&self) -> usize { 23 }

    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let lifetime_secs = expect_numeric(23, raw)?;
        let trit = if lifetime_secs <= 0.0 {
            Trit::V1
        } else if lifetime_secs <= 86400.0 {
            Trit::V2
        } else {
            Trit::V3
        };
        Ok((trit, Confidence::High))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PEACE — Trits 25–27
// ═══════════════════════════════════════════════════════════════════════════

pub struct DeriveEncryption;

impl DerivationRule for DeriveEncryption {
    fn dimension(&self) -> usize { 24 }

    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let score = expect_numeric(24, raw)?;
        let trit = if score < 0.34 {
            Trit::V1
        } else if score < 0.67 {
            Trit::V2
        } else {
            Trit::V3
        };
        let confidence = if score < 0.1 || score > 0.9 {
            Confidence::High
        } else {
            Confidence::Medium
        };
        Ok((trit, confidence))
    }
}

pub struct DeriveTrackerCount;

impl DerivationRule for DeriveTrackerCount {
    fn dimension(&self) -> usize { 25 }

    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let count = expect_numeric(25, raw)? as u32;
        let trit = match count {
            10.. => Trit::V1,
            1..=9 => Trit::V2,
            0 => Trit::V3,
        };
        Ok((trit, Confidence::High))
    }
}

pub struct DeriveAuditStatus;

impl DerivationRule for DeriveAuditStatus {
    fn dimension(&self) -> usize { 26 }

    fn derive(&self, raw: &RawValue) -> Result<(Trit, Confidence), DerivationError> {
        let pattern = expect_pattern(26, raw)?;
        match pattern.to_lowercase().as_str() {
            "no" | "none" | "unknown" => Ok((Trit::V1, Confidence::High)),
            "self" | "self-certified" | "claimed" | "badge" => Ok((Trit::V2, Confidence::Medium)),
            "audited" | "soc2" | "iso27001" | "verified" | "certified" => Ok((Trit::V3, Confidence::High)),
            _ => Ok((Trit::V1, Confidence::Low)),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Registry — All 27 rules in order
// ═══════════════════════════════════════════════════════════════════════════

pub fn all_rules() -> Vec<Box<dyn DerivationRule>> {
    vec![
        // WHO (1–4)
        Box::new(DeriveEntityKind),
        Box::new(DeriveAudience),
        Box::new(DeriveOperatorTransparency),
        Box::new(DeriveHostingModel),
        // WHAT (5–8)
        Box::new(DeriveFormFactor),
        Box::new(DeriveContentType),
        Box::new(DeriveConsumerType),
        Box::new(DeriveIntelligence),
        // WHERE (9–12)
        Box::new(DeriveVisibility),
        Box::new(DeriveAuthModel),
        Box::new(DeriveInfraScale),
        Box::new(DeriveConnectionProtocol),
        // WHEN (13–16)
        Box::new(DeriveEra),
        Box::new(DeriveAvailability),
        Box::new(DeriveDataFreshness),
        Box::new(DeriveLatencyProfile),
        // WHY (17–20)
        Box::new(DerivePaymentModel),
        Box::new(DeriveDataAppetite),
        Box::new(DerivePolicyPresence),
        Box::new(DeriveCostModel),
        // HOW (21–24)
        Box::new(DeriveDeliveryModel),
        Box::new(DeriveDataFlow),
        Box::new(DeriveUpdateModel),
        Box::new(DeriveStatePersistence),
        // PEACE (25–27)
        Box::new(DeriveEncryption),
        Box::new(DeriveTrackerCount),
        Box::new(DeriveAuditStatus),
    ]
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_rules_cover_all_dims() {
        let rules = all_rules();
        assert_eq!(rules.len(), 27);
        for (i, rule) in rules.iter().enumerate() {
            assert_eq!(rule.dimension(), i, "rule {} covers wrong dimension", i);
        }
    }

    #[test]
    fn entity_kind_derivation() {
        let rule = DeriveEntityKind;
        let (t, c) = rule.derive(&RawValue::Pattern("government".into())).unwrap();
        assert_eq!(t, Trit::V3);
        assert_eq!(c, Confidence::High);

        let (t, _) = rule.derive(&RawValue::Pattern("personal".into())).unwrap();
        assert_eq!(t, Trit::V1);
    }

    #[test]
    fn era_derivation() {
        let rule = DeriveEra;
        let (t, _) = rule.derive(&RawValue::Numeric(2005.0)).unwrap();
        assert_eq!(t, Trit::V1);

        let (t, _) = rule.derive(&RawValue::Numeric(2015.0)).unwrap();
        assert_eq!(t, Trit::V2);

        let (t, _) = rule.derive(&RawValue::Numeric(2024.0)).unwrap();
        assert_eq!(t, Trit::V3);
    }

    #[test]
    fn tracker_count_reversed_scale() {
        let rule = DeriveTrackerCount;
        let (t, _) = rule.derive(&RawValue::Numeric(50.0)).unwrap();
        assert_eq!(t, Trit::V1);

        let (t, _) = rule.derive(&RawValue::Numeric(3.0)).unwrap();
        assert_eq!(t, Trit::V2);

        let (t, _) = rule.derive(&RawValue::Numeric(0.0)).unwrap();
        assert_eq!(t, Trit::V3);
    }

    #[test]
    fn latency_profile_thresholds() {
        let rule = DeriveLatencyProfile;
        let (t, _) = rule.derive(&RawValue::Numeric(10000.0)).unwrap();
        assert_eq!(t, Trit::V1);

        let (t, _) = rule.derive(&RawValue::Numeric(500.0)).unwrap();
        assert_eq!(t, Trit::V2);

        let (t, _) = rule.derive(&RawValue::Numeric(5.0)).unwrap();
        assert_eq!(t, Trit::V3);
    }

    #[test]
    fn visibility_from_status_code() {
        let rule = DeriveVisibility;
        let (t, _) = rule.derive(&RawValue::Numeric(200.0)).unwrap();
        assert_eq!(t, Trit::V3);

        let (t, _) = rule.derive(&RawValue::Numeric(401.0)).unwrap();
        assert_eq!(t, Trit::V2);

        let (t, _) = rule.derive(&RawValue::Numeric(0.0)).unwrap();
        assert_eq!(t, Trit::V1);
    }

    #[test]
    fn encryption_measures_entity_not_fabric() {
        let rule = DeriveEncryption;

        let (t, _) = rule.derive(&RawValue::Numeric(0.0)).unwrap();
        assert_eq!(t, Trit::V1);

        let (t, _) = rule.derive(&RawValue::Numeric(1.0)).unwrap();
        assert_eq!(t, Trit::V3);
    }

    #[test]
    fn type_mismatch_errors() {
        let rule = DeriveEntityKind;
        let result = rule.derive(&RawValue::Numeric(42.0));
        assert!(result.is_err());
    }

    #[test]
    fn state_persistence_thresholds() {
        let rule = DeriveStatePersistence;
        let (t, _) = rule.derive(&RawValue::Numeric(0.0)).unwrap();
        assert_eq!(t, Trit::V1);

        let (t, _) = rule.derive(&RawValue::Numeric(3600.0)).unwrap();
        assert_eq!(t, Trit::V2);

        let (t, _) = rule.derive(&RawValue::Numeric(2592000.0)).unwrap();
        assert_eq!(t, Trit::V3);
    }

    #[test]
    fn derive_google_address() {
        let rules = all_rules();
        let raw_measurements: Vec<RawValue> = vec![
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
        ];

        let mut trits = [Trit::V1; 27];
        for (i, (rule, raw)) in rules.iter().zip(raw_measurements.iter()).enumerate() {
            let (trit, _confidence) = rule.derive(raw).unwrap();
            trits[i] = trit;
        }

        let addr = crate::addr::CubeAddr::new(trits);
        let expected = crate::addr::CubeAddr::from_category_string(
            "WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313",
        )
        .unwrap();

        assert_eq!(addr, expected, "Derived address doesn't match spec: {} vs {}", addr, expected);
    }
}
