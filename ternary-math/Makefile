# ternary-math build targets
# Copyright (c) 2025-2026 Capomastro Holdings Ltd.

WASM_PACK_MIN_MAJOR := 0
WASM_PACK_MIN_MINOR := 13
WASM_PACK_MIN_PATCH := 1
RUST_WASM_TARGET    := wasm32-unknown-unknown

.PHONY: all native test wasm wasm-check wasm-test clean audit

all: native wasm

native:
	cargo build --locked --release

test:
	cargo test --locked
	cargo test --locked --no-default-features

wasm:
	@command -v wasm-pack >/dev/null 2>&1 || { echo "ERROR: wasm-pack not found. Install: cargo install wasm-pack --version 0.13.1"; exit 1; }
	@WASM_VER=$$(wasm-pack --version 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+'); \
	 MAJOR=$$(echo $$WASM_VER | cut -d. -f1); \
	 MINOR=$$(echo $$WASM_VER | cut -d. -f2); \
	 PATCH=$$(echo $$WASM_VER | cut -d. -f3); \
	 if [ "$$MAJOR" -lt $(WASM_PACK_MIN_MAJOR) ] || \
	    ([ "$$MAJOR" -eq $(WASM_PACK_MIN_MAJOR) ] && [ "$$MINOR" -lt $(WASM_PACK_MIN_MINOR) ]) || \
	    ([ "$$MAJOR" -eq $(WASM_PACK_MIN_MAJOR) ] && [ "$$MINOR" -eq $(WASM_PACK_MIN_MINOR) ] && [ "$$PATCH" -lt $(WASM_PACK_MIN_PATCH) ]); then \
	   echo "ERROR: wasm-pack $$WASM_VER below minimum $(WASM_PACK_MIN_MAJOR).$(WASM_PACK_MIN_MINOR).$(WASM_PACK_MIN_PATCH)"; exit 1; \
	 fi; echo "wasm-pack $$WASM_VER OK"
	wasm-pack build --target web --no-default-features --release
	@echo "Output: pkg/ternary_math.js, pkg/ternary_math_bg.wasm, pkg/ternary_math.d.ts"

wasm-check:
	cargo check --target $(RUST_WASM_TARGET) --no-default-features

wasm-test:
	wasm-pack test --headless --chrome --no-default-features

audit:
	cargo audit
	@echo "Checking for banned primitives in production tree..."
	@if cargo tree --edges=normal 2>/dev/null | grep -qE "(rsa|sha2|aes-gcm)"; then \
	   echo "FAIL: banned crate in production tree"; \
	   cargo tree --edges=normal | grep -E "(rsa|sha2|aes-gcm)"; exit 1; \
	 else echo "PASS: no banned crates"; fi

clean:
	cargo clean
	rm -rf pkg/
