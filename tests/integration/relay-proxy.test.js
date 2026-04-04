const http = require('http');

const BASE_URL = process.env.TEST_BASE_URL || 'http://localhost:5000';

function request(path, options = {}) {
  return new Promise((resolve, reject) => {
    const url = new URL(path, BASE_URL);
    const req = http.get(
      url.toString(),
      { headers: options.headers || {} },
      (res) => {
        let data = '';
        res.on('data', (chunk) => (data += chunk));
        res.on('end', () => {
          let parsed;
          try { parsed = JSON.parse(data); } catch { parsed = data; }
          resolve({ status: res.statusCode || 0, body: parsed });
        });
      }
    );
    req.on('error', reject);
    req.setTimeout(10000, () => { req.destroy(); reject(new Error('request timeout')); });
  });
}

function getValidToken() {
  return process.env.RELAY_API_TOKEN || null;
}

describe('Relay HTTP Proxy — /api/salvi/inter-cube/slots', () => {

  test('401 for missing Authorization header', async () => {
    const res = await request('/api/salvi/inter-cube/slots');
    expect(res.status).toBe(401);
    expect(res.body).toEqual({
      error: 'Invalid or missing relay token',
      hint: "Verify your RELAY_AUTH_TOKEN matches the server's RELAY_API_TOKEN.",
    });
  });

  test('401 for wrong Bearer token', async () => {
    const res = await request('/api/salvi/inter-cube/slots', {
      headers: { Authorization: 'Bearer wrong-token-value-0000' },
    });
    expect(res.status).toBe(401);
    expect(res.body).toEqual({
      error: 'Invalid or missing relay token',
      hint: "Verify your RELAY_AUTH_TOKEN matches the server's RELAY_API_TOKEN.",
    });
  });

  test('401 for malformed Authorization (no Bearer prefix)', async () => {
    const res = await request('/api/salvi/inter-cube/slots', {
      headers: { Authorization: 'Token some-value' },
    });
    expect(res.status).toBe(401);
    expect(res.body.error).toBe('Invalid or missing relay token');
  });

  test('503 when no daemons connected', async () => {
    const token = getValidToken();
    if (!token) { console.log('RELAY_API_TOKEN not set — skipping 503 test'); return; }
    const res = await request('/api/salvi/inter-cube/slots', {
      headers: { Authorization: 'Bearer ' + token },
    });
    expect(res.status).toBe(503);
    expect(res.body).toEqual({
      error: 'No Array3 daemons connected to relay',
      hint: 'Verify Array3 services are running on the host machine and connected to the relay.',
    });
  });

  test('all error responses use strict { error, hint } structure', async () => {
    const res401 = await request('/api/salvi/inter-cube/slots');
    expect(res401.status).toBe(401);
    expect(typeof res401.body.error).toBe('string');
    expect(typeof res401.body.hint).toBe('string');
    expect(res401.body.error.length).toBeGreaterThan(0);
    expect(res401.body.hint.length).toBeGreaterThan(0);
    expect(Object.keys(res401.body).sort()).toEqual(['error', 'hint']);

    const token = getValidToken();
    if (token) {
      const res503 = await request('/api/salvi/inter-cube/slots', {
        headers: { Authorization: 'Bearer ' + token },
      });
      expect(res503.status).toBe(503);
      expect(typeof res503.body.error).toBe('string');
      expect(typeof res503.body.hint).toBe('string');
      expect(Object.keys(res503.body).sort()).toEqual(['error', 'hint']);
    }
  });

  test('relay/status returns connection diagnostics without auth', async () => {
    const res = await request('/api/salvi/inter-cube/relay/status');
    expect(res.status).toBe(200);
    expect(typeof res.body.connectedNodes).toBe('number');
    expect(Array.isArray(res.body.nodes)).toBe(true);
    expect(typeof res.body.pendingQueues).toBe('number');
  });

  test('route-specific rate limiter returns 429 after 30 requests', async () => {
    const results = [];
    for (let i = 0; i < 35; i++) {
      const res = await request('/api/salvi/inter-cube/slots');
      results.push(res.status);
      if (res.status === 429 && res.body.error === 'Too many requests') {
        expect(res.body).toEqual({
          error: 'Too many requests',
          hint: 'Rate limit is 30 requests per minute. Wait and retry.',
        });
        break;
      }
    }
    expect(results).toContain(429);
  }, 30000);
});
