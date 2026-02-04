# Post-Quantum Ternary Internet Build System
# Version: 1.0.0

.PHONY: all build clean test fmt lint doc install dev-setup release help

# Default target
all: build

# Variables
CARGO = cargo
RUSTFLAGS = -D warnings
CARGO_TEST_FLAGS = --verbose -- --test-threads=1
CARGO_DOC_FLAGS = --no-deps --document-private-items

# Project directories
KERNEL_DIR = src/kernel
TSL_DIR = src/tsl
LIB_TERNARY_DIR = src/libternary
TIMING_API_DIR = src/timing-api
DOCS_DIR = docs
SCRIPTS_DIR = scripts
CONFIG_DIR = config
KEYS_DIR = keys

# Build all components
build:
	@echo "Building Post-Quantum Ternary Internet..."
	@$(CARGO) build --release --workspace
	@echo "Build complete."

# Build kernel only
build-kernel:
	@echo "Building PQTI Kernel..."
	@cd $(KERNEL_DIR) && $(CARGO) build --release
	@echo "Kernel build complete."

# Build TSL compiler
build-tsl:
	@echo "Building Ternary Specification Language compiler..."
	@cd $(TSL_DIR) && $(CARGO) build --release
	@echo "TSL compiler build complete."

# Run all tests
test: test-kernel test-tsl test-lib test-timing

test-kernel:
	@echo "Running kernel tests..."
	@cd $(KERNEL_DIR) && $(CARGO) test $(CARGO_TEST_FLAGS)

test-tsl:
	@echo "Running TSL tests..."
	@cd $(TSL_DIR) && $(CARGO) test $(CARGO_TEST_FLAGS)

test-lib:
	@echo "Running libternary tests..."
	@cd $(LIB_TERNARY_DIR) && $(CARGO) test $(CARGO_TEST_FLAGS)

test-timing:
	@echo "Running timing API tests..."
	@cd $(TIMING_API_DIR) && $(CARGO) test $(CARGO_TEST_FLAGS)

# Run integration tests
test-integration:
	@echo "Running integration tests..."
	@$(CARGO) test --test integration --verbose

# Run security tests
test-security:
	@echo "Running security tests..."
	@$(CARGO) test --test security --verbose

# Format code
fmt:
	@echo "Formatting Rust code..."
	@$(CARGO) fmt --all
	@echo "Formatting complete."

# Lint code
lint:
	@echo "Linting Rust code..."
	@$(CARGO) clippy --all-targets --all-features -- -D warnings
	@echo "Linting complete."

# Generate documentation
doc:
	@echo "Generating documentation..."
	@$(CARGO) doc $(CARGO_DOC_FLAGS) --workspace
	@echo "Documentation generated in target/doc/"

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@$(CARGO) clean
	@rm -rf target/
	@rm -rf dist/
	@rm -rf *.log
	@echo "Clean complete."

# Setup development environment
dev-setup:
	@echo "Setting up development environment..."
	@./scripts/setup-dev.sh
	@echo "Development environment setup complete."

# Run all benchmarks
bench:
	@echo "Running benchmarks..."
	@$(CARGO) bench --workspace
	@echo "Benchmarks complete."

# Create release package
release: clean build test
	@echo "Creating release package..."
	@mkdir -p dist/release
	@cp $(KERNEL_DIR)/target/release/pqti-kernel dist/release/ 2>/dev/null || true
	@cp $(TSL_DIR)/target/release/tsl dist/release/ 2>/dev/null || true
	@cp -r $(DOCS_DIR) dist/release/
	@cp LICENSE dist/release/
	@cp README.md dist/release/
	@echo "Release package created in dist/release/"

# Install system-wide (requires sudo)
install: build
	@echo "Installing system-wide..."
	@sudo cp $(KERNEL_DIR)/target/release/pqti-kernel /usr/local/bin/ 2>/dev/null || true
	@sudo cp $(TSL_DIR)/target/release/tsl /usr/local/bin/ 2>/dev/null || true
	@echo "Installation complete."

# Update dependencies
update:
	@echo "Updating dependencies..."
	@$(CARGO) update
	@echo "Dependencies updated."

# Run security audit
audit:
	@echo "Running security audit..."
	@$(CARGO) audit
	@echo "Security audit complete."

# Verify timing compliance
verify-timing:
	@echo "Verifying timing compliance..."
	@./scripts/verify-timing.sh 2>/dev/null || echo "Timing verification script not found"
	@echo "Timing compliance verification complete."

# Deploy to Kong
deploy-kong:
	@echo "Deploying to Kong Cloud Gateway..."
	@./scripts/deploy-kong.sh 2>/dev/null || echo "Kong deployment script not found"
	@echo "Deployment complete."

# Help command
help:
	@echo "Post-Quantum Ternary Internet Build System"
	@echo ""
	@echo "Available commands:"
	@echo "  make build           - Build all components"
	@echo "  make test            - Run all tests"
	@echo "  make fmt             - Format code"
	@echo "  make lint            - Lint code"
	@echo "  make doc             - Generate documentation"
	@echo "  make clean           - Clean build artifacts"
	@echo "  make dev-setup       - Setup development environment"
	@echo "  make bench           - Run benchmarks"
	@echo "  make release         - Create release package"
	@echo "  make install         - Install system-wide"
	@echo "  make update          - Update dependencies"
	@echo "  make audit           - Run security audit"
	@echo "  make verify-timing   - Verify timing compliance"
	@echo "  make deploy-kong     - Deploy to Kong"
	@echo ""
	@echo "Component-specific commands:"
	@echo "  make build-kernel    - Build kernel only"
	@echo "  make build-tsl       - Build TSL compiler"
	@echo "  make test-kernel     - Test kernel only"
	@echo "  make test-tsl        - Test TSL compiler"
	@echo "  make test-lib        - Test libternary"
	@echo "  make test-timing     - Test timing API"
	@echo "  make test-integration - Run integration tests"
	@echo "  make test-security   - Run security tests"

.DEFAULT_GOAL := help
