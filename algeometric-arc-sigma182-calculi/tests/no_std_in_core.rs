// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
//
// Static guard test: no `use std::` outside the bridge module. The
// crate must remain `no_std`-clean on the default feature path.

use std::fs;
use std::path::PathBuf;

fn src_dir() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("src");
    p
}

#[test]
fn no_std_uses_in_core_modules() {
    let allowed_files = ["bridge.rs"];

    for entry in fs::read_dir(src_dir()).expect("src/ exists") {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if allowed_files.contains(&name.as_str()) {
            continue;
        }
        let body = fs::read_to_string(&path).unwrap();
        for line in body.lines() {
            let trim = line.trim_start();
            // Skip comments
            if trim.starts_with("//") {
                continue;
            }
            assert!(
                !trim.starts_with("use std::") && !trim.contains(" use std::"),
                "no_std violation: `use std::` in `src/{}` (line: `{}`)",
                name,
                trim,
            );
            assert!(
                !trim.starts_with("extern crate std"),
                "no_std violation: `extern crate std` in `src/{}`",
                name,
            );
        }
    }
}
