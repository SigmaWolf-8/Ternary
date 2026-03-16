/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * PT26-DSA BRIDGE — TypeScript interface to PT26-DSA Hypercube Walk Signatures
 * @version 1.0.0
 *
 * Repository: SigmaWolf-8/Ternary
 * Location:   server/crypto/pt26-bridge.ts
 *
 * All operations use the native Rust N-API addon (sponge-native.node).
 * PT-26 keygen/sign/verify run entirely in the Rust kernel crate.
 */

import { createRequire as _createRequire } from 'module';
import { fileURLToPath as _fileURLToPath } from 'url';
import { dirname as _dirname, resolve as _resolve } from 'path';

interface NativeAddon {
  pt26Keygen(address: Buffer, secret: Buffer): { publicKey: Buffer; secretKey: Buffer; address: number[] };
  pt26Sign(secretKey: Buffer, message: Buffer): Buffer;
  pt26Verify(publicKey: Buffer, message: Buffer, signature: Buffer): boolean;
  pt26PublicKeySize(): number;
  pt26SignatureSize(): number;
}

function _getRequire(): NodeRequire {
  if (typeof require !== 'undefined') return require;
  return _createRequire(import.meta.url);
}

function _resolveAddonPath(filename: string): string {
  if (typeof __filename !== 'undefined') {
    return _resolve(_dirname(__filename), filename);
  } else if (typeof import.meta?.url !== 'undefined') {
    const _f = _fileURLToPath(import.meta.url);
    return _resolve(_dirname(_f), filename);
  }
  return _resolve(process.cwd(), 'server/crypto', filename);
}

let nativeAddon: NativeAddon | null = null;

function loadNativeAddon(): NativeAddon {
  if (nativeAddon) return nativeAddon;

  const _require = _getRequire();
  const paths = [
    _resolveAddonPath('sponge-native.node'),
    _resolve(process.cwd(), 'ternary-math/napi/index.node'),
    _resolve(process.cwd(), 'server/crypto/sponge-native.node'),
  ];

  for (const p of paths) {
    try {
      const addon = _require(p);
      if (typeof addon.pt26Keygen === 'function') {
        nativeAddon = addon as NativeAddon;
        return nativeAddon;
      }
    } catch {
      // Try next path
    }
  }

  throw new Error(
    'PT-26 native addon not available. Run `cargo build --release` in ternary-math/napi. ' +
    'The addon must export pt26Keygen, pt26Sign, pt26Verify.'
  );
}

export function isNativeAvailable(): boolean {
  try { loadNativeAddon(); return true; } catch { return false; }
}

export interface Pt26KeyPair {
  publicKey: Buffer;
  secretKey: Buffer;
  address: number[];
}

export function keygen(address: Buffer, secret: Buffer): Pt26KeyPair {
  const addon = loadNativeAddon();
  const result = addon.pt26Keygen(address, secret);
  return {
    publicKey: Buffer.from(result.publicKey),
    secretKey: Buffer.from(result.secretKey),
    address: result.address,
  };
}

export function sign(secretKey: Buffer, message: Buffer): Buffer {
  const addon = loadNativeAddon();
  return Buffer.from(addon.pt26Sign(secretKey, message));
}

export function verify(publicKey: Buffer, message: Buffer, signature: Buffer): boolean {
  const addon = loadNativeAddon();
  return addon.pt26Verify(publicKey, message, signature);
}

export function publicKeySize(): number {
  const addon = loadNativeAddon();
  return addon.pt26PublicKeySize();
}

export function signatureSize(): number {
  const addon = loadNativeAddon();
  return addon.pt26SignatureSize();
}
