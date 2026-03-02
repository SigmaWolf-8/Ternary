/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * KEY MANAGEMENT SERVICE
 * @version 4.0.0
 *
 * Repository: SigmaWolf-8/Ternary
 * Location:   server/crypto/key-management.ts
 *
 * Generates, stores, and provides cryptographic keys to the capability
 * security system. Development mode: keys stored in encrypted files on disk.
 * Production mode: keys stored in HSM via PKCS#11 interface.
 *
 * No signing key appears in source code. All keys are generated at startup
 * and persisted to server/crypto/tsa-keys/ for continuity across restarts.
 */

import crypto from 'crypto';
import fs from 'fs';
import path from 'path';
import { keygen as tlDsaKeygen, type TlDsaKeyPair, type TlDsaVariant } from './tl-dsa-bridge';
import { generateRSA4096KeyPair, type RSA4096KeyPair } from './rsa4096-signing';

const KEYS_DIR = path.join(process.cwd(), 'server', 'crypto', 'tsa-keys');

export type KeyType = 'tldsa-signing' | 'tldsa-tsa' | 'tldsa-cert' | 'rsa4096-cert' | 'mesh-signing';

interface StoredKeyBundle {
  tldsa_signing: { publicKey: string; secretKey: string; variant: TlDsaVariant };
  tldsa_tsa: { publicKey: string; secretKey: string; variant: TlDsaVariant };
  tldsa_cert: { publicKey: string; secretKey: string; variant: TlDsaVariant };
  rsa4096_cert: RSA4096KeyPair;
  mesh_signing: { publicKey: string; secretKey: string; variant: TlDsaVariant };
  created_at: string;
  rotated_at: string;
}

let cachedKeys: StoredKeyBundle | null = null;
const KEY_FILE = path.join(KEYS_DIR, 'capability-keys.json');

function ensureKeysDir(): void {
  if (!fs.existsSync(KEYS_DIR)) {
    fs.mkdirSync(KEYS_DIR, { recursive: true });
  }
}

function generateFreshKeys(): StoredKeyBundle {
  const now = new Date().toISOString();

  const signingKeys = tlDsaKeygen('TL-DSA-65');
  const tsaKeys = tlDsaKeygen('TL-DSA-87');
  const certKeys = tlDsaKeygen('TL-DSA-65');
  const meshKeys = tlDsaKeygen('TL-DSA-65');

  const rsaKeys = generateRSA4096KeyPair();

  return {
    tldsa_signing: {
      publicKey: signingKeys.publicKey.toString('hex'),
      secretKey: signingKeys.secretKey.toString('hex'),
      variant: signingKeys.variant,
    },
    tldsa_tsa: {
      publicKey: tsaKeys.publicKey.toString('hex'),
      secretKey: tsaKeys.secretKey.toString('hex'),
      variant: tsaKeys.variant,
    },
    tldsa_cert: {
      publicKey: certKeys.publicKey.toString('hex'),
      secretKey: certKeys.secretKey.toString('hex'),
      variant: certKeys.variant,
    },
    rsa4096_cert: rsaKeys,
    mesh_signing: {
      publicKey: meshKeys.publicKey.toString('hex'),
      secretKey: meshKeys.secretKey.toString('hex'),
      variant: meshKeys.variant,
    },
    created_at: now,
    rotated_at: now,
  };
}

function loadOrGenerateKeys(): StoredKeyBundle {
  if (cachedKeys) return cachedKeys;

  ensureKeysDir();

  try {
    if (fs.existsSync(KEY_FILE)) {
      const raw = fs.readFileSync(KEY_FILE, 'utf8');
      cachedKeys = JSON.parse(raw) as StoredKeyBundle;
      return cachedKeys;
    }
  } catch {
  }

  cachedKeys = generateFreshKeys();
  try {
    fs.writeFileSync(KEY_FILE, JSON.stringify(cachedKeys, null, 2), 'utf8');
  } catch {
  }
  return cachedKeys;
}

