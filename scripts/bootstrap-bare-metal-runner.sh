#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────────
# bootstrap-bare-metal-runner.sh
#
# Operator-side bootstrap for the self-hosted GitHub Actions runner
# that powers the `bare-metal-qemu` workflow
# (.github/workflows/bare-metal-qemu.yml). Run this on the runner
# host machine — NOT inside the Replit workspace and NOT inside CI.
#
# What this script does (idempotent — safe to re-run):
#   1. Installs OS prerequisites (qemu-system-x86, KVM tools,
#      build-essential, curl, jq, libicu, libssl) on the host.
#   2. Adds the runner user (default: $USER) to the `kvm` group so
#      QEMU can use hardware virtualisation.
#   3. Installs the pinned Rust toolchain
#      (channel + components from
#       src/kernel/bare-metal/rust-toolchain.toml — currently
#       `nightly` + `rust-src`) under the runner user's $HOME.
#   4. Downloads, unpacks, and registers the GitHub Actions runner
#      under $RUNNER_HOME (default: $HOME/actions-runner) with the
#      `bare-metal` label and the operator-provided token.
#   5. Installs the runner as a systemd service (`actions.runner.*`)
#      and starts it. The next push that touches `src/kernel/**`,
#      `AASC/**`, `ternary-math/**`, or
#      the workflow itself will route to this runner.
#
# Required env vars (operator obtains these from the repo's GitHub
# settings page → Actions → Runners → New self-hosted runner):
#   RUNNER_URL    — e.g. https://github.com/SigmaWolf-8/Ternary
#   RUNNER_TOKEN  — short-lived registration token (single-use)
#
# Optional env vars:
#   RUNNER_NAME   — default: hostname
#   RUNNER_HOME   — default: $HOME/actions-runner
#   RUNNER_USER   — default: $USER (script also supports running
#                   under sudo with RUNNER_USER set to the target
#                   account)
#   RUNNER_LABELS — default: "self-hosted,bare-metal,Linux,X64"
#                   (the workflow only needs `bare-metal`; the
#                   others are conventional)
#   RUNNER_VERSION — default: "2.319.1" (override only to pin to a
#                    newer release tag from
#                    https://github.com/actions/runner/releases)
#
# Reference: https://docs.github.com/en/actions/hosting-your-own-runners/managing-self-hosted-runners
# Audit doc: docs/audit/bare-metal-incorporation.md (§ "Bare-metal
# runner provisioning")
#
# Capomastro Holdings Ltd. — Applied Physics Division
# ──────────────────────────────────────────────────────────────────
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[0;33m'
BOLD='\033[1m'
NC='\033[0m'

log()  { echo -e "${CYAN}[runner-bootstrap]${NC} $*"; }
ok()   { echo -e "${GREEN}[  OK ]${NC} $*"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
fail() { echo -e "${RED}[FAIL]${NC} $*" >&2; exit 1; }

# ──────────────────────────────────────────────────────────────────
# Pre-flight checks
# ──────────────────────────────────────────────────────────────────
if [[ "$(uname -s)" != "Linux" ]]; then
    fail "This script only supports Linux hosts (the bare-metal-qemu workflow assumes Linux + KVM)."
fi

if [[ -z "${RUNNER_URL:-}" || -z "${RUNNER_TOKEN:-}" ]]; then
    fail "RUNNER_URL and RUNNER_TOKEN are required. Get the token from:
       Settings → Actions → Runners → New self-hosted runner (Linux x64).
       The token is single-use and expires after ~1 hour.
       NOTE: this is the *registration* token, not the *removal* token.
             GitHub mints them separately. Only set RUNNER_REMOVAL_TOKEN
             below if you want a clean re-registration on a host that
             already has a stale .runner file."
fi

RUNNER_NAME="${RUNNER_NAME:-$(hostname)}"
RUNNER_USER="${RUNNER_USER:-$USER}"
RUNNER_HOME="${RUNNER_HOME:-/home/$RUNNER_USER/actions-runner}"
RUNNER_LABELS="${RUNNER_LABELS:-self-hosted,bare-metal,Linux,X64}"
RUNNER_VERSION="${RUNNER_VERSION:-2.319.1}"
# Optional — distinct from RUNNER_TOKEN. Only required when re-registering
# a host that already has a stale .runner file. Mint via:
#   Settings → Actions → Runners → <existing runner> → Remove → copy token.
RUNNER_REMOVAL_TOKEN="${RUNNER_REMOVAL_TOKEN:-}"

case "$RUNNER_LABELS" in
    *bare-metal*) ;;
    *) fail "RUNNER_LABELS must include 'bare-metal' so the workflow's runs-on selector matches." ;;
