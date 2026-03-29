use crate::manifest::{AppKind, Architecture, FirstRunAction, Manifest};
use anyhow::Result;

fn wix_safe_id(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for c in input.chars() {
        if c.is_ascii_alphanumeric() || c == '_' || c == '.' {
            result.push(c);
        } else {
            result.push('_');
        }
    }
    if result.starts_with(|c: char| c.is_ascii_digit()) {
        result.insert(0, '_');
    }
    result
}

fn split_command(command: &str) -> (String, String) {
    let parts: Vec<&str> = command.splitn(2, ' ').collect();
    let exe = parts[0].to_string();
    let args = if parts.len() > 1 {
        parts[1].to_string()
    } else {
        String::new()
    };
    (exe, args)
}

fn ensure_exe_suffix(name: &str) -> String {
    if name.ends_with(".exe") {
        name.to_string()
    } else {
        format!("{}.exe", name)
    }
}

pub const PRODUCT_WXS_TEMPLATE: &str = include_str!("../templates/product.wxs");
pub const UI_DIALOG_TEMPLATE: &str = include_str!("../templates/ui/dialogs.wxs");

pub struct WixGenerator<'a> {
    manifest: &'a Manifest,
    arch: &'a Architecture,
    product_code: String,
    manifest_dir: std::path::PathBuf,
}

pub struct WixOutput {
    pub product_wxs: String,
    pub dialogs_wxs: String,
}

impl<'a> WixGenerator<'a> {
    pub fn new(
        manifest: &'a Manifest,
        arch: &'a Architecture,
        product_code: String,
        manifest_dir: std::path::PathBuf,
    ) -> Self {
        Self {
            manifest,
            arch,
            product_code,
            manifest_dir,
        }
    }

    pub fn generate(&self) -> Result<WixOutput> {
        let product_wxs = self.generate_product_wxs()?;
        let dialogs_wxs = UI_DIALOG_TEMPLATE.to_string();
        Ok(WixOutput {
            product_wxs,
            dialogs_wxs,
        })
    }

