fn main() {
    println!("cargo:rerun-if-changed=linker.ld");
    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=src/serial.rs");
    println!("cargo:rerun-if-changed=src/selftest.rs");
}
