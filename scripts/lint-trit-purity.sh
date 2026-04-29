#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# lint-trit-purity.sh — hard gate for the Salvi Framework's "trits, not bytes"
# invariant. Exits non-zero (and prints offending lines) when any of the
# following appear in non-allowlisted code.
#
# This gate exists because the same byte/hex propagation mistake — most
# recently in RepoSync — has been made repeatedly. New code MUST take
# operator-supplied symmetric key material as balanced trits, not bytes
# and not hex strings.
#
# THE FIVE RULES
# --------------
#   R1: No foreign crypto imports OR fully-qualified calls
#       Forbidden roots: sha2, sha3, blake2, blake3, md5, hmac, ring,
#                        openssl, aes_gcm, aead, chacha20, poly1305,
#                        argon2, scrypt
#       Matches:  use sha2::… ;  pub use sha3::… ;  sha2::Sha256::new() ;
#                 ::aes_gcm::… ;  any reference to those crate roots in
#                 Rust source.
#       Replace with framework primitives:
#         crypto::keyed_sponge::KeyedTernarySponge   (stream cipher / KDF)
#         crypto::sponge::sponge_hash_bytes          (hashing)
#         crypto::tl_dsa::{sign,verify}              (signing)
#         crypto::tl_kem::{encapsulate,decapsulate}  (key exchange)
#
#   R1b: Cargo.toml deps must not pull in forbidden crates outside allowlist.
#
#   R2: Symmetric / pre-shared keys MUST be balanced trits.
#       Forbidden: any field named shared_key / pre_shared_key / psk /
#       sym_key / symmetric_key typed as a byte container in any common
#       form (`[u8; …]`, `Vec<u8>`, `&[u8]`, `Box<[u8]>`, `Cow<…, [u8]>`,
#       `Option<Vec<u8>>`, `String`).
#       Required:  `Vec<i8>` of length SHARED_KEY_TRITS (243), every
#                  entry in {-1, 0, +1}, validated at the entrypoint.
#
#   R3: No hex-encoded SECRET key fields in operator-facing config.
#       Forbidden:  shared_key_hex, pre_shared_key_hex, psk_hex,
#                   secret_key_hex, sym_key_hex, key_hex — in Rust struct
#                   fields, JSON config keys, TOML keys, env var names.
#       Required:   *_trits — a string of '-', '0', '+' characters,
#                   one per balanced trit.
#       (Public-key hex fields — public_key_hex, pubkey_hex — are
#        intentionally NOT flagged: public keys are not secrets.)
#
#   R4: hex / base64 decode MUST NOT be used to parse key material.
#       Triggered when hex::decode / FromHex / from_hex_string /
#       base64::decode appears in any function whose body mentions a
#       symmetric-key identifier (block-scoped, not line-local).
#
# ALLOWLIST (these directories may legitimately use byte representations
# because they implement the framework primitives themselves, run
# benchmarks against industry crypto, or document the wire format):
#   benchmarks/                          industry-vs-framework benchmarks
#   ternary-math/benches/                A/B benchmarks
#   ternary-math/src/{tl_dsa,tl_kem}.rs  lattice math packed-byte storage
#   src/kernel/src/crypto/               framework primitives
#   tools/plenum-pack/src/tis27.rs       TIS-27 wire format helpers
#
# Branch protection: in addition to running this gate in CI, the
# `Trit Purity Gate` check should be marked as REQUIRED in the GitHub
# branch protection rules for the default branch. Configure under
# Settings → Branches → Branch protection rules. Without that, a PR can
# theoretically be merged with a failing gate.
#
# Run locally:  bash scripts/lint-trit-purity.sh
# Run in CI:    .github/workflows/trit-purity.yml invokes this script.
# ─────────────────────────────────────────────────────────────────────────────

set -u
status=0

if ! command -v rg >/dev/null 2>&1; then
  echo "lint-trit-purity: ripgrep ('rg') is required but not installed" >&2
  exit 2
fi

# Globs shared by the source-code rules. Documentation and the gate
# infrastructure itself are excluded so the rule text doesn't trip the rule.
GLOBS=(
  --glob '!deployments/**'
  --glob '!attached_assets/**'
  --glob '!target/**'
  --glob '!node_modules/**'
  --glob '!dist/**'
  --glob '!**/*.lock'
  --glob '!**/package-lock.json'
  --glob '!**/Cargo.lock'
  --glob '!benchmarks/**'
  --glob '!ternary-math/benches/**'
  --glob '!src/kernel/src/crypto/**'
  --glob '!ternary-math/src/tl_dsa.rs'
  --glob '!ternary-math/src/tl_kem.rs'
  --glob '!ternary-math/tests/phase_cross_compat.rs'
  --glob '!tools/plenum-pack/src/tis27.rs'
  --glob '!scripts/lint-trit-purity.sh'
  --glob '!.github/workflows/trit-purity.yml'
  --glob '!AGENTS.md'
  --glob '!CONTRIBUTING.md'
  --glob '!docs/**'
  --glob '!**/*.md'
)

