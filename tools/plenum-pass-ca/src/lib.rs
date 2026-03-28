#[cfg(windows)]
mod msi_ffi {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use std::path::PathBuf;

    type MsiHandle = u32;
    const ERROR_SUCCESS: u32 = 0;
    const ERROR_INSTALL_FAILURE: u32 = 1603;
    const ERROR_MORE_DATA: u32 = 234;

    #[link(name = "msi")]
    extern "system" {
        fn MsiGetPropertyW(
            h_install: MsiHandle,
            sz_name: *const u16,
            sz_value_buf: *mut u16,
            pcch_value_buf: *mut u32,
        ) -> u32;
        fn MsiSetPropertyW(h_install: MsiHandle, sz_name: *const u16, sz_value: *const u16) -> u32;
    }

    #[link(name = "user32")]
    extern "system" {
        fn MessageBoxW(hwnd: usize, text: *const u16, caption: *const u16, u_type: u32) -> i32;
        fn GetForegroundWindow() -> usize;
        fn CreateWindowExW(
            ex_style: u32,
            class_name: *const u16,
            window_name: *const u16,
            style: u32,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
            parent: usize,
            menu: usize,
            instance: usize,
            param: usize,
        ) -> usize;
        fn DestroyWindow(hwnd: usize) -> i32;
        fn ShowWindow(hwnd: usize, cmd: i32) -> i32;
        fn SendMessageW(hwnd: usize, msg: u32, w_param: usize, l_param: isize) -> isize;
        fn SetFocus(hwnd: usize) -> usize;
        fn GetDlgItem(hdlg: usize, id: i32) -> usize;
        fn EnableWindow(hwnd: usize, enable: i32) -> i32;
        fn SetWindowTextW(hwnd: usize, text: *const u16) -> i32;
        fn GetWindowTextW(hwnd: usize, text: *mut u16, max: i32) -> i32;
        fn GetWindowTextLengthW(hwnd: usize) -> i32;
        fn DefWindowProcW(hwnd: usize, msg: u32, w_param: usize, l_param: isize) -> isize;
        fn PostQuitMessage(exit_code: i32);
        fn GetMessageW(msg: *mut [u8; 48], hwnd: usize, filter_min: u32, filter_max: u32) -> i32;
        fn TranslateMessage(msg: *const [u8; 48]) -> i32;
        fn DispatchMessageW(msg: *const [u8; 48]) -> isize;
        fn RegisterClassExW(wc: *const WndClassExW) -> u16;
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetModuleHandleW(name: *const u16) -> usize;
    }

    #[repr(C)]
    struct WndClassExW {
        cb_size: u32,
        style: u32,
        wnd_proc: unsafe extern "system" fn(usize, u32, usize, isize) -> isize,
        cls_extra: i32,
        wnd_extra: i32,
        instance: usize,
        icon: usize,
        cursor: usize,
        background: usize,
        menu_name: *const u16,
        class_name: *const u16,
        icon_sm: usize,
    }

    fn to_wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn get_property(h_install: MsiHandle, name: &str) -> Result<String, u32> {
        let name_w = to_wide(name);
        let mut size: u32 = 0;
        let ret =
            unsafe { MsiGetPropertyW(h_install, name_w.as_ptr(), std::ptr::null_mut(), &mut size) };
        if ret != ERROR_SUCCESS && ret != ERROR_MORE_DATA {
            return Err(ret);
        }
        size += 1;
        let mut buf: Vec<u16> = vec![0; size as usize];
        let ret =
            unsafe { MsiGetPropertyW(h_install, name_w.as_ptr(), buf.as_mut_ptr(), &mut size) };
        if ret != ERROR_SUCCESS {
            return Err(ret);
        }
        let os_str = OsString::from_wide(&buf[..size as usize]);
        Ok(os_str.to_string_lossy().into_owned())
    }

    fn set_property(h_install: MsiHandle, name: &str, value: &str) -> Result<(), u32> {
        let name_w = to_wide(name);
        let value_w = to_wide(value);
        let ret = unsafe { MsiSetPropertyW(h_install, name_w.as_ptr(), value_w.as_ptr()) };
        if ret != ERROR_SUCCESS {
            Err(ret)
        } else {
            Ok(())
        }
    }