    fn generate_product_wxs(&self) -> Result<String> {
        let template = PRODUCT_WXS_TEMPLATE;
        let data_directory = self.manifest.data_directory();
        let binary_name = &self.manifest.app.binary;

        let path_condition = if self.manifest.install.add_to_path {
            "1".to_string()
        } else {
            "0".to_string()
        };

        let directory_structure = self.generate_directory_structure();
        let extra_refs = self.generate_extra_component_refs();
        let (first_run_actions, first_run_sequence) = self.generate_first_run(binary_name);

        let result = template
            .replace("{{PRODUCT_NAME}}", &self.manifest.app.display_name)
            .replace("{{PRODUCT_SHORT_NAME}}", &self.manifest.app.name)
            .replace("{{PRODUCT_VERSION}}", &self.manifest.app.version)
            .replace("{{UPGRADE_CODE}}", &self.manifest.app.upgrade_code)
            .replace("{{PRODUCT_CODE}}", &self.product_code)
            .replace("{{MANUFACTURER}}", &self.manifest.app.publisher)
            .replace("{{DATA_DIRECTORY}}", &data_directory)
            .replace("{{BINARY_NAME}}", binary_name)
            .replace("{{ICON_PATH}}", &self.manifest.app.icon)
            .replace("{{LICENSE}}", &self.manifest.app.license)
            .replace("{{LICENSE_RTF_PATH}}", &self.resolve_license_rtf_path())
            .replace("{{APP_TYPE}}", &self.manifest.app_type.kind.to_string())
            .replace("{{PATH_CONDITION}}", &path_condition)
            .replace("{{DIRECTORY_STRUCTURE}}", &directory_structure)
            .replace("{{EXTRA_COMPONENT_REFS}}", &extra_refs)
            .replace("{{SHORTCUT_ENTRIES}}", &self.generate_shortcuts())
            .replace("{{DESKTOP_SHORTCUT_ENTRIES}}", &self.generate_desktop_shortcuts())
            .replace("{{SHORTCUT_ICONS}}", &self.generate_shortcut_icons())
            .replace(
                "{{SERVICE_ACCOUNT_RESOLVER}}",
                &self.generate_account_resolver_package_scope(),
            )
            .replace("{{FIRST_RUN_ACTIONS}}", &first_run_actions)
            .replace("{{FIRST_RUN_SEQUENCE}}", &first_run_sequence)
            .replace(
                "{{STATUS_PORT_REGISTRY}}",
                &self
                    .manifest
                    .app_type
                    .status_port
                    .map(|p| {
                        format!(
                            "        <RegistryValue Name=\"StatusPort\" Type=\"integer\" Value=\"{}\" />\n",
                            p
                        )
                    })
                    .unwrap_or_default(),
            )
            .replace(
                "{{CONFIGURE_COMMAND_REGISTRY}}",
                &self
                    .manifest
                    .app_type
                    .configure_command
                    .as_ref()
                    .map(|cmd| {
                        format!(
                            "        <RegistryValue Name=\"ConfigureCommand\" Type=\"string\" Value=\"{}\" />\n",
                            cmd
                        )
                    })
                    .unwrap_or_default(),
            )
            .replace(
                "{{ARCH_LAUNCH_CONDITION}}",
                self.arch.arch_launch_condition(),
            )
            .replace("{{ARCH_LAUNCH_MESSAGE}}", self.arch.arch_launch_message())
            .replace(
                "{{ARCH_MISMATCH_CONDITION}}",
                self.arch.arch_mismatch_condition(),
            )
            .replace(
                "{{PASSPHRASE_REQUIRED_PROPERTY}}",
                if self.has_passphrase_actions() {
                    "    <Property Id=\"PASSPHRASE_REQUIRED\" Value=\"1\" />"
                } else {
                    ""
                },
            )
            .replace(
                "{{PASSPHRASE_MIN_LENGTH}}",
                &self.get_passphrase_min_length().to_string(),
            )
            .replace(
                "{{SERVICE_CONFIG_REQUIRED_PROPERTY}}",
                if matches!(
                    self.manifest.app_type.kind,
                    AppKind::Service | AppKind::Hybrid
                ) {
                    "    <Property Id=\"SERVICE_CONFIG_REQUIRED\" Value=\"1\" />"
                } else {
                    ""
                },
            )
            .replace(
                "{{SERVICE_ACCOUNT_DISPLAY}}",
                self.manifest
                    .app_type
                    .service_account
                    .as_deref()
                    .unwrap_or("LocalSystem"),
            )
            .replace(
                "{{SERVICE_ACCOUNT_MODE_DEFAULT}}",
                if self.manifest.app_type.service_account.is_some() {
                    "virtual"
                } else {
                    "localsystem"
                },
            )
            .replace(
                "{{PRESERVE_DATA}}",
                if self
                    .manifest
                    .uninstall
                    .as_ref()
                    .map_or(true, |u| u.preserve_data)
                {
                    "yes"
                } else {
                    "no"
                },
            )
            .replace(
                "{{PRESERVE_MESSAGE}}",
                self.manifest
                    .uninstall
                    .as_ref()
                    .and_then(|u| u.preserve_message.as_deref())
                    .unwrap_or(""),
            );

        Ok(result)
    }

    pub fn wix_build_args(
        &self,
        wxs_path: &str,
        dialogs_wxs_path: &str,
        msi_path: &str,
        binary_source_dir: &str,
        manifest_dir: &str,
        installer_assets_dir: &str,
    ) -> Vec<String> {
        let mut args = vec![
            "build".to_string(),
            "-arch".to_string(),
            self.arch.wix_platform().to_string(),
            wxs_path.to_string(),
            dialogs_wxs_path.to_string(),
            "-o".to_string(),
            msi_path.to_string(),
            "-ext".to_string(),
            "WixToolset.UI.wixext".to_string(),
            "-ext".to_string(),
            "WixToolset.Util.wixext".to_string(),
            "-d".to_string(),
            format!("BinarySourceDir={}", binary_source_dir),
            "-d".to_string(),
            format!("ManifestDir={}", manifest_dir),
            "-d".to_string(),
            format!("InstallerAssetsDir={}", installer_assets_dir),
        ];

        if matches!(
            self.manifest.app_type.kind,
            AppKind::Service | AppKind::Hybrid
        ) {
            args.push("-ext".to_string());
            args.push("WixToolset.Firewall.wixext".to_string());
        }

        args
    }