# Cargo manifests use a separate, narrower glob set (only Cargo.toml files,
# allowlist still applies).
CARGO_GLOBS=(
  --glob '*Cargo.toml'
  --glob '!deployments/**'
  --glob '!benchmarks/**'
  --glob '!ternary-math/Cargo.toml'
  --glob '!target/**'
  --glob '!node_modules/**'
  --glob '!dist/**'
)

FORBIDDEN_CRATES='sha2|sha3|blake2|blake3|md5|hmac|ring|openssl|aes_gcm|aead|chacha20|poly1305|argon2|scrypt'

section() { printf '\n=== %s ===\n' "$1"; }
fail()    { status=1; }

# ─── R1: foreign crypto imports + fully-qualified calls in Rust sources ──────
section "R1 — Foreign crypto imports / fully-qualified usage"
# (a) `use ...` and `pub use ...` lines.
hits_use=$(rg -n --no-heading -t rust \
  "^[[:space:]]*(pub[[:space:]]+)?use[[:space:]]+(::[[:space:]]*)?(${FORBIDDEN_CRATES})([:: ]|;)" \
  "${GLOBS[@]}" 2>/dev/null || true)
# (b) Fully-qualified path usage anywhere on a non-comment Rust line.
#     Filter out plain-text mentions inside string literals and comments.
hits_fqp=$(rg -n --no-heading -t rust \
  "(^|[^A-Za-z0-9_:])(${FORBIDDEN_CRATES})::[A-Za-z_]" \
  "${GLOBS[@]}" 2>/dev/null \
  | rg -v '^[^:]+:[0-9]+:[[:space:]]*//' \
  | rg -v '^[^:]+:[0-9]+:[[:space:]]*\*' \
  || true)
hits=$(printf '%s\n%s\n' "$hits_use" "$hits_fqp" | grep -v '^$' | sort -u || true)
if [ -n "$hits" ]; then
  printf '%s\n' "$hits"
  echo ""
  echo "  FIX: replace with framework primitives:"
  echo "    crypto::keyed_sponge::KeyedTernarySponge   (stream cipher / KDF)"
  echo "    crypto::sponge::sponge_hash_bytes          (hashing)"
  echo "    crypto::tl_dsa::{sign,verify}              (signing)"
  echo "    crypto::tl_kem::{encapsulate,decapsulate}  (key exchange)"
  fail
fi

# ─── R1b: forbidden crates declared as Cargo dependencies ────────────────────
section "R1b — Forbidden crates declared as Cargo dependencies"
# Match `cratename = …` or `cratename.workspace = …` at start of TOML key,
# and `<cratename> = { … }` table form.
hits=$(rg -n --no-heading \
  "^[[:space:]]*(${FORBIDDEN_CRATES})[[:space:]]*(=|\.workspace[[:space:]]*=|\.version[[:space:]]*=)" \
  "${CARGO_GLOBS[@]}" 2>/dev/null || true)
if [ -n "$hits" ]; then
  printf '%s\n' "$hits"
  echo ""
  echo "  FIX: remove the dependency. Use framework primitives instead."
  echo "  If this is a benchmark crate, place it under benchmarks/ or"
  echo "  ternary-math/Cargo.toml (allowlisted)."
  fail
fi

# ─── R2: symmetric / pre-shared keys typed as bytes ──────────────────────────
section "R2 — Symmetric pre-shared keys typed as bytes / hex / String"
# Any byte-container form on a key field. The combined regex covers:
#   [u8; N]          fixed array
#   Vec<u8>          owned
#   &[u8]            borrowed slice
#   &'a [u8]         borrowed slice with lifetime
#   Box<[u8]>        boxed slice
#   Cow<…, [u8]>     copy-on-write slice
#   Option<Vec<u8>>  optional owned
#   Option<&[u8]>    optional borrow
#   String           hex/base64 string holder (closed by , ) or })
hits=$(rg -n --no-heading -t rust \
  '\b(shared_key|pre_shared_key|psk|sym_key|symmetric_key)[[:space:]]*:[[:space:]]*(Option[[:space:]]*<[[:space:]]*)?(\[u8;|Vec[[:space:]]*<[[:space:]]*u8|&[[:space:]]*(['"'"'][a-zA-Z_]+[[:space:]]+)?\[[[:space:]]*u8|Box[[:space:]]*<[[:space:]]*\[[[:space:]]*u8|Cow[[:space:]]*<[^>]*\[[[:space:]]*u8|String[[:space:]]*[,)}>])' \
  "${GLOBS[@]}" 2>/dev/null || true)
if [ -n "$hits" ]; then
  printf '%s\n' "$hits"
  echo ""
  echo "  FIX: keys are trits, not bytes. Use:"
  echo "    pub shared_key: Vec<i8>,   // length = SHARED_KEY_TRITS (243)"
  echo "  and validate every entry is in {-1, 0, +1} at the entrypoint."
  fail
