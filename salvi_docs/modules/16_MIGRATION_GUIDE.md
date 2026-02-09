# Migration Guide: Lamport OTS to XMSS/LMS

**Purpose:** Guide for migrating from legacy Lamport One-Time Signatures to SP 800-208 compliant XMSS and LMS schemes.  
**Urgency:** High — Lamport OTS is deprecated and will be removed in a future release.

---

## Why Migrate?

### Lamport OTS Limitations

1. **Single-use keys**: Each Lamport key pair can sign exactly ONE message. A second use completely breaks security.
2. **Large signatures**: Lamport signatures are ~16 KB per signature (256 hash pairs).
3. **No standard**: Lamport OTS is not covered by any NIST standard or CNSA 2.0 requirement.
4. **No state tracking**: The legacy implementation has no built-in protection against key reuse.

### XMSS/LMS Advantages

1. **Multi-use**: XMSS-20 supports 1,048,576 signatures from a single key pair.
2. **Standardized**: Both are specified in NIST SP 800-208.
3. **CNSA 2.0 compliant**: Required by NSA for post-quantum signature operations.
4. **State tracking**: Built-in monotonic index advancement with `StateExhausted` error.
5. **Smaller signatures**: More compact per-signature overhead with authentication paths.

---

## Migration Steps

### Step 1: Choose XMSS or LMS

| Feature | XMSS | LMS |
|---------|------|-----|
| Standard | SP 800-208 (Section 5) | SP 800-208 (Section 4) |
| Tree structure | Merkle + L-tree | Merkle |
| OTS scheme | WOTS+ | LM-OTS |
| Parameter flexibility | Height: 10/16/20 | Height: 5/10/15/20/25, W: 1/2/4/8 |
| Best for | Long-lived signing keys | High-volume signing |
| Signature size | Medium | Configurable (trade size vs speed) |

**Recommendation:** Use XMSS for firmware signing and certificate authorities. Use LMS for high-volume document signing where parameter tuning (W value) matters.

### Step 2: Replace Key Generation

**Before (Lamport):**
```rust
use salvi_kernel::crypto::signature::{LamportKeypair, lamport_keygen};

let seed = vec![1i8, 0, -1, 1, 0, -1];
let keypair = lamport_keygen(&seed);
// WARNING: This key can only sign ONE message
```

**After (XMSS):**
```rust
use salvi_kernel::crypto::signature::{XmssKeypair, XmssParams};

let params = XmssParams::sha256_h16(); // 65,536 signatures
let seed = vec![1i8, 0, -1, 1, 0, -1];
let keypair = XmssKeypair::generate(&params, &seed);
// This key can sign 65,536 messages
```

**After (LMS):**
```rust
use salvi_kernel::crypto::signature::{LmsKeypair, LmsParams};

let params = LmsParams::sha256_h20_w4(); // 1,048,576 signatures
let seed = vec![1i8, 0, -1, 1, 0, -1];
let keypair = LmsKeypair::generate(&params, &seed);
```

### Step 3: Replace Signing

**Before (Lamport):**
```rust
let message = b"data to sign";
let signature = lamport_sign(message, &keypair.private_key);
// After this call, keypair.private_key MUST NEVER be used again
```

**After (XMSS):**
```rust
let message = b"data to sign";
let mut current_index: u32 = load_persisted_index(); // CRITICAL
let signature = keypair.sign(message, current_index);
current_index += 1;
persist_index(current_index); // CRITICAL: must persist before next sign
```

**After (LMS):**
```rust
let message = b"data to sign";
let mut current_index: u32 = load_persisted_index();
let signature = keypair.sign(message, current_index);
current_index += 1;
persist_index(current_index);
```

### Step 4: Replace Verification

**Before (Lamport):**
```rust
let valid = lamport_verify(message, &signature, &keypair.public_key);
```

**After (XMSS/LMS):**
```rust
let valid = keypair.verify(message, &signature);
// Verification is stateless — no index management needed
```

### Step 5: Add State Persistence

This is the most critical difference. XMSS and LMS are **stateful** — the signing index must advance monotonically and MUST be persisted.

```rust
// Recommended state persistence pattern
struct SigningState {
    keypair: XmssKeypair,  // or LmsKeypair
    current_index: u32,
    max_index: u32,
}

impl SigningState {
    fn sign(&mut self, message: &[u8]) -> Result<Signature, CryptoError> {
        if self.current_index >= self.max_index {
            return Err(CryptoError::StateExhausted);
        }
        let sig = self.keypair.sign(message, self.current_index);
        self.current_index += 1;
        self.persist()?;  // MUST persist before returning
        Ok(sig)
    }

    fn persist(&self) -> Result<(), IoError> {
        // Write current_index to durable storage
        // Use fsync/fdatasync to ensure durability
        // Consider write-ahead logging for crash safety
    }
}
```

**Failure mode:** If the process crashes after signing but before persisting the index, the same index could be reused on restart. This breaks security. Use write-ahead logging or atomic file operations.

---

## Choosing Parameters

### XMSS Height Selection

| Height | Total Signatures | Use Case |
|--------|-----------------|----------|
| 10 | 1,024 | Short-lived certificates, testing |
| 16 | 65,536 | Firmware signing (typical product lifecycle) |
| 20 | 1,048,576 | Root CAs, long-lived infrastructure |

### LMS Parameter Selection

| Height | W | Signatures | Sig Size | Speed | Use Case |
|--------|---|-----------|----------|-------|----------|
| 10 | 1 | 1,024 | Largest | Fastest | Low-latency signing |
| 15 | 4 | 32,768 | Medium | Medium | General purpose |
| 20 | 8 | 1,048,576 | Smallest | Slowest | Storage-constrained |
| 25 | 4 | 33M | Medium | Medium | Very high volume |

---

## Dual-Signing Transition Period

During migration, you may want to produce both Lamport and XMSS/LMS signatures to maintain backward compatibility:

```rust
struct DualSigner {
    legacy: LamportKeypair,
    modern: XmssKeypair,
    modern_index: u32,
}

impl DualSigner {
    fn dual_sign(&mut self, message: &[u8]) -> DualSignature {
        let legacy_sig = lamport_sign(message, &self.legacy.private_key);
        let modern_sig = self.modern.sign(message, self.modern_index);
        self.modern_index += 1;
        DualSignature { legacy: legacy_sig, modern: modern_sig }
    }
}
```

**Timeline:** Dual-signing should last no more than one key rotation cycle. After all verifiers support XMSS/LMS, remove Lamport entirely.

---

## Verification Compatibility

Verifiers should accept both old Lamport and new XMSS/LMS signatures during transition:

```rust
fn verify_any(message: &[u8], sig: &AnySignature) -> bool {
    match sig {
        AnySignature::Lamport(s) => lamport_verify(message, s, &lamport_pk),
        AnySignature::Xmss(s) => xmss_keypair.verify(message, s),
        AnySignature::Lms(s) => lms_keypair.verify(message, s),
    }
}
```

---

## Checklist

- [ ] Identify all Lamport OTS usage in codebase
- [ ] Choose XMSS or LMS (or both) per use case
- [ ] Select appropriate height and W parameters
- [ ] Implement state persistence for signing index
- [ ] Test crash recovery (index not lost on restart)
- [ ] Deploy dual-signing during transition
- [ ] Update all verifiers to accept XMSS/LMS
- [ ] Remove Lamport signing after transition complete
- [ ] Update algorithm agility policy to `CnsaOnly` mode
- [ ] Run CAVP vectors to validate implementation