    fn generate_directory_structure(&self) -> String {
        let dir_path = &self.manifest.install.directory;
        let parts: Vec<&str> = dir_path.split('\\').collect();
        let binary_name = &self.manifest.app.binary;
        let icon_path = &self.manifest.app.icon;

        let mut xml = String::new();
        xml.push_str("    <StandardDirectory Id=\"ProgramFiles64Folder\">\n");

        let mut indent = 6;
        for (i, part) in parts.iter().enumerate() {
            let id = if i == parts.len() - 1 {
                "INSTALLFOLDER".to_string()
            } else {
                format!("Dir_{}", wix_safe_id(part))
            };
            xml.push_str(&format!(
                "{}<Directory Id=\"{}\" Name=\"{}\">\n",
                " ".repeat(indent),
                id,
                part
            ));
            indent += 2;
        }

        xml.push_str(&format!(
            "{}<Component Id=\"MainBinary\" Guid=\"*\">\n",
            " ".repeat(indent)
        ));
        xml.push_str(&format!(
            "{}  <File Id=\"MainExe\" Source=\"$(var.BinarySourceDir)\\{}\" KeyPath=\"yes\" />\n",
            " ".repeat(indent),
            binary_name
        ));

        xml.push_str(&self.generate_service_elements(indent + 2));

        xml.push_str(&format!("{}</Component>\n", " ".repeat(indent)));

        xml.push_str(&format!(
            "{}<Component Id=\"IconFile\" Guid=\"*\">\n",
            " ".repeat(indent)
        ));
        xml.push_str(&format!(
            "{}  <File Id=\"ProductIconFile\" Source=\"$(var.ManifestDir)\\{}\" />\n",
            " ".repeat(indent),
            icon_path
        ));
        xml.push_str(&format!("{}</Component>\n", " ".repeat(indent)));

        xml.push_str(&self.generate_tray_component(indent));

        if self.has_passphrase_actions() {
            xml.push_str(&format!(
                "{}<Component Id=\"PassHelperBinary\" Guid=\"*\">\n",
                " ".repeat(indent)
            ));
            xml.push_str(&format!(
                "{}  <File Id=\"PassHelperExe\" Source=\"$(var.BinarySourceDir)\\plenum-pass-helper.exe\" />\n",
                " ".repeat(indent),
            ));
            xml.push_str(&format!("{}</Component>\n", " ".repeat(indent)));
        }

        for (i, extra_bin) in self.manifest.install.extra_binaries.iter().enumerate() {
            let comp_id = format!("ExtraBinary_{}", i);
            let file_id = format!("ExtraExe_{}", i);
            xml.push_str(&format!(
                "{}<Component Id=\"{}\" Guid=\"*\">\n",
                " ".repeat(indent),
                comp_id
            ));
            xml.push_str(&format!(
                "{}  <File Id=\"{}\" Source=\"$(var.BinarySourceDir)\\{}\" />\n",
                " ".repeat(indent),
                file_id,
                extra_bin
            ));
            xml.push_str(&format!("{}</Component>\n", " ".repeat(indent)));
        }

        for _ in 0..parts.len() {
            indent -= 2;
            xml.push_str(&format!("{}</Directory>\n", " ".repeat(indent)));
        }

        xml.push_str("    </StandardDirectory>");
        xml
    }

    fn generate_service_elements(&self, indent: usize) -> String {
        match self.manifest.app_type.kind {
            AppKind::Service | AppKind::Hybrid => {
                let default_account = self
                    .manifest
                    .app_type
                    .service_account
                    .as_deref()
                    .unwrap_or("LocalSystem");

                let description = self.manifest.app.description.as_deref().unwrap_or("");
                let pad = " ".repeat(indent);
                format!(
                    r#"{pad}<ServiceInstall Id="{name}Service"
{pad}                Type="ownProcess"
{pad}                Name="{name}"
{pad}                DisplayName="{display}"
{pad}                Description="{desc}"
{pad}                Start="[SERVICE_START_TYPE]"
{pad}                Account="[RESOLVED_SERVICE_ACCOUNT]"
{pad}                Password="[SERVICE_ACCOUNT_PASSWORD]"
{pad}                ErrorControl="normal"
{pad}                Vital="yes">
{pad}  <util:ServiceConfig
{pad}    FirstFailureActionType="restart"
{pad}    SecondFailureActionType="restart"
{pad}    ThirdFailureActionType="none"
{pad}    RestartServiceDelayInSeconds="5"
{pad}    ResetPeriodInDays="1" />
{pad}</ServiceInstall>
{pad}<ServiceControl Id="{name}ServiceControl"
{pad}                Name="{name}"
{pad}                Start="install"
{pad}                Stop="both"
{pad}                Remove="both"
{pad}                Wait="yes" />
"#,
                    pad = pad,
                    name = self.manifest.app.name,
                    display = self.manifest.app.display_name,
                    desc = description,
                )
            }
            _ => String::new(),
        }
    }

