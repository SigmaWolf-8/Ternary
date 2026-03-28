mod manifest;
mod tis27;
mod wix;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use manifest::{generate_product_code, generate_upgrade_code, Manifest};
use std::path::{Path, PathBuf};
use wix::WixGenerator;

#[derive(Parser)]
#[command(
    name = "plenum-pack",
    version = "1.0.0",
    about = "Manifest-driven MSI build tool for PlenumNET applications"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Build {
        #[arg(long)]
        arch: Option<String>,
        #[arg(long)]
        sign: bool,
        #[arg(long, help = "Generate WiX .wxs source files without compiling")]
        dry_run: bool,
        #[arg(long, default_value = ".")]
        manifest_dir: PathBuf,
        #[arg(
            long,
            help = "Directory containing compiled binaries (overrides default target/ lookup)"
        )]
        binary_dir: Option<PathBuf>,
    },
    Validate {
        #[arg(long)]
        workspace: bool,
        #[arg(long, default_value = ".")]
        manifest_dir: PathBuf,
        #[arg(long, help = "Also check that referenced binaries exist on disk")]
        check_binaries: bool,
    },
    Inspect {
        msi_path: PathBuf,
        #[arg(long, help = "Manifest directory for metadata extraction")]
        manifest_dir: Option<PathBuf>,
    },
    New {
        name: String,
        #[arg(long, default_value = ".")]
        output_dir: PathBuf,
    },
    Verify {
        checksums_file: PathBuf,
    },
    Checksum {
        #[arg(help = "Files to compute TIS-27 checksums for")]
        files: Vec<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build {
            arch,
            sign,
            dry_run,
            manifest_dir,
            binary_dir,
        } => cmd_build(&manifest_dir, arch.as_deref(), sign, dry_run, binary_dir),
        Commands::Validate {
            workspace,
            manifest_dir,
            check_binaries,
        } => cmd_validate(&manifest_dir, workspace, check_binaries),
        Commands::Inspect {
            msi_path,
            manifest_dir,
        } => cmd_inspect(&msi_path, manifest_dir.as_deref()),
        Commands::New { name, output_dir } => cmd_new(&name, &output_dir),
        Commands::Verify { checksums_file } => cmd_verify(&checksums_file),
        Commands::Checksum { files } => cmd_checksum(&files),
    }
}

