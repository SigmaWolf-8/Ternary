use anyhow::{bail, Context, Result};
use regex::Regex;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub app: AppSection,
    pub install: InstallSection,
    pub app_type: AppTypeSection,
    #[serde(default)]
    pub first_run: Option<FirstRunSection>,
    #[serde(default)]
    pub shortcuts: Option<ShortcutsSection>,
    #[serde(default)]
    pub uninstall: Option<UninstallSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSection {
    pub name: String,
    pub display_name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub version: String,
    pub publisher: String,
    pub binary: String,
    pub icon: String,
    #[serde(default = "default_license")]
    pub license: String,
    pub upgrade_code: String,
}

fn default_license() -> String {
    "Proprietary".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallSection {
    pub directory: String,
    #[serde(default)]
    pub data_directory: Option<String>,
    #[serde(default)]
    pub add_to_path: bool,
    pub architecture: Vec<Architecture>,
    #[serde(default)]
    pub extra_binaries: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Architecture {
    Aarch64,
    #[serde(alias = "x86_64")]
    X86_64,
}

impl Architecture {
    pub fn wix_platform(&self) -> &str {
        match self {
            Architecture::Aarch64 => "arm64",
            Architecture::X86_64 => "x64",
        }
    }

    pub fn arch_launch_condition(&self) -> &str {
        match self {
            Architecture::X86_64 => "VersionNT64",
            Architecture::Aarch64 => "VersionNT64",
        }
    }

    pub fn arch_mismatch_condition(&self) -> &str {
        match self {
            Architecture::X86_64 => "NOT DETECTED_ARCH = &quot;ARM64&quot;",
            Architecture::Aarch64 => "DETECTED_ARCH = &quot;ARM64&quot;",
        }
    }

    pub fn arch_launch_message(&self) -> &str {
        match self {
            Architecture::Aarch64 => {
                "This installer requires a 64-bit ARM (ARM64) operating system. Your system does not meet this requirement. Please download the x64 installer."
            }
            Architecture::X86_64 => {
                "This installer requires a 64-bit x64 (Intel/AMD) operating system. Your system does not meet this requirement. Please download the ARM64 installer."
            }
        }
    }

    pub fn msi_suffix(&self) -> &str {
        match self {
            Architecture::Aarch64 => "arm64",
            Architecture::X86_64 => "x64",
        }
    }

    pub fn rust_target(&self) -> &str {
        match self {
            Architecture::Aarch64 => "aarch64-pc-windows-msvc",
            Architecture::X86_64 => "x86_64-pc-windows-msvc",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppTypeSection {
    pub kind: AppKind,
    #[serde(default)]
    pub autostart: bool,
    #[serde(default)]
    pub tray_icon: Option<String>,
    #[serde(default)]
    pub tray_tooltip: Option<String>,
    #[serde(default)]
    pub configure_command: Option<String>,
    #[serde(default)]
    pub service_account: Option<String>,
    #[serde(default)]
    pub status_port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AppKind {
    TrayAgent,
    Service,
    CliTool,
    Hybrid,
}

impl std::fmt::Display for AppKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppKind::TrayAgent => write!(f, "tray_agent"),
            AppKind::Service => write!(f, "service"),
            AppKind::CliTool => write!(f, "cli_tool"),
            AppKind::Hybrid => write!(f, "hybrid"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirstRunSection {
    pub actions: Vec<FirstRunAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FirstRunAction {
    PromptPassphrase {
        min_length: u32,
        #[serde(default)]
        confirm: bool,
    },
    RunCommand {
        command: String,
        #[serde(default)]
        env_passphrase: Option<String>,
    },
    CopyToClipboard {
        command: String,
        message: String,
    },
    Launch {
        command: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutsSection {
    #[serde(default)]
    pub start_menu: Vec<ShortcutEntry>,
    #[serde(default)]
    pub desktop: Vec<ShortcutEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutEntry {
    pub name: String,
    pub target: String,
    #[serde(default)]
    pub args: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UninstallSection {
    #[serde(default = "default_true")]
    pub preserve_data: bool,
    #[serde(default)]
    pub preserve_message: Option<String>,
}

fn default_true() -> bool {
    true
}

impl Manifest {
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read manifest at {}", path.display()))?;
        let manifest: Manifest = toml::from_str(&content)
            .with_context(|| format!("Failed to parse manifest at {}", path.display()))?;
        Ok(manifest)
    }

    pub fn data_directory(&self) -> String {
        self.install
            .data_directory
            .clone()
            .unwrap_or_else(|| self.app.name.clone())
    }

    pub fn validate(&self, manifest_dir: &Path) -> Result<Vec<String>> {
        self.validate_inner(manifest_dir, true)
    }

    pub fn validate_schema_only(&self, manifest_dir: &Path) -> Result<Vec<String>> {
        self.validate_inner(manifest_dir, false)
    }

    fn validate_inner(&self, manifest_dir: &Path, check_binaries: bool) -> Result<Vec<String>> {
        let mut errors = Vec::new();

        if self.app.name.is_empty() {
            errors.push("app.name is required".to_string());
        }
        if self.app.display_name.is_empty() {
            errors.push("app.display_name is required".to_string());
        }
        if self.app.version.is_empty() {
            errors.push("app.version is required".to_string());
        }
        if self.app.publisher.is_empty() {
            errors.push("app.publisher is required".to_string());
        }
        if self.app.binary.is_empty() {
            errors.push("app.binary is required".to_string());
        }
        if self.app.icon.is_empty() {
            errors.push("app.icon is required".to_string());
        }
        if self.app.upgrade_code.is_empty() {
            errors.push("app.upgrade_code is required".to_string());
        }

        if let Err(e) = Version::parse(&self.app.version) {
            errors.push(format!(
                "app.version '{}' is not valid semver: {}",
                self.app.version, e
            ));
        }

        let guid_re = Regex::new(
            r"^[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}$",
        )
        .unwrap();
        if !guid_re.is_match(&self.app.upgrade_code) {
            errors.push(format!(
                "app.upgrade_code '{}' is not a valid GUID format",
                self.app.upgrade_code
            ));
        }

        if self.install.architecture.is_empty() {
            errors.push("install.architecture must list at least one target".to_string());
        }

        if self.install.directory.is_empty() {
            errors.push("install.directory is required".to_string());
        }

        self.validate_cross_field_consistency(&mut errors);
        self.validate_first_run_security(&mut errors);
        self.validate_file_references(manifest_dir, check_binaries, &mut errors);

        Ok(errors)
    }

    fn validate_cross_field_consistency(&self, errors: &mut Vec<String>) {
        match self.app_type.kind {
            AppKind::TrayAgent => {
                if self.app_type.service_account.is_some() {
                    errors.push("tray_agent kind must not specify service_account".to_string());
                }
            }
            AppKind::Service => {
                if self.app_type.tray_icon.is_some() {
                    errors.push(
                        "service kind must not specify tray_icon without also being hybrid"
                            .to_string(),
                    );
                }
            }
            AppKind::CliTool => {
                if self.app_type.autostart {
                    errors.push("cli_tool kind does not support autostart".to_string());
                }
                if self.app_type.service_account.is_some() {
                    errors.push("cli_tool kind must not specify service_account".to_string());
                }
            }
            AppKind::Hybrid => {}
        }

        if let Some(ref acct) = self.app_type.service_account {
            if acct.is_empty() {
                errors.push("service_account cannot be empty".to_string());
            } else {
                let account_name_re = Regex::new(r"^[A-Za-z0-9_\-\\. ]+$").unwrap();
                if !account_name_re.is_match(acct) {
                    errors.push(format!(
                        "service_account '{}' contains invalid characters",
                        acct
                    ));
                }
            }
        }
    }

    fn validate_first_run_security(&self, errors: &mut Vec<String>) {
        let shell_var_re = Regex::new(r"(%[A-Za-z_]+%|\$[A-Za-z_]+|\$\(.*\)|`.*`)").unwrap();
        let legacy_passphrase_re = Regex::new(r"\{\{passphrase\}\}").unwrap();
        let crs_endpoint_re = Regex::new(r"\{\{CRS_ENDPOINT\}\}").unwrap();

        if let Some(ref first_run) = self.first_run {
            for action in &first_run.actions {
                match action {
                    FirstRunAction::RunCommand { command, .. } => {
                        if legacy_passphrase_re.is_match(command) {
                            errors.push(format!(
                                "Command '{}' uses deprecated {{{{passphrase}}}} interpolation. Use env_passphrase instead.",
                                command
                            ));
                        }
                        let cleaned = crs_endpoint_re.replace_all(command, "");
                        if shell_var_re.is_match(&cleaned) {
                            errors.push(format!(
                                "Command '{}' contains shell variable expansion patterns. Only {{{{CRS_ENDPOINT}}}} interpolation is permitted.",
                                command
                            ));
                        }
                    }
                    FirstRunAction::CopyToClipboard { command, .. } => {
                        if legacy_passphrase_re.is_match(command) {
                            errors.push(format!(
                                "Command '{}' uses deprecated {{{{passphrase}}}} interpolation.",
                                command
                            ));
                        }
                        let cleaned = crs_endpoint_re.replace_all(command, "");
                        if shell_var_re.is_match(&cleaned) {
                            errors.push(format!(
                                "copy_to_clipboard command '{}' contains shell variable expansion patterns. Only {{{{CRS_ENDPOINT}}}} interpolation is permitted.",
                                command
                            ));
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn validate_file_references(
        &self,
        manifest_dir: &Path,
        check_binaries: bool,
        errors: &mut Vec<String>,
    ) {
        let icon_path = manifest_dir.join(&self.app.icon);
        if !icon_path.exists() {
            errors.push(format!(
                "Icon file '{}' not found at {}",
                self.app.icon,
                icon_path.display()
            ));
        } else if icon_path.extension().map_or(false, |e| e == "ico") {
            self.validate_ico_file(&icon_path, &self.app.icon, errors);
        }

        if !self.app.binary.ends_with(".exe") {
            errors.push(format!(
                "app.binary '{}' must end with .exe",
                self.app.binary
            ));
        }

        if check_binaries {
            let binary_path = manifest_dir.join(&self.app.binary);
            if !binary_path.exists() {
                errors.push(format!(
                    "Binary '{}' not found at {}",
                    self.app.binary,
                    binary_path.display()
                ));
            }
        }

        for extra in &self.install.extra_binaries {
            if !extra.ends_with(".exe") {
                errors.push(format!(
                    "extra_binaries entry '{}' must end with .exe",
                    extra
                ));
            }
            if check_binaries {
                let extra_path = manifest_dir.join(extra);
                if !extra_path.exists() {
                    errors.push(format!(
                        "Extra binary '{}' not found at {}",
                        extra,
                        extra_path.display()
                    ));
                }
            }
        }

        if self.app.license != "Proprietary" && !self.app.license.is_empty() {
            let license_path = manifest_dir.join(&self.app.license);
            if license_path.exists() {
            } else if ![
                "MIT",
                "Apache-2.0",
                "BSD-3-Clause",
                "GPL-3.0",
                "Proprietary",
            ]
            .contains(&self.app.license.as_str())
            {
                errors.push(format!(
                    "License '{}' is neither a recognized identifier nor an existing file",
                    self.app.license
                ));
            }
        }

        if let Some(ref tray_icon) = self.app_type.tray_icon {
            let tray_icon_path = manifest_dir.join(tray_icon);
            if !tray_icon_path.exists() {
                errors.push(format!(
                    "Tray icon file '{}' not found at {}",
                    tray_icon,
                    tray_icon_path.display()
                ));
            } else if tray_icon_path.extension().map_or(false, |e| e == "ico") {
                self.validate_ico_file(&tray_icon_path, tray_icon, errors);
            }
        }
    }

    fn validate_ico_file(&self, path: &Path, label: &str, errors: &mut Vec<String>) {
        if let Ok(data) = std::fs::read(path) {
            if data.len() < 6 {
                errors.push(format!("ICO file '{}' is too small to be valid", label));
                return;
            }
            let _reserved = u16::from_le_bytes([data[0], data[1]]);
            let ico_type = u16::from_le_bytes([data[2], data[3]]);
            let count = u16::from_le_bytes([data[4], data[5]]);

            if ico_type != 1 {
                errors.push(format!(
                    "ICO file '{}' has invalid type {} (expected 1)",
                    label, ico_type
                ));
                return;
            }

            if count == 0 {
                errors.push(format!("ICO file '{}' contains no image entries", label));
                return;
            }

            let mut sizes: Vec<u32> = Vec::new();
            for i in 0..count as usize {
                let offset = 6 + i * 16;
                if offset + 16 > data.len() {
                    break;
                }
                let w = if data[offset] == 0 {
                    256
                } else {
                    data[offset] as u32
                };
                let h = if data[offset + 1] == 0 {
                    256
                } else {
                    data[offset + 1] as u32
                };
                sizes.push(std::cmp::max(w, h));
            }

            let required = [16u32, 32, 48, 256];
            let missing: Vec<u32> = required
                .iter()
                .filter(|r| !sizes.contains(r))
                .copied()
                .collect();

            if !missing.is_empty() {
                errors.push(format!(
                    "ICO file '{}' is missing recommended resolutions: {}px (has: {}px). \
                     For best Windows display, include 16, 32, 48, and 256px variants.",
                    label,
                    missing
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    sizes
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                ));
            }
        }
    }
}

pub fn validate_workspace(manifests: &[(&Path, Manifest)]) -> Vec<String> {
    let mut errors = Vec::new();
    let mut upgrade_codes = HashSet::new();
    let mut status_ports: Vec<(String, u16)> = Vec::new();

    for (path, manifest) in manifests {
        if !upgrade_codes.insert(&manifest.app.upgrade_code) {
            errors.push(format!(
                "Duplicate upgrade_code '{}' found in {}",
                manifest.app.upgrade_code,
                path.display()
            ));
        }

        if let Some(port) = manifest.app_type.status_port {
            for (other_name, other_port) in &status_ports {
                if *other_port == port {
                    errors.push(format!(
                        "Status port {} collision between '{}' and '{}' (in {})",
                        port,
                        other_name,
                        manifest.app.name,
                        path.display()
                    ));
                }
            }
            status_ports.push((manifest.app.name.clone(), port));
        }
    }

    errors
}

pub fn generate_product_code(upgrade_code: &str, version: &str) -> String {
    let namespace = uuid::Uuid::parse_str(upgrade_code).unwrap_or(uuid::Uuid::nil());
    let product_code = uuid::Uuid::new_v5(&namespace, version.as_bytes());
    product_code.to_string().to_uppercase()
}

pub fn generate_upgrade_code() -> String {
    uuid::Uuid::new_v4().to_string().to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_code_deterministic() {
        let uc = "A1B2C3D4-E5F6-7890-ABCD-EF1234567890";
        let pc1 = generate_product_code(uc, "1.0.0");
        let pc2 = generate_product_code(uc, "1.0.0");
        assert_eq!(pc1, pc2);
    }

    #[test]
    fn test_product_code_differs_by_version() {
        let uc = "A1B2C3D4-E5F6-7890-ABCD-EF1234567890";
        let pc1 = generate_product_code(uc, "1.0.0");
        let pc2 = generate_product_code(uc, "2.0.0");
        assert_ne!(pc1, pc2);
    }

    #[test]
    fn test_product_code_differs_by_upgrade_code() {
        let uc1 = "A1B2C3D4-E5F6-7890-ABCD-EF1234567890";
        let uc2 = "B2C3D4E5-F6A7-8901-BCDE-F12345678901";
        let pc1 = generate_product_code(uc1, "1.0.0");
        let pc2 = generate_product_code(uc2, "1.0.0");
        assert_ne!(pc1, pc2);
    }
}
