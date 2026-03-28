use anyhow::{bail, Result};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct InstalledApp {
    pub name: String,
    pub display_name: String,
    pub version: String,
    pub app_type: String,
    pub install_path: String,
    pub data_directory: String,
    pub binary: String,
    pub status: AppStatus,
    pub status_port: Option<u16>,
    pub configure_command: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppStatus {
    Active,
    Warning,
    Inactive,
    Locked,
    Unknown,
}

impl InstalledApp {
    pub fn status_text(&self) -> &str {
        match self.status {
            AppStatus::Active => "Active",
            AppStatus::Warning => "Warning",
            AppStatus::Inactive => "Inactive",
            AppStatus::Locked => "Locked",
            AppStatus::Unknown => "Unknown",
        }
    }

    pub fn status_color(&self) -> &str {
        match self.status {
            AppStatus::Active => "#4A9EF5",
            AppStatus::Warning => "#78828C",
            AppStatus::Inactive => "#3D444B",
            AppStatus::Locked => "#4A9EF5",
            AppStatus::Unknown => "#3D444B",
        }
    }

    pub fn data_dir_path(&self) -> PathBuf {
        #[cfg(windows)]
        {
            let appdata =
                std::env::var("APPDATA").unwrap_or_else(|_| String::from("C:\\Users\\Public"));
            PathBuf::from(appdata).join(&self.data_directory)
        }
        #[cfg(not(windows))]
        {
            let home = std::env::var("HOME").unwrap_or_else(|_| String::from("/tmp"));
            PathBuf::from(home)
                .join(".local")
                .join("share")
                .join(&self.data_directory)
        }
    }

    pub fn log_dir_path(&self) -> PathBuf {
        self.data_dir_path().join("logs")
    }
}

pub struct AppRegistry {
    apps: Vec<InstalledApp>,
}

impl AppRegistry {
    pub fn new() -> Self {
        Self { apps: Vec::new() }
    }

    pub fn apps(&self) -> &[InstalledApp] {
        &self.apps
    }

    pub fn status_summary(&self) -> (usize, usize) {
        let active = self
            .apps
            .iter()
            .filter(|a| a.status == AppStatus::Active)
            .count();
        let inactive = self.apps.len() - active;
        (active, inactive)
    }

    pub fn discover_installed_apps(&mut self) -> Result<()> {
        self.apps.clear();

        #[cfg(windows)]
        {
            self.discover_from_registry()?;
        }

        #[cfg(not(windows))]
        {
            self.discover_from_mock()?;
        }

        Ok(())
    }

    pub fn refresh_status(&mut self) -> Result<()> {
        for app in &mut self.apps {
            app.status = Self::check_app_status(app)?;
        }
        Ok(())
    }

    pub fn start_app(&self, app_name: &str) -> Result<()> {
        let app = self.find_app(app_name)?;
        match app.app_type.as_str() {
            "service" => self.elevate_service_action("start", &app.name),
            "tray_agent" => {
                let binary_path = PathBuf::from(&app.install_path).join(&app.binary);
                #[cfg(windows)]
                {
                    std::process::Command::new(binary_path)
                        .spawn()
                        .map_err(|e| anyhow::anyhow!("Failed to start {}: {}", app.name, e))?;
                }
                Ok(())
            }
            "hybrid" => {
                self.elevate_service_action("start", &app.name)?;
                let binary_path = PathBuf::from(&app.install_path).join(&app.binary);
                #[cfg(windows)]
                {
                    std::process::Command::new(binary_path)
                        .spawn()
                        .map_err(|e| anyhow::anyhow!("Failed to start {}: {}", app.name, e))?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn stop_app(&self, app_name: &str) -> Result<()> {
        let app = self.find_app(app_name)?;
        match app.app_type.as_str() {
            "service" | "hybrid" => self.elevate_service_action("stop", &app.name),
            "tray_agent" => {
                #[cfg(windows)]
                {
                    let _ = std::process::Command::new("taskkill")
                        .args(["/im", &app.binary, "/f"])
                        .status();
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn configure_app(&self, app_name: &str) -> Result<()> {
        let app = self.find_app(app_name)?;

        if let Some(ref cmd) = app.configure_command {
            let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
            let exe = parts[0];
            let args = parts.get(1).unwrap_or(&"");
            let exe_path = PathBuf::from(&app.install_path).join(exe);

            #[cfg(windows)]
            {
                let mut command = std::process::Command::new(&exe_path);
                if !args.is_empty() {
                    command.args(args.split_whitespace());
                }
                command
                    .spawn()
                    .map_err(|e| anyhow::anyhow!("Failed to run configure command: {}", e))?;
            }

            #[cfg(not(windows))]
            {
                println!(
                    "(Configure: {} {} — requires Windows)",
                    exe_path.display(),
                    args
                );
            }
        } else {
            let data_dir = app.data_dir_path();
            let target = if data_dir.exists() {
                data_dir
            } else {
                PathBuf::from(&app.install_path)
            };
            #[cfg(windows)]
            {
                let _ = std::process::Command::new("explorer")
                    .arg(target.to_str().unwrap_or("."))
                    .spawn();
            }
            #[cfg(not(windows))]
            {
                println!("Configure fallback: open {}", target.display());
            }
        }

        Ok(())
    }

    pub fn open_logs(&self, app_name: &str) -> Result<()> {
        let app = self.find_app(app_name)?;
        let log_dir = app.log_dir_path();

        #[cfg(windows)]
        {
            let _ = std::process::Command::new("explorer")
                .arg(log_dir.to_str().unwrap_or("."))
                .spawn();
        }

        Ok(())
    }

    fn find_app(&self, name: &str) -> Result<&InstalledApp> {
        self.apps
            .iter()
            .find(|a| a.name == name)
            .ok_or_else(|| anyhow::anyhow!("App '{}' not found", name))
    }

    fn elevate_service_action(&self, action: &str, service_name: &str) -> Result<()> {
        let elevate_path = self.find_elevate_helper()?;

        #[cfg(windows)]
        {
            verify_helper_signature(&elevate_path)?;

            let params = format!("{} {}", action, service_name);
            let result = runas_shellexecute(elevate_path.to_str().unwrap(), &params);
            if let Err(e) = result {
                bail!(
                    "UAC elevation failed for '{}' — {}: {}",
                    service_name,
                    action,
                    e
                );
            }
        }

        #[cfg(not(windows))]
        {
            println!(
                "(Elevation helper: {} {} — requires Windows)",
                action, service_name
            );
            let _ = elevate_path;
        }

        Ok(())
    }

    fn find_elevate_helper(&self) -> Result<PathBuf> {
        for app in &self.apps {
            if app.name == "PlenumNET-Launcher" {
                let helper = PathBuf::from(&app.install_path).join("plenum-launcher-elevate.exe");
                if helper.exists() {
                    return Ok(helper);
                }
            }
        }

        #[cfg(windows)]
        {
            let default_path = PathBuf::from(
                r"C:\Program Files\Capomastro\PlenumNET-Launcher\plenum-launcher-elevate.exe",
            );
            if default_path.exists() {
                return Ok(default_path);
            }
        }

        bail!("Elevation helper not found. Reinstall PlenumNET Launcher.")
    }

    fn check_app_status(app: &InstalledApp) -> Result<AppStatus> {
        match app.app_type.as_str() {
            "service" => Self::check_service_status(&app.name),
            "tray_agent" | "hybrid" => Self::check_tray_status(app),
            _ => Ok(AppStatus::Unknown),
        }
    }

    #[cfg(windows)]
    fn discover_from_registry(&mut self) -> Result<()> {
        use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_64KEY};
        use winreg::RegKey;

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let apps_key = hklm.open_subkey_with_flags(
            r"Software\Capomastro\PlenumNET\Apps",
            KEY_READ | KEY_WOW64_64KEY,
        );

        if let Ok(key) = apps_key {
            for name in key.enum_keys().filter_map(|k| k.ok()) {
                if let Ok(product_key) =
                    key.open_subkey_with_flags(&name, KEY_READ | KEY_WOW64_64KEY)
                {
                    let app = InstalledApp {
                        name: name.clone(),
                        display_name: product_key
                            .get_value::<String, _>("DisplayName")
                            .unwrap_or_else(|_| name.clone()),
                        version: product_key
                            .get_value::<String, _>("Version")
                            .unwrap_or_default(),
                        app_type: product_key
                            .get_value::<String, _>("AppType")
                            .unwrap_or_default(),
                        install_path: product_key
                            .get_value::<String, _>("InstallPath")
                            .unwrap_or_default(),
                        data_directory: product_key
                            .get_value::<String, _>("DataDirectory")
                            .unwrap_or_else(|_| name.clone()),
                        binary: product_key
                            .get_value::<String, _>("Binary")
                            .unwrap_or_default(),
                        status: AppStatus::Unknown,
                        status_port: product_key
                            .get_value::<u32, _>("StatusPort")
                            .ok()
                            .map(|p| p as u16),
                        configure_command: product_key
                            .get_value::<String, _>("ConfigureCommand")
                            .ok(),
                    };
                    self.apps.push(app);
                }
            }
        }

        self.refresh_status()?;
        Ok(())
    }

    #[cfg(not(windows))]
    fn discover_from_mock(&mut self) -> Result<()> {
        Ok(())
    }

    #[cfg(windows)]
    fn check_service_status(service_name: &str) -> Result<AppStatus> {
        let output = std::process::Command::new("sc.exe")
            .args(["query", service_name])
            .output();

        match output {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                if stdout.contains("RUNNING") {
                    Ok(AppStatus::Active)
                } else if stdout.contains("STOPPED") {
                    Ok(AppStatus::Inactive)
                } else {
                    Ok(AppStatus::Warning)
                }
            }
            Err(_) => Ok(AppStatus::Unknown),
        }
    }

    #[cfg(not(windows))]
    fn check_service_status(_service_name: &str) -> Result<AppStatus> {
        Ok(AppStatus::Unknown)
    }

    fn check_tray_status(app: &InstalledApp) -> Result<AppStatus> {
        if let Some(port) = app.status_port {
            match std::net::TcpStream::connect_timeout(
                &format!("127.0.0.1:{}", port).parse().unwrap(),
                std::time::Duration::from_secs(2),
            ) {
                Ok(_) => Ok(AppStatus::Active),
                Err(_) => Ok(AppStatus::Inactive),
            }
        } else {
            #[cfg(windows)]
            {
                let output = std::process::Command::new("tasklist")
                    .args(["/fi", &format!("imagename eq {}", app.binary)])
                    .output();
                match output {
                    Ok(o) => {
                        let stdout = String::from_utf8_lossy(&o.stdout);
                        if stdout.contains(&app.binary) {
                            Ok(AppStatus::Active)
                        } else {
                            Ok(AppStatus::Inactive)
                        }
                    }
                    Err(_) => Ok(AppStatus::Unknown),
                }
            }
            #[cfg(not(windows))]
            {
                Ok(AppStatus::Unknown)
            }
        }
    }
}

#[cfg(windows)]
fn verify_helper_signature(helper_path: &PathBuf) -> Result<()> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    fn to_wide_nul(s: &str) -> Vec<u16> {
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    #[repr(C)]
    struct WinTrustFileInfo {
        cb_struct: u32,
        file_path: *const u16,
        file_handle: *mut std::ffi::c_void,
        known_subject: *const [u8; 16],
    }

    #[repr(C)]
    struct WinTrustData {
        cb_struct: u32,
        policy_callback_data: *mut std::ffi::c_void,
        sip_client_data: *mut std::ffi::c_void,
        ui_choice: u32,
        revocation_checks: u32,
        union_choice: u32,
        union_data: *mut std::ffi::c_void,
        state_action: u32,
        state_data: *mut std::ffi::c_void,
        url_reference: *const u16,
        provider_flags: u32,
        ui_context: u32,
        signature_settings: *mut std::ffi::c_void,
    }

    extern "system" {
        fn WinVerifyTrust(
            hwnd: *mut std::ffi::c_void,
            action_id: *const [u8; 16],
            data: *mut WinTrustData,
        ) -> i32;
    }

    const WINTRUST_ACTION_GENERIC_VERIFY_V2: [u8; 16] = [
        0x6d, 0xc5, 0xaa, 0x00, 0x44, 0xcd, 0xd0, 0x11, 0x8c, 0xc2, 0x00, 0xc0, 0x4f, 0xc2,
        0x95, 0xee,
    ];
    const WTD_UI_NONE: u32 = 2;
    const WTD_REVOKE_NONE: u32 = 0;
    const WTD_CHOICE_FILE: u32 = 1;
    const WTD_STATEACTION_VERIFY: u32 = 1;

    let file_path_wide = to_wide_nul(helper_path.to_str().unwrap_or(""));

    let mut file_info = WinTrustFileInfo {
        cb_struct: std::mem::size_of::<WinTrustFileInfo>() as u32,
        file_path: file_path_wide.as_ptr(),
        file_handle: std::ptr::null_mut(),
        known_subject: std::ptr::null(),
    };

    let mut trust_data = WinTrustData {
        cb_struct: std::mem::size_of::<WinTrustData>() as u32,
        policy_callback_data: std::ptr::null_mut(),
        sip_client_data: std::ptr::null_mut(),
        ui_choice: WTD_UI_NONE,
        revocation_checks: WTD_REVOKE_NONE,
        union_choice: WTD_CHOICE_FILE,
        union_data: &mut file_info as *mut WinTrustFileInfo as *mut std::ffi::c_void,
        state_action: WTD_STATEACTION_VERIFY,
        state_data: std::ptr::null_mut(),
        url_reference: std::ptr::null(),
        provider_flags: 0,
        ui_context: 0,
        signature_settings: std::ptr::null_mut(),
    };

    let result = unsafe {
        WinVerifyTrust(
            std::ptr::null_mut(),
            &WINTRUST_ACTION_GENERIC_VERIFY_V2,
            &mut trust_data,
        )
    };

    if result != 0 {
        bail!(
            "Elevation helper signature verification failed (error 0x{:08X}). \
             Refusing to elevate an unsigned or tampered binary: {}",
            result,
            helper_path.display()
        );
    }

    Ok(())
}

#[cfg(windows)]
fn runas_shellexecute(exe_path: &str, params: &str) -> Result<()> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;

    fn to_wide(s: &str) -> Vec<u16> {
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    let verb = to_wide("runas");
    let file = to_wide(exe_path);
    let parameters = to_wide(params);

    #[repr(C)]
    struct ShellExecuteInfoW {
        cb_size: u32,
        f_mask: u32,
        hwnd: *mut std::ffi::c_void,
        lp_verb: *const u16,
        lp_file: *const u16,
        lp_parameters: *const u16,
        lp_directory: *const u16,
        n_show: i32,
        h_inst_app: *mut std::ffi::c_void,
        lp_id_list: *mut std::ffi::c_void,
        lp_class: *const u16,
        hkey_class: *mut std::ffi::c_void,
        dw_hot_key: u32,
        h_icon_or_monitor: *mut std::ffi::c_void,
        h_process: *mut std::ffi::c_void,
    }

    extern "system" {
        fn ShellExecuteExW(pExecInfo: *mut ShellExecuteInfoW) -> i32;
        fn WaitForSingleObject(hHandle: *mut std::ffi::c_void, dwMilliseconds: u32) -> u32;
        fn CloseHandle(hObject: *mut std::ffi::c_void) -> i32;
        fn GetExitCodeProcess(hProcess: *mut std::ffi::c_void, lpExitCode: *mut u32) -> i32;
    }

    const SEE_MASK_NOCLOSEPROCESS: u32 = 0x00000040;
    const SW_HIDE: i32 = 0;
    const INFINITE: u32 = 0xFFFFFFFF;

    let mut info = ShellExecuteInfoW {
        cb_size: std::mem::size_of::<ShellExecuteInfoW>() as u32,
        f_mask: SEE_MASK_NOCLOSEPROCESS,
        hwnd: ptr::null_mut(),
        lp_verb: verb.as_ptr(),
        lp_file: file.as_ptr(),
        lp_parameters: parameters.as_ptr(),
        lp_directory: ptr::null(),
        n_show: SW_HIDE,
        h_inst_app: ptr::null_mut(),
        lp_id_list: ptr::null_mut(),
        lp_class: ptr::null(),
        hkey_class: ptr::null_mut(),
        dw_hot_key: 0,
        h_icon_or_monitor: ptr::null_mut(),
        h_process: ptr::null_mut(),
    };

    let success = unsafe { ShellExecuteExW(&mut info) };
    if success == 0 {
        bail!("ShellExecuteEx failed — user may have declined UAC prompt");
    }

    if !info.h_process.is_null() {
        unsafe {
            WaitForSingleObject(info.h_process, INFINITE);
            let mut exit_code: u32 = 0;
            GetExitCodeProcess(info.h_process, &mut exit_code);
            CloseHandle(info.h_process);
            if exit_code != 0 {
                bail!(
                    "Elevation helper exited with code {} — check Event Viewer for details",
                    exit_code
                );
            }
        }
    }

    Ok(())
}

#[cfg(not(windows))]
fn runas_shellexecute(_exe_path: &str, _params: &str) -> Result<()> {
    Ok(())
}