fn cmd_build(
    manifest_dir: &Path,
    arch_filter: Option<&str>,
    sign: bool,
    dry_run: bool,
    binary_dir_override: Option<PathBuf>,
) -> Result<()> {
    let manifest_path = manifest_dir.join("plenum-app.toml");
    let manifest = Manifest::load(&manifest_path)?;

    let errors = manifest.validate_schema_only(manifest_dir)?;
    if !errors.is_empty() {
        eprintln!("Manifest validation failed:");
        for e in &errors {
            eprintln!("  - {}", e);
        }
        bail!("{} validation error(s) found", errors.len());
    }

    let output_dir = manifest_dir.join("plenum-pack-output");
    std::fs::create_dir_all(&output_dir)?;

    for arch in &manifest.install.architecture {
        if let Some(filter) = arch_filter {
            let matches = match filter {
                "aarch64" | "arm64" => arch == &manifest::Architecture::Aarch64,
                "x86_64" | "x64" => arch == &manifest::Architecture::X86_64,
                _ => bail!("Unknown architecture filter: {}", filter),
            };
            if !matches {
                continue;
            }
        }

        let product_code = generate_product_code(&manifest.app.upgrade_code, &manifest.app.version);
        let generator =
            WixGenerator::new(&manifest, arch, product_code, manifest_dir.to_path_buf());
        let wix_output = generator.generate()?;

        let wxs_filename = format!(
            "{}-{}-{}.wxs",
            manifest.app.name,
            manifest.app.version,
            arch.msi_suffix()
        );
        let wxs_path = output_dir.join(&wxs_filename);
        std::fs::write(&wxs_path, &wix_output.product_wxs)?;
        println!("Generated WiX source: {}", wxs_path.display());

        let dialogs_wxs_path = output_dir.join("dialogs.wxs");
        std::fs::write(&dialogs_wxs_path, &wix_output.dialogs_wxs)?;
        println!("Generated WiX dialogs: {}", dialogs_wxs_path.display());

        if dry_run {
            println!("Dry run — skipping WiX compilation and signing");
            continue;
        }

        if !check_wix_available() {
            bail!(
                "WiX Toolset v4 is not installed. Install it with:\n  \
                 dotnet tool install --global wix\n  \
                 Requires .NET 8.0 SDK or later."
            );
        }

        let msi_filename = format!(
            "{}-{}-{}.msi",
            manifest.app.name,
            manifest.app.version,
            arch.msi_suffix()
        );
        let msi_path = output_dir.join(&msi_filename);

        let binary_source_dir =
            resolve_binary_dir(binary_dir_override.as_deref(), manifest_dir, arch)?;

        let main_binary_path = binary_source_dir.join(&manifest.app.binary);
        if !main_binary_path.exists() {
            bail!(
                "Main binary '{}' not found at {}. Build the project first or specify --binary-dir.",
                manifest.app.binary,
                main_binary_path.display()
            );
        }
        for extra in &manifest.install.extra_binaries {
            let extra_path = binary_source_dir.join(extra);
            if !extra_path.exists() {
                bail!(
                    "Extra binary '{}' not found at {}. Build the project first or specify --binary-dir.",
                    extra,
                    extra_path.display()
                );
            }
        }
        if manifest.first_run.as_ref().map_or(false, |fr| {
            fr.actions
                .iter()
                .any(|a| matches!(a, manifest::FirstRunAction::PromptPassphrase { .. }))
        }) {
            let helper_path = binary_source_dir.join("plenum-pass-helper.exe");
            if !helper_path.exists() {
                bail!(
                    "Passphrase helper 'plenum-pass-helper.exe' not found at {}. Build plenum-pass-helper first.",
                    helper_path.display()
                );
            }
            let ca_dll_path = binary_source_dir.join("plenum_pass_ca.dll");
            if !ca_dll_path.exists() {
                bail!(
                    "Passphrase DLL CA 'plenum_pass_ca.dll' not found at {}. Build plenum-pass-ca first.",
                    ca_dll_path.display()
                );
            }
        }

        let installer_assets_dir = find_installer_assets_dir(manifest_dir)?;
        let build_args = generator.wix_build_args(
            wxs_path.to_str().unwrap(),
            dialogs_wxs_path.to_str().unwrap(),
            msi_path.to_str().unwrap(),
            binary_source_dir.to_str().unwrap(),
            manifest_dir.to_str().unwrap(),
            installer_assets_dir.to_str().unwrap(),
        );

        if sign {
            let binary_path = binary_source_dir.join(&manifest.app.binary);
            println!("Signing binary: {}", binary_path.display());
            sign_binary(&binary_path)?;

            for extra in &manifest.install.extra_binaries {
                let extra_path = binary_source_dir.join(extra);
                if extra_path.exists() {
                    println!("Signing extra binary: {}", extra_path.display());
                    sign_binary(&extra_path)?;
                }
            }

            if manifest.first_run.as_ref().map_or(false, |fr| {
                fr.actions
                    .iter()
                    .any(|a| matches!(a, manifest::FirstRunAction::PromptPassphrase { .. }))
            }) {
                let helper_path = binary_source_dir.join("plenum-pass-helper.exe");
                if helper_path.exists() {
                    println!("Signing pass helper: {}", helper_path.display());
                    sign_binary(&helper_path)?;
                }
                let ca_dll_path = binary_source_dir.join("plenum_pass_ca.dll");
                if ca_dll_path.exists() {
                    println!("Signing passphrase CA DLL: {}", ca_dll_path.display());
                    sign_binary(&ca_dll_path)?;
                }
            }
        }

        println!("Compiling MSI: {}", msi_path.display());
        println!("Binary source: {}", binary_source_dir.display());

        let wix_version_raw = std::process::Command::new("wix")
            .args(["--version"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
        let wix_version = wix_version_raw.split('+').next().unwrap_or("").to_string();
        println!("WiX version: {}", if wix_version.is_empty() { "unknown" } else { &wix_version });

        let mut required_exts = vec!["WixToolset.UI.wixext", "WixToolset.Util.wixext"];
        if matches!(
            manifest.app_type.kind,
            manifest::AppKind::Service | manifest::AppKind::Hybrid
        ) {
            required_exts.push("WixToolset.Firewall.wixext");
        }
        for ext in &required_exts {
            let ext_versioned = if wix_version.is_empty() {
                ext.to_string()
            } else {
                format!("{}/{}", ext, wix_version)
            };
            println!("Ensuring WiX extension: {}", ext_versioned);
            let add_result = std::process::Command::new("wix")
                .args(["extension", "add", "--global", &ext_versioned])
                .output();
            match add_result {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if !output.status.success() {
                        if stderr.contains("already exists") || stdout.contains("already exists") {
                            println!("  [OK] {} (already installed)", ext);
                        } else {
                            eprintln!("  WARN: wix extension add {} failed: {}", ext_versioned, stderr.trim());
                        }
                    } else {
                        println!("  [OK] {} installed", ext);
                    }
                }
                Err(e) => eprintln!("  WARN: Failed to run wix extension add: {}", e),
            }
        }

        let status = std::process::Command::new("wix")
            .args(&build_args)
            .status()
            .context("Failed to execute WiX build command")?;

        if !status.success() {
            bail!("WiX build failed for {}", wxs_filename);
        }

        if sign {
            println!("Signing MSI: {}", msi_path.display());
            sign_msi(&msi_path)?;
        }

        println!("Built MSI: {}", msi_path.display());
    }

    Ok(())
}

fn resolve_binary_dir(
    cli_override: Option<&Path>,
    manifest_dir: &Path,
    arch: &manifest::Architecture,
) -> Result<PathBuf> {
    if let Some(dir) = cli_override {
        return Ok(dir.to_path_buf());
    }

    if let Ok(env_dir) = std::env::var("PLENUM_BINARY_DIR") {
        return Ok(PathBuf::from(env_dir));
    }

    let workspace_root = find_workspace_root(manifest_dir);
    let candidate = workspace_root
        .join("target")
        .join(arch.rust_target())
        .join("release");
    if candidate.exists() {
        return Ok(candidate);
    }

    let local_candidate = manifest_dir
        .join("target")
        .join(arch.rust_target())
        .join("release");
    Ok(local_candidate)
}

fn find_workspace_root(start: &Path) -> PathBuf {
    let mut dir = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
    loop {
        if dir.join("Cargo.toml").exists() {
            let content = std::fs::read_to_string(dir.join("Cargo.toml")).unwrap_or_default();
            if content.contains("[workspace]") {
                return dir;
            }
        }
        if !dir.pop() {
            break;
        }
    }
    start.to_path_buf()
}

fn find_installer_assets_dir(manifest_dir: &Path) -> Result<PathBuf> {
    let local_assets = manifest_dir.join("assets").join("installer");
    if local_assets.is_dir() {
        return Ok(local_assets);
    }

    let workspace_root = find_workspace_root(manifest_dir);
    let workspace_assets = workspace_root
        .join("assets")
        .join("icons")
        .join("installer");
    if workspace_assets.is_dir() {
        return Ok(workspace_assets);
    }

    bail!(
        "Installer assets directory not found. Looked in:\n  {}\n  {}",
        local_assets.display(),
        workspace_assets.display()
    );
}

fn cmd_validate(manifest_dir: &Path, workspace: bool, check_binaries: bool) -> Result<()> {
    if workspace {
        let manifests = discover_manifests(manifest_dir)?;
        let mut all_errors = Vec::new();
        let mut loaded: Vec<(&Path, Manifest)> = Vec::new();

        for manifest_path in &manifests {
            let dir = manifest_path.parent().unwrap_or(manifest_dir);
            match Manifest::load(manifest_path) {
                Ok(m) => {
                    let errors = if check_binaries {
                        m.validate(dir)?
                    } else {
                        m.validate_schema_only(dir)?
                    };
                    for e in &errors {
                        all_errors.push(format!("{}: {}", manifest_path.display(), e));
                    }
                    loaded.push((manifest_path.as_path(), m));
                }
                Err(e) => {
                    all_errors.push(format!("{}: {}", manifest_path.display(), e));
                }
            }
        }

        let workspace_errors = manifest::validate_workspace(
            &loaded
                .iter()
                .map(|(p, m)| (*p, m.clone()))
                .collect::<Vec<_>>(),
        );
        all_errors.extend(workspace_errors);

        if all_errors.is_empty() {
            println!("All {} manifests validated successfully", manifests.len());
            Ok(())
        } else {
            eprintln!("Workspace validation failed:");
            for e in &all_errors {
                eprintln!("  - {}", e);
            }
            bail!("{} error(s) found across workspace", all_errors.len());
        }
    } else {
        let manifest_path = manifest_dir.join("plenum-app.toml");
        let manifest = Manifest::load(&manifest_path)?;
        let errors = if check_binaries {
            manifest.validate(manifest_dir)?
        } else {
            manifest.validate_schema_only(manifest_dir)?
        };
        if errors.is_empty() {
            println!(
                "Manifest validated successfully: {}",
                manifest.app.display_name
            );
            Ok(())
        } else {
            eprintln!("Validation failed:");
            for e in &errors {
                eprintln!("  - {}", e);
            }
            bail!("{} validation error(s) found", errors.len());
        }
    }
}

fn cmd_inspect(msi_path: &Path, manifest_dir: Option<&Path>) -> Result<()> {
    if !msi_path.exists() {
        bail!("MSI file not found: {}", msi_path.display());
    }

    let metadata = std::fs::metadata(msi_path)?;
    println!("MSI Inspection: {}", msi_path.display());
    println!("  File size: {} bytes", metadata.len());
    println!("---");

    let mut msi_properties: Vec<(String, String)> = Vec::new();
    let mut msi_properties_extracted = false;

    let wix_query_output = std::process::Command::new("wix")
        .args([
            "msi",
            "listproperties",
            msi_path.to_str().unwrap_or_default(),
        ])
        .output();

    if let Ok(o) = wix_query_output {
        if o.status.success() {
            msi_properties_extracted = true;
            let stdout = String::from_utf8_lossy(&o.stdout);
            println!("MSI Properties (from MSI tables):");
            for line in stdout.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    println!("  {}", trimmed);
                    if let Some((key, value)) = trimmed.split_once('=') {
                        msi_properties.push((key.trim().to_string(), value.trim().to_string()));
                    }
                }
            }
        }
    }

    if !msi_properties_extracted {
        let msiinfo_output = std::process::Command::new("msiinfo")
            .args(["suminfo", msi_path.to_str().unwrap()])
            .output();
        match msiinfo_output {
            Ok(o) if o.status.success() => {
                msi_properties_extracted = true;
                let stdout = String::from_utf8_lossy(&o.stdout);
                println!("MSI Summary Information:");
                for line in stdout.lines() {
                    println!("  {}", line);
                }
            }
            _ => {}
        }
    }

    if !msi_properties_extracted {
        println!("Note: WiX Toolset and msiinfo not available for direct MSI table queries.");
        println!("Extracting basic properties from MSI file header...");
        extract_msi_header_properties(msi_path);
    }

    if let Some(dir) = manifest_dir {
        let manifest_path = dir.join("plenum-app.toml");
        if manifest_path.exists() {
            println!("\nManifest metadata:");
            match Manifest::load(&manifest_path) {
                Ok(m) => {
                    println!("  Name:         {}", m.app.name);
                    println!("  Display Name: {}", m.app.display_name);
                    println!("  Version:      {}", m.app.version);
                    println!("  Publisher:     {}", m.app.publisher);
                    println!("  Upgrade Code: {}", m.app.upgrade_code);
                    println!("  App Type:     {}", m.app_type.kind);
                    println!("  Binary:       {}", m.app.binary);
                    println!("  Install Dir:  {}", m.install.directory);
                    println!(
                        "  Architectures: {}",
                        m.install
                            .architecture
                            .iter()
                            .map(|a| a.wix_platform().to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                    if let Some(desc) = &m.app.description {
                        println!("  Description:  {}", desc);
                    }

                    if msi_properties_extracted && !msi_properties.is_empty() {
                        println!("\nManifest vs MSI comparison:");
                        let mut mismatches = 0;

                        let find_prop = |key: &str| -> Option<&str> {
                            msi_properties
                                .iter()
                                .find(|(k, _)| k == key)
                                .map(|(_, v)| v.as_str())
                        };

                        if let Some(msi_name) = find_prop("ProductName") {
                            if msi_name != m.app.display_name {
                                eprintln!(
                                    "  MISMATCH ProductName: manifest='{}', MSI='{}'",
                                    m.app.display_name, msi_name
                                );
                                mismatches += 1;
                            } else {
                                println!("  OK ProductName: {}", msi_name);
                            }
                        }

                        if let Some(msi_version) = find_prop("ProductVersion") {
                            if msi_version != m.app.version {
                                eprintln!(
                                    "  MISMATCH ProductVersion: manifest='{}', MSI='{}'",
                                    m.app.version, msi_version
                                );
                                mismatches += 1;
                            } else {
                                println!("  OK ProductVersion: {}", msi_version);
                            }
                        }

                        if let Some(msi_manufacturer) = find_prop("Manufacturer") {
                            if msi_manufacturer != m.app.publisher {
                                eprintln!(
                                    "  MISMATCH Manufacturer: manifest='{}', MSI='{}'",
                                    m.app.publisher, msi_manufacturer
                                );
                                mismatches += 1;
                            } else {
                                println!("  OK Manufacturer: {}", msi_manufacturer);
                            }
                        }

                        if let Some(msi_upgrade_code) = find_prop("UpgradeCode") {
                            let normalized_msi = msi_upgrade_code
                                .replace('{', "")
                                .replace('}', "")
                                .to_uppercase();
                            let normalized_manifest = m.app.upgrade_code.to_uppercase();
                            if normalized_msi != normalized_manifest {
                                eprintln!(
                                    "  MISMATCH UpgradeCode: manifest='{}', MSI='{}'",
                                    m.app.upgrade_code, msi_upgrade_code
                                );
                                mismatches += 1;
                            } else {
                                println!("  OK UpgradeCode: {}", msi_upgrade_code);
                            }
                        }

                        if mismatches > 0 {
                            eprintln!("\n{} metadata mismatch(es) detected", mismatches);
                            std::process::exit(1);
                        } else {
                            println!("\nAll metadata matches.");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("  (Failed to load manifest: {})", e);
                }
            }
        }
    }

    Ok(())
}

fn extract_msi_header_properties(msi_path: &Path) {
    if let Ok(bytes) = std::fs::read(msi_path) {
        println!("MSI Header Analysis:");
        if bytes.len() >= 8 {
            let magic = &bytes[0..8];
            if magic[0..4] == [0xD0, 0xCF, 0x11, 0xE0] {
                println!("  Format: OLE Compound Document (valid MSI)");
                if bytes.len() >= 30 {
                    let minor_ver = u16::from_le_bytes([bytes[24], bytes[25]]);
                    let major_ver = u16::from_le_bytes([bytes[26], bytes[27]]);
                    println!("  OLE version: {}.{}", major_ver, minor_ver);
                }
            } else {
                println!("  WARNING: File does not have valid MSI/OLE header signature");
            }
        }
        println!("  (Install WiX Toolset or msitools for full MSI property extraction)");
    }
}

fn cmd_new(name: &str, output_dir: &Path) -> Result<()> {
    let upgrade_code = generate_upgrade_code();
    let manifest_content = format!(
        r#"[app]
name = "{name}"
display_name = "{name} — PlenumNET Application"
description = "A PlenumNET application"
version = "0.1.0"
publisher = "Capomastro Holdings Ltd."
binary = "{binary}.exe"
icon = "assets/{binary}.ico"
license = "Proprietary"
upgrade_code = "{upgrade_code}"

[install]
directory = "Capomastro\\{name}"
data_directory = "{name}"
add_to_path = false
architecture = ["aarch64", "x86_64"]

[app_type]
kind = "cli_tool"
autostart = false

[shortcuts]
start_menu = [
    {{ name = "{name}", target = "{binary}.exe", icon = "assets/{binary}.ico" }},
]

[uninstall]
preserve_data = true
preserve_message = "Your {name} data has been preserved in %APPDATA%\\{name}. Delete manually if no longer needed."
"#,
        name = name,
        binary = name.to_lowercase().replace(' ', "-"),
        upgrade_code = upgrade_code,
    );

    let manifest_path = output_dir.join("plenum-app.toml");
    std::fs::write(&manifest_path, &manifest_content)?;
    println!("Created new manifest: {}", manifest_path.display());
    println!("Upgrade code: {}", upgrade_code);

    Ok(())
}

fn cmd_verify(checksums_file: &Path) -> Result<()> {
    let content =
        std::fs::read_to_string(checksums_file).context("Failed to read checksums file")?;

    let mut failures = 0;
    let mut successes = 0;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.splitn(2, "  ").collect();
        if parts.len() != 2 {
            eprintln!("Malformed line: {}", line);
            failures += 1;
            continue;
        }

        let expected_hash = parts[0];
        let filename = parts[1];

        if !Path::new(filename).exists() {
            eprintln!("FAIL: {} (file not found)", filename);
            failures += 1;
            continue;
        }

        let actual_hash = compute_tis27(Path::new(filename))?;
        if actual_hash == expected_hash {
            println!("OK: {}", filename);
            successes += 1;
        } else {
            eprintln!(
                "FAIL: {} (expected {}, got {})",
                filename, expected_hash, actual_hash
            );
            failures += 1;
        }
    }

    println!("\n{} files verified, {} failures", successes, failures);
    if failures > 0 {
        bail!("{} verification failure(s)", failures);
    }
    Ok(())
}

fn cmd_checksum(files: &[PathBuf]) -> Result<()> {
    for file_path in files {
        if !file_path.exists() {
            bail!("File not found: {}", file_path.display());
        }
        let hash = compute_tis27(file_path)?;
        println!("{}  {}", hash, file_path.display());
    }
    Ok(())
}

fn compute_tis27(path: &Path) -> Result<String> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    Ok(tis27::hash_hex_tis(&data))
}

fn discover_manifests(workspace_root: &Path) -> Result<Vec<PathBuf>> {
    let mut manifests = Vec::new();
    for entry in walkdir::WalkDir::new(workspace_root)
        .max_depth(4)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_name() == "plenum-app.toml" {
            manifests.push(entry.path().to_path_buf());
        }
    }
    Ok(manifests)
}

