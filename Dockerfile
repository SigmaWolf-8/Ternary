# ═══════════════════════════════════════════════════════════════════
# PlenumNET Inter-Cube Daemon — Container Recipe
# Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
# Applied Physics Division
#
# WHAT THIS FILE DOES:
#   Packages the inter-cube daemon into a container that any
#   computer can run — your laptop, a cloud server, or 27 at once.
#
# HOW TO BUILD:
#   docker build -t plenumnet/inter-cube .
#
# HOW TO RUN:
#   docker run -p 8080:8080 -p 51820:51820 plenumnet/inter-cube
#
# The build has two stages:
#   Stage 1 "builder" — Downloads Rust, compiles your code, runs tests
#   Stage 2 "runtime" — Copies just the compiled binary into a tiny image
#
# The final container is small (~80MB) because the Rust compiler
# (~1GB) is thrown away after the binary is built.
# ═══════════════════════════════════════════════════════════════════

# ── STAGE 1: Compile the code ────────────────────────────────────
# Start with a computer that has Rust 1.83 pre-installed.
FROM rust:1.83-bookworm AS builder

# Set the working directory. Everything below happens inside /app.
WORKDIR /app

# Copy the workspace file first. This tells Rust which crates
# (code packages) exist and how they relate to each other.
COPY Cargo.toml ./

# Copy the two Rust crates the daemon needs:
#
#   ternary-math/    — The GF(3) arithmetic engine.
#                      Pure math, zero external dependencies.
#
#   services/inter-cube/  — The four services:
#       GLB = Geometric Load Balancer (routes packets by math)
#       CON = Cube Overlay Network (encrypted tunnels)
#       CRS = Cube Registration Service (address allocation)
#       FTS = Fault Tolerance Service (health monitoring)
#
COPY ternary-math/ ternary-math/
COPY services/inter-cube/ services/inter-cube/

# We also need these in the workspace even though we're only
# building inter-cube. Cargo requires all workspace members to
# exist. Create minimal stubs for the ones we don't need here.
RUN mkdir -p src/kernel/src && \
    echo '[package]\nname = "ternary-kernel"\nversion = "0.1.0"\nedition = "2021"' > src/kernel/Cargo.toml && \
    echo "" > src/kernel/src/lib.rs && \
    mkdir -p services/pqti-service/src && \
    echo '[package]\nname = "pqti-service"\nversion = "0.1.0"\nedition = "2021"' > services/pqti-service/Cargo.toml && \
    echo "" > services/pqti-service/src/lib.rs

# Compile the inter-cube daemon in release mode (optimized).
# This downloads dependencies, compiles everything, and produces
# a single binary: target/release/inter-cube-daemon
RUN cargo build -p inter-cube --release

# Run all 57 tests. If any test fails, the build stops here.
# You will never deploy broken code.
RUN cargo test -p inter-cube --release

# ── STAGE 2: Create the runtime image ───────────────────────────
# Start fresh with a minimal Linux (no Rust compiler, no source).
FROM debian:bookworm-slim

# Install only what the binary needs at runtime:
#   ca-certificates = so HTTPS connections work
#   curl = for the health check (see below)
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from Stage 1.
# Everything else (compiler, source, build artifacts) is discarded.
COPY --from=builder /app/target/release/inter-cube-daemon /usr/local/bin/

# ── Settings (override these when running the container) ─────────
#
# CUBE_MODE controls what this container does:
#   "all"  = Run all four services in one process (default, good for demos)
#   "crs"  = Run only the registration desk (central coordinator)
#   "cube" = Run as a normal cube (registers with a remote CRS)
#
# CUBE_CRS_URL is where cubes find the registration desk:
#   Example: http://crs:8080  (inside Docker network)
#   Example: https://plenumnet.replit.app/api/salvi/inter-cube/crs  (your live Replit)
#
# CUBE_ENDPOINT is this cube's public address (how others reach it):
#   Example: cube-1:51820  (inside Docker network)
#   Example: 203.0.113.5:51820  (on the real internet)
#
ENV CUBE_MODE=all
ENV CUBE_CRS_URL=http://localhost:8080
ENV CUBE_ENDPOINT=0.0.0.0:51820
ENV CUBE_LOG_LEVEL=info
ENV RUST_LOG=info

# Open two ports:
#   8080  = CRS API (registration, lookup, heartbeat)
#   51820 = Cube-to-cube tunnel traffic
EXPOSE 8080 51820

# Health check: Docker pings this endpoint every 30 seconds.
# If it fails 3 times, Docker marks the container as unhealthy.
HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
    CMD curl -sf http://localhost:8080/health || exit 1

# When the container starts, run the daemon.
CMD ["inter-cube-daemon"]
