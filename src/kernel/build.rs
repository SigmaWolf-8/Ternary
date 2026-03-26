fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    if !target.contains("none") {
        return;
    }

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    let arch = if target.starts_with("x86_64") {
        "x86_64"
    } else if target.starts_with("aarch64") {
        "aarch64"
    } else if target.starts_with("riscv64") {
        "riscv64"
    } else {
        return;
    };

    let linker_script = format!("{}/linker-{}.ld", manifest_dir, arch);
    println!(
        "cargo:rustc-link-arg-bin=plenumnet-kernel=-T{}",
        linker_script
    );
    println!("cargo:rerun-if-changed=linker-{}.ld", arch);
}
