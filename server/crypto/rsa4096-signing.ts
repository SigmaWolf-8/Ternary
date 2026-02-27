/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * RSA-4096 SIGNING MODULE
 * @version 4.0.0
 *
 * Repository: SigmaWolf-8/Ternary
 * Location:   server/crypto/rsa4096-signing.ts
 *
 * Real RSA-4096 signing using Node.js native crypto module.
 * Provides backward-compatible verification for courts and systems
 * that do not yet support post-quantum algorithms.
 * Used alongside TL-DSA for dual-signature capability certificates.
 */

import crypto from 'crypto';

export interface RSA4096KeyPair {
  publicKey: string;
  privateKey: string;
}

export function generateRSA4096KeyPair(): RSA4096KeyPair {
  const { publicKey, privateKey } = crypto.generateKeyPairSync('rsa', {
    modulusLength: 4096,
    publicKeyEncoding: { type: 'spki', format: 'pem' },
    privateKeyEncoding: { type: 'pkcs8', format: 'pem' },
  });
  return { publicKey: publicKey as unknown as string, privateKey: privateKey as unknown as string };
}

export function signRSA4096(privateKey: string, data: Buffer): Buffer {
  return crypto.sign('sha256', data, privateKey);
}

export function verifyRSA4096(publicKey: string, data: Buffer, signature: Buffer): boolean {
  return crypto.verify('sha256', data, publicKey, signature);
}
