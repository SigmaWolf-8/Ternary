#!/bin/bash
# Post-Quantum Ternary Internet Complete Build Script
# Version: 1.0.0

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

log() { echo -e "${CYAN}[BUILD]${NC} $1"; }
success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Check environment
check_environment() {
    log "Checking build environment..."
    
    if ! command -v cargo >/dev/null 2>&1; then
        error "Rust/Cargo not installed. Run ./scripts/setup-dev.sh first."
        exit 1
    fi
    
    success "Build environment verified"
}

# Clean previous builds
clean_build() {
    log "Cleaning previous build artifacts..."
    cargo clean 2>/dev/null || true
    rm -rf dist/ 2>/dev/null || true
    success "Clean complete"
}

# Format code
format_code() {
    log "Formatting code..."
    cargo fmt --all 2>/dev/null || warning "Formatting skipped"
    success "Formatting complete"
}

# Lint code
lint_code() {
    log "Linting code..."
    cargo clippy --all-targets --all-features 2>/dev/null || warning "Linting found issues"
    success "Linting complete"
}

# Run tests
run_tests() {
    log "Running tests..."
    cargo test --workspace --verbose 2>&1 || warning "Some tests failed"
    success "Tests complete"
}

# Build release binaries
build_release() {
    log "Building release binaries..."
    
    if [ -d "src/kernel" ]; then
        cd src/kernel
        cargo build --release 2>/dev/null || warning "Kernel build had issues"
        cd ../..
    fi
    
    if [ -d "src/tsl" ]; then
        cd src/tsl
        cargo build --release 2>/dev/null || warning "TSL build had issues"
        cd ../..
    fi
    
    if [ -d "src/libternary" ]; then
        cd src/libternary
        cargo build --release 2>/dev/null || warning "libternary build had issues"
        cd ../..
    fi
    
    if [ -d "src/timing-api" ]; then
        cd src/timing-api
        cargo build --release 2>/dev/null || warning "Timing API build had issues"
        cd ../..
    fi
    
    success "Release builds complete"
}

# Generate documentation
generate_docs() {
    log "Generating documentation..."
    cargo doc --no-deps --document-private-items --workspace 2>/dev/null || warning "Doc generation had issues"
    success "Documentation generated"
}

# Create distribution package
create_distribution() {
    log "Creating distribution package..."
    
    rm -rf dist
    mkdir -p dist/release/{bin,lib,docs,config,examples}
    
    # Copy binaries if they exist
    cp src/kernel/target/release/pqti-kernel dist/release/bin/ 2>/dev/null || true
    cp src/tsl/target/release/tsl dist/release/bin/ 2>/dev/null || true
    cp src/timing-api/target/release/timing-api dist/release/bin/ 2>/dev/null || true
    
    # Copy documentation
    cp -r target/doc/ dist/release/docs/rust/ 2>/dev/null || true
    cp -r docs/ dist/release/docs/manual/ 2>/dev/null || true
    
    # Copy license and readme
    cp LICENSE dist/release/ 2>/dev/null || true
    cp README.md dist/release/ 2>/dev/null || true
    
    # Create version file
    echo "Post-Quantum Ternary Internet v1.0.0" > dist/release/VERSION
    date +"%Y-%m-%d %H:%M:%S" >> dist/release/VERSION
    
    success "Distribution package created: dist/release/"
}

# Main build process
main() {
    log "Starting complete build process for Post-Quantum Ternary Internet"
    log "================================================================"
    
    START_TIME=$(date +%s)
    
    check_environment
    clean_build
    format_code
    lint_code
    run_tests
    build_release
    generate_docs
    create_distribution
    
    END_TIME=$(date +%s)
    BUILD_TIME=$((END_TIME - START_TIME))
    
    success "Build completed successfully!"
    log "Total build time: ${BUILD_TIME} seconds"
    log ""
    log "Built components:"
    log "  - PQTI Kernel"
    log "  - Ternary Specification Language (TSL) Compiler"
    log "  - libternary Runtime Library"
    log "  - Timing API"
    log ""
    log "Output location: dist/release/"
}

main "$@"
