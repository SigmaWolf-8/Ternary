// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL

fn main() {
    let version = std::fs::read_to_string("../../VERSION")
        .expect("Cannot read ../../VERSION")
        .trim()
        .to_string();

    println!("cargo:rustc-env=PLENUMNET_VERSION={}", version);
    println!("cargo:rerun-if-changed=../../VERSION");

    let cargo_toml_path = "Cargo.toml";
    let cargo_toml = std::fs::read_to_string(cargo_toml_path)
        .expect("Cannot read Cargo.toml");

    let mut updated = String::new();
    let mut synced = false;
    for line in cargo_toml.lines() {
        if line.starts_with("version = ") && !synced {
            updated.push_str(&format!("version = \"{}\"", version));
            synced = true;
        } else {
            updated.push_str(line);
        }
        updated.push('\n');
    }

    if synced && updated != cargo_toml {
        std::fs::write(cargo_toml_path, &updated)
            .expect("Cannot write Cargo.toml");
    }
}