esac

echo -e "${BOLD}"
echo "════════════════════════════════════════════════════════════════"
echo "  PlenumNET Bare-Metal Runner — Bootstrap"
echo "  Capomastro Holdings Ltd. — Applied Physics Division"
echo "════════════════════════════════════════════════════════════════"
echo -e "${NC}"
log "Runner URL    : $RUNNER_URL"
log "Runner name   : $RUNNER_NAME"
log "Runner user   : $RUNNER_USER"
log "Runner home   : $RUNNER_HOME"
log "Runner labels : $RUNNER_LABELS"
log "Runner version: $RUNNER_VERSION"
echo ""

# ──────────────────────────────────────────────────────────────────
# Step 1 — OS prerequisites
# ──────────────────────────────────────────────────────────────────
log "Step 1/5: Installing OS prerequisites (QEMU, KVM, build tools)..."
if ! command -v sudo >/dev/null 2>&1; then
    fail "sudo is required to install system packages."
fi

if command -v apt-get >/dev/null 2>&1; then
    sudo apt-get update -qq
    sudo apt-get install -y -qq \
        qemu-system-x86 qemu-utils \
        cpu-checker \
        build-essential pkg-config \
        curl jq ca-certificates \
        libicu-dev libssl-dev libkrb5-3 zlib1g \
        git
elif command -v dnf >/dev/null 2>&1; then
    sudo dnf install -y \
        qemu-system-x86 qemu-img \
        gcc make pkgconfig \
        curl jq ca-certificates \
        libicu krb5-libs zlib openssl \
        git
else
    fail "Unsupported package manager (only apt-get and dnf are wired)."
fi
ok "OS prerequisites installed."

# KVM acceleration check — non-fatal so the script still works on
# nested-virt hosts where kvm-ok returns warnings but QEMU still runs.
if command -v kvm-ok >/dev/null 2>&1; then
    if ! kvm-ok >/dev/null 2>&1; then
        warn "kvm-ok reports KVM is not usable — QEMU will fall back to TCG (slow)."
        warn "Boot timings under TCG can exceed the workflow's 90s timeout."
    else
        ok "KVM acceleration available."
    fi
fi

# ──────────────────────────────────────────────────────────────────
# Step 2 — KVM group membership
# ──────────────────────────────────────────────────────────────────
log "Step 2/5: Granting $RUNNER_USER access to /dev/kvm..."
if getent group kvm >/dev/null 2>&1; then
    sudo usermod -aG kvm "$RUNNER_USER"
    ok "$RUNNER_USER added to 'kvm' group (re-login required for the group to take effect)."
else
    warn "No 'kvm' group on this host — skipping. The runner will use TCG."
fi

# ──────────────────────────────────────────────────────────────────
# Step 3 — Rust toolchain (matches src/kernel/bare-metal/rust-toolchain.toml)
# ──────────────────────────────────────────────────────────────────
log "Step 3/5: Installing pinned Rust toolchain (nightly + rust-src) under $RUNNER_USER..."
if ! sudo -u "$RUNNER_USER" -H bash -c 'command -v rustup >/dev/null 2>&1'; then
    sudo -u "$RUNNER_USER" -H bash -c \
        'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly --profile minimal'
fi
sudo -u "$RUNNER_USER" -H bash -c \
    'source "$HOME/.cargo/env"; rustup toolchain install nightly --component rust-src'
ok "Nightly Rust + rust-src ready for $RUNNER_USER."

