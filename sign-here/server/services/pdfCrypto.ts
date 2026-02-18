/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
 * Patent(s) Pending.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * Unauthorized copying, modification, distribution, or use of this file,
 * via any medium, is strictly prohibited without the prior written
 * permission of Capomastro Holdings Ltd.
 *
 * See LICENSE in the repository root for full terms.
 */
import crypto from "crypto";

const ALGORITHM = "aes-256-gcm";
const IV_LENGTH = 16;
const AUTH_TAG_LENGTH = 16;
const SALT_LENGTH = 32;
const ENCRYPTED_PREFIX = "enc:";
const HKDF_PREFIX = "hkdf:";
const HKDF_INFO = "signhere-cnsa2-document-encryption";

function deriveKeyHKDF(salt: Buffer): Buffer {
  const secret = process.env.SESSION_SECRET;
  if (!secret) {
    throw new Error("SESSION_SECRET environment variable is required for document encryption");
  }
  const ikm = Buffer.from(secret, "utf8");
  const info = Buffer.from(HKDF_INFO, "utf8");
  return Buffer.from(crypto.hkdfSync("sha512", ikm, salt, info, 32));
}

function getLegacyKey(): Buffer {
  const secret = process.env.SESSION_SECRET;
  if (!secret) {
    throw new Error("SESSION_SECRET environment variable is required for document encryption");
  }
  return crypto.createHash("sha256").update(secret).digest();
}

export function encryptPdf(base64Data: string): string {
  const salt = crypto.randomBytes(SALT_LENGTH);
  const key = deriveKeyHKDF(salt);
  const iv = crypto.randomBytes(IV_LENGTH);
  const cipher = crypto.createCipheriv(ALGORITHM, key, iv);

  const encrypted = Buffer.concat([
    cipher.update(base64Data, "utf8"),
    cipher.final(),
  ]);

  const authTag = cipher.getAuthTag();

  const combined = Buffer.concat([salt, iv, authTag, encrypted]);
  return HKDF_PREFIX + combined.toString("base64");
}

export function decryptPdf(storedData: string): string {
  if (storedData.startsWith(HKDF_PREFIX)) {
    const combined = Buffer.from(storedData.slice(HKDF_PREFIX.length), "base64");

    const salt = combined.subarray(0, SALT_LENGTH);
    const iv = combined.subarray(SALT_LENGTH, SALT_LENGTH + IV_LENGTH);
    const authTag = combined.subarray(SALT_LENGTH + IV_LENGTH, SALT_LENGTH + IV_LENGTH + AUTH_TAG_LENGTH);
    const encrypted = combined.subarray(SALT_LENGTH + IV_LENGTH + AUTH_TAG_LENGTH);

    const key = deriveKeyHKDF(salt);
    const decipher = crypto.createDecipheriv(ALGORITHM, key, iv);
    decipher.setAuthTag(authTag);

    const decrypted = Buffer.concat([
      decipher.update(encrypted),
      decipher.final(),
    ]);

    return decrypted.toString("utf8");
  }

  if (storedData.startsWith(ENCRYPTED_PREFIX)) {
    const key = getLegacyKey();
    const combined = Buffer.from(storedData.slice(ENCRYPTED_PREFIX.length), "base64");

    const iv = combined.subarray(0, IV_LENGTH);
    const authTag = combined.subarray(IV_LENGTH, IV_LENGTH + AUTH_TAG_LENGTH);
    const encrypted = combined.subarray(IV_LENGTH + AUTH_TAG_LENGTH);

    const decipher = crypto.createDecipheriv(ALGORITHM, key, iv);
    decipher.setAuthTag(authTag);

    const decrypted = Buffer.concat([
      decipher.update(encrypted),
      decipher.final(),
    ]);

    return decrypted.toString("utf8");
  }

  return storedData;
}

export function isEncrypted(data: string): boolean {
  return data.startsWith(HKDF_PREFIX) || data.startsWith(ENCRYPTED_PREFIX);
}
