import crypto from "crypto";

const ALGORITHM = "aes-256-gcm";
const IV_LENGTH = 16;
const AUTH_TAG_LENGTH = 16;
const ENCRYPTED_PREFIX = "enc:";

function getEncryptionKey(): Buffer {
  const secret = process.env.SESSION_SECRET;
  if (!secret) {
    throw new Error("SESSION_SECRET environment variable is required for PDF encryption");
  }
  return crypto.createHash("sha256").update(secret).digest();
}

export function encryptPdf(base64Data: string): string {
  const key = getEncryptionKey();
  const iv = crypto.randomBytes(IV_LENGTH);
  const cipher = crypto.createCipheriv(ALGORITHM, key, iv);

  const encrypted = Buffer.concat([
    cipher.update(base64Data, "utf8"),
    cipher.final(),
  ]);

  const authTag = cipher.getAuthTag();

  const combined = Buffer.concat([iv, authTag, encrypted]);
  return ENCRYPTED_PREFIX + combined.toString("base64");
}

export function decryptPdf(storedData: string): string {
  if (!storedData.startsWith(ENCRYPTED_PREFIX)) {
    return storedData;
  }

  const key = getEncryptionKey();
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

export function isEncrypted(data: string): boolean {
  return data.startsWith(ENCRYPTED_PREFIX);
}
