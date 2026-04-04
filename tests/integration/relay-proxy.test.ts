import http from 'http';

const BASE_URL = process.env.TEST_BASE_URL || 'http://localhost:5000';

function request(
  path: string,
  options: { headers?: Record<string, string> } = {}
): Promise<{ status: number; body: any }> {
  return new Promise((resolve, reject) => {
    const url = new URL(path, BASE_URL);
    const req = http.get(
      url.toString(),
      { headers: options.headers || {} },
      (res) => {
        let data = '';
        res.on('data', (chunk) => (data += chunk));
        res.on('end', () => {
          let parsed: any;
          try {
            parsed = JSON.parse(data);
          } catch {
            parsed = data;
          }
          resolve({ status: res.statusCode || 0, body: parsed });
        });
      }
    );
    req.on('error', reject);
    req.setTimeout(10000, () => {
      req.destroy();
      reject(new Error('request timeout'));
    });
  });
}

describe('Relay HTTP Proxy — /api/salvi/inter-cube/slots', () => {
  test('(a) 401 for missing Authorization header', async () => {
    const res = await request('/api/salvi/inter-cube/slots');
    expect(res.status).toBe(401);
    expect(res.body.error).toBe('Invalid or missing relay token');
    expect(res.body.hint).toBe("Verify your RELAY_AUTH_TOKEN matches the server's RELAY_API_TOKEN.");
  });

  test('(a) 401 for wrong Bearer token', async () => {
    const res = await request('/api/salvi/inter-cube/slots', {
      headers: { Authorization: 'Bearer wrong-token-value' },
    });
    expect(res.status).toBe(401);
    expect(res.body.error).toBe('Invalid or missing relay token');
    expect(typeof res.body.hint).toBe('string');
  });

  test('(b) 503 when no remote daemons connected via relay', async () => {
    const validToken = process.env.RELAY_API_TOKEN;
    if (!validToken) {
      console.log('RELAY_API_TOKEN not set — skipping authenticated test');
      return;
    }
    const res = await request('/api/salvi/inter-cube/slots', {
      headers: { Authorization: `Bearer ${validToken}` },
    });
    expect([200, 503, 504]).toContain(res.status);
    if (res.status === 503) {
      expect(res.body.error).toBe('No Array3 daemons connected to relay');
      expect(res.body.hint).toContain('Array3 services');
    }
  });

  test('(f) all error responses use { error, hint } structure', async () => {
    const res = await request('/api/salvi/inter-cube/slots');
    expect(res.status).toBe(401);
    expect(typeof res.body.error).toBe('string');
    expect(typeof res.body.hint).toBe('string');
    expect(res.body.error.length).toBeGreaterThan(0);
    expect(res.body.hint.length).toBeGreaterThan(0);
  });

  test('relay/status returns connection diagnostics without auth', async () => {
    const res = await request('/api/salvi/inter-cube/relay/status');
    expect(res.status).toBe(200);
    expect(typeof res.body.connectedNodes).toBe('number');
    expect(Array.isArray(res.body.nodes)).toBe(true);
    expect(typeof res.body.pendingQueues).toBe('number');
  });
});
