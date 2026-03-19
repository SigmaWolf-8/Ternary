# Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
# Patent(s) Pending — All Rights Reserved
# Applied Physics Division
#
# Multi-stage build for PlenumNET Inter-Cube Infrastructure Daemon

# ── Stage 1: Build ──────────────────────────────────────────────────
FROM rust:1.78-slim-bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config && rm -rf /var/lib/apt/lists/*

WORKDIR /build

COPY ternary-math/ ternary-math/
COPY services/inter-cube/ services/inter-cube/

WORKDIR /build/services/inter-cube
ENV CARGO_TARGET_DIR=/build/target
RUN cargo build --release --bin inter-cube-daemon

# ── Stage 2: Runtime ────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates wget && rm -rf /var/lib/apt/lists/*

RUN groupadd -r plenum && useradd -r -g plenum -s /usr/sbin/nologin plenum

COPY --from=builder /build/target/release/inter-cube-daemon /usr/local/bin/inter-cube-daemon

RUN chmod +x /usr/local/bin/inter-cube-daemon

USER plenum

EXPOSE 8080
EXPOSE 51820/udp

ENV CUBE_MODE=all
ENV RUST_LOG=info

HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
  CMD wget -qO- http://localhost:8080/health || exit 1

ENTRYPOINT ["/usr/local/bin/inter-cube-daemon"]
