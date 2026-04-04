use crate::discovery::AppStatus;

pub struct StatusColors;

impl StatusColors {
    pub const ACTIVE: &'static str = "#4A9EF5";
    pub const WARNING: &'static str = "#78828C";
    pub const INACTIVE: &'static str = "#3D444B";
    pub const LOCKED: &'static str = "#4A9EF5";
}

pub struct BrandPalette;

impl BrandPalette {
    pub const BG_PAGE_DARK: &'static str = "#0F0C0A";
    pub const BG_CARD_DARK: &'static str = "#181411";
    pub const BG_MUTED_DARK: &'static str = "#1D1915";
    pub const BORDER_DARK: &'static str = "#272220";
    pub const TEXT_HEADING_DARK: &'static str = "#F0EDE8";
    pub const TEXT_NAV_DARK: &'static str = "#E4DFD5";
    pub const TEXT_BODY_DARK: &'static str = "#C9C1B4";
    pub const TEXT_LABEL_DARK: &'static str = "#998F82";
    pub const TEXT_FAINT_DARK: &'static str = "#6B655E";
    pub const ACCENT_PRIMARY: &'static str = "#4A9EF5";
    pub const ACCENT_HOVER: &'static str = "#38BDF8";
    pub const SURFACE_IRON: &'static str = "#3D444B";
    pub const SURFACE_SLATE: &'static str = "#78828C";

    pub const BG_PAGE_LIGHT: &'static str = "#FAF8F6";
    pub const BG_CARD_LIGHT: &'static str = "#FFFFFF";
    pub const BG_MUTED_LIGHT: &'static str = "#F0ECE8";
    pub const BORDER_LIGHT: &'static str = "#D9D3CC";
    pub const TEXT_HEADING_LIGHT: &'static str = "#1A1614";
    pub const TEXT_NAV_LIGHT: &'static str = "#2C2722";
    pub const TEXT_BODY_LIGHT: &'static str = "#4A4440";
    pub const TEXT_LABEL_LIGHT: &'static str = "#7A7168";
    pub const TEXT_FAINT_LIGHT: &'static str = "#A89E94";
    pub const ACCENT_PRIMARY_LIGHT: &'static str = "#2D7DD2";
    pub const ACCENT_HOVER_LIGHT: &'static str = "#1A6FC2";
    pub const SURFACE_IRON_LIGHT: &'static str = "#C8CDD2";
    pub const SURFACE_SLATE_LIGHT: &'static str = "#8E959C";
}

pub fn status_to_text(status: &AppStatus) -> &'static str {
    match status {
        AppStatus::Active => "Active",
        AppStatus::Warning => "Warning",
        AppStatus::Inactive => "Inactive",
        AppStatus::Locked => "Locked",
        AppStatus::Unknown => "Unknown",
    }
}

pub fn status_to_color(status: &AppStatus) -> &'static str {
    match status {
        AppStatus::Active => StatusColors::ACTIVE,
        AppStatus::Warning => StatusColors::WARNING,
        AppStatus::Inactive => StatusColors::INACTIVE,
        AppStatus::Locked => StatusColors::LOCKED,
        AppStatus::Unknown => StatusColors::INACTIVE,
    }
}

pub struct HighContrastColors;

impl HighContrastColors {
    pub const ACTIVE: &'static str = "#00FF00";
    pub const WARNING: &'static str = "#FFFF00";
    pub const INACTIVE: &'static str = "#808080";
    pub const LOCKED: &'static str = "#00FFFF";
    pub const TEXT: &'static str = "#FFFFFF";
    pub const BACKGROUND: &'static str = "#000000";
}

pub fn status_to_color_hc(status: &AppStatus) -> &'static str {
    match status {
        AppStatus::Active => HighContrastColors::ACTIVE,
        AppStatus::Warning => HighContrastColors::WARNING,
        AppStatus::Inactive => HighContrastColors::INACTIVE,
        AppStatus::Locked => HighContrastColors::LOCKED,
        AppStatus::Unknown => HighContrastColors::INACTIVE,
    }
}

pub fn format_tray_tooltip(apps: &[crate::discovery::InstalledApp]) -> String {
    let active = apps
        .iter()
        .filter(|a| a.status == AppStatus::Active)
        .count();
    let total = apps.len();
    if total == 0 {
        return "PlenumNET Launcher -- No apps installed".to_string();
    }
    let mut tip = format!("PlenumNET Launcher -- {}/{} active\n", active, total);
    for app in apps {
        tip.push_str(&format!(
            "\n{}: {}",
            app.display_name,
            status_to_text(&app.status)
        ));
    }
    tip
}