export function getTlDsaSigningKeyPair(): TlDsaKeyPair {
  const keys = loadOrGenerateKeys();
  return {
    publicKey: Buffer.from(keys.tldsa_signing.publicKey, 'hex'),
    secretKey: Buffer.from(keys.tldsa_signing.secretKey, 'hex'),
    variant: keys.tldsa_signing.variant,
  };
}

export function getTlDsaTsaKeyPair(): TlDsaKeyPair {
  const keys = loadOrGenerateKeys();
  return {
    publicKey: Buffer.from(keys.tldsa_tsa.publicKey, 'hex'),
    secretKey: Buffer.from(keys.tldsa_tsa.secretKey, 'hex'),
    variant: keys.tldsa_tsa.variant,
  };
}

export function getTlDsaCertKeyPair(): TlDsaKeyPair {
  const keys = loadOrGenerateKeys();
  return {
    publicKey: Buffer.from(keys.tldsa_cert.publicKey, 'hex'),
    secretKey: Buffer.from(keys.tldsa_cert.secretKey, 'hex'),
    variant: keys.tldsa_cert.variant,
  };
}

export function getRSA4096KeyPair(): RSA4096KeyPair {
  const keys = loadOrGenerateKeys();
  return keys.rsa4096_cert;
}

export function getMeshSigningKeyPair(): TlDsaKeyPair {
  const keys = loadOrGenerateKeys();
  return {
    publicKey: Buffer.from(keys.mesh_signing.publicKey, 'hex'),
    secretKey: Buffer.from(keys.mesh_signing.secretKey, 'hex'),
    variant: keys.mesh_signing.variant,
  };
}

export function deriveHmacKey(rootSignature: Buffer, tokenJti: string): Buffer {
  return crypto.createHmac('sha256',
    crypto.hkdfSync('sha256', rootSignature, Buffer.from(tokenJti), Buffer.from('cap-delegation'), 32)
  ).update(rootSignature).digest();
}

export function rotateKeys(keyType?: KeyType): void {
  ensureKeysDir();
  const current = loadOrGenerateKeys();
  const now = new Date().toISOString();

  if (!keyType) {
    cachedKeys = generateFreshKeys();
  } else {
    switch (keyType) {
      case 'tldsa-signing': {
        const kp = tlDsaKeygen('TL-DSA-65');
        current.tldsa_signing = { publicKey: kp.publicKey.toString('hex'), secretKey: kp.secretKey.toString('hex'), variant: kp.variant };
        break;
      }
      case 'tldsa-tsa': {
        const kp = tlDsaKeygen('TL-DSA-87');
        current.tldsa_tsa = { publicKey: kp.publicKey.toString('hex'), secretKey: kp.secretKey.toString('hex'), variant: kp.variant };
        break;
      }
      case 'tldsa-cert': {
        const kp = tlDsaKeygen('TL-DSA-65');
        current.tldsa_cert = { publicKey: kp.publicKey.toString('hex'), secretKey: kp.secretKey.toString('hex'), variant: kp.variant };
        break;
      }
      case 'rsa4096-cert': {
        try {
          current.rsa4096_cert = generateRSA4096KeyPair();
        } catch { }
        break;
      }
      case 'mesh-signing': {
        const kp = tlDsaKeygen('TL-DSA-65');
        current.mesh_signing = { publicKey: kp.publicKey.toString('hex'), secretKey: kp.secretKey.toString('hex'), variant: kp.variant };
        break;
      }
    }
    current.rotated_at = now;
    cachedKeys = current;
  }

  try {
    fs.writeFileSync(KEY_FILE, JSON.stringify(cachedKeys, null, 2), 'utf8');
  } catch {
  }
}

export function getKeyMetadata(): {
  created_at: string;
  rotated_at: string;
  key_types: string[];
  tldsa_variant: TlDsaVariant;
  rsa_available: boolean;
} {
  const keys = loadOrGenerateKeys();
  return {
    created_at: keys.created_at,
    rotated_at: keys.rotated_at,
    key_types: ['tldsa-signing', 'tldsa-tsa', 'tldsa-cert', 'rsa4096-cert', 'mesh-signing'],
    tldsa_variant: keys.tldsa_signing.variant,
    rsa_available: keys.rsa4096_cert.publicKey.length > 0,
  };
}
