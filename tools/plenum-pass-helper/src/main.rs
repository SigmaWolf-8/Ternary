use anyhow::{bail, Result};
use clap::Parser;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "plenum-pass-helper",
    about = "Secure passphrase collection for PlenumNET installer first-run"
)]
struct Cli {
    #[arg(long, default_value = "12", help = "Minimum passphrase length")]
    min_length: usize,

    #[arg(long, help = "Require passphrase confirmation")]
    confirm: bool,

    #[arg(long, help = "Product display name for prompt")]
    product_name: Option<String>,

    #[arg(
        long,
        help = "Read passphrase from file (silent install mode). File must have restricted ACLs."
    )]
    from_file: Option<PathBuf>,

    #[arg(
        long,
        help = "Read passphrase from stdin (pipe mode for automated workflows)"
    )]
    from_stdin: bool,

    #[arg(
        long,
        help = "Write passphrase from stdin to a secure temp file with restricted ACLs. \
                Used by MSI GUI install to bridge immediate-to-deferred CA data flow \
                without exposing secrets on the command line."
    )]
    write_secure_temp: Option<PathBuf>,

    #[arg(
        long,
        help = "Securely delete a temp passphrase file created by --write-secure-temp"
    )]
    cleanup_temp: Option<PathBuf>,

    #[arg(
        long,
        help = "Environment variable name for passphrase injection into child process"
    )]
    exec_env: Option<String>,

    #[arg(trailing_var_arg = true, help = "Child command to execute (after --)")]
    child_args: Vec<String>,
}

fn log_to_install_file(product: &str, message: &str) {
    let temp = std::env::var("TEMP")
        .or_else(|_| std::env::var("TMP"))
        .unwrap_or_else(|_| String::from("."));
    let log_name = format!(
        "PlenumNET_{}_install.log",
        product
            .replace(' ', "_")
            .replace(|c: char| !c.is_alphanumeric() && c != '_', "")
    );
    let log_path = std::path::Path::new(&temp).join(log_name);
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        use std::time::SystemTime;
        let elapsed = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let _ = writeln!(f, "[{}] {}", elapsed, message);
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let product = cli
        .product_name
        .as_deref()
        .unwrap_or("PlenumNET Application");

    if let Some(ref temp_path) = cli.write_secure_temp {
        return write_secure_temp_file(temp_path, product);
    }

    if let Some(ref temp_path) = cli.cleanup_temp {
        return cleanup_temp_file(temp_path, product);
    }

    let passphrase = if cli.from_stdin {
        let mut pass = String::new();
        io::stdin().read_line(&mut pass)?;
        let pass = pass.trim().to_string();
        if pass.is_empty() {
            let msg = "No passphrase provided on stdin. \
                 Pipe the passphrase into this process or use --from-file instead.";
            log_to_install_file(product, msg);
            bail!("{}", msg);
        }
        pass
    } else if let Some(ref source_file) = cli.from_file {
        if let Err(e) = validate_source_file_security(source_file) {
            log_to_install_file(
                product,
                &format!("PASSPHRASE_FILE security check failed: {}", e),
            );
            return Err(e);
        }
        let content = std::fs::read_to_string(source_file).map_err(|e| {
            let msg = format!(
                "Failed to read PASSPHRASE_FILE '{}': {}",
                source_file.display(),
                e
            );
            log_to_install_file(product, &msg);
            anyhow::anyhow!("{}", msg)
        })?;
        let stripped = content
            .strip_suffix("\r\n")
            .or_else(|| content.strip_suffix('\n'))
            .unwrap_or(&content)
            .to_string();
        if stripped.is_empty() {
            let msg = format!("PASSPHRASE_FILE is empty: {}", source_file.display());
            log_to_install_file(product, &msg);
            bail!("{}", msg);
        }
        stripped
    } else {
        println!("=== {} First-Run Setup ===", product);
        println!();
        println!("Create a passphrase to protect your signing key.");
        println!("Minimum {} characters required.", cli.min_length);
        println!();

        let p = read_passphrase("Enter passphrase: ")?;

        if cli.confirm {
            let confirmed = read_passphrase("Confirm passphrase: ")?;
            if p != confirmed {
                bail!("Passphrases do not match");
            }
        }
        p
    };

    if passphrase.chars().count() < cli.min_length {
        let msg = format!(
            "Passphrase too short: {} characters (minimum {})",
            passphrase.chars().count(),
            cli.min_length
        );
        log_to_install_file(product, &msg);
        bail!("{}", msg);
    }

    if let Some(ref env_var) = cli.exec_env {
        if cli.child_args.is_empty() {
            bail!("--exec-env requires a child command after --");
        }
        let child_exe = &cli.child_args[0];
        let child_args = &cli.child_args[1..];

        let status = std::process::Command::new(child_exe)
            .args(child_args)
            .env(env_var, &passphrase)
            .status()
            .map_err(|e| anyhow::anyhow!("Failed to spawn child process '{}': {}", child_exe, e))?;

        zeroize_string(passphrase);

        if !status.success() {
            bail!(
                "Child process '{}' exited with code {}",
                child_exe,
                status.code().unwrap_or(-1)
            );
        }

        println!("First-run initialization completed successfully.");
    } else {
        zeroize_string(passphrase);
        println!("Passphrase accepted.");
    }

    Ok(())
}

