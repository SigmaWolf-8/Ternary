# TDNS v2.3 — Dockerfile
# Capomastro Holdings Ltd. — Applied Physics Division
#
# Multi-stage build:
#   Stage 1 (builder): compile both binaries
#   Stage 2 (runtime): minimal image with just the binaries
#
# Usage:
#   docker build -t tdns-v2 .
#   docker run -p 3927:3927 tdns-v2
#   docker run tdns-v2 tdns-scan https://github.com

# ─── Builder Stage ────────────────────────────────────────────────────────────

FROM rust:1.85-bookworm AS builder

WORKDIR /build

# Cache dependency compilation
COPY Cargo.toml Cargo.lock* ./
RUN mkdir -p src/bin && \
    echo "fn main() {}" > src/bin/tdns_scan.rs && \
    echo "fn main() {}" > src/bin/tdns_server.rs && \
    echo "pub fn placeholder() {}" > src/lib.rs && \
    cargo build --release 2>/dev/null || true && \
    rm -rf src

# Copy actual source
COPY src/ src/
COPY tests/ tests/

# Build release binaries
RUN cargo build --release --bins

# Run tests
RUN cargo test --release

# ─── Runtime Stage ────────────────────────────────────────────────────────────

FROM debian:bookworm-slim AS runtime

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binaries from builder
COPY --from=builder /build/target/release/tdns-server /usr/local/bin/tdns-server
COPY --from=builder /build/target/release/tdns-scan /usr/local/bin/tdns-scan

# Non-root user
RUN useradd -r -s /bin/false tdns
USER tdns

# Default port
EXPOSE 3927

# Health check
HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -sf http://localhost:3927/api/v1/health || exit 1

# Default: run the server
ENTRYPOINT ["tdns-server"]
CMD ["--host", "0.0.0.0", "--port", "3927"]
