// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// Attestation Report Versioning — Task #119 Task 5

//! Schema version registry and upgrade window management.
//!
//! Reports carry a u16 schema version. Unknown versions produce
//! "unparseable" alerts (NOT suspicion increments), suppressed
//! during declared upgrade windows (auto-expiring, max 4h).

/// Maximum upgrade window duration: 4 hours in seconds.
const MAX_UPGRADE_WINDOW_S: u32 = 14_400;

/// Upgrade window state.
#[derive(Debug, Clone)]
pub struct UpgradeWindow {
    /// Start timestamp (femtoseconds since Salvi Epoch).
    pub start_fs: u128,
    /// Duration in seconds.
    pub duration_s: u32,
}

impl UpgradeWindow {
    /// Create a new upgrade window. Duration capped at 4 hours.
    pub fn new(start_fs: u128, duration_s: u32) -> Self {
        Self {
            start_fs,
            duration_s: duration_s.min(MAX_UPGRADE_WINDOW_S),
        }
    }

    /// Check if the window is active at the given time.
    pub fn is_active(&self, current_fs: u128) -> bool {
        if current_fs < self.start_fs {
            return false;
        }
        let elapsed_fs = current_fs - self.start_fs;
        let duration_fs = self.duration_s as u128 * 1_000_000_000_000_000;
        elapsed_fs < duration_fs
    }
}

/// Schema version registry entry.
#[derive(Debug, Clone)]
pub struct SchemaVersionEntry {
    /// Schema version number.
    pub version: u16,
    /// TLSponge-385 hash of the schema definition.
    pub schema_hash: Vec<u8>,
}

/// Manages known schema versions and upgrade windows.
#[derive(Debug)]
pub struct VersionRegistry {
    /// Known schema versions (from TL-DSA-signed PlenumConfig artifact).
    entries: Vec<SchemaVersionEntry>,
    /// Active upgrade window (if any).
    upgrade_window: Option<UpgradeWindow>,
}

impl VersionRegistry {
    pub fn new() -> Self {
        Self {
            entries: vec![SchemaVersionEntry {
                version: super::report::SCHEMA_VERSION,
                schema_hash: Vec::new(), // populated from PlenumConfig
            }],
            upgrade_window: None,
        }
    }

    /// Check if a version is known.
    pub fn is_known(&self, version: u16) -> bool {
        self.entries.iter().any(|e| e.version == version)
    }

    /// Add a known version (from signed PlenumConfig update).
    pub fn add_version(&mut self, entry: SchemaVersionEntry) {
        if !self.is_known(entry.version) {
            self.entries.push(entry);
        }
    }

    /// Declare an upgrade window.
    pub fn declare_upgrade_window(&mut self, start_fs: u128, duration_s: u32) {
        self.upgrade_window = Some(UpgradeWindow::new(start_fs, duration_s));
    }

    /// Close the upgrade window.
    pub fn close_upgrade_window(&mut self) {
        self.upgrade_window = None;
    }

    /// Check if version mismatch alerts should be suppressed.
    pub fn suppress_version_alerts(&self, current_fs: u128) -> bool {
        self.upgrade_window.as_ref()
            .map(|w| w.is_active(current_fs))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_version_accepted() {
        let reg = VersionRegistry::new();
        assert!(reg.is_known(super::super::report::SCHEMA_VERSION));
        assert!(!reg.is_known(99));
    }

    #[test]
    fn unknown_version_not_suspicion() {
        let reg = VersionRegistry::new();
        // Unknown version → "unparseable", not suspicion increment
        assert!(!reg.is_known(2));
    }

    #[test]
    fn version_zero_invalid() {
        let reg = VersionRegistry::new();
        assert!(!reg.is_known(0));
    }

    #[test]
    fn upgrade_window_auto_expires() {
        let mut reg = VersionRegistry::new();
        let start: u128 = 100 * 1_000_000_000_000_000;
        reg.declare_upgrade_window(start, 3600); // 1 hour

        // Active during window
        assert!(reg.suppress_version_alerts(start + 1800 * 1_000_000_000_000_000));

        // Expired after duration
        assert!(!reg.suppress_version_alerts(start + 3601 * 1_000_000_000_000_000));
    }

    #[test]
    fn upgrade_window_capped_at_4h() {
        let mut reg = VersionRegistry::new();
        let start: u128 = 0;
        reg.declare_upgrade_window(start, 100_000); // Requested 100K seconds

        // Should cap at 14400s (4h)
        assert!(reg.suppress_version_alerts(start + 14399 * 1_000_000_000_000_000));
        assert!(!reg.suppress_version_alerts(start + 14401 * 1_000_000_000_000_000));
    }
}
