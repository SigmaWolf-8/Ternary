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
        println!("{} active · {} inactive", active, inactive);
    }
}

#[cfg(windows)]
fn build_tray_menu(
    tray: &mut tray_item::TrayItem,
    registry: &Arc<Mutex<discovery::AppRegistry>>,
    tx: &std::sync::mpsc::Sender<String>,
) -> Result<()> {
    tray.add_label("PlenumNET Launcher")?;

    let dashboard_tx = tx.clone();
    tray.add_menu_item("Open Dashboard", move || {
        let _ = dashboard_tx.send("dashboard".to_string());
    })?;

    {
        let reg = registry.lock().unwrap();

        for app in reg.apps() {
            let label = format!(
                "{} — {} ({})",
                app.display_name,
                app.status_text(),
                app.app_type
            );
            tray.add_label(&label)?;

            let app_name = app.name.clone();
            let is_active = app.status == discovery::AppStatus::Active;

            if is_active {
                let stop_tx = tx.clone();
                let stop_name = app_name.clone();
                tray.add_menu_item(&format!("  Stop {}", app.display_name), move || {
                    let _ = stop_tx.send(format!("stop:{}", stop_name));
                })?;

                let restart_tx = tx.clone();
                let restart_name = app_name.clone();
                tray.add_menu_item(&format!("  Restart {}", app.display_name), move || {
                    let _ = restart_tx.send(format!("restart:{}", restart_name));
                })?;
            } else {
                let start_tx = tx.clone();
                let start_name = app_name.clone();
                tray.add_menu_item(&format!("  Start {}", app.display_name), move || {
                    let _ = start_tx.send(format!("start:{}", start_name));
                })?;
            }

            let config_tx = tx.clone();
            let config_name = app_name.clone();
            tray.add_menu_item(&format!("  Configure {}", app.display_name), move || {
                let _ = config_tx.send(format!("configure:{}", config_name));
            })?;

            let logs_tx = tx.clone();
            let logs_name = app_name.clone();
            tray.add_menu_item(&format!("  Open Logs {}", app.display_name), move || {
                let _ = logs_tx.send(format!("logs:{}", logs_name));
            })?;

        }
    }

    tray.add_label("Theme")?;

    let theme_system_tx = tx.clone();
    tray.add_menu_item("  System (follow OS)", move || {
        let _ = theme_system_tx.send("theme:system".to_string());
    })?;

    let theme_light_tx = tx.clone();
    tray.add_menu_item("  Light", move || {
        let _ = theme_light_tx.send("theme:light".to_string());
    })?;

    let theme_dark_tx = tx.clone();
    tray.add_menu_item("  Dark", move || {
        let _ = theme_dark_tx.send("theme:dark".to_string());
    })?;

    let about_tx = tx.clone();
    tray.add_menu_item("About PlenumNET", move || {
        let _ = about_tx.send("about".to_string());
    })?;

    let quit_tx = tx.clone();
    tray.add_menu_item("Quit", move || {
        let _ = quit_tx.send("quit".to_string());
    })?;

    Ok(())
}

