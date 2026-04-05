fn main() {
    let version_file = std::fs::read_to_string("../../VERSION")
        .expect("Cannot read ../../VERSION — this file is the single source of truth for the project version");
    let version_file = version_file.trim();
    let cargo_version = env!("CARGO_PKG_VERSION");
    if version_file != cargo_version {
        panic!(
            "\n\nVERSION MISMATCH: VERSION file says '{}' but Cargo.toml says '{}'\n\
             Fix: update version in services/inter-cube/Cargo.toml to match the VERSION file.\n",
            version_file, cargo_version
        );
    }
    println!("cargo:rerun-if-changed=../../VERSION");
}
