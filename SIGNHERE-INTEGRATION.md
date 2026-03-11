# SignHere Integration Guide — TL-Sponge-385 Document Hashing

**Version**: 1.0.0  
**Date**: 2026-03-09  
**Author**: RSalvi@Salvigroup.com  
**Repository**: SigmaWolf-8/Ternary  

This document provides actionable instructions for the SignHere team to integrate
PlenumNET's TL-Sponge-385 cryptographic hash into the e-signature workflow, replacing
ML-DSA/SHA-based document hashing with TL-Sponge-385.

---

## 1. Architecture Overview

| Component | Value |
|---|---|
| Algorithm | TL-Sponge-385 |
| Security variant | TL-Sponge-385 |
| OID | `1.3.6.1.4.1.0.100.3.1` |
| State | 729 trits (3⁶) |
| Rate | 243 trits (3⁵) |
| Capacity | 486 trits |
| Rounds | 9 (3× safety margin) |
| Output | 49 bytes (98 hex chars, 243 trits) |
| Security | 385-bit post-quantum |
| Theta | 7-neighbor substitution (±1, ±7, ±13) |
| Pi | Scatter permutation: π(i) = (376·i + 1) mod 729 |

---

## 2. Endpoint Reference

### 2.1 Document Hashing

```
POST /api/salvi/crypto/hash
```

**Request** (JSON):
```json
{
  "data": "<base64-encoded document bytes>"
}
```

**Request** (raw binary):
```
POST /api/salvi/crypto/hash
Content-Type: application/octet-stream
Body: <raw file bytes>
```

**Response**:
```json
{
  "success": true,
  "algorithm": "tl-sponge",
  "oid": "1.3.6.1.4.1.0.100.3.1",
  "hash": "<98-character hex digest>",
  "bytes": 49,
  "trits": 243,
  "security": "385-bit post-quantum",
  "inputSize": 1234,
  "construction": "TL-Sponge-385 (729-trit state, 243-trit rate, 9 rounds, 7-neighbor theta)"
}
```

**Limits**: 10 MB max input size. Rate limited.

### 2.2 TSA Timestamping (with TL-Sponge-385)

```
POST /api/tsa/timestamp/json
```

**Request**:
```json
{
  "hash": "<98-char hex digest from /api/salvi/crypto/hash>",
  "algorithm": "tl-sponge"
}
```

**Response**: RFC 3161 timestamp token with the TL-Sponge-385 hash embedded.

### 2.3 Timestamp Verification

```
POST /api/tsa/verify
```

**Request**:
```json
{
  "token": "<base64-encoded timestamp token>"
}
```

### 2.4 TL-DSA Signing

```
POST /api/pqti/tldsa/sign
```

Use this endpoint for post-quantum digital signatures on documents. TL-DSA is the
correct signing algorithm — do NOT use ML-DSA endpoints.

### 2.5 Hedera Witnessing

```
POST /api/hedera/v1/witness
```

Submit witness hashes to Hedera Consensus Service for immutable proof of operations.

---

## 3. Migration Steps

### Step 1: Hash the Document

Replace any SHA-256/SHA3-256 document hashing with:

```bash
curl -X POST https://plenumnet.replit.app/api/salvi/crypto/hash \
  -H "Content-Type: application/json" \
  -d '{"data":"<base64-encoded-document>"}'
```

Store the returned `hash` value (98 hex characters).

### Step 2: Timestamp the Hash

```bash
curl -X POST https://plenumnet.replit.app/api/tsa/timestamp/json \
  -H "Content-Type: application/json" \
  -d '{"hash":"<hash-from-step-1>","algorithm":"tl-sponge"}'
```

Store the returned timestamp token for audit and verification.

### Step 3: Sign with TL-DSA

```bash
curl -X POST https://plenumnet.replit.app/api/pqti/tldsa/sign \
  -H "Content-Type: application/json" \
  -d '{"message":"<hash-from-step-1>"}'
```

### Step 4: Witness on Hedera (optional, for Fortified tier)

```bash
curl -X POST https://plenumnet.replit.app/api/hedera/v1/witness \
  -H "Content-Type: application/json" \
  -d '{"hash":"<hash-from-step-1>","operation":"signhere-document-sign"}'
```

---

## 4. Salvi Framework Crypto Naming Convention

All ternary-native cryptographic primitives use the `TL-` prefix (Ternary Lattice):

| Algorithm | Type | Role | Security variant |
|---|---|---|---|
| **TL-DSA** | Signature | Post-quantum digital signatures | TL-DSA-44 / -65 / -87 |
| **TL-KEM** | Key encapsulation | Post-quantum key exchange | TL-KEM-512 / -768 / -1024 |
| **TL-Sponge** | Cryptographic hash | Document hashing, identity binding, Merkle trees | TL-Sponge-385 |
| **TIS-27** | Fast integrity (43-bit) | Wire packet checksums, scan hashing | N/A |

**TIS-27 has 43-bit cryptographic security** (proven by wide-trail analysis, TM-2026-008).
Same sponge construction as TL-Sponge-385, sized for speed. For document hashing,
signing, or identity binding, use TL-Sponge-385 (385-bit post-quantum security).

---

## 5. Key Differences from Previous Integration

| Aspect | Before | After |
|---|---|---|
| Hash algorithm | SHA-256 or SHA3-256 | TL-Sponge-385 |
| Hash length | 32 bytes (64 hex) | 49 bytes (98 hex) |
| OID | `2.16.840.1.101.3.4.2.1` | `1.3.6.1.4.1.0.100.3.1` |
| Security level | 128-bit classical | 385-bit post-quantum |
| Signing endpoint | `/api/salvi/crypto/ml-dsa` (removed) | `/api/pqti/tldsa/sign` |
| Witness endpoint | `/api/salvi/witness/sign` (removed) | `/api/hedera/v1/witness` |
| TSA algorithm param | `"sha256"` or `"sha3-256"` | `"tl-sponge"` |
| Merkle audit log | SHA3-256 internally | TL-Sponge-385 internally |

---

## 6. Verification Checklist

- [ ] Document hash returns exactly 98 hex characters
- [ ] TSA accepts `algorithm: "tl-sponge"` without error
- [ ] Same document always produces the same hash (deterministic)
- [ ] Timestamp tokens verify successfully via `/api/tsa/verify`
- [ ] TL-DSA signatures are produced at `/api/pqti/tldsa/sign`
- [ ] Hash field lengths updated in SignHere database (49 bytes / 98 hex chars)
- [ ] Old SHA-256/SHA3-256 hashing code removed from SignHere pipeline

---

## 7. Error Handling

| HTTP Status | Meaning |
|---|---|
| 400 | Missing or invalid request body |
| 413 | Input exceeds 10 MB limit |
| 429 | Rate limit exceeded |
| 500 | Internal hashing error |

All error responses include `{ "success": false, "error": "<message>" }`.

---

## 8. Contact

For integration support, reach out to RSalvi@Salvigroup.com.
