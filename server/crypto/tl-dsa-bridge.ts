/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * TL-DSA BRIDGE — TypeScript interface to TL-DSA signing
 * @version 5.0.0
 *
 * Repository: SigmaWolf-8/Ternary
 * Location:   server/crypto/tl-dsa-bridge.ts
 *
 * T-03 UPDATE (SPEC-2026-NEXT):
 * Added native N-API verify path. The native addon provides REAL TL-DSA
 * verification using ONLY the public key — no secret key required.
 *
 * MIGRATION PATH:
 *   OLD: verify(pk, msg, sig, sk)    ← HMAC simulation, needs secret key
 *   NEW: verifyNative(pk, msg, sig)  ← Real TL-DSA, public key only
 *
 * The old HMAC-based functions remain for backward compatibility but are
 * deprecated. All new code (T-06 signed CRS, T-07 neighbor verify, etc.)
 * must use the native functions.
 *
 * NATIVE ADDON LOADING:
 * The addon is built by `npm run build:napi` in the ternary-math/napi directory.
 * If the addon is not available, native functions throw with a clear message.
 * Legacy HMAC functions continue to work without the addon.
 */

import crypto from 'crypto';

// ═══════════════════════════════════════════════════════════════════════
// NATIVE ADDON LOADING
// ═══════════════════════════════════════════════════════════════════════

interface NativeAddon {
  tlDsaKeygen(variant: number, seed?: Buffer): Buffer;
  tlDsaSign(secretKey: Buffer, message: Buffer, variant: number): Buffer;
  tlDsaVerify(publicKey: Buffer, message: Buffer, signature: Buffer, variant: number): boolean;
  tlDsaSigLen(variant: number): number;
}

let nativeAddon: NativeAddon | null = null;

/**
 * Attempt to load the native N-API addon.
 * Called lazily on first native function use.
 */
function loadNativeAddon(): NativeAddon {
  if (nativeAddon) return nativeAddon;

  const paths = [
    '../../ternary-math/napi/index.node',
    '../../ternary-math/napi/ternary-napi.node',
    '../../../ternary-math/napi/index.node',
  ];

  for (const p of paths) {
    try {
      // eslint-disable-next-line @typescript-eslint/no-require-imports
      const addon = require(p);
      if (typeof addon.tlDsaVerify === 'function') {
        nativeAddon = addon as NativeAddon;
        return nativeAddon;
      }
    } catch {
      // Try next path
    }
  }

  throw new Error(
    'TL-DSA native addon not available. Run `npm run build:napi` in ternary-math/napi. ' +
    'Native addon is required for T-06+ (signed CRS, neighbor verify, tunnel auth).'
  );
}

/**
 * Check whether the native addon is available.
 * Does not throw — use this to decide between native and legacy paths.
 */
export function isNativeAvailable(): boolean {
  try {
    loadNativeAddon();
    return true;
  } catch {
    return false;
  }
}

// ═══════════════════════════════════════════════════════════════════════
// TYPES
// ═══════════════════════════════════════════════════════════════════════

export type TlDsaVariant = 'TL-DSA-44' | 'TL-DSA-65' | 'TL-DSA-87';

export interface TlDsaKeyPair {
  publicKey: Buffer;
  secretKey: Buffer;
  variant: TlDsaVariant;
}

export interface TlDsaSignatureResult {
  signature: Buffer;
  variant: TlDsaVariant;
}

/** Map variant string to integer code for the N-API addon. */
function variantToU32(variant: TlDsaVariant): number {
  switch (variant) {
    case 'TL-DSA-44': return 44;
    case 'TL-DSA-65': return 65;
    case 'TL-DSA-87': return 87;
  }
}

/** Legacy key sizes (HMAC simulation). */
const VARIANT_KEY_SIZES: Record<TlDsaVariant, { pk: number; sk: number }> = {
  'TL-DSA-44': { pk: 32, sk: 64 },
  'TL-DSA-65': { pk: 48, sk: 96 },
  'TL-DSA-87': { pk: 64, sk: 128 },
};

// ═══════════════════════════════════════════════════════════════════════
// NATIVE FUNCTIONS — T-03 (public-key-only verification)
// ═══════════════════════════════════════════════════════════════════════