fn write_secure_temp_file(temp_path: &PathBuf, product: &str) -> Result<()> {
    let mut passphrase = String::new();
    io::stdin().read_line(&mut passphrase)?;
    let passphrase = passphrase.trim_end_matches(|c| c == '\n' || c == '\r');

    if passphrase.is_empty() {
        let msg = "No passphrase provided on stdin for --write-secure-temp.";
        log_to_install_file(product, msg);
        bail!("{}", msg);
    }

    if let Some(parent) = temp_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    {
        let mut f = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(temp_path)
            .map_err(|e| {
                let msg = format!(
                    "Failed to create secure temp file '{}': {}",
                    temp_path.display(),
                    e
                );
                log_to_install_file(product, &msg);
                anyhow::anyhow!("{}", msg)
            })?;
        f.write_all(passphrase.as_bytes())?;
        f.flush()?;
    }

    #[cfg(windows)]
    {
        let path_str = temp_path.display().to_string();
        let icacls_result = std::process::Command::new("icacls")
            .args([
                &path_str,
                "/inheritance:r",
                "/grant:r",
                &format!("{}:(R)", whoami()),
            ])
            .output();
        match icacls_result {
            Ok(o) if o.status.success() => {}
            Ok(o) => {
                let _ = std::fs::remove_file(temp_path);
                let msg = format!(
                    "Failed to set ACLs on temp passphrase file: icacls exited with {}",
                    o.status
                );
                log_to_install_file(product, &msg);
                bail!("{}", msg);
            }
            Err(e) => {
                let _ = std::fs::remove_file(temp_path);
                let msg = format!("Failed to execute icacls for ACL hardening: {}", e);
                log_to_install_file(product, &msg);
                bail!("{}", msg);
            }
        }
    }

    log_to_install_file(
        product,
        &format!(
            "Secure temp passphrase file written: {}",
            temp_path.display()
        ),
    );
    Ok(())
}

fn cleanup_temp_file(temp_path: &PathBuf, product: &str) -> Result<()> {
    if temp_path.exists() {
        if let Ok(metadata) = std::fs::metadata(temp_path) {
            let len = metadata.len() as usize;
            if let Ok(mut f) = std::fs::OpenOptions::new().write(true).open(temp_path) {
                let zeros = vec![0u8; len];
                let _ = f.write_all(&zeros);
                let _ = f.flush();
            }
        }
        std::fs::remove_file(temp_path).map_err(|e| {
            let msg = format!(
                "Failed to remove temp passphrase file '{}': {}",
                temp_path.display(),
                e
            );
            log_to_install_file(product, &msg);
            anyhow::anyhow!("{}", msg)
        })?;
        log_to_install_file(
            product,
            &format!(
                "Secure temp passphrase file cleaned up: {}",
                temp_path.display()
            ),
        );
    }
    Ok(())
}