#[cfg(windows)]
fn run_tray_app(
    config: config::LauncherConfig,
    registry: Arc<Mutex<discovery::AppRegistry>>,
) -> Result<()> {
    use tray_item::{IconSource, TrayItem};

    let mut tray = TrayItem::new("PlenumNET Launcher", IconSource::Resource("plenumnet-launcher"))?;
    let (tx, rx) = std::sync::mpsc::channel();

    build_tray_menu(&mut tray, &registry, &tx)?;

    let poll_secs = config.poll_interval_secs;
    let status_registry = Arc::clone(&registry);
    let status_tx = tx.clone();
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
                    let _ = status_tx.send("rebuild_menu".to_string());
                }
            }
        }
    });

    loop {
        match rx.recv() {
            Ok(msg) => {
                if msg == "quit" {
                    break;
                } else if msg == "rebuild_menu" {
                    let mut new_tray = TrayItem::new("PlenumNET Launcher", IconSource::Resource("plenumnet-launcher"))
                        .unwrap_or_else(|_| {
                            eprintln!("Failed to rebuild tray menu");
                            std::process::exit(1);
                        });
                    let _ = build_tray_menu(&mut new_tray, &registry, &tx);
                    tray = new_tray;
                } else if msg == "dashboard" {
                    let exe = std::env::current_exe().unwrap_or_default();
                    let _ = std::process::Command::new(exe)
                        .arg("--popup")
                        .spawn();
                } else if msg == "about" {
                    show_notification(
                        &format!(
                            "PlenumNET Launcher v{}\nCapomastro Holdings Ltd.\nhttps://plenumnet.com",
                            env!("CARGO_PKG_VERSION")
                        ),
                        "About PlenumNET",
                    );
                } else if let Some(app_name) = msg.strip_prefix("start:") {
                    let reg = registry.lock().unwrap();
                    match reg.start_app(app_name) {
                        Ok(()) => {
                            show_notification(
                                &format!("{} started successfully.", app_name),
                                "PlenumNET Launcher",
                            );
                        }
                        Err(e) => {
                            show_notification(
                                &format!("Failed to start {}: {}", app_name, e),
                                "PlenumNET Launcher — Error",
                            );
                        }
                    }
                    drop(reg);
                    let _ = tx.send("rebuild_menu".to_string());
                } else if let Some(app_name) = msg.strip_prefix("stop:") {
                    let reg = registry.lock().unwrap();
                    match reg.stop_app(app_name) {
                        Ok(()) => {
                            show_notification(
                                &format!("{} stopped.", app_name),
                                "PlenumNET Launcher",
                            );
                        }
                        Err(e) => {
                            show_notification(
                                &format!("Failed to stop {}: {}", app_name, e),
                                "PlenumNET Launcher — Error",
                            );
                        }
                    }
                    drop(reg);
                    let _ = tx.send("rebuild_menu".to_string());
                } else if let Some(app_name) = msg.strip_prefix("restart:") {
                    let reg = registry.lock().unwrap();
                    let stop_result = reg.stop_app(app_name);
                    if let Err(e) = &stop_result {
                        show_notification(
                            &format!("Failed to stop {}: {}", app_name, e),
                            "PlenumNET Launcher — Error",
                        );
                    } else {
                        std::thread::sleep(std::time::Duration::from_secs(2));
                        match reg.start_app(app_name) {
                            Ok(()) => {
                                show_notification(
                                    &format!("{} restarted successfully.", app_name),
                                    "PlenumNET Launcher",
                                );
                            }
                            Err(e) => {
                                show_notification(
                                    &format!("Failed to restart {}: {}", app_name, e),
                                    "PlenumNET Launcher — Error",
                                );
                            }
                        }
                    }
                    drop(reg);
                    let _ = tx.send("rebuild_menu".to_string());
                } else if let Some(app_name) = msg.strip_prefix("configure:") {
                    let reg = registry.lock().unwrap();
                    if let Err(e) = reg.configure_app(app_name) {
                        show_notification(
                            &format!("Failed to configure {}: {}", app_name, e),
                            "PlenumNET Launcher — Error",
                        );
                    }
                } else if let Some(app_name) = msg.strip_prefix("logs:") {
                    let reg = registry.lock().unwrap();
                    if let Err(e) = reg.open_logs(app_name) {
                        show_notification(
                            &format!("Failed to open logs for {}: {}", app_name, e),
                            "PlenumNET Launcher — Error",
                        );
                    }
                } else if let Some(theme_str) = msg.strip_prefix("theme:") {
                    let new_theme = match theme_str {
                        "light" => config::ThemeMode::Light,
                        "dark" => config::ThemeMode::Dark,
                        _ => config::ThemeMode::System,
                    };
                    let mut cfg = config::LauncherConfig::load().unwrap_or_default();
                    cfg.theme = new_theme;
                    if let Err(e) = cfg.save() {
                        show_notification(
                            &format!("Failed to save theme setting: {}", e),
                            "PlenumNET Launcher — Error",
                        );
                    } else {
                        let resolved = cfg.resolve_theme();
                        show_notification(
                            &format!("Theme set to {:?}", resolved),
                            "PlenumNET Launcher",
                        );
                    }
                }
            }
            Err(_) => break,
        }
    }

    Ok(())
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
        .with_decorations(false)
        .with_always_on_top(true)
        .with_resizable(false)
        .build(&event_loop)
        .map_err(|e| anyhow::anyhow!("Failed to create window: {}", e))?;

    let _webview = WebViewBuilder::new(&window)
        .with_url("https://plenumnet.replit.app")
        .with_initialization_script(
            r#"document.addEventListener('keydown', function(e) {
                if (e.key === 'Escape') { window.close(); }
            });"#,
        )
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to create webview: {}", e))?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::Focused(false),
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}
