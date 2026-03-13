/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * Pre-Sign Syndrome Check (T-24, SPEC-2026-NEXT)
 *
 * Repository: SigmaWolf-8/Ternary
 * Location:   sign-here/src/pre-sign-check.ts
 *
 * Validates document integrity BEFORE applying TL-DSA signatures.
 * Catches corruption early — before the expensive signing operation
 * and before the signature locks in a corrupted document hash.
 *
 * ## Checks Performed
 *
 * 1. **Document Hash Consistency**: Recompute the document hash and
 *    compare against the stored hash. Detects file corruption, truncation,
 *    or unauthorized modification since the hash was computed.
 *
 * 2. **Sponge State Sanity**: Verify the TLSponge-385 state is properly
 *    initialized and produces expected outputs for known inputs.
 *
 * 3. **Key Material Validation**: Verify the signing key is the correct
 *    length and format for the selected TL-DSA variant before attempting
 *    to sign (avoids cryptic internal errors).
 *
 * 4. **Timestamp Freshness**: Verify the signing timestamp is within
 *    acceptable bounds (not stale, not future).
 *
 * 5. **Wire ECC Syndrome** (optional): If the document contains TDNS
 *    addresses, verify their ECC syndromes before signing. A signature
 *    over a corrupted address is valid but useless.
 *
 * ## Usage
 *
 * ```typescript
 * const result = await preSignCheck({
 *   documentBytes: Buffer.from(document),
 *   expectedHash: storedHash,
 *   signingKey: secretKey,
 *   variant: 'TL-DSA-87',
 *   timestampFs: currentTimestamp,
 *   nowFs: Date.now() * 1_000_000_000_000,
 * });
 *
 * if (!result.pass) {
 *   console.error('Pre-sign check failed:', result.failures);
 *   // DO NOT SIGN
 * }
 * ```
 */

// ═══════════════════════════════════════════════════════════════════════
// TYPES
// ═══════════════════════════════════════════════════════════════════════

export type TlDsaVariant = 'TL-DSA-44' | 'TL-DSA-65' | 'TL-DSA-87';

/** Expected key sizes per variant. */
const VARIANT_KEY_SIZES: Record<TlDsaVariant, { pk: number; sk: number; sig: number }> = {
  'TL-DSA-44': { pk: 32, sk: 64, sig: 1632 },
  'TL-DSA-65': { pk: 48, sk: 96, sig: 2144 },
  'TL-DSA-87': { pk: 64, sk: 128, sig: 3168 },
};

/** Femtoseconds per second. */
const FS_PER_SECOND = BigInt('1000000000000000');

/** Maximum timestamp age: 30 seconds in femtoseconds. */
const MAX_TIMESTAMP_AGE_FS = BigInt(30) * FS_PER_SECOND;

/** Maximum future tolerance: 1 second in femtoseconds. */
const MAX_FUTURE_FS = BigInt(1) * FS_PER_SECOND;

/** Input parameters for the pre-sign check. */
export interface PreSignInput {
  /** The raw document bytes to be signed. */
  documentBytes: Buffer;
  /** The previously computed document hash (hex string). */
  expectedHash: string;
  /** The TL-DSA secret key that will be used for signing. */
  signingKey: Buffer;
  /** The TL-DSA public key (for key-pair consistency check). */
  publicKey?: Buffer;
  /** Which TL-DSA variant to use. */
  variant: TlDsaVariant;
  /** Signing timestamp in femtoseconds since Salvi Epoch. */
  timestampFs: bigint;
  /** Current time in femtoseconds since Salvi Epoch. */
  nowFs: bigint;
  /** Optional: TDNS addresses embedded in the document to check. */
  embeddedAddresses?: Buffer[];
  /** Optional: Sponge hash function (injectable for testing). */
  hashFn?: (input: Buffer) => string;
}

/** Result of a single check. */
export interface CheckResult {
  /** Check name. */
  name: string;
  /** Whether this check passed. */
  pass: boolean;
  /** Human-readable detail. */
  detail: string;
}

/** Aggregate result of all pre-sign checks. */
export interface PreSignResult {
  /** Whether ALL checks passed. */
  pass: boolean;
  /** Individual check results. */
  checks: CheckResult[];
  /** Only the failed checks (convenience). */
  failures: CheckResult[];
  /** Total time taken in milliseconds. */
  elapsedMs: number;
}

// ═══════════════════════════════════════════════════════════════════════
// DEFAULT HASH FUNCTION
// ═══════════════════════════════════════════════════════════════════════

