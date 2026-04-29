# AGENTS.md

> Non-negotiable rules for any agent (human or AI) writing code in this
> repository. Read this before your first edit. The rules in **Trits, Not
> Bytes** below are enforced by the `trit-purity` CI gate; PRs that violate
> them will not merge.

---

## 1. Trits, Not Bytes

The Salvi Framework is a **trit-native** post-quantum stack. Operator-supplied
key material, transport ciphers, and high-level cryptographic primitives are
expressed in balanced ternary (`{-1, 0, +1}`), not in bytes and not in hex.

This rule has been violated repeatedly — most recently in RepoSync, which
originally took a 48-byte / 96-character-hex pre-shared key and ran a
hand-rolled byte permutation over it. **That mistake is now structurally
impossible to repeat**, because every PR runs through `scripts/lint-trit-purity.sh`
in CI.

### The five enforced rules

| # | Forbidden                                                                                          | Required                                                                                  |
|---|----------------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------------|
| **R1** | Foreign crypto in Rust source — `use sha2` / `pub use sha3` / fully-qualified `aes_gcm::…` / etc. for `sha2`, `sha3`, `blake2`, `blake3`, `md5`, `hmac`, `ring`, `openssl`, `aes_gcm`, `aead`, `chacha20`, `poly1305`, `argon2`, `scrypt`, outside `benchmarks/` and the framework's own crypto modules | `crypto::keyed_sponge::KeyedTernarySponge`, `crypto::sponge::sponge_hash_bytes`, `crypto::tl_dsa`, `crypto::tl_kem` |
| **R2** | Forbidden crates declared as Cargo dependencies (`sha2 = "…"`, `chacha20.workspace = true`, etc.) outside the allowlist | Don't add the dependency.                                                                  |
| **R3** | `shared_key` / `pre_shared_key` / `psk` / `sym_key` / `symmetric_key` typed as any byte container — `[u8; N]`, `Vec<u8>`, `&[u8]`, `Box<[u8]>`, `Cow<…, [u8]>`, `Option<Vec<u8>>`, `String` | `pub shared_key: Vec<i8>` of length `SHARED_KEY_TRITS` (243), every entry in `{-1, 0, +1}` |
| **R4** | Hex-encoded SECRET key fields in Rust struct fields, JSON config, TOML config, or env-var names — `shared_key_hex`, `psk_hex`, `secret_key_hex`, `key_hex`, `SHARED_KEY_HEX`, etc. | `*_trits` — a string of `-`, `0`, `+` characters, one per balanced trit                   |
| **R5** | `hex::decode` / `FromHex` / `from_hex_string` / `base64::decode` invoked anywhere in the same function body as a symmetric-key identifier (block-scoped via brace-depth) | Parse the trit string directly: `c.chars().filter_map(...)`                               |

Public-key hex transport (e.g. `public_key_hex` for TL-DSA pubkeys returned
from an API) is intentionally **not** flagged. Public keys are not secrets;
hex-encoding them for display or wire transport is standard. If you have a
public key in a local variable, name it `pubkey_hex` (not `key_hex`) so both
the rule and future readers can tell at a glance that it's not a secret.

### The reference implementation

`src/kernel/src/repo_sync.rs` and `plenum-reposync/src/main.rs` are the
canonical example of how operator-facing symmetric crypto is supposed to
look. Copy that pattern when adding any new service that takes a shared
key on the wire:

1. **Config field**: `pub shared_key: Vec<i8>` with `SHARED_KEY_TRITS = 243`.
2. **Validation**: a `Config::validate()` method that rejects wrong length,
   out-of-range trits, and the all-zero placeholder. Call it at the
   library entrypoint (`pub fn run(...)`), not just at the launcher.
3. **Cipher**: `KeyedTernarySponge::new(&key).absorb(&nonce_trits).squeeze(...)` —
   never a hand-rolled permutation.
4. **Per-frame nonce**: `(timestamp_nanos, atomic_counter)` — uniqueness,
   not randomness, is the security requirement.
5. **Byte ↔ trit mapping**: 6 trits per byte (`3^6 = 729 > 256`) for
   nonce absorption (injective); 8 trits per byte with rejection sampling
   (`v < 6400 = 25 * 256`, output `v % 256`) for keystream output (unbiased).
6. **Installer config** parses `shared_key_trits` as a `-/0/+` string. No
   hex.

### How to run the gate locally

```bash
bash scripts/lint-trit-purity.sh
```

Exit code `0` = clean. Exit code `1` = violations printed with file:line
references and the suggested fix.

### Allowlist

The gate skips paths that legitimately use byte representations:

- `benchmarks/` — A/B benchmarks against industry crypto.
- `ternary-math/benches/` — same.
- `ternary-math/src/tl_dsa.rs`, `tl_kem.rs` — lattice math packed-byte storage.
- `ternary-math/tests/phase_cross_compat.rs` — cross-implementation parity
  test that asserts byte-level equivalence with the TypeScript Phase
  Encryption v3 backend; the hex strings are test vectors, not key transport.
- `src/kernel/src/crypto/**` — framework primitives that bridge byte ↔ trit
  internally; the byte-accepting helpers are never the ones that operator
  config feeds into.
- `tools/plenum-pack/src/tis27.rs` — TIS-27 wire format helpers.
- `deployments/`, `attached_assets/`, `target/`, `node_modules/`, `dist/`,
  lockfiles, `**/*.md` — out of scope.

If you genuinely need to add a new allowlist entry, add it to the `GLOBS`
array in `scripts/lint-trit-purity.sh` **in the same PR** as the code that
needs it, and explain why in the commit message.

### Branch protection (one-time GitHub setting, NOT in the repo)

The CI workflow alone does not block merge — it only fails the check. To
make the gate truly unbypassable, the `Trit Purity Gate / lint-trit-purity`
status check must be marked **Required** under
`Settings → Branches → Branch protection rules → main` (and any other
protected branch). Without this, a PR could theoretically be force-merged
with a failing gate. This is the single remaining trust boundary; everything
else is enforced in code.

---

## 2. Other invariants (carried from the project root)

- **All constants live in `AASC/src/constants.rs`** — no magic numbers
  scattered across modules. The user has rejected past PRs over this.
- **Do not modify `deployments/` or `attached_assets/`** (except mandatory
  PAT redaction).
- **Do not invent formulas.** If a derivation isn't in the spec, ask before
  writing it down.
- **PAT redaction**: `s/ghp_[A-Za-z0-9]{36}/<REDACTED>/g` before committing
  anything from chat logs or attached assets.
- **No fabrication** in canonical maps, module lists, or documentation. The
  AASC canonical map at `client/public/aasc_canonical_map.svg` is
  reconciled against the ground truth in `AASC/src/lib.rs`; check both
  before editing either.

---

## 3. When the gate flags your PR

1. **Read the offending line.** The script prints `file:line:source` and a
   suggested fix.
2. **Fix it the trit-native way.** Copy the pattern from `repo_sync.rs`. If
   you can't see how, read that file first — don't ask the user to
   explain it again.
3. **Re-run locally**: `bash scripts/lint-trit-purity.sh`.
4. **Push.** The CI workflow `Trit Purity Gate` will re-verify.

If you genuinely believe a violation is a false positive, say so explicitly
in the PR description and propose either a refined regex or a new allowlist
entry. Do **not** silently ignore the gate.
