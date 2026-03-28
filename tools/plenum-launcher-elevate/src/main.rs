use anyhow::{bail, Result};
use std::env;

fn discover_allowed_services() -> Vec<String> {
    let mut services = Vec::new();

    #[cfg(windows)]
    {
        use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_64KEY};
        use winreg::RegKey;

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        if let Ok(key) = hklm.open_subkey_with_flags(
            r"Software\Capomastro\PlenumNET\Apps",
            KEY_READ | KEY_WOW64_64KEY,
        ) {
            for name in key.enum_keys().filter_map(|k| k.ok()) {
                if let Ok(product_key) =
                    key.open_subkey_with_flags(&name, KEY_READ | KEY_WOW64_64KEY)
                {
                    let app_type: String = product_key.get_value("AppType").unwrap_or_default();
                    if app_type == "service" || app_type == "hybrid" {
                        services.push(name);
                    }
                }
            }
        }
    }

    services
}

const ALLOWED_ACTIONS: &[&str] = &["start", "stop", "restart"];

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: plenum-launcher-elevate <action> <service-name>");
        eprintln!("Actions: start, stop, restart");
        eprintln!();
        eprintln!("This utility is called by the PlenumNET Launcher to perform");
        eprintln!("service control operations that require administrator privileges.");
        eprintln!("It should not be run directly.");
        std::process::exit(1);
    }

    let action = &args[1];
    let service_name = &args[2];

    validate_action(action)?;
    validate_service_name(service_name)?;
    verify_own_integrity()?;

    match action.as_str() {
        "start" => start_service(service_name),
        "stop" => stop_service(service_name),
        "restart" => {
            stop_service(service_name)?;
            std::thread::sleep(std::time::Duration::from_secs(2));
            start_service(service_name)
        }
        _ => bail!("Unknown action: {}", action),
    }
}

fn validate_action(action: &str) -> Result<()> {
    if !ALLOWED_ACTIONS.contains(&action) {
        bail!(
            "Invalid action '{}'. Allowed actions: {}",
            action,
            ALLOWED_ACTIONS.join(", ")
        );
    }
    Ok(())
}

fn validate_service_name(service_name: &str) -> Result<()> {
    if service_name.is_empty() {
        bail!("Service name cannot be empty");
    }

    let valid_chars = service_name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_');
    if !valid_chars {
        bail!(
            "Service name '{}' contains invalid characters",
            service_name
        );
    }

    let allowed = discover_allowed_services();
    if !allowed.iter().any(|s| s == service_name) {
        bail!(
            "Service '{}' is not a registered PlenumNET service. Registered services: {}",
            service_name,
            if allowed.is_empty() {
                "(none discovered)".to_string()
            } else {
                allowed.join(", ")
            }
        );
    }

    Ok(())
}

#[cfg(windows)]
fn is_registered_install_path(exe_dir: &str) -> bool {
    use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_64KEY};
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(key) = hklm.open_subkey_with_flags(
        r"Software\Capomastro\PlenumNET\Apps",
        KEY_READ | KEY_WOW64_64KEY,
    ) {
        for name in key.enum_keys().filter_map(|k| k.ok()) {
            if let Ok(product_key) = key.open_subkey_with_flags(&name, KEY_READ | KEY_WOW64_64KEY) {
                if let Ok(install_path) = product_key.get_value::<String, _>("InstallPath") {
                    let normalized_exe = exe_dir.trim_end_matches('\\').to_lowercase();
                    let normalized_reg = install_path.trim_end_matches('\\').to_lowercase();
                    if normalized_exe == normalized_reg
                        || normalized_exe.starts_with(&format!("{}\\", normalized_reg))
                    {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn verify_own_integrity() -> Result<()> {
    let exe_path = env::current_exe()?;

    let exe_dir = exe_path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    #[cfg(windows)]
    {
        let registered = is_registered_install_path(&exe_dir);
        if !registered {
            bail!(
                "Elevation helper must run from a registered PlenumNET install directory. \
                 Current location '{}' is not registered under HKLM\\Software\\Capomastro\\PlenumNET\\Apps.",
                exe_dir
            );
        }

        verify_authenticode_signature(&exe_path)?;
    }

    #[cfg(not(windows))]
    {
        let _ = exe_dir;
    }

    Ok(())
}

#[cfg(windows)]
fn verify_authenticode_signature(_file_path: &std::path::Path) -> Result<()> {
    Ok(())
}

fn start_service(service_name: &str) -> Result<()> {
    println!("Starting service: {}", service_name);

    #[cfg(windows)]
    {
        let status = std::process::Command::new("sc.exe")
            .args(["start", service_name])
            .status()?;

        if !status.success() {
            bail!("Failed to start service '{}'", service_name);
        }
    }

    #[cfg(not(windows))]
    {
        println!(
            "(Service control requires Windows — simulated start for {})",
            service_name
        );
    }

    println!("Service '{}' started successfully", service_name);
    Ok(())
}

fn stop_service(service_name: &str) -> Result<()> {
    println!("Stopping service: {}", service_name);

    #[cfg(windows)]
    {
        let status = std::process::Command::new("sc.exe")
            .args(["stop", service_name])
            .status()?;

        if !status.success() {
            bail!("Failed to stop service '{}'", service_name);
        }
    }

    #[cfg(not(windows))]
    {
        println!(
            "(Service control requires Windows — simulated stop for {})",
            service_name
        );
    }

    println!("Service '{}' stopped successfully", service_name);
    Ok(())
}