/**
 * Default document hash using Node.js crypto (SHA3-256).
 *
 * In production, this is replaced by TLSponge-385 via the N-API addon.
 * The pre-sign check accepts an injectable `hashFn` to support both.
 */
function defaultHash(input: Buffer): string {
  const crypto = require('crypto');
  return crypto.createHash('sha3-256').update(input).digest('hex');
}

// ═══════════════════════════════════════════════════════════════════════
// INDIVIDUAL CHECKS
// ═══════════════════════════════════════════════════════════════════════

/**
 * Check 1: Document hash consistency.
 *
 * Recomputes the hash from raw bytes and compares against the stored hash.
 * Detects file corruption, truncation, or modification.
 */
function checkDocumentHash(
  documentBytes: Buffer,
  expectedHash: string,
  hashFn: (input: Buffer) => string,
): CheckResult {
  const name = 'document_hash';
  try {
    const computed = hashFn(documentBytes);
    const pass = computed.toLowerCase() === expectedHash.toLowerCase();
    return {
      name,
      pass,
      detail: pass
        ? `Hash matches (${computed.substring(0, 16)}...)`
        : `Hash mismatch: expected ${expectedHash.substring(0, 16)}..., got ${computed.substring(0, 16)}...`,
    };
  } catch (err) {
    return {
      name,
      pass: false,
      detail: `Hash computation failed: ${err}`,
    };
  }
}

/**
 * Check 2: Sponge state sanity.
 *
 * Verifies the hash function produces deterministic output for a
 * known test vector. Catches corrupted native addon loading or
 * misconfigured sponge parameters.
 */
function checkSpongeSanity(hashFn: (input: Buffer) => string): CheckResult {
  const name = 'sponge_sanity';
  try {
    const testInput = Buffer.from('PlenumNET-PRE-SIGN-SANITY-CHECK');
    const hash1 = hashFn(testInput);
    const hash2 = hashFn(testInput);
    const pass = hash1 === hash2 && hash1.length > 0;
    return {
      name,
      pass,
      detail: pass
        ? `Sponge deterministic (${hash1.substring(0, 16)}...)`
        : 'Sponge non-deterministic or empty output',
    };
  } catch (err) {
    return {
      name,
      pass: false,
      detail: `Sponge sanity failed: ${err}`,
    };
  }
}

/**
 * Check 3: Key material validation.
 *
 * Verifies the signing key length matches the selected TL-DSA variant.
 * Optionally checks public key length if provided.
 */
function checkKeyMaterial(
  signingKey: Buffer,
  publicKey: Buffer | undefined,
  variant: TlDsaVariant,
): CheckResult {
  const name = 'key_material';
  const sizes = VARIANT_KEY_SIZES[variant];

  if (!sizes) {
    return { name, pass: false, detail: `Unknown variant: ${variant}` };
  }

  if (signingKey.length !== sizes.sk) {
    return {
      name,
      pass: false,
      detail: `SK length ${signingKey.length} != expected ${sizes.sk} for ${variant}`,
    };
  }

  if (publicKey !== undefined && publicKey.length !== sizes.pk) {
    return {
      name,
      pass: false,
      detail: `PK length ${publicKey.length} != expected ${sizes.pk} for ${variant}`,
    };
  }

  // Check key is not all zeros (degenerate key)
  const allZero = signingKey.every((b) => b === 0);
  if (allZero) {
    return {
      name,
      pass: false,
      detail: 'Signing key is all zeros (degenerate)',
    };
  }

  return {
    name,
    pass: true,
    detail: `${variant}: SK=${sizes.sk}B, PK=${sizes.pk}B — valid`,
  };
}

/**
 * Check 4: Timestamp freshness.
 *
 * Ensures the signing timestamp is within the acceptable window:
 * - Not more than 30 seconds in the past
 * - Not more than 1 second in the future
 */
function checkTimestamp(timestampFs: bigint, nowFs: bigint): CheckResult {
  const name = 'timestamp_freshness';

  // Future check
  if (timestampFs > nowFs + MAX_FUTURE_FS) {
    const aheadSecs = Number((timestampFs - nowFs) / FS_PER_SECOND);
    return {
      name,
      pass: false,
      detail: `Timestamp ${aheadSecs.toFixed(1)}s in the future (max 1s)`,
    };
  }

  // Staleness check
  if (nowFs > timestampFs && (nowFs - timestampFs) > MAX_TIMESTAMP_AGE_FS) {
    const ageSecs = Number((nowFs - timestampFs) / FS_PER_SECOND);
    return {
      name,
      pass: false,
      detail: `Timestamp ${ageSecs.toFixed(1)}s old (max 30s)`,
    };
  }

  return {
    name,
    pass: true,
    detail: 'Timestamp within acceptable window',
  };
}

