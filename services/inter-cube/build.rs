fn main() {
    let version = std::fs::read_to_string("../../VERSION")
        .expect("Cannot read ../../VERSION")
        .trim()
        .to_string();
    println!("cargo:rustc-env=PLENUMNET_VERSION={}", version);
    println!("cargo:rerun-if-changed=../../VERSION");
}