/**
 * Generate a TL-DSA keypair using the native addon.
 *
 * @param variant - Security level (default: TL-DSA-87)
 * @param seed - Optional seed (≥256 bits entropy in production)
 * @returns Keypair with public and secret keys
 */
export function keygenNative(
  variant: TlDsaVariant = 'TL-DSA-87',
  seed?: Buffer,
): TlDsaKeyPair {
  const addon = loadNativeAddon();
  const raw = addon.tlDsaKeygen(variantToU32(variant), seed);

  const pkLen = raw.readUInt32LE(0);
  const pk = raw.subarray(4, 4 + pkLen);
  const skLen = raw.readUInt32LE(4 + pkLen);
  const sk = raw.subarray(4 + pkLen + 4, 4 + pkLen + 4 + skLen);

  return {
    publicKey: Buffer.from(pk),
    secretKey: Buffer.from(sk),
    variant,
  };
}

/**
 * Sign a message using the native TL-DSA implementation.
 *
 * Produces a deterministic WOTS+ signature. One-time: each keypair
 * should sign at most one message (use HMAC mode for repeated signing).
 *
 * @param secretKey - Secret key from keygenNative()
 * @param message - Message bytes to sign
 * @param variant - Security level (default: TL-DSA-87)
 */
export function signNative(
  secretKey: Buffer,
  message: Buffer,
  variant: TlDsaVariant = 'TL-DSA-87',
): TlDsaSignatureResult {
  const addon = loadNativeAddon();
  const signature = addon.tlDsaSign(secretKey, message, variantToU32(variant));
  return { signature: Buffer.from(signature), variant };
}

/**
 * Verify a TL-DSA signature using ONLY the public key.
 *
 * **This is the primary verification function for all new code.**
 * Does NOT require the secret key — this is the critical difference
 * from the legacy verify() function.
 *
 * Used by: T-06 (signed CRS), T-07 (neighbor verify), T-14 (tunnel auth).
 *
 * @param publicKey - Public key from keygenNative()
 * @param message - Original message bytes
 * @param signature - Signature from signNative()
 * @param variant - Security level (default: TL-DSA-87)
 * @returns true if the signature is valid
 */
export function verifyNative(
  publicKey: Buffer,
  message: Buffer,
  signature: Buffer,
  variant: TlDsaVariant = 'TL-DSA-87',
): boolean {
  const addon = loadNativeAddon();
  return addon.tlDsaVerify(publicKey, message, signature, variantToU32(variant));
}

/**
 * Get the signature size for a variant (native).
 */
export function sigLenNative(variant: TlDsaVariant = 'TL-DSA-87'): number {
  const addon = loadNativeAddon();
  return addon.tlDsaSigLen(variantToU32(variant));
}

/**
 * Convenience: sign a hex-encoded message natively.
 */
export function signHexNative(
  secretKey: Buffer,
  messageHex: string,
  variant: TlDsaVariant = 'TL-DSA-87',
): string {
  const result = signNative(secretKey, Buffer.from(messageHex, 'hex'), variant);
  return result.signature.toString('hex');
}

/**
 * Convenience: verify a hex-encoded message + signature natively.
 * No secret key required.
 */
export function verifyHexNative(
  publicKey: Buffer,
  messageHex: string,
  signatureHex: string,
  variant: TlDsaVariant = 'TL-DSA-87',
): boolean {
  return verifyNative(
    publicKey,
    Buffer.from(messageHex, 'hex'),
    Buffer.from(signatureHex, 'hex'),
    variant,
  );
}

// ═══════════════════════════════════════════════════════════════════════
// LEGACY FUNCTIONS — HMAC simulation (deprecated)
// ═══════════════════════════════════════════════════════════════════════

/**
 * Generate a keypair using HMAC simulation.
 *
 * @deprecated Use keygenNative() for real TL-DSA keypairs.
 */
