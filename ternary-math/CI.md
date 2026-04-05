# ternary-math CI Matrix

## Rust Version
- **MSRV**: 1.77.0
- **Stable**: latest stable (primary)

## Cargo.lock Policy
Cargo.lock is committed to VCS. All deps use exact `=` pins.
Both CI and Makefile use `--locked`.

## Targets

| Target | Features | Build | Test | Duration |
|--------|----------|-------|------|----------|
| x86_64-unknown-linux-gnu | default | yes | yes | ~90s |
| x86_64-unknown-linux-gnu | no-default-features | yes | yes | ~60s |
| wasm32-unknown-unknown | no-default-features | yes | wasm-pack | ~45s |

## Pipeline

1. `cargo fmt --check`
2. `cargo clippy --all-targets -- -D warnings`
3. `make native`
4. `make test`
5. `make wasm`
6. `make wasm-test`
7. `make audit`