fi

# ─── R3: hex-encoded SECRET key fields in operator-facing config ─────────────
section "R3 — Hex-encoded SECRET key fields (Rust struct / JSON / TOML / env)"
hits=$(rg -n --no-heading \
  '("?\b(shared_key_hex|pre_shared_key_hex|psk_hex|secret_key_hex|sym_key_hex|symmetric_key_hex|key_hex)\b"?)[[:space:]]*[:=]' \
  "${GLOBS[@]}" 2>/dev/null \
  | rg -v 'public_key_hex|pubkey_hex|pub_key_hex' \
  || true)
# Env var pattern: SHARED_KEY_HEX, PSK_HEX, etc.
env_hits=$(rg -n --no-heading \
  '\b(SHARED_KEY_HEX|PRE_SHARED_KEY_HEX|PSK_HEX|SECRET_KEY_HEX|SYM_KEY_HEX|SYMMETRIC_KEY_HEX|KEY_HEX)\b' \
  "${GLOBS[@]}" 2>/dev/null \
  | rg -v 'PUBLIC_KEY_HEX|PUBKEY_HEX|PUB_KEY_HEX' \
  || true)
hits=$(printf '%s\n%s\n' "$hits" "$env_hits" | grep -v '^$' | sort -u || true)
if [ -n "$hits" ]; then
  printf '%s\n' "$hits"
  echo ""
  echo "  FIX: replace '*_hex' SECRET key fields with '*_trits' — a string of"
  echo "  '-', '0', '+' characters (one per balanced trit)."
  fail
fi

# ─── R4: hex/base64 decode used inside a function that mentions key material ─
section "R4 — hex / base64 decode used near key material (block-scoped)"
KEY_TOKENS='shared_key|pre_shared_key|\bpsk\b|secret_key|sym_key|symmetric_key'
DECODE_TOKENS='hex::decode|FromHex|from_hex_string|base64::decode|BASE64_STANDARD\.decode'
# Block-scoped check: for every Rust file, look at each function body and
# flag if it contains BOTH a decode call and a key-name reference.
flagged_files=""
while IFS= read -r -d '' file; do
  # Quick file-level prefilter to skip files that obviously don't touch both.
  if ! rg -q "$DECODE_TOKENS" "$file" 2>/dev/null; then continue; fi
  if ! rg -qi "$KEY_TOKENS" "$file" 2>/dev/null; then continue; fi
  # Pull each line that has either a key token or a decode call, with the line
  # number, then collapse into per-fn co-occurrence using awk over the brace
  # depth of the file as a coarse function-scope proxy.
  matches=$(awk -v decode_re="$DECODE_TOKENS" -v key_re="$KEY_TOKENS" '
    BEGIN { depth = 0; fn_start = 0; has_decode = 0; has_key = 0; fn_name = "<top>"; }
    {
      line = $0
      # detect fn start at depth 0 / 1 (impl methods)
      if (depth <= 1 && match(line, /fn[[:space:]]+([A-Za-z_][A-Za-z0-9_]*)/)) {
        fn_name = substr(line, RSTART+3, RLENGTH-3)
        sub(/^[[:space:]]+/, "", fn_name)
      }
      # entering a block?
      open_count = gsub(/\{/, "{", line)
      close_count = gsub(/\}/, "}", line)
      if (depth == 0 && open_count > 0) {
        fn_start = NR; has_decode = 0; has_key = 0; fn_name_at_start = fn_name
      }
      depth += open_count - close_count
      if (depth < 0) depth = 0
      if (line ~ decode_re) has_decode = 1
      if (tolower(line) ~ tolower(key_re)) has_key = 1
      if (depth == 0 && fn_start > 0) {
        if (has_decode && has_key) {
          printf "%s:%d: function `%s` uses decode and key material together\n", FILENAME, fn_start, fn_name_at_start
        }
        fn_start = 0; has_decode = 0; has_key = 0
      }
    }
  ' "$file")
  if [ -n "$matches" ]; then
    flagged_files="${flagged_files}${matches}\n"
  fi
done < <(rg -l -t rust "$DECODE_TOKENS" "${GLOBS[@]}" -0 2>/dev/null || true)

if [ -n "$flagged_files" ]; then
  printf '%b' "$flagged_files"
  echo ""
  echo "  FIX: do not parse key material from hex / base64."
  echo "  Use the '-/0/+' trit string format defined in plenum-reposync/src/main.rs."
  echo "  Hex helpers are fine for digest display / logging — but not for keys."
  fail
fi

if [ "$status" -eq 0 ]; then
  printf '\nlint-trit-purity: OK — no forbidden byte/hex key paths or foreign crypto imports.\n'
else
  printf '\nlint-trit-purity: FAILED — fix the violations above before merging.\n' >&2
  printf 'See AGENTS.md "Trits, not bytes" for the full rule.\n' >&2
fi

exit "$status"