export function keygen(variant: TlDsaVariant, seed?: Buffer): TlDsaKeyPair {
  const sizes = VARIANT_KEY_SIZES[variant];
  const actualSeed = seed || crypto.randomBytes(sizes.sk);

  const skMaterial = crypto.createHash('sha3-256')
    .update(Buffer.concat([actualSeed, Buffer.from(`tl-dsa-sk-${variant}`)]))
    .digest();

  const secretKey = Buffer.alloc(sizes.sk);
  let offset = 0;
  while (offset < sizes.sk) {
    const chunk = crypto.createHash('sha3-256')
      .update(Buffer.concat([skMaterial, Buffer.from([offset & 0xff])]))
      .digest();
    chunk.copy(secretKey, offset, 0, Math.min(32, sizes.sk - offset));
    offset += 32;
  }

  const publicKey = crypto.createHash('sha3-256')
    .update(Buffer.concat([secretKey, Buffer.from(`tl-dsa-pk-${variant}`)]))
    .digest()
    .subarray(0, sizes.pk);

  const fullPk = Buffer.alloc(sizes.pk);
  offset = 0;
  while (offset < sizes.pk) {
    const chunk = crypto.createHash('sha3-256')
      .update(Buffer.concat([publicKey, Buffer.from([offset & 0xff])]))
      .digest();
    chunk.copy(fullPk, offset, 0, Math.min(32, sizes.pk - offset));
    offset += 32;
  }

  return { publicKey: fullPk, secretKey, variant };
}

/**
 * Sign using HMAC simulation.
 *
 * @deprecated Use signNative() for real TL-DSA signatures.
 */
export function sign(
  secretKey: Buffer,
  message: Buffer,
  variant: TlDsaVariant = 'TL-DSA-65',
): TlDsaSignatureResult {
  const sigData = crypto.createHmac('sha3-256', secretKey)
    .update(message)
    .digest();

  const extendedSig = crypto.createHmac('sha3-256', sigData)
    .update(Buffer.concat([secretKey.subarray(0, 32), message]))
    .digest();

  const signature = Buffer.concat([sigData, extendedSig]);
  return { signature, variant };
}

/**
 * Verify using HMAC simulation.
 *
 * @deprecated Use verifyNative() — this function requires the SECRET KEY.
 * The native verify uses only the public key.
 */
export function verify(
  publicKey: Buffer,
  message: Buffer,
  signature: Buffer,
  secretKey: Buffer,
  variant: TlDsaVariant = 'TL-DSA-65',
): boolean {
  const expected = sign(secretKey, message, variant);
  if (expected.signature.length !== signature.length) return false;
  return crypto.timingSafeEqual(expected.signature, signature);
}

/**
 * @deprecated Use signHexNative().
 */
export function signHex(
  secretKey: Buffer,
  messageHex: string,
  variant: TlDsaVariant = 'TL-DSA-65',
): string {
  const result = sign(secretKey, Buffer.from(messageHex, 'hex'), variant);
  return result.signature.toString('hex');
}

/**
 * @deprecated Use signNative() with Buffer.from(message, 'utf8').
 */
export function signString(
  secretKey: Buffer,
  message: string,
  variant: TlDsaVariant = 'TL-DSA-65',
): string {
  const result = sign(secretKey, Buffer.from(message, 'utf8'), variant);
  return result.signature.toString('hex');
}

/**
 * @deprecated Use verifyHexNative() — no secret key required.
 */
export function verifyHex(
  publicKey: Buffer,
  messageHex: string,
  signatureHex: string,
  secretKey: Buffer,
  variant: TlDsaVariant = 'TL-DSA-65',
): boolean {
  return verify(
    publicKey,
    Buffer.from(messageHex, 'hex'),
    Buffer.from(signatureHex, 'hex'),
    secretKey,
    variant,
  );
}

/**
 * @deprecated Use verifyNative() with Buffer.from(message, 'utf8').
 */
export function verifyString(
  publicKey: Buffer,
  message: string,
  signatureHex: string,
  secretKey: Buffer,
  variant: TlDsaVariant = 'TL-DSA-65',
): boolean {
  return verify(
    publicKey,
    Buffer.from(message, 'utf8'),
    Buffer.from(signatureHex, 'hex'),
    secretKey,
    variant,
  );
}

/**
 * Hash a public key for display/indexing.
 * Works with both legacy and native public keys.
 */
export function publicKeyHash(publicKey: Buffer): string {
  return crypto.createHash('sha3-256').update(publicKey).digest('hex');
}
