#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod discovery;
mod status;

use anyhow::Result;
use std::sync::{Arc, Mutex};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    #[cfg(windows)]
    if args.iter().any(|a| a == "--popup") {
        return run_popup_panel();
    }

    let config = config::LauncherConfig::load()?;
    let app_registry = Arc::new(Mutex::new(discovery::AppRegistry::new()));

    {
        let mut registry = app_registry.lock().unwrap();
        registry.discover_installed_apps()?;
    }

    let resolved_theme = config.resolve_theme();
    println!("PlenumNET Launcher v{}", env!("CARGO_PKG_VERSION"));
    println!("Theme: {:?} (resolved: {:?})", config.theme, resolved_theme);
    {
        let registry = app_registry.lock().unwrap();
        let tooltip = status::format_tray_tooltip(registry.apps());
        println!("{}", tooltip);
        if registry.apps().is_empty() {
            println!("  Visit plenumnet.com/products to get started.");
        }
    }

    #[cfg(windows)]
    {
        run_tray_app(config, app_registry)?;
    }

    #[cfg(not(windows))]
    {
        println!("\nLauncher tray mode requires Windows. Running in console mode.");
        loop_console_status(app_registry)?;
    }

    Ok(())
}

#[cfg(not(windows))]
fn loop_console_status(registry: Arc<Mutex<discovery::AppRegistry>>) -> Result<()> {
    println!("\nPress Ctrl+C to exit.");
    let poll_interval = std::time::Duration::from_secs(10);
    loop {
        std::thread::sleep(poll_interval);
        let mut reg = registry.lock().unwrap();
        reg.refresh_status()?;
        let (active, inactive) = reg.status_summary();
        println!("{} active -- {} inactive", active, inactive);
    }
}

