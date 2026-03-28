use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherConfig {
    #[serde(default)]
    pub theme: ThemeMode,
    #[serde(default = "default_poll_interval")]
    pub poll_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

impl Default for ThemeMode {
    fn default() -> Self {
        ThemeMode::System
    }
}

fn default_poll_interval() -> u64 {
    10
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            theme: ThemeMode::default(),
            poll_interval_secs: default_poll_interval(),
        }
    }
}

impl LauncherConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: LauncherConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path();
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }

    fn config_path() -> PathBuf {
        #[cfg(windows)]
        {
            let appdata =
                std::env::var("APPDATA").unwrap_or_else(|_| String::from("C:\\ProgramData"));
            PathBuf::from(appdata)
                .join("PlenumNET-Launcher")
                .join("launcher.toml")
        }
        #[cfg(not(windows))]
        {
            let home = std::env::var("HOME").unwrap_or_else(|_| String::from("/tmp"));
            PathBuf::from(home)
                .join(".config")
                .join("plenumnet-launcher")
                .join("launcher.toml")
        }
    }

    pub fn resolve_theme(&self) -> ResolvedTheme {
        match self.theme {
            ThemeMode::Light => ResolvedTheme::Light,
            ThemeMode::Dark => ResolvedTheme::Dark,
            ThemeMode::System => self.detect_system_theme(),
        }
    }

    fn detect_system_theme(&self) -> ResolvedTheme {
        #[cfg(windows)]
        {
            use winreg::enums::HKEY_CURRENT_USER;
            use winreg::RegKey;
            if Self::detect_high_contrast() {
                return ResolvedTheme::HighContrast;
            }
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            if let Ok(key) =
                hkcu.open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize")
            {
                if let Ok(val) = key.get_value::<u32, _>("AppsUseLightTheme") {
                    return if val == 0 {
                        ResolvedTheme::Dark
                    } else {
                        ResolvedTheme::Light
                    };
                }
            }
            ResolvedTheme::Dark
        }
        #[cfg(not(windows))]
        {
            ResolvedTheme::Dark
        }
    }

    #[cfg(windows)]
    fn detect_high_contrast() -> bool {
        use winreg::enums::HKEY_CURRENT_USER;
        use winreg::RegKey;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(key) = hkcu.open_subkey(r"Control Panel\Accessibility\HighContrast") {
            if let Ok(flags) = key.get_value::<String, _>("Flags") {
                if let Ok(f) = flags.parse::<u32>() {
                    return (f & 1) != 0;
                }
            }
        }
        false
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedTheme {
    Light,
    Dark,
    HighContrast,
}
