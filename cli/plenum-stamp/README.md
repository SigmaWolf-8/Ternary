# plenum-stamp

PlenumNET RFC 3161 TSA CLI — sign and verify files with cryptographic timestamps.

Zero dependencies. Node.js 18+ only.

## Install

```bash
npm install -g plenum-stamp
```

Or run directly:

```bash
npx plenum-stamp sign myfile.tar.gz
```

## Quick Start

```bash
# 1. Sign a file (creates myfile.tar.gz.tsp + myfile.tar.gz.tsp.json)
plenum-stamp sign myfile.tar.gz

# 2. Verify the timestamp
plenum-stamp verify myfile.tar.gz

# 3. Inspect token metadata
plenum-stamp info myfile.tar.gz.tsp
```

## Commands

### `sign <file>`

Hashes the file with SHA-256 and submits the hash to the PlenumNET TSA. Saves the RFC 3161 timestamp token as `<file>.tsp` and human-readable metadata as `<file>.tsp.json`.

```bash
plenum-stamp sign dist/release-v2.0.0.tar.gz
```

Output:
```
  Stamped successfully.
  Serial:   142
  Time:     2026-03-03 14:30:00 UTC
  Policy:   PlenumNET SECURE
  Token:    release-v2.0.0.tar.gz.tsp (3.7 KB)
  Metadata: release-v2.0.0.tar.gz.tsp.json
```

### `verify <file>`

Verifies a timestamp token against the TSA. Pass the original file (looks for `<file>.tsp`) or the `.tsp` file directly.

```bash
plenum-stamp verify dist/release-v2.0.0.tar.gz
```

### `info <file.tsp>`

Displays stored metadata for a timestamp token without contacting the TSA.

```bash
plenum-stamp info dist/release-v2.0.0.tar.gz.tsp
```

### `cert`

Downloads the TSA's public certificate for offline verification.

```bash
plenum-stamp cert
# Saves: plenumnet-tsa.pem
```

## Options

| Flag | Description | Default |
|---|---|---|
| `-e, --endpoint <url>` | TSA endpoint URL | `https://plenumnet.replit.app` |
| `-t, --token <token>` | API authentication token | — |
| `-a, --algorithm <alg>` | Hash algorithm | `sha256` |
| `-p, --policy <oid>` | TSA policy OID | server default |
| `-f, --format <fmt>` | Output format: `text` or `json` | `text` |
| `-o, --output <path>` | Output file path | `<file>.tsp` |
| `--compact` | Use compact calendar encoding | — |
| `--calendars <list>` | Calendar systems (comma-separated or `*`) | — |

## Environment Variables

| Variable | Description |
|---|---|
| `PLENUM_ENDPOINT` | TSA base URL (overridden by `--endpoint`) |
| `PLENUM_API_TOKEN` | Bearer token for authentication (overridden by `--token`) |

## GitHub Actions

Use the companion GitHub Action to sign release artifacts automatically:

```yaml
- uses: SigmaWolf-8/Ternary/.github/actions/plenum-stamp@main
  with:
    files: 'dist/*.tar.gz'
    api-token: ${{ secrets.PLENUM_API_TOKEN }}
```

See [`.github/actions/plenum-stamp/action.yml`](../../.github/actions/plenum-stamp/action.yml) for full documentation.

## File Formats

### `.tsp` — Timestamp Token

DER-encoded RFC 3161 `TimeStampToken` (CMS SignedData). Contains:
- RSA-4096 signature from the PlenumNET TSA
- Post-quantum TL-DSA dual signature
- Embedded calendar context (42 calendar systems)
- HPTP high-precision timing metadata

Verifiable with standard tools:
```bash
openssl ts -verify -data myfile.tar.gz -in myfile.tar.gz.tsp -CAfile plenumnet-tsa.pem
```

### `.tsp.json` — Metadata

Human-readable JSON with the original filename, hash, serial number, generation time, and policy. Used by `plenum-stamp verify` and `plenum-stamp info` for offline reference.

## License

Proprietary — Capomastro Holdings Ltd. See LICENSE in the repository root.
