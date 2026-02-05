import crypto from 'crypto';

export interface SecretsConfig {
  encryptionKey: Buffer;
  encryptionAlgorithm: string;
  ivLength: number;
}

const ENCRYPTION_KEY = process.env.SECRETS_ENCRYPTION_KEY || crypto.randomBytes(32).toString('hex');

export const secretsConfig: SecretsConfig = {
  encryptionKey: Buffer.from(ENCRYPTION_KEY, 'hex'),
  encryptionAlgorithm: 'aes-256-gcm',
  ivLength: 16
};

export function encryptSecret(plaintext: string): string {
  const iv = crypto.randomBytes(secretsConfig.ivLength);
  const cipher = crypto.createCipheriv(
    secretsConfig.encryptionAlgorithm,
    secretsConfig.encryptionKey,
    iv
  ) as crypto.CipherGCM;
  
  let encrypted = cipher.update(plaintext, 'utf8', 'hex');
  encrypted += cipher.final('hex');
  
  const authTag = cipher.getAuthTag();
  
  return `${iv.toString('hex')}:${authTag.toString('hex')}:${encrypted}`;
}

export function decryptSecret(ciphertext: string): string {
  const [ivHex, authTagHex, encrypted] = ciphertext.split(':');
  
  if (!ivHex || !authTagHex || !encrypted) {
    throw new Error('Invalid ciphertext format');
  }
  
  const iv = Buffer.from(ivHex, 'hex');
  const authTag = Buffer.from(authTagHex, 'hex');
  
  const decipher = crypto.createDecipheriv(
    secretsConfig.encryptionAlgorithm,
    secretsConfig.encryptionKey,
    iv
  ) as crypto.DecipherGCM;
  
  decipher.setAuthTag(authTag);
  
  let decrypted = decipher.update(encrypted, 'hex', 'utf8');
  decrypted += decipher.final('utf8');
  
  return decrypted;
}

export function rotateEncryptionKey(newKey: string, encryptedSecrets: string[]): string[] {
  const decrypted = encryptedSecrets.map(decryptSecret);
  
  secretsConfig.encryptionKey = Buffer.from(newKey, 'hex');
  
  return decrypted.map(encryptSecret);
}
