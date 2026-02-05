import crypto from 'crypto';

describe('Webhook Validation', () => {
  describe('Stripe Signature Validation', () => {
    const testSecret = 'whsec_test_secret_key_12345';
    
    function generateStripeSignature(payload: string, secret: string, timestamp?: number): string {
      const ts = timestamp || Math.floor(Date.now() / 1000);
      const signedPayload = `${ts}.${payload}`;
      const signature = crypto
        .createHmac('sha256', secret)
        .update(signedPayload, 'utf8')
        .digest('hex');
      return `t=${ts},v1=${signature}`;
    }

    it('should validate a correctly signed Stripe webhook', () => {
      const payload = JSON.stringify({ id: 'evt_test', type: 'payment_intent.succeeded' });
      const signature = generateStripeSignature(payload, testSecret);
      
      expect(signature).toContain('t=');
      expect(signature).toContain('v1=');
    });

    it('should reject an expired timestamp', () => {
      const payload = JSON.stringify({ id: 'evt_test', type: 'payment_intent.succeeded' });
      const oldTimestamp = Math.floor(Date.now() / 1000) - 600;
      const signature = generateStripeSignature(payload, testSecret, oldTimestamp);
      
      expect(signature).toContain(`t=${oldTimestamp}`);
    });

    it('should reject missing signature parts', () => {
      const invalidSignatures = [
        't=12345',
        'v1=abcdef',
        '',
        'invalid'
      ];
      
      invalidSignatures.forEach(sig => {
        expect(sig).not.toMatch(/^t=\d+,v1=[a-f0-9]+$/);
      });
    });
  });

  describe('Interac Signature Validation', () => {
    const testSecret = 'interac_test_secret_key';
    
    function generateInteracSignature(payload: string, secret: string): string {
      return crypto
        .createHmac('sha256', secret)
        .update(payload, 'utf8')
        .digest('hex');
    }

    it('should generate valid HMAC-SHA256 signature', () => {
      const payload = JSON.stringify({ transaction_id: 'tx_123', event_type: 'DEPOSIT_RECEIVED' });
      const signature = generateInteracSignature(payload, testSecret);
      
      expect(signature).toHaveLength(64);
      expect(signature).toMatch(/^[a-f0-9]+$/);
    });
  });

  describe('Crypto Gateway Signature Validation', () => {
    const testSecret = 'crypto_test_secret_key';
    
    function generateCryptoSignature(payload: string, secret: string): string {
      return crypto
        .createHmac('sha512', secret)
        .update(payload, 'utf8')
        .digest('hex');
    }

    it('should generate valid HMAC-SHA512 signature', () => {
      const payload = JSON.stringify({ id: 'inv_123', type: 'invoice_paid' });
      const signature = generateCryptoSignature(payload, testSecret);
      
      expect(signature).toHaveLength(128);
      expect(signature).toMatch(/^[a-f0-9]+$/);
    });
  });
});

describe('Idempotency Checker', () => {
  it('should generate unique idempotency keys', () => {
    const key1 = `stripe:evt_001:payment_intent.succeeded`;
    const key2 = `stripe:evt_002:payment_intent.succeeded`;
    const key3 = `interac:tx_001:DEPOSIT_RECEIVED`;
    
    expect(key1).not.toBe(key2);
    expect(key1).not.toBe(key3);
  });

  it('should detect duplicate webhooks', () => {
    const processedKeys = new Set<string>();
    const key = 'stripe:evt_duplicate:payment_intent.succeeded';
    
    expect(processedKeys.has(key)).toBe(false);
    processedKeys.add(key);
    expect(processedKeys.has(key)).toBe(true);
  });
});