#[cfg(windows)]
fn run_tray_app(
    config: config::LauncherConfig,
    registry: Arc<Mutex<discovery::AppRegistry>>,
) -> Result<()> {
    use tao::event::{Event, WindowEvent};
    use tao::event_loop::{ControlFlow, EventLoopBuilder};
    use tray_icon::{TrayIconBuilder, Icon as TrayIcon};
    use muda::{Menu, MenuItem, MenuEvent, Submenu, PredefinedMenuItem};

    let icon_bytes = include_bytes!("../assets/plenum-launcher-tray.ico");
    let icon = load_tray_icon(icon_bytes)?;

    let popup_handle: Arc<Mutex<Option<std::process::Child>>> = Arc::new(Mutex::new(None));

    let event_loop = EventLoopBuilder::<String>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let (menu, menu_ids) = build_context_menu(&registry);

    let _tray = TrayIconBuilder::new()
        .with_tooltip("PlenumNET Launcher")
        .with_icon(icon.clone())
        .with_menu(Box::new(menu))
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to create system tray: {}", e))?;

    let menu_ids = Arc::new(Mutex::new(menu_ids));

    let poll_secs = config.poll_interval_secs;
    let status_registry = Arc::clone(&registry);
    let status_proxy = proxy.clone();
    std::thread::spawn(move || {
        let poll_interval = std::time::Duration::from_secs(poll_secs);
        loop {
            std::thread::sleep(poll_interval);
            if let Ok(mut reg) = status_registry.lock() {
                let before: Vec<_> = reg
                    .apps()
                    .iter()
                    .map(|a| (a.name.clone(), a.status.clone()))
                    .collect();
                let _ = reg.refresh_status();
                let mut changed = false;
                for (name, old_status) in &before {
                    if let Some(app) = reg.apps().iter().find(|a| &a.name == name) {
                        if app.status != *old_status {
                            changed = true;
                        }
                    }
                }
                if changed {
                    let _ = status_proxy.send_event("rebuild_menu".to_string());
                }
            }
        }
    });

    let menu_receiver = MenuEvent::receiver();
    let tray_receiver = tray_icon::TrayIconEvent::receiver();

    let popup_handle_click = Arc::clone(&popup_handle);
    event_loop.run(move |event, _event_loop, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Ok(_tray_event) = tray_receiver.try_recv() {
            open_or_focus_popup(&popup_handle_click);
        }

        if let Ok(menu_event) = menu_receiver.try_recv() {
            let ids = menu_ids.lock().unwrap();
            let event_id = menu_event.id().clone();

            if Some(&event_id) == ids.open_dashboard.as_ref() {
                open_or_focus_popup(&popup_handle_click);
            } else if Some(&event_id) == ids.quit.as_ref() {
                *control_flow = ControlFlow::Exit;
            } else if Some(&event_id) == ids.about.as_ref() {
                show_notification(
                    &format!(
                        "PlenumNET Launcher v{}\nCapomastro Holdings Ltd.\nhttps://plenumnet.com",
                        env!("CARGO_PKG_VERSION")
                    ),
                    "About PlenumNET",
                );
            } else if let Some(idx) = ids.start.iter().position(|id| *id == event_id) {
                let reg = registry.lock().unwrap();
                if let Some(app) = reg.apps().get(idx) {
                    let app_name = app.name.clone();
                    match reg.start_app(&app_name) {
                        Ok(()) => show_notification(&format!("{} started successfully.", app_name), "PlenumNET Launcher"),
                        Err(e) => show_notification(&format!("Failed to start {}: {}", app_name, e), "PlenumNET Launcher -- Error"),
                    }
                }
            } else if let Some(idx) = ids.stop.iter().position(|id| *id == event_id) {
                let reg = registry.lock().unwrap();
                if let Some(app) = reg.apps().get(idx) {
                    let app_name = app.name.clone();
                    match reg.stop_app(&app_name) {
                        Ok(()) => show_notification(&format!("{} stopped.", app_name), "PlenumNET Launcher"),
                        Err(e) => show_notification(&format!("Failed to stop {}: {}", app_name, e), "PlenumNET Launcher -- Error"),
                    }
                }
            } else if let Some(idx) = ids.restart.iter().position(|id| *id == event_id) {
                let reg = registry.lock().unwrap();
                if let Some(app) = reg.apps().get(idx) {
                    let app_name = app.name.clone();
                    let _ = reg.stop_app(&app_name);
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    match reg.start_app(&app_name) {
                        Ok(()) => show_notification(&format!("{} restarted successfully.", app_name), "PlenumNET Launcher"),
                        Err(e) => show_notification(&format!("Failed to restart {}: {}", app_name, e), "PlenumNET Launcher -- Error"),
                    }
                }
            } else if let Some(idx) = ids.configure.iter().position(|id| *id == event_id) {
                let reg = registry.lock().unwrap();
                if let Some(app) = reg.apps().get(idx) {
                    let _ = reg.configure_app(&app.name);
                }
            } else if let Some(idx) = ids.open_logs.iter().position(|id| *id == event_id) {
                let reg = registry.lock().unwrap();
                if let Some(app) = reg.apps().get(idx) {
                    let _ = reg.open_logs(&app.name);
                }
            } else if Some(&event_id) == ids.theme_system.as_ref() {
                save_theme(config::ThemeMode::System);
            } else if Some(&event_id) == ids.theme_light.as_ref() {
                save_theme(config::ThemeMode::Light);
            } else if Some(&event_id) == ids.theme_dark.as_ref() {
                save_theme(config::ThemeMode::Dark);
            }
        }

        match event {
            Event::UserEvent(ref msg) if msg == "rebuild_menu" => {
                // Menu rebuild would require recreating the tray; skip for now
                // as status polling already refreshes on next cycle
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}

#[cfg(windows)]
fn save_theme(mode: config::ThemeMode) {
    let mut cfg = config::LauncherConfig::load().unwrap_or_default();
    cfg.theme = mode;
    if let Err(e) = cfg.save() {
        show_notification(&format!("Failed to save theme setting: {}", e), "PlenumNET Launcher -- Error");
    } else {
        let resolved = cfg.resolve_theme();
        show_notification(&format!("Theme set to {:?}", resolved), "PlenumNET Launcher");
    }
}

#[cfg(windows)]
struct MenuIds {
    open_dashboard: Option<muda::MenuId>,
    quit: Option<muda::MenuId>,
    about: Option<muda::MenuId>,
    start: Vec<muda::MenuId>,
    stop: Vec<muda::MenuId>,
    restart: Vec<muda::MenuId>,
    configure: Vec<muda::MenuId>,
    open_logs: Vec<muda::MenuId>,
    theme_system: Option<muda::MenuId>,
    theme_light: Option<muda::MenuId>,
    theme_dark: Option<muda::MenuId>,
}

#[cfg(windows)]
fn build_context_menu(registry: &Arc<Mutex<discovery::AppRegistry>>) -> (muda::Menu, MenuIds) {
    use muda::{Menu, MenuItem, Submenu, PredefinedMenuItem};

    let menu = Menu::new();
    let mut ids = MenuIds {
        open_dashboard: None,
        quit: None,
        about: None,
        start: Vec::new(),
        stop: Vec::new(),
        restart: Vec::new(),
        configure: Vec::new(),
        open_logs: Vec::new(),
        theme_system: None,
        theme_light: None,
        theme_dark: None,
    };

    let dashboard_item = MenuItem::new("Open Dashboard", true, None);
    ids.open_dashboard = Some(dashboard_item.id().clone());
    let _ = menu.append(&dashboard_item);

    let _ = menu.append(&PredefinedMenuItem::separator());

    let reg = registry.lock().unwrap();
    for (idx, app) in reg.apps().iter().enumerate() {
        let label = format!(
            "{} -- {} ({})",
            app.display_name,
            app.status_text(),
            app.app_type
        );
        let header = MenuItem::new(&label, false, None);
        let _ = menu.append(&header);

        let is_active = app.status == discovery::AppStatus::Active;
        if is_active {
            let stop_item = MenuItem::new(&format!("  Stop {}", app.display_name), true, None);
            ids.stop.push(stop_item.id().clone());
            // Pad other vectors to keep indices aligned
            while ids.start.len() <= idx {
                ids.start.push(muda::MenuId::new("__placeholder_start__"));
            }
            let _ = menu.append(&stop_item);

            let restart_item = MenuItem::new(&format!("  Restart {}", app.display_name), true, None);
            ids.restart.push(restart_item.id().clone());
            while ids.restart.len() <= idx {
                ids.restart.push(muda::MenuId::new("__placeholder_restart__"));
            }
            let _ = menu.append(&restart_item);
        } else {
            let start_item = MenuItem::new(&format!("  Start {}", app.display_name), true, None);
            ids.start.push(start_item.id().clone());
            while ids.stop.len() <= idx {
                ids.stop.push(muda::MenuId::new("__placeholder_stop__"));
            }
            let _ = menu.append(&start_item);
        }

        let config_item = MenuItem::new(&format!("  Configure {}", app.display_name), true, None);
        ids.configure.push(config_item.id().clone());
        let _ = menu.append(&config_item);

        let logs_item = MenuItem::new(&format!("  Open Logs {}", app.display_name), true, None);
        ids.open_logs.push(logs_item.id().clone());
        let _ = menu.append(&logs_item);
    }

    let _ = menu.append(&PredefinedMenuItem::separator());

    let theme_sub = Submenu::new("Theme", true);
    let sys_item = MenuItem::new("System (follow OS)", true, None);
    ids.theme_system = Some(sys_item.id().clone());
    let _ = theme_sub.append(&sys_item);
    let light_item = MenuItem::new("Light", true, None);
    ids.theme_light = Some(light_item.id().clone());
    let _ = theme_sub.append(&light_item);
    let dark_item = MenuItem::new("Dark", true, None);
    ids.theme_dark = Some(dark_item.id().clone());
    let _ = theme_sub.append(&dark_item);
    let _ = menu.append(&theme_sub);

    let about_item = MenuItem::new("About PlenumNET", true, None);
    ids.about = Some(about_item.id().clone());
    let _ = menu.append(&about_item);

    let _ = menu.append(&PredefinedMenuItem::separator());

    let quit_item = MenuItem::new("Quit", true, None);
    ids.quit = Some(quit_item.id().clone());
    let _ = menu.append(&quit_item);

    (menu, ids)
}

#[cfg(windows)]
fn open_or_focus_popup(popup_handle: &Arc<Mutex<Option<std::process::Child>>>) {
    let mut handle = popup_handle.lock().unwrap();

    if let Some(ref mut child) = *handle {
        match child.try_wait() {
            Ok(Some(_)) => {}
            Ok(None) => {
                return;
            }
            Err(_) => {}
        }
    }

    let exe = std::env::current_exe().unwrap_or_default();
    use std::os::windows::process::CommandExt;
    match std::process::Command::new(exe)
        .arg("--popup")
        .creation_flags(0x08000000)
        .spawn()
    {
        Ok(child) => {
            *handle = Some(child);
        }
        Err(e) => {
            eprintln!("Failed to spawn popup: {}", e);
        }
    }
}

#[cfg(windows)]
fn load_tray_icon(ico_data: &[u8]) -> Result<tray_icon::Icon> {
    if ico_data.len() < 22 {
        return Err(anyhow::anyhow!("ICO data too short to contain a valid image"));
    }

    let image_count = u16::from_le_bytes([ico_data[4], ico_data[5]]) as usize;
    if image_count == 0 {
        return Err(anyhow::anyhow!("ICO file contains no images"));
    }

    let mut best_idx = 0usize;
    let mut best_area = 0u32;
    for i in 0..image_count {
        let off = 6 + i * 16;
        if off + 16 > ico_data.len() {
            break;
        }
        let w = if ico_data[off] == 0 { 256u32 } else { ico_data[off] as u32 };
        let h = if ico_data[off + 1] == 0 { 256u32 } else { ico_data[off + 1] as u32 };
        let area = w * h;
        if area > best_area {
            best_area = area;
            best_idx = i;
        }
    }

    let entry = 6 + best_idx * 16;
    let w = if ico_data[entry] == 0 { 256u32 } else { ico_data[entry] as u32 };
    let h = if ico_data[entry + 1] == 0 { 256u32 } else { ico_data[entry + 1] as u32 };
    let data_size = u32::from_le_bytes([
        ico_data[entry + 8],
        ico_data[entry + 9],
        ico_data[entry + 10],
        ico_data[entry + 11],
    ]) as usize;
    let data_offset = u32::from_le_bytes([
        ico_data[entry + 12],
        ico_data[entry + 13],
        ico_data[entry + 14],
        ico_data[entry + 15],
    ]) as usize;

    if data_offset + data_size > ico_data.len() {
        return Err(anyhow::anyhow!("ICO image data extends beyond file boundary"));
    }

    let img_data = &ico_data[data_offset..data_offset + data_size];
    let is_png = img_data.len() >= 8 && img_data[0..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    if is_png {
        if let Ok(decoder) = image::codecs::png::PngDecoder::new(std::io::Cursor::new(img_data)) {
            use image::ImageDecoder;
            let (pw, ph) = decoder.dimensions();
            let mut rgba_buf = vec![0u8; (pw * ph * 4) as usize];
            if decoder.read_image(&mut rgba_buf).is_ok() {
                return tray_icon::Icon::from_rgba(rgba_buf, pw, ph)
                    .map_err(|e| anyhow::anyhow!("Icon from PNG RGBA failed: {}", e));
            }
        }
    }

    if !is_png && img_data.len() >= 40 {
        let bpp = u16::from_le_bytes([img_data[14], img_data[15]]) as u32;
        if bpp == 32 {
            let pixel_offset = 40usize;
            let stride = (w * 4) as usize;
            let expected = pixel_offset + stride * h as usize;
            if img_data.len() >= expected {
                let mut rgba = Vec::with_capacity((w * h * 4) as usize);
                for row in (0..h).rev() {
                    let row_start = pixel_offset + (row as usize) * stride;
                    for col in 0..w as usize {
                        let px = row_start + col * 4;
                        rgba.push(img_data[px + 2]);
                        rgba.push(img_data[px + 1]);
                        rgba.push(img_data[px]);
                        rgba.push(img_data[px + 3]);
                    }
                }
                return tray_icon::Icon::from_rgba(rgba, w, h)
                    .map_err(|e| anyhow::anyhow!("Icon from BMP RGBA failed: {}", e));
            }
        }
    }

    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for _ in 0..(w * h) {
        rgba.extend_from_slice(&[0x1B, 0x99, 0x8B, 0xFF]);
    }
    tray_icon::Icon::from_rgba(rgba, w, h)
        .map_err(|e| anyhow::anyhow!("Icon fallback creation failed: {}", e))
}

#[cfg(windows)]
fn show_notification(message: &str, title: &str) {
    let _ = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &format!(
                r#"[System.Reflection.Assembly]::LoadWithPartialName('System.Windows.Forms') | Out-Null; $n = New-Object System.Windows.Forms.NotifyIcon; $n.Icon = [System.Drawing.SystemIcons]::Information; $n.Visible = $true; $n.ShowBalloonTip(5000, '{}', '{}', 'Info'); Start-Sleep -Seconds 3; $n.Dispose()"#,
                title.replace('\'', "''"),
                message.replace('\'', "''"),
            ),
        ])
        .spawn();
}

#[cfg(not(windows))]
fn show_notification(message: &str, title: &str) {
    eprintln!("[{}] {}", title, message);
}

#[cfg(windows)]
fn run_popup_panel() -> Result<()> {
    use tao::event::{Event, WindowEvent};
    use tao::event_loop::{ControlFlow, EventLoop};
    use tao::window::WindowBuilder;
    use tao::dpi::{LogicalSize, LogicalPosition};
    use wry::WebViewBuilder;

    let panel_width = 420.0_f64;
    let panel_height = 640.0_f64;

    let event_loop = EventLoop::new();

    let monitor = event_loop
        .primary_monitor()
        .or_else(|| event_loop.available_monitors().next());

    let (pos_x, pos_y) = if let Some(m) = &monitor {
        let size = m.size();
        let scale = m.scale_factor();
        let w = size.width as f64 / scale;
        let h = size.height as f64 / scale;
        (w - panel_width - 12.0, h - panel_height - 60.0)
    } else {
        (800.0, 200.0)
    };

    let window = WindowBuilder::new()
        .with_title("PlenumNET")
        .with_inner_size(LogicalSize::new(panel_width, panel_height))
        .with_position(LogicalPosition::new(pos_x, pos_y))
        .with_decorations(true)
        .with_always_on_top(true)
        .with_resizable(false)
        .build(&event_loop)
        .map_err(|e| anyhow::anyhow!("Failed to create window: {}", e))?;

    let _webview = WebViewBuilder::new()
        .with_url("https://plenumnet.replit.app/widget")
        .build(&window)
        .map_err(|e| anyhow::anyhow!("Failed to create webview: {}", e))?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit;
        }
    });
}
