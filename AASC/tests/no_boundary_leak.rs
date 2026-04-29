// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
//
// Static guard test: the only file in `src/` allowed to expose `u8`
// as a *boundary type* (i.e. for byte ↔ TritVec conversion) is
// `bridge.rs`. Internal use of `u8` (constants, table indices, table
// lookups, GAIT positions, etc.) is permitted everywhere.
//
// We enforce the rule by greping for the canonical boundary signatures
// `&[u8]`, `Vec<u8>`, and `to_bytes`/`from_bytes` patterns and asserting
// they appear only in `bridge.rs`.

use std::fs;
use std::path::PathBuf;

fn src_dir() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("src");
    p
}

fn read_rs_files() -> Vec<(String, String)> {
    let mut out = Vec::new();
    for entry in fs::read_dir(src_dir()).expect("src/ exists") {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let body = fs::read_to_string(&path).unwrap();
        out.push((name, body));
    }
    out
}

#[test]
fn boundary_byte_types_only_in_bridge() {
    // Note: `&[u8]` and `Vec<u8>` are permitted in the *type-definition*
    // files (`trit.rs`, `tritvec.rs`) because they appear there as
    // **Rep-B alphabet cell transport** — i.e. they carry digit values
    // 0/1/2 *as the narrowest host cell that fits a single trit*. That
    // is the alphabet boundary, not the bytes-I/O boundary.
    //
    // The bytes-I/O boundary (opaque byte buffers, packed-b³ encoding,
    // network frames) is the I-3 boundary and lives only in `bridge.rs`.
    // `[u8;` covers fixed-size byte-array boundary forms like `&[u8; 32]`
    // and `[u8; N]` parameter types — these are the same boundary risk
    // as `&[u8]` and `Vec<u8>`.
    let banned_patterns = ["&[u8]", "Vec<u8>", "&mut [u8]", "[u8;"];
    // Whitelist: type-definition / arithmetic-internal files where
    // `&[u8]` / `Vec<u8>` carry **Rep-B alphabet cells** or **u8
    // accumulator slots**, never opaque bytes.
    let allowed_files = [
        "bridge.rs",
        "trit.rs",
        "tritvec.rs",
        "arithmetic.rs",     // internal u8 carry/borrow slots only
        "milesian.rs",       // doc comment mentions `&'static str` glyphs (no &[u8])
        "plenum_square.rs",  // 3×3 magic-square table holds Rep-B trit cells
    ];

    for (name, body) in read_rs_files() {
        if allowed_files.contains(&name.as_str()) {
            continue;
        }
        for pat in banned_patterns {
            assert!(
                !body.contains(pat),
                "boundary leak: `{}` appears in `src/{}` (only `bridge.rs` may use byte boundary types)",
                pat,
                name,
            );
        }
    }
}

#[test]
fn boundary_byte_function_names_only_in_bridge() {
    // `to_bytes_*` / `from_bytes_*` boundary names are reserved for
    // the bridge module.
    let allowed_files = ["bridge.rs"];

    for (name, body) in read_rs_files() {
        if allowed_files.contains(&name.as_str()) {
            continue;
        }
        // Allow doc references in /// comments, but not function defs.
        for line in body.lines() {
            let trim = line.trim_start();
            if trim.starts_with("///") || trim.starts_with("//!") || trim.starts_with("//") {
                continue;
            }
            assert!(
                !trim.contains("fn to_bytes") && !trim.contains("fn from_bytes"),
                "boundary leak: byte conversion fn declared in `src/{}` (line: `{}`)",
                name,
                trim,
            );
        }
    }
}