    fn zeroize_string(s: &mut String) {
        unsafe {
            let bytes = s.as_bytes_mut();
            for b in bytes.iter_mut() {
                std::ptr::write_volatile(b, 0);
            }
        }
    }

    fn write_passphrase_to_temp(passphrase: &str, h_install: MsiHandle) -> Result<String, u32> {
        let temp_folder = match get_property(h_install, "TempFolder") {
            Ok(t) if !t.is_empty() => t,
            _ => match std::env::var("TEMP").or_else(|_| std::env::var("TMP")) {
                Ok(t) => {
                    let mut t = t;
                    if !t.ends_with('\\') {
                        t.push('\\');
                    }
                    t
                }
                Err(_) => return Err(ERROR_INSTALL_FAILURE),
            },
        };

        let product_code = get_property(h_install, "ProductCode").unwrap_or_default();
        let sanitized_code = product_code
            .replace('{', "")
            .replace('}', "")
            .replace('-', "");
        let filename = format!("pnet_pass_{}.tmp", sanitized_code);
        let temp_path = PathBuf::from(&temp_folder).join(&filename);
        let path_str = temp_path.display().to_string();

        {
            use std::io::Write;
            let file = match std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&temp_path)
            {
                Ok(f) => f,
                Err(_) => return Err(ERROR_INSTALL_FAILURE),
            };

            let icacls_result = std::process::Command::new("icacls")
                .args([
                    &path_str,
                    "/inheritance:r",
                    "/grant:r",
                    &format!(
                        "{}:(R,W)",
                        std::env::var("USERNAME").unwrap_or_else(|_| "CURRENT_USER".into())
                    ),
                ])
                .output();
            match icacls_result {
                Ok(output) if output.status.success() => {}
                _ => {
                    let _ = std::fs::remove_file(&temp_path);
                    return Err(ERROR_INSTALL_FAILURE);
                }
            }

            let mut writer = std::io::BufWriter::new(file);
            if writer.write_all(passphrase.as_bytes()).is_err() {
                let _ = std::fs::remove_file(&temp_path);
                return Err(ERROR_INSTALL_FAILURE);
            }
            if writer.flush().is_err() {
                let _ = std::fs::remove_file(&temp_path);
                return Err(ERROR_INSTALL_FAILURE);
            }
        }

        let icacls_restrict = std::process::Command::new("icacls")
            .args([
                &path_str,
                "/inheritance:r",
                "/grant:r",
                &format!(
                    "{}:(R)",
                    std::env::var("USERNAME").unwrap_or_else(|_| "CURRENT_USER".into())
                ),
            ])
            .output();
        match icacls_restrict {
            Ok(output) if output.status.success() => {}
            _ => {
                let _ = std::fs::remove_file(&temp_path);
                return Err(ERROR_INSTALL_FAILURE);
            }
        }