# ──────────────────────────────────────────────────────────────────
# Step 4 — Download + register the GitHub Actions runner
# ──────────────────────────────────────────────────────────────────
log "Step 4/5: Installing actions-runner $RUNNER_VERSION at $RUNNER_HOME..."

sudo -u "$RUNNER_USER" -H mkdir -p "$RUNNER_HOME"
RUNNER_TARBALL="actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz"
RUNNER_URL_TARBALL="https://github.com/actions/runner/releases/download/v${RUNNER_VERSION}/${RUNNER_TARBALL}"

if [[ ! -f "$RUNNER_HOME/run.sh" ]]; then
    sudo -u "$RUNNER_USER" -H bash -c "
        cd '$RUNNER_HOME'
        curl -O -L '$RUNNER_URL_TARBALL'
        tar xzf '$RUNNER_TARBALL'
        rm -f '$RUNNER_TARBALL'
    "
    ok "Runner archive unpacked."
else
    ok "Runner archive already present — skipping download."
fi

# Configure. `--replace` is the supported re-registration path: it takes the
# *registration* token and force-replaces any existing runner that owns the
# same name in GitHub's runner table, so the operator does NOT need a
# separate removal token in the common case.
#
# We only call `./config.sh remove` (which DOES need a removal token) when
# the operator explicitly supplies RUNNER_REMOVAL_TOKEN. Without that token
# the call would fail silently and leave a stale `.runner` file on disk —
# `--replace` then takes care of the GitHub-side cleanup.
log "Registering runner with GitHub..."
if [[ -f "$RUNNER_HOME/.runner" && -n "$RUNNER_REMOVAL_TOKEN" ]]; then
    log "Stale .runner detected and RUNNER_REMOVAL_TOKEN supplied — unconfiguring first."
    sudo -u "$RUNNER_USER" -H bash -c "
        cd '$RUNNER_HOME'
        ./config.sh remove --token '$RUNNER_REMOVAL_TOKEN' || true
    "
elif [[ -f "$RUNNER_HOME/.runner" ]]; then
    log "Stale .runner detected (no removal token) — relying on --replace for the GitHub-side swap."
    # Drop the local marker so config.sh does not refuse to write a new one.
    sudo -u "$RUNNER_USER" -H rm -f "$RUNNER_HOME/.runner" "$RUNNER_HOME/.credentials" "$RUNNER_HOME/.credentials_rsaparams"
fi

sudo -u "$RUNNER_USER" -H bash -c "
    cd '$RUNNER_HOME'
    ./config.sh \
        --url '$RUNNER_URL' \
        --token '$RUNNER_TOKEN' \
        --name '$RUNNER_NAME' \
        --labels '$RUNNER_LABELS' \
        --work '_work' \
        --unattended \
        --replace
"
ok "Runner registered with labels: $RUNNER_LABELS"

# ──────────────────────────────────────────────────────────────────
# Step 5 — Install + start the systemd service
# ──────────────────────────────────────────────────────────────────
log "Step 5/5: Installing actions-runner as a systemd service..."
(
    cd "$RUNNER_HOME"
    sudo ./svc.sh install "$RUNNER_USER"
    sudo ./svc.sh start
)
ok "Runner service installed and started."

echo ""
echo -e "${BOLD}${GREEN}════════════════════════════════════════════════════════════════"
echo -e "  Bootstrap complete."
echo -e "════════════════════════════════════════════════════════════════${NC}"
echo ""
log "Verify the runner appears as 'Idle' here:"
log "  ${RUNNER_URL}/settings/actions/runners"
log ""
log "Trigger the workflow:"
log "  gh workflow run bare-metal-qemu.yml --ref main"
log "  # or: push any commit touching src/kernel/** to trigger automatically."
log ""
log "Tail the service logs:"
log "  sudo journalctl -u actions.runner.* -f"
echo ""
warn "If $RUNNER_USER was added to the 'kvm' group during this run,"
warn "restart the runner service so the new group membership takes effect:"
warn "  sudo systemctl restart 'actions.runner.*'"