fn check_wix_available() -> bool {
    std::process::Command::new("wix")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

const TIMESTAMP_RETRY_COUNT: u32 = 3;
const TIMESTAMP_RETRY_DELAY_SECS: u64 = 30;

fn run_signtool_with_retry(args: &[&str], description: &str) -> Result<()> {
    for attempt in 1..=TIMESTAMP_RETRY_COUNT {
        let status = std::process::Command::new("signtool")
            .args(args)
            .status()
            .context(format!("Failed to execute signtool for {}", description))?;

        if status.success() {
            return Ok(());
        }

        if attempt < TIMESTAMP_RETRY_COUNT {
            eprintln!(
                "Signing attempt {}/{} failed for {} — retrying in {}s",
                attempt, TIMESTAMP_RETRY_COUNT, description, TIMESTAMP_RETRY_DELAY_SECS
            );
            std::thread::sleep(std::time::Duration::from_secs(TIMESTAMP_RETRY_DELAY_SECS));
        } else {
            bail!(
                "Signing failed for {} after {} attempts",
                description,
                TIMESTAMP_RETRY_COUNT
            );
        }
    }
    unreachable!()
}

fn import_pfx_to_store(cert_path: &str, cert_pass: &str) -> Result<String> {
    let import_output = std::process::Command::new("certutil")
        .args(["-f", "-p", cert_pass, "-importpfx", cert_path])
        .output()
        .context("Failed to execute certutil for PFX import")?;

    if !import_output.status.success() {
        bail!(
            "certutil -importpfx failed: {}",
            String::from_utf8_lossy(&import_output.stderr)
        );
    }

    let thumbprint_output = std::process::Command::new("certutil")
        .args(["-dump", cert_path])
        .output()
        .context("Failed to execute certutil -dump")?;

    let stdout = String::from_utf8_lossy(&thumbprint_output.stdout);
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.len() == 40 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            return Ok(trimmed.to_string());
        }
        if trimmed.contains("Cert Hash(sha1):") || trimmed.contains("(sha1)") {
            if let Some(hash_part) = trimmed.split(':').last() {
                let hash = hash_part.trim().replace(' ', "");
                if hash.len() == 40 {
                    return Ok(hash);
                }
            }
        }
    }

    bail!("Could not extract certificate thumbprint from PFX")
}

