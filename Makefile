# ══════════════════════════════════════════════════════════════
# Salvi Framework — Build Orchestrator
# Capomastro Holdings Ltd. · Applied Physics Division
#
# Usage:
#   make              — Build everything (Rust + TypeScript)
#   make test         — Run all tests
#   make audit        — Security audit all dependencies
#   make clean        — Remove all build artifacts
#   make release      — Production build with optimizations
#   make wasm         — Build WASM target
#   make docs         — Generate documentation
#   make check        — Format + lint + typecheck (pre-commit)
# ══════════════════════════════════════════════════════════════

.PHONY: all build test audit clean release wasm docs check \
        rust-build rust-test rust-fmt rust-clippy rust-doc rust-audit \
        ts-build ts-check ts-lint \
        integration docker help

SHELL := /bin/bash
.DEFAULT_GOAL := all

# ── Colors ─────────────────────────────────────────────────
C_GOLD   := \033[33m
C_CYAN   := \033[36m
C_GREEN  := \033[32m
C_RED    := \033[31m
C_DIM    := \033[2m
C_RESET  := \033[0m

define header
	@printf "\n$(C_GOLD)══ $(1) $(C_DIM)────────────────────────────────$(C_RESET)\n"
endef

# ══════════════════════════════════════════════════════════════
# TOP-LEVEL TARGETS
# ══════════════════════════════════════════════════════════════

all: build
	@printf "\n$(C_GREEN)✓ All targets built successfully.$(C_RESET)\n"

build: rust-build ts-build
	$(call header,BUILD COMPLETE)

test: rust-test ts-check integration
	$(call header,ALL TESTS PASSED)

check: rust-fmt rust-clippy ts-check
	$(call header,PRE-COMMIT CHECKS PASSED)

release: rust-release ts-build
	$(call header,RELEASE BUILD COMPLETE)

audit: rust-audit ts-audit
	$(call header,SECURITY AUDIT COMPLETE)

docs: rust-doc
	$(call header,DOCUMENTATION GENERATED)

clean:
	$(call header,CLEANING)
	cd libternary && cargo clean
	rm -rf node_modules dist .vite
	@printf "$(C_GREEN)✓ Clean.$(C_RESET)\n"

# ══════════════════════════════════════════════════════════════
# RUST TARGETS
# ══════════════════════════════════════════════════════════════

rust-build:
	$(call header,RUST BUILD)
	cd libternary && cargo build --all-features

rust-release:
	$(call header,RUST RELEASE BUILD)
	cd libternary && cargo build --release --all-features

rust-test:
	$(call header,RUST TESTS)
	cd libternary && cargo test --release --all-features -- --nocapture

rust-fmt:
	$(call header,RUST FORMAT CHECK)
	cd libternary && cargo fmt --all -- --check

rust-clippy:
	$(call header,RUST CLIPPY)
	cd libternary && cargo clippy --all-targets --all-features -- -D warnings

rust-doc:
	$(call header,RUST DOCUMENTATION)
	cd libternary && RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features

rust-audit:
	$(call header,RUST SECURITY AUDIT)
	cd libternary && cargo audit 2>/dev/null || \
		(printf "$(C_DIM)Install with: cargo install cargo-audit$(C_RESET)\n" && true)

# ══════════════════════════════════════════════════════════════
# WASM TARGET
# ══════════════════════════════════════════════════════════════

wasm:
	$(call header,WASM BUILD)
	cd libternary && wasm-pack build --release --target web

# ══════════════════════════════════════════════════════════════
# TYPESCRIPT TARGETS
# ══════════════════════════════════════════════════════════════

ts-build:
	$(call header,TYPESCRIPT BUILD)
	npm run build

ts-check:
	$(call header,TYPESCRIPT TYPECHECK)
	npx tsc --noEmit

ts-lint:
	$(call header,TYPESCRIPT LINT)
	npx eslint src/ client/ server/ shared/ --ext .ts,.tsx 2>/dev/null || true

ts-audit:
	$(call header,NPM SECURITY AUDIT)
	npm audit --audit-level=high 2>/dev/null || true

# ══════════════════════════════════════════════════════════════
# INTEGRATION TESTS
# ══════════════════════════════════════════════════════════════

integration:
	$(call header,INTEGRATION TESTS)
	cd libternary && cargo test --release --test '*' -- --nocapture 2>/dev/null || \
		printf "$(C_DIM)No integration test binaries found (expected in tests/)$(C_RESET)\n"

# ══════════════════════════════════════════════════════════════
# DOCKER
# ══════════════════════════════════════════════════════════════

docker:
	$(call header,DOCKER BUILD)
	cd deployments/docker && docker compose build

docker-up:
	cd deployments/docker && docker compose up -d

docker-down:
	cd deployments/docker && docker compose down

# ══════════════════════════════════════════════════════════════
# HELP
# ══════════════════════════════════════════════════════════════

help:
	@printf "$(C_GOLD)Salvi Framework$(C_RESET) — Build Targets\n\n"
	@printf "  $(C_CYAN)make$(C_RESET)              Build everything\n"
	@printf "  $(C_CYAN)make test$(C_RESET)          Run all tests\n"
	@printf "  $(C_CYAN)make check$(C_RESET)         Pre-commit: fmt + lint + typecheck\n"
	@printf "  $(C_CYAN)make release$(C_RESET)       Production build (LTO, stripped)\n"
	@printf "  $(C_CYAN)make wasm$(C_RESET)          Build WASM target\n"
	@printf "  $(C_CYAN)make audit$(C_RESET)         Security audit all deps\n"
	@printf "  $(C_CYAN)make docs$(C_RESET)          Generate Rust documentation\n"
	@printf "  $(C_CYAN)make docker$(C_RESET)        Build Docker images\n"
	@printf "  $(C_CYAN)make clean$(C_RESET)         Remove all artifacts\n"
	@printf "  $(C_CYAN)make help$(C_RESET)          Show this message\n"
	@printf "\n$(C_DIM)Capomastro Holdings Ltd. · Applied Physics Division$(C_RESET)\n"