        Ok(path_str)
    }

    static mut DIALOG_RESULT: Option<String> = None;
    static mut DIALOG_MIN_LEN: usize = 12;

    const WM_COMMAND: u32 = 0x0111;
    const WM_DESTROY: u32 = 0x0002;
    const WM_CREATE: u32 = 0x0001;
    const WS_OVERLAPPED: u32 = 0x00000000;
    const WS_CAPTION: u32 = 0x00C00000;
    const WS_SYSMENU: u32 = 0x00080000;
    const WS_VISIBLE: u32 = 0x10000000;
    const WS_CHILD: u32 = 0x40000000;
    const WS_BORDER: u32 = 0x00800000;
    const WS_TABSTOP: u32 = 0x00010000;
    const ES_PASSWORD: u32 = 0x0020;
    const BS_PUSHBUTTON: u32 = 0x00000000;
    const MB_OK: u32 = 0x00000000;
    const MB_ICONERROR: u32 = 0x00000010;
    const IDCANCEL: i32 = 2;
    const ID_PASS1: i32 = 101;
    const ID_PASS2: i32 = 102;
    const ID_OK: i32 = 103;
    const ID_CANCEL: i32 = 104;

    unsafe extern "system" fn passphrase_wnd_proc(
        hwnd: usize,
        msg: u32,
        w_param: usize,
        l_param: isize,
    ) -> isize {
        match msg {
            WM_CREATE => {
                let static_class = to_wide("STATIC");
                let edit_class = to_wide("EDIT");
                let button_class = to_wide("BUTTON");

                let label1 = to_wide("Protect your signing key — enter passphrase:");
                CreateWindowExW(
                    0,
                    static_class.as_ptr(),
                    label1.as_ptr(),
                    WS_CHILD | WS_VISIBLE,
                    10,
                    10,
                    360,
                    20,
                    hwnd,
                    0,
                    0,
                    0,
                );

                CreateWindowExW(
                    0,
                    edit_class.as_ptr(),
                    std::ptr::null(),
                    WS_CHILD | WS_VISIBLE | WS_BORDER | WS_TABSTOP | ES_PASSWORD,
                    10,
                    35,
                    360,
                    25,
                    hwnd,
                    ID_PASS1 as usize,
                    0,
                    0,
                );

                let label2 = to_wide("Confirm passphrase:");
                CreateWindowExW(
                    0,
                    static_class.as_ptr(),
                    label2.as_ptr(),
                    WS_CHILD | WS_VISIBLE,
                    10,
                    70,
                    360,
                    20,
                    hwnd,
                    0,
                    0,
                    0,
                );

                CreateWindowExW(
                    0,
                    edit_class.as_ptr(),
                    std::ptr::null(),
                    WS_CHILD | WS_VISIBLE | WS_BORDER | WS_TABSTOP | ES_PASSWORD,
                    10,
                    95,
                    360,
                    25,
                    hwnd,
                    ID_PASS2 as usize,
                    0,
                    0,
                );

                let ok_text = to_wide("OK");
                CreateWindowExW(
                    0,
                    button_class.as_ptr(),
                    ok_text.as_ptr(),
                    WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_PUSHBUTTON,
                    200,
                    135,
                    80,
                    30,
                    hwnd,
                    ID_OK as usize,
                    0,
                    0,
                );

                let cancel_text = to_wide("Cancel");
                CreateWindowExW(
                    0,
                    button_class.as_ptr(),
                    cancel_text.as_ptr(),
                    WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_PUSHBUTTON,
                    290,
                    135,
                    80,
                    30,
                    hwnd,
                    ID_CANCEL as usize,
                    0,
                    0,
                );

                let edit1 = GetDlgItem(hwnd, ID_PASS1);
                SetFocus(edit1);

                0
            }
            WM_COMMAND => {
                let id = (w_param & 0xFFFF) as i32;
                if id == ID_OK {
                    let edit1 = GetDlgItem(hwnd, ID_PASS1);
                    let edit2 = GetDlgItem(hwnd, ID_PASS2);

                    let len1 = GetWindowTextLengthW(edit1) as usize;
                    let len2 = GetWindowTextLengthW(edit2) as usize;

                    let mut buf1 = vec![0u16; len1 + 1];
                    let mut buf2 = vec![0u16; len2 + 1];
                    GetWindowTextW(edit1, buf1.as_mut_ptr(), buf1.len() as i32);
                    GetWindowTextW(edit2, buf2.as_mut_ptr(), buf2.len() as i32);

                    let pass1 = OsString::from_wide(&buf1[..len1])
                        .to_string_lossy()
                        .into_owned();
                    let mut pass2 = OsString::from_wide(&buf2[..len2])
                        .to_string_lossy()
                        .into_owned();

                    buf1.iter_mut().for_each(|b| *b = 0);
                    buf2.iter_mut().for_each(|b| *b = 0);

                    if pass1.chars().count() < DIALOG_MIN_LEN {
                        let msg = to_wide(&format!(
                            "Passphrase must be at least {} characters.",
                            DIALOG_MIN_LEN
                        ));
                        let title = to_wide("PlenumNET");
                        MessageBoxW(hwnd, msg.as_ptr(), title.as_ptr(), MB_OK | MB_ICONERROR);
                        return 0;
                    }

                    if pass1 != pass2 {
                        zeroize_string(&mut pass2);
                        let msg = to_wide("Passphrases do not match.");
                        let title = to_wide("PlenumNET");
                        MessageBoxW(hwnd, msg.as_ptr(), title.as_ptr(), MB_OK | MB_ICONERROR);
                        return 0;
                    }

                    zeroize_string(&mut pass2);
                    DIALOG_RESULT = Some(pass1);
                    DestroyWindow(hwnd);
                } else if id == ID_CANCEL || id == IDCANCEL {
                    DIALOG_RESULT = None;
                    DestroyWindow(hwnd);
                }
                0
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, msg, w_param, l_param),
        }
    }

    fn show_passphrase_dialog(product_name: &str, min_len: usize) -> Option<String> {
        unsafe {
            DIALOG_RESULT = None;
            DIALOG_MIN_LEN = min_len;

            let class_name = to_wide("PlenumPassDlg");
            let instance = GetModuleHandleW(std::ptr::null());

            let wc = WndClassExW {
                cb_size: std::mem::size_of::<WndClassExW>() as u32,
                style: 0,
                wnd_proc: passphrase_wnd_proc,
                cls_extra: 0,
                wnd_extra: 0,
                instance,
                icon: 0,
                cursor: 0,
                background: 6,
                menu_name: std::ptr::null(),
                class_name: class_name.as_ptr(),
                icon_sm: 0,
            };

            RegisterClassExW(&wc);

            let title = to_wide(&format!("{} — Passphrase", product_name));
            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                title.as_ptr(),
                WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_VISIBLE,
                100,
                100,
                400,
                210,
                0,
                0,
                instance,
                0,
            );

            if hwnd == 0 {
                return None;
            }

            ShowWindow(hwnd, 1);

            let mut msg = [0u8; 48];
            while GetMessageW(&mut msg, 0, 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            DIALOG_RESULT.take()
        }
    }

    pub fn collect_and_write_passphrase_impl(h_install: MsiHandle) -> u32 {
        let product_name =
            get_property(h_install, "ProductName").unwrap_or_else(|_| "PlenumNET".into());
        let min_length_str = get_property(h_install, "PASSPHRASE_MIN_LENGTH").unwrap_or_default();
        let min_length: usize = min_length_str.parse().unwrap_or(12);

        let mut passphrase = match show_passphrase_dialog(&product_name, min_length) {
            Some(p) => p,
            None => return ERROR_INSTALL_FAILURE,
        };

        let result = match write_passphrase_to_temp(&passphrase, h_install) {
            Ok(path_str) => {
                if set_property(h_install, "PASSPHRASE_TEMPFILE", &path_str).is_err() {
                    let _ = std::fs::remove_file(&path_str);
                    ERROR_INSTALL_FAILURE
                } else {
                    ERROR_SUCCESS
                }
            }
            Err(e) => e,
        };

        zeroize_string(&mut passphrase);
        result
    }

    pub fn cleanup_passphrase_temp_impl(h_install: MsiHandle) -> u32 {
        let temp_path = match get_property(h_install, "PASSPHRASE_TEMPFILE") {
            Ok(p) if !p.is_empty() => PathBuf::from(p),
            _ => {
                let temp_folder = get_property(h_install, "TempFolder").unwrap_or_else(|_| {
                    std::env::var("TEMP")
                        .or_else(|_| std::env::var("TMP"))
                        .unwrap_or_else(|_| String::from("."))
                });
                let product_code = get_property(h_install, "ProductCode").unwrap_or_default();
                let sanitized = product_code
                    .replace('{', "")
                    .replace('}', "")
                    .replace('-', "");
                PathBuf::from(temp_folder).join(format!("pnet_pass_{}.tmp", sanitized))
            }
        };

        if temp_path.exists() {
            if let Ok(metadata) = std::fs::metadata(&temp_path) {
                use std::io::Write;
                let len = metadata.len() as usize;
                if let Ok(mut f) = std::fs::OpenOptions::new().write(true).open(&temp_path) {
                    let zeros = vec![0u8; len];
                    let _ = f.write_all(&zeros);
                    let _ = f.flush();
                }
            }
            let _ = std::fs::remove_file(&temp_path);
        }

        ERROR_SUCCESS
    }
}

#[no_mangle]
#[cfg(windows)]
pub extern "system" fn CollectAndWritePassphrase(h_install: u32) -> u32 {
    msi_ffi::collect_and_write_passphrase_impl(h_install)
}

#[no_mangle]
#[cfg(windows)]
pub extern "system" fn CleanupPassphraseTemp(h_install: u32) -> u32 {
    msi_ffi::cleanup_passphrase_temp_impl(h_install)
}

#[no_mangle]
#[cfg(not(windows))]
pub extern "C" fn CollectAndWritePassphrase(_h_install: u32) -> u32 {
    0
}

#[no_mangle]
#[cfg(not(windows))]
pub extern "C" fn CleanupPassphraseTemp(_h_install: u32) -> u32 {
    0
}