    fn generate_account_resolver_package_scope(&self) -> String {
        match self.manifest.app_type.kind {
            AppKind::Service | AppKind::Hybrid => {
                let default_account = self
                    .manifest
                    .app_type
                    .service_account
                    .as_deref()
                    .unwrap_or("LocalSystem");
                self.generate_account_resolver(default_account)
            }
            _ => String::new(),
        }
    }

    fn generate_account_resolver(&self, default_account: &str) -> String {
        format!(
            r#"    <Property Id="RESOLVED_SERVICE_ACCOUNT" Value="{default}" />
    <SetProperty Id="RESOLVED_SERVICE_ACCOUNT" Value="NT SERVICE\{name}" After="CostFinalize" Sequence="execute">
      <![CDATA[SERVICE_ACCOUNT_MODE = "virtual"]]>
    </SetProperty>
    <SetProperty Id="RESOLVED_SERVICE_ACCOUNT" Value="LocalSystem" After="CostFinalize" Sequence="execute">
      <![CDATA[SERVICE_ACCOUNT_MODE = "localsystem"]]>
    </SetProperty>
    <SetProperty Id="RESOLVED_SERVICE_ACCOUNT" Value="[SERVICE_CUSTOM_ACCOUNT]" After="CostFinalize" Sequence="execute">
      <![CDATA[SERVICE_ACCOUNT_MODE = "custom" AND SERVICE_CUSTOM_ACCOUNT <> ""]]>
    </SetProperty>
"#,
            default = default_account,
            name = self.manifest.app.name,
        )
    }

    fn generate_tray_component(&self, indent: usize) -> String {
        match self.manifest.app_type.kind {
            AppKind::TrayAgent | AppKind::Hybrid => {
                let pad = " ".repeat(indent);
                let autostart_condition = if self.manifest.app_type.autostart {
                    "1"
                } else {
                    "0"
                };

                format!(
                    r#"{pad}<Component Id="AutostartEntry" Guid="*" Condition="{cond}">
{pad}  <RegistryValue Root="HKCU"
{pad}                 Key="Software\Microsoft\Windows\CurrentVersion\Run"
{pad}                 Name="{name}"
{pad}                 Type="string"
{pad}                 Value="&quot;[INSTALLFOLDER]{binary}&quot;"
{pad}                 KeyPath="yes" />
{pad}</Component>
"#,
                    pad = pad,
                    cond = autostart_condition,
                    name = self.manifest.app.name,
                    binary = self.manifest.app.binary,
                )
            }
            _ => String::new(),
        }
    }

    fn generate_extra_component_refs(&self) -> String {
        let mut refs = String::new();

        match self.manifest.app_type.kind {
            AppKind::TrayAgent | AppKind::Hybrid => {
                refs.push_str("      <ComponentRef Id=\"AutostartEntry\" />\n");
            }
            _ => {}
        }

        if self.manifest.shortcuts.as_ref().map_or(false, |s| !s.desktop.is_empty()) {
            refs.push_str("      <ComponentRef Id=\"DesktopShortcuts\" />\n");
        }

        if self.has_passphrase_actions() {
            refs.push_str("      <ComponentRef Id=\"PassHelperBinary\" />\n");
        }

        for (i, _) in self.manifest.install.extra_binaries.iter().enumerate() {
            refs.push_str(&format!(
                "      <ComponentRef Id=\"ExtraBinary_{}\" />\n",
                i
            ));
        }

        refs
    }

