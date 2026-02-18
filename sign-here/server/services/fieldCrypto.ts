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
const FIELD_PREFIX = "fenc:";
const HKDF_INFO = "signhere-cnsa2-field-encryption";

function deriveKey(salt: Buffer): Buffer {
  const secret = process.env.SESSION_SECRET;
  if (!secret) {
    throw new Error("SESSION_SECRET required for field encryption");
  }
  return Buffer.from(crypto.hkdfSync("sha512", Buffer.from(secret, "utf8"), salt, Buffer.from(HKDF_INFO, "utf8"), 32));
}

export function encryptField(plaintext: string): string {
  if (!plaintext || plaintext.startsWith(FIELD_PREFIX)) return plaintext;
  const salt = crypto.randomBytes(SALT_LENGTH);
  const key = deriveKey(salt);
  const iv = crypto.randomBytes(IV_LENGTH);
  const cipher = crypto.createCipheriv(ALGORITHM, key, iv);
  const encrypted = Buffer.concat([cipher.update(plaintext, "utf8"), cipher.final()]);
  const authTag = cipher.getAuthTag();
  const combined = Buffer.concat([salt, iv, authTag, encrypted]);
  return FIELD_PREFIX + combined.toString("base64");
}

export function decryptField(stored: string): string {
  if (!stored || !stored.startsWith(FIELD_PREFIX)) return stored;
  try {
    const combined = Buffer.from(stored.slice(FIELD_PREFIX.length), "base64");
    const salt = combined.subarray(0, SALT_LENGTH);
    const iv = combined.subarray(SALT_LENGTH, SALT_LENGTH + IV_LENGTH);
    const authTag = combined.subarray(SALT_LENGTH + IV_LENGTH, SALT_LENGTH + IV_LENGTH + AUTH_TAG_LENGTH);
    const encrypted = combined.subarray(SALT_LENGTH + IV_LENGTH + AUTH_TAG_LENGTH);
    const key = deriveKey(salt);
    const decipher = crypto.createDecipheriv(ALGORITHM, key, iv);
    decipher.setAuthTag(authTag);
    return Buffer.concat([decipher.update(encrypted), decipher.final()]).toString("utf8");
  } catch {
    return stored;
  }
}

export function isFieldEncrypted(data: string): boolean {
  return !!data && data.startsWith(FIELD_PREFIX);
}

export function encryptJson(obj: any): string | null {
  if (obj === null || obj === undefined) return null;
  return encryptField(JSON.stringify(obj));
}

export function decryptJson(stored: string | null): any {
  if (!stored) return null;
  const decrypted = decryptField(stored);
  try {
    return JSON.parse(decrypted);
  } catch {
    return stored;
  }
}