fn sign_binary(binary_path: &Path) -> Result<()> {
    if !binary_path.exists() {
        bail!("Binary not found for signing: {}", binary_path.display());
    }

    println!("Signing binary: {}", binary_path.display());

    let cert_path = std::env::var("PLENUM_SIGN_CERT")
        .context("PLENUM_SIGN_CERT environment variable not set for code signing")?;
    let cert_pass = std::env::var("PLENUM_SIGN_PASS")
        .context("PLENUM_SIGN_PASS environment variable not set for code signing")?;
    let timestamp_url = std::env::var("PLENUM_TIMESTAMP_URL")
        .unwrap_or_else(|_| "http://timestamp.digicert.com".to_string());

    let thumbprint = import_pfx_to_store(&cert_path, &cert_pass)?;

    let binary_str = binary_path.to_str().unwrap();
    run_signtool_with_retry(
        &[
            "sign",
            "/sha1",
            &thumbprint,
            "/tr",
            &timestamp_url,
            "/td",
            "sha256",
            "/fd",
            "sha256",
            binary_str,
        ],
        &format!("binary {}", binary_path.display()),
    )
}

fn sign_msi(msi_path: &Path) -> Result<()> {
    println!("Signing MSI: {}", msi_path.display());

    let cert_path = std::env::var("PLENUM_SIGN_CERT")?;
    let cert_pass = std::env::var("PLENUM_SIGN_PASS")?;
    let timestamp_url = std::env::var("PLENUM_TIMESTAMP_URL")
        .unwrap_or_else(|_| "http://timestamp.digicert.com".to_string());

    let thumbprint = import_pfx_to_store(&cert_path, &cert_pass)?;
    let msi_str = msi_path.to_str().unwrap();

    run_signtool_with_retry(
        &[
            "sign",
            "/sha1",
            &thumbprint,
            "/tr",
            &timestamp_url,
            "/td",
            "sha256",
            "/fd",
            "sha256",
            msi_str,
        ],
        &format!("MSI {}", msi_path.display()),
    )?;

    let verify_status = std::process::Command::new("signtool")
        .args(["verify", "/pa", "/tw", msi_str])
        .status()
        .context("Failed to verify MSI signature")?;

    if !verify_status.success() {
        bail!(
            "MSI signature verification failed for {} — timestamp may be missing",
            msi_path.display()
        );
    }

    Ok(())
}