    fn generate_shortcuts(&self) -> String {
        let mut entries = String::new();
        if let Some(ref shortcuts) = self.manifest.shortcuts {
            for (idx, shortcut) in shortcuts.start_menu.iter().enumerate() {
                let icon_ref = if shortcut.icon.is_some() {
                    let icon_path = self.manifest_dir.join(shortcut.icon.as_ref().unwrap());
                    if icon_path.exists() {
                        format!("ShortcutIcon_{}", idx)
                    } else {
                        "ProductIcon".to_string()
                    }
                } else {
                    "ProductIcon".to_string()
                };
                entries.push_str(&format!(
                    r#"            <Shortcut Id="shortcut_{id}" Name="{name}" Target="[INSTALLFOLDER]{target}"{args} Icon="{icon}" Directory="PlenumNetStartMenuFolder" />"#,
                    id = wix_safe_id(&shortcut.name),
                    name = shortcut.name,
                    target = shortcut.target,
                    args = shortcut
                        .args
                        .as_ref()
                        .map(|a| format!(r#" Arguments="{}""#, a))
                        .unwrap_or_default(),
                    icon = icon_ref,
                ));
                entries.push('\n');
            }
        }
        entries
    }

    fn generate_desktop_shortcuts(&self) -> String {
        if let Some(ref shortcuts) = self.manifest.shortcuts {
            if shortcuts.desktop.is_empty() {
                return String::new();
            }
            let mut entries = String::new();
            entries.push_str("    <StandardDirectory Id=\"DesktopFolder\">\n");
            entries.push_str("      <Component Id=\"DesktopShortcuts\" Guid=\"*\">\n");
            entries.push_str(&format!(
                "        <RegistryValue Root=\"HKCU\" Key=\"Software\\Capomastro\\PlenumNET\\{}\" Name=\"DesktopShortcutInstalled\" Type=\"integer\" Value=\"1\" KeyPath=\"yes\" />\n",
                self.manifest.app.name
            ));
            for (idx, shortcut) in shortcuts.desktop.iter().enumerate() {
                let icon_ref = if shortcut.icon.is_some() {
                    let icon_path = self.manifest_dir.join(shortcut.icon.as_ref().unwrap());
                    if icon_path.exists() {
                        format!("DesktopIcon_{}", idx)
                    } else {
                        "ProductIcon".to_string()
                    }
                } else {
                    "ProductIcon".to_string()
                };
                entries.push_str(&format!(
                    r#"        <Shortcut Id="desktop_{id}" Name="{name}" Target="[INSTALLFOLDER]{target}"{args} Icon="{icon}" WorkingDirectory="INSTALLFOLDER" />
"#,
                    id = wix_safe_id(&shortcut.name),
                    name = shortcut.name,
                    target = shortcut.target,
                    args = shortcut
                        .args
                        .as_ref()
                        .map(|a| format!(r#" Arguments="{}""#, a))
                        .unwrap_or_default(),
                    icon = icon_ref,
                ));
            }
            entries.push_str("      </Component>\n");
            entries.push_str("    </StandardDirectory>\n");
            entries
        } else {
            String::new()
        }
    }

    fn generate_shortcut_icons(&self) -> String {
        let mut icons = String::new();
        if let Some(ref shortcuts) = self.manifest.shortcuts {
            for (idx, shortcut) in shortcuts.start_menu.iter().enumerate() {
                if let Some(ref icon) = shortcut.icon {
                    let icon_path = self.manifest_dir.join(icon);
                    if icon_path.exists() {
                        let resolved = icon_path
                            .canonicalize()
                            .unwrap_or(icon_path)
                            .to_string_lossy()
                            .to_string();
                        icons.push_str(&format!(
                            "    <Icon Id=\"ShortcutIcon_{}\" SourceFile=\"{}\" />\n",
                            idx, resolved
                        ));
                    }
                }
            }
            for (idx, shortcut) in shortcuts.desktop.iter().enumerate() {
                if let Some(ref icon) = shortcut.icon {
                    let icon_path = self.manifest_dir.join(icon);
                    if icon_path.exists() {
                        let resolved = icon_path
                            .canonicalize()
                            .unwrap_or(icon_path)
                            .to_string_lossy()
                            .to_string();
                        icons.push_str(&format!(
                            "    <Icon Id=\"DesktopIcon_{}\" SourceFile=\"{}\" />\n",
                            idx, resolved
                        ));
                    }
                }
            }
        }
        icons
    }

    fn generate_first_run(&self, _binary_name: &str) -> (String, String) {
        let mut actions = String::new();
        let mut sequence_entries = Vec::new();

        if let Some(ref first_run) = self.manifest.first_run {
            let needs_passphrase = self.has_passphrase_actions();

            if needs_passphrase {
                let min_len = self.get_passphrase_min_length();
                let passphrase_commands = self.collect_passphrase_commands();

                actions.push_str(
                    r#"    <Binary Id="PlenumPassCA" SourceFile="$(var.BinarySourceDir)\plenum_pass_ca.dll" />
    <CustomAction Id="CollectAndWritePassphrase"
                  BinaryRef="PlenumPassCA" DllEntry="CollectAndWritePassphrase"
                  Execute="immediate" Return="check" />
    <CustomAction Id="CleanupGuiPassphraseTempFile"
                  BinaryRef="PlenumPassCA" DllEntry="CleanupPassphraseTemp"
                  Execute="immediate" Return="ignore" />
    <CustomAction Id="RollbackCleanupPassphraseTempFile"
                  BinaryRef="PlenumPassCA" DllEntry="CleanupPassphraseTemp"
                  Execute="rollback" Return="ignore" />
"#,
                );

                for (idx, (env_var, exe_name, args)) in passphrase_commands.iter().enumerate() {
                    let gui_id = format!("GuiPassExec_{}", idx);
                    let silent_id = format!("SilentPassExec_{}", idx);

                    actions.push_str(&format!(
                        r#"    <CustomAction Id="{gui_id}"
                  Directory="INSTALLFOLDER"
                  ExeCommand="[INSTALLFOLDER]plenum-pass-helper.exe --from-file &quot;[PASSPHRASE_TEMPFILE]&quot; --min-length {min_len} --exec-env {env_var} -- &quot;[INSTALLFOLDER]{exe_name}&quot; {args}"
                  Execute="deferred" Impersonate="yes" Return="check" />
    <CustomAction Id="{silent_id}"
                  Directory="INSTALLFOLDER"
                  ExeCommand="[INSTALLFOLDER]plenum-pass-helper.exe --from-file &quot;[PASSPHRASE_FILE]&quot; --min-length {min_len} --exec-env {env_var} -- &quot;[INSTALLFOLDER]{exe_name}&quot; {args}"
                  Execute="deferred" Impersonate="yes" Return="check" />
"#,
                        gui_id = gui_id,
                        silent_id = silent_id,
                        min_len = min_len,
                        env_var = env_var,
                        exe_name = exe_name,
                        args = args,
                    ));
                }

                actions.push_str(
                    r#"    <CustomAction Id="FailSilentNoPassphrase" Execute="deferred" Impersonate="no" Return="check"
                  Directory="INSTALLFOLDER"
                  ExeCommand="cmd.exe /c &quot;echo [%DATE% %TIME%] ERROR: Silent install of [ProductName] requires PASSPHRASE_FILE property. Use: msiexec /i package.msi /qn PASSPHRASE_FILE=C:\path\to\passphrase.txt >> [TempFolder]PlenumNET_[ProductName]_install.log 2>&amp;1 &amp;&amp; exit /b 1603&quot;" />
"#,
                );
                sequence_entries.push("PassphraseInit".to_string());
            }

            if matches!(
                self.manifest.app_type.kind,
                AppKind::Service | AppKind::Hybrid
            ) {
                let svc_name = &self.manifest.app.name;
                actions.push_str(&format!(
                    r#"    <SetProperty Id="ApplyServiceStartType" Value="{svc_name}" Before="ApplyServiceStartType" Sequence="execute" />
    <CustomAction Id="ApplyServiceStartType" Execute="deferred" Impersonate="no" Return="check"
                  Directory="INSTALLFOLDER"
                  ExeCommand="cmd.exe /c &quot;sc.exe config {svc_name} start= [SERVICE_START_TYPE]&quot;" />
"#,
                    svc_name = svc_name,
                ));
                sequence_entries.push("ApplyServiceStartType".to_string());
            }

            for (i, action) in first_run.actions.iter().enumerate() {
                let action_id = format!("FirstRun_{}", i);
                match action {
                    FirstRunAction::PromptPassphrase { .. } => {
                        continue;
                    }
                    FirstRunAction::RunCommand {
                        command,
                        env_passphrase,
                    } => {
                        if env_passphrase.as_ref().map_or(false, |v| !v.is_empty()) {
                            continue;
                        }
                        let cmd = self.interpolate_command(command);
                        let (exe, args) = split_command(&cmd);
                        let exe_name = ensure_exe_suffix(&exe);
                        actions.push_str(&format!(
                            r#"    <CustomAction Id="{action_id}" Directory="INSTALLFOLDER"
                  ExeCommand="&quot;[INSTALLFOLDER]{exe_name}&quot; {args}"
                  Execute="deferred" Impersonate="no" Return="check" />
"#,
                            action_id = action_id,
                            exe_name = exe_name,
                            args = args,
                        ));
                        sequence_entries.push(action_id);
                    }
                    FirstRunAction::Launch { command } => {
                        let cmd = self.interpolate_command(command);
                        let exe_cmd = cmd
                            .replace("[INSTALLFOLDER]", "")
                            .replace("[InstallFolder]", "");
                        actions.push_str(&format!(
                            r#"    <CustomAction Id="{action_id}" Directory="INSTALLFOLDER"
                  ExeCommand="{exe_cmd}"
                  Execute="commit" Impersonate="yes" Return="asyncNoWait" />
"#,
                            action_id = action_id,
                            exe_cmd = exe_cmd,
                        ));
                        sequence_entries.push(action_id);
                    }
                    FirstRunAction::CopyToClipboard { command, message } => {
                        let cmd = self.interpolate_command(command);
                        let (exe, args) = split_command(&cmd);
                        let exe_name = ensure_exe_suffix(&exe);
                        let data_dir = self
                            .manifest
                            .install
                            .data_directory
                            .as_deref()
                            .unwrap_or(&self.manifest.app.name);
                        let export_file = format!("[AppDataFolder]{}\\export.txt", data_dir);
                        let silent_action_id = format!("{}_Silent", action_id);
                        let gui_action_id = format!("{}_Gui", action_id);
                        actions.push_str(&format!(
                            r#"    <!-- {msg} (silent mode: file-only, no clipboard) -->
    <CustomAction Id="{silent_id}" Directory="INSTALLFOLDER"
                  ExeCommand="cmd.exe /c &quot;if not exist &quot;[AppDataFolder]{data_dir}&quot; mkdir &quot;[AppDataFolder]{data_dir}&quot; &amp;&amp; &quot;[INSTALLFOLDER]{exe_name}&quot; {args} > &quot;{export}&quot; 2>&amp;1&quot;"
                  Execute="deferred" Impersonate="yes" Return="check" />
    <!-- {msg} (GUI mode: file + clipboard) -->
    <CustomAction Id="{gui_id}" Directory="INSTALLFOLDER"
                  ExeCommand="cmd.exe /c &quot;if not exist &quot;[AppDataFolder]{data_dir}&quot; mkdir &quot;[AppDataFolder]{data_dir}&quot; &amp;&amp; &quot;[INSTALLFOLDER]{exe_name}&quot; {args} > &quot;{export}&quot; 2>&amp;1 &amp;&amp; powershell -NoProfile -Command &quot;Get-Content '{export}' | Set-Clipboard&quot;&quot;"
                  Execute="deferred" Impersonate="yes" Return="check" />
"#,
                            msg = message,
                            silent_id = silent_action_id,
                            gui_id = gui_action_id,
                            exe_name = exe_name,
                            args = args,
                            export = export_file,
                            data_dir = data_dir,
                        ));
                        sequence_entries.push(format!("{}|{}", gui_action_id, silent_action_id));
                    }
                }
            }
        }

        let passphrase_commands = self.collect_passphrase_commands();
        let sequence = if !sequence_entries.is_empty() {
            let mut seq = String::from("    <InstallExecuteSequence>\n");
            let mut prev = "InstallFiles".to_string();
            for entry in &sequence_entries {
                if entry == "PassphraseInit" {
                    seq.push_str(&format!(
                        "      <Custom Action=\"FailSilentNoPassphrase\" After=\"{prev}\" Condition=\"NOT Installed AND NOT PASSPHRASE_FILE AND UILevel &lt; 4\" />\n",
                        prev = prev,
                    ));
                    seq.push_str(&format!(
                        "      <Custom Action=\"RollbackCleanupPassphraseTempFile\" After=\"{prev}\" Condition=\"NOT Installed AND UILevel &gt;= 4\" />\n",
                        prev = prev,
                    ));
                    seq.push_str(&format!(
                        "      <Custom Action=\"CollectAndWritePassphrase\" After=\"{prev}\" Condition=\"NOT Installed AND UILevel &gt;= 4\" />\n",
                        prev = prev,
                    ));
                    prev = "CollectAndWritePassphrase".to_string();
                    for idx in 0..passphrase_commands.len() {
                        let gui_id = format!("GuiPassExec_{}", idx);
                        let silent_id = format!("SilentPassExec_{}", idx);
                        seq.push_str(&format!(
                            "      <Custom Action=\"{gui_id}\" After=\"{prev}\" Condition=\"NOT Installed AND UILevel &gt;= 4\" />\n",
                            gui_id = gui_id,
                            prev = prev,
                        ));
                        seq.push_str(&format!(
                            "      <Custom Action=\"{silent_id}\" After=\"{prev}\" Condition=\"NOT Installed AND PASSPHRASE_FILE AND UILevel &lt; 4\" />\n",
                            silent_id = silent_id,
                            prev = prev,
                        ));
                        prev = silent_id;
                    }
                    seq.push_str(&format!(
                        "      <Custom Action=\"CleanupGuiPassphraseTempFile\" After=\"{prev}\" Condition=\"NOT Installed AND UILevel &gt;= 4\" />\n",
                        prev = prev,
                    ));
                    prev = "CleanupGuiPassphraseTempFile".to_string();
                } else if entry.contains('|') {
                    let parts: Vec<&str> = entry.split('|').collect();
                    let gui_id = parts[0];
                    let silent_id = parts[1];
                    seq.push_str(&format!(
                        "      <Custom Action=\"{gui_id}\" After=\"{prev}\" Condition=\"NOT Installed AND UILevel &gt;= 4\" />\n",
                        gui_id = gui_id,
                        prev = prev,
                    ));
                    seq.push_str(&format!(
                        "      <Custom Action=\"{silent_id}\" After=\"{prev}\" Condition=\"NOT Installed AND UILevel &lt; 4\" />\n",
                        silent_id = silent_id,
                        prev = prev,
                    ));
                    prev = silent_id.to_string();
                } else {
                    seq.push_str(&format!(
                        "      <Custom Action=\"{action}\" After=\"{prev}\" Condition=\"NOT Installed\" />\n",
                        action = entry,
                        prev = prev,
                    ));
                    prev = entry.clone();
                }
            }
            seq.push_str("    </InstallExecuteSequence>\n");
            seq
        } else {
            String::new()
        };

        (actions, sequence)
    }

    fn interpolate_command(&self, command: &str) -> String {
        command.replace("{{CRS_ENDPOINT}}", "[CRS_ENDPOINT]")
    }

    fn get_passphrase_params(&self) -> String {
        let mut params = String::new();
        if let Some(ref first_run) = self.manifest.first_run {
            for action in &first_run.actions {
                if let FirstRunAction::PromptPassphrase {
                    min_length,
                    confirm,
                } = action
                {
                    params.push_str(&format!(" --min-length {}", min_length));
                    if *confirm {
                        params.push_str(" --confirm");
                    }
                    params.push_str(&format!(
                        " --product-name &quot;{}&quot;",
                        self.manifest.app.display_name
                    ));
                    break;
                }
            }
        }
        params
    }

    fn has_passphrase_actions(&self) -> bool {
        self.manifest.first_run.as_ref().map_or(false, |fr| {
            fr.actions.iter().any(|a| {
                matches!(a, FirstRunAction::PromptPassphrase { .. })
                    || matches!(
                        a,
                        FirstRunAction::RunCommand {
                            env_passphrase: Some(v),
                            ..
                        } if !v.is_empty()
                    )
            })
        })
    }

    fn resolve_license_rtf_path(&self) -> String {
        let license = &self.manifest.app.license;

        if license.ends_with(".rtf") {
            let candidate = self.manifest_dir.join(license);
            if candidate.exists() {
                return candidate
                    .canonicalize()
                    .unwrap_or(candidate)
                    .to_string_lossy()
                    .to_string();
            }
        }

        let rtf_candidate = self.manifest_dir.join("license.rtf");
        if rtf_candidate.exists() {
            return rtf_candidate
                .canonicalize()
                .unwrap_or(rtf_candidate)
                .to_string_lossy()
                .to_string();
        }

        "$(var.InstallerAssetsDir)\\license.rtf".to_string()
    }

    fn collect_passphrase_commands(&self) -> Vec<(String, String, String)> {
        let mut commands = Vec::new();
        if let Some(ref first_run) = self.manifest.first_run {
            for action in &first_run.actions {
                if let FirstRunAction::RunCommand {
                    command,
                    env_passphrase: Some(env_var),
                } = action
                {
                    if env_var.is_empty() {
                        continue;
                    }
                    let cmd = self.interpolate_command(command);
                    let (exe, args) = split_command(&cmd);
                    let exe_name = ensure_exe_suffix(&exe);
                    commands.push((env_var.clone(), exe_name, args));
                }
            }
        }
        commands
    }

    fn get_passphrase_min_length(&self) -> u32 {
        self.manifest.first_run.as_ref().map_or(12, |fr| {
            fr.actions
                .iter()
                .find_map(|a| {
                    if let FirstRunAction::PromptPassphrase { min_length, .. } = a {
                        Some(*min_length)
                    } else {
                        None
                    }
                })
                .unwrap_or(12)
        })
    }

    fn find_passphrase_env_var(&self) -> Option<String> {
        self.manifest.first_run.as_ref().and_then(|fr| {
            fr.actions.iter().find_map(|a| {
                if let FirstRunAction::RunCommand {
                    env_passphrase: Some(v),
                    ..
                } = a
                {
                    if !v.is_empty() {
                        Some(v.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        })
    }
}