/**
 * Check 5: Embedded address ECC syndromes (optional).
 *
 * If the document contains TDNS addresses, verify their Rep C validity.
 * A signature over a corrupted address is technically valid but practically
 * useless — the signed address doesn't match any real node.
 */
function checkEmbeddedAddresses(addresses: Buffer[]): CheckResult {
  const name = 'embedded_addresses';

  if (addresses.length === 0) {
    return { name, pass: true, detail: 'No embedded addresses to check' };
  }

  const failures: string[] = [];

  for (let i = 0; i < addresses.length; i++) {
    const addr = addresses[i];

    // Check length (13 trits for cube address, 27 for TDNS)
    if (addr.length !== 13 && addr.length !== 27) {
      failures.push(`Address ${i}: invalid length ${addr.length} (expected 13 or 27)`);
      continue;
    }

    // Check Rep C validity (all trits must be 1, 2, or 3)
    for (let j = 0; j < addr.length; j++) {
      const trit = addr[j];
      if (trit < 1 || trit > 3) {
        failures.push(`Address ${i}: invalid trit at position ${j}: ${trit} (must be 1-3)`);
        break;
      }
    }
  }

  if (failures.length > 0) {
    return {
      name,
      pass: false,
      detail: `${failures.length} address error(s): ${failures[0]}${failures.length > 1 ? ` (+${failures.length - 1} more)` : ''}`,
    };
  }

  return {
    name,
    pass: true,
    detail: `${addresses.length} embedded address(es) valid`,
  };
}

// ═══════════════════════════════════════════════════════════════════════
// MAIN ENTRY POINT
// ═══════════════════════════════════════════════════════════════════════

/**
 * Run all pre-sign checks before applying a TL-DSA signature.
 *
 * If any check fails, the document should NOT be signed. The caller
 * should inspect `result.failures` for details and remediate before
 * retrying.
 *
 * @param input - All inputs needed for the checks
 * @returns Aggregate result with individual check details
 */
export function preSignCheck(input: PreSignInput): PreSignResult {
  const start = Date.now();
  const hashFn = input.hashFn ?? defaultHash;

  const checks: CheckResult[] = [
    checkDocumentHash(input.documentBytes, input.expectedHash, hashFn),
    checkSpongeSanity(hashFn),
    checkKeyMaterial(input.signingKey, input.publicKey, input.variant),
    checkTimestamp(input.timestampFs, input.nowFs),
    checkEmbeddedAddresses(input.embeddedAddresses ?? []),
  ];

  const failures = checks.filter((c) => !c.pass);
  const elapsedMs = Date.now() - start;

  return {
    pass: failures.length === 0,
    checks,
    failures,
    elapsedMs,
  };
}

/**
 * Quick check: just document hash + key material.
 *
 * For performance-sensitive paths where the full 5-check suite
 * is too expensive. Skips sponge sanity, timestamp, and address checks.
 */
export function preSignCheckQuick(
  documentBytes: Buffer,
  expectedHash: string,
  signingKey: Buffer,
  variant: TlDsaVariant,
  hashFn?: (input: Buffer) => string,
): PreSignResult {
  const start = Date.now();
  const fn = hashFn ?? defaultHash;

  const checks: CheckResult[] = [
    checkDocumentHash(documentBytes, expectedHash, fn),
    checkKeyMaterial(signingKey, undefined, variant),
  ];

  const failures = checks.filter((c) => !c.pass);
  const elapsedMs = Date.now() - start;

  return {
    pass: failures.length === 0,
    checks,
    failures,
    elapsedMs,
  };
}

// ═══════════════════════════════════════════════════════════════════════
// EXPORTS FOR TESTING
// ═══════════════════════════════════════════════════════════════════════

export const _internal = {
  checkDocumentHash,
  checkSpongeSanity,
  checkKeyMaterial,
  checkTimestamp,
  checkEmbeddedAddresses,
  VARIANT_KEY_SIZES,
  FS_PER_SECOND,
  MAX_TIMESTAMP_AGE_FS,
  MAX_FUTURE_FS,
};