#[cfg(windows)]
fn whoami() -> String {
    std::env::var("USERNAME").unwrap_or_else(|_| "CURRENT_USER".to_string())
}

fn zeroize_string(mut s: String) {
    unsafe {
        let bytes = s.as_bytes_mut();
        for b in bytes.iter_mut() {
            std::ptr::write_volatile(b, 0);
        }
    }
    drop(s);
}

fn validate_source_file_security(path: &PathBuf) -> Result<()> {
    if !path.exists() {
        bail!(
            "PASSPHRASE_FILE does not exist: {}. Error code 1603.",
            path.display()
        );
    }

    let path_str = path.to_str().unwrap_or("");

    if path_str.starts_with("\\\\") {
        if path_str.contains(':') {
            bail!(
                "PASSPHRASE_FILE UNC path contains alternate data stream reference: {}. Rejected for security.",
                path.display()
            );
        }
    } else if path_str.len() >= 2 && path_str.as_bytes()[1] == b':' {
        let after_drive = &path_str[2..];
        if after_drive.contains(':') {
            bail!(
                "PASSPHRASE_FILE contains alternate data stream reference: {}. Rejected for security.",
                path.display()
            );
        }
    } else if path_str.contains(':') {
        bail!(
            "PASSPHRASE_FILE contains alternate data stream reference: {}. Rejected for security.",
            path.display()
        );
    }

    #[cfg(windows)]
    {
        let script = r#"param([string]$FilePath)
$acl = Get-Acl -LiteralPath $FilePath
$currentSid = [System.Security.Principal.WindowsIdentity]::GetCurrent().User.Value
foreach($ace in $acl.Access) {
    if ($ace.AccessControlType -ne 'Allow') { continue }
    if ($ace.FileSystemRights -band [System.Security.AccessControl.FileSystemRights]::Read) {
        $sid = $ace.IdentityReference.Translate([System.Security.Principal.SecurityIdentifier]).Value
        if ($sid -ne $currentSid) {
            Write-Output "DENIED:$sid"
            exit 1
        }
    }
}
Write-Output 'OK'"#;
        let output = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                script,
                "-FilePath",
                &path.display().to_string(),
            ])
            .output();
        match output {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                if stdout.contains("DENIED:") || !o.status.success() {
                    bail!(
                        "PASSPHRASE_FILE has insecure ACL: {}. \
                         Only the installing user may have read access. \
                         Found unauthorized read principal. \
                         Restrict file ACLs before retrying. Error code 1603.",
                        path.display()
                    );
                }
                if !stdout.contains("OK") {
                    bail!(
                        "PASSPHRASE_FILE ACL check returned unexpected output for {}. \
                         Cannot verify file permissions. Error code 1603.",
                        path.display()
                    );
                }
            }
            Err(e) => {
                bail!(
                    "Cannot verify PASSPHRASE_FILE permissions: PowerShell ACL check failed \
                     ({}). Ensure PowerShell is available. Error code 1603.",
                    e
                );
            }
        }
    }

    Ok(())
}

fn read_passphrase(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;

    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawHandle;
        let handle = io::stdin();
        let raw = handle.as_raw_handle();
        let mut mode: u32 = 0;
        unsafe {
            let kernel32 = windows_sys::Win32::System::Console::GetConsoleMode;
            kernel32(raw as isize, &mut mode);
            windows_sys::Win32::System::Console::SetConsoleMode(
                raw as isize,
                mode & !windows_sys::Win32::System::Console::ENABLE_ECHO_INPUT,
            );
        }
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        unsafe {
            windows_sys::Win32::System::Console::SetConsoleMode(raw as isize, mode);
        }
        println!();
        Ok(input.trim().to_string())
    }

    #[cfg(not(windows))]
    {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        println!();
        Ok(input.trim().to_string())
    }
}
