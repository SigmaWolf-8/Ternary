const http = require('http');
const crypto = require('crypto');

const BASE_URL = process.env.TEST_BASE_URL || 'http://localhost:5000';

function request(path, options = {}) {
  return new Promise((resolve, reject) => {
    const url = new URL(path, BASE_URL);
    const reqOptions = {
      hostname: url.hostname,
      port: url.port,
      path: url.pathname + url.search,
      method: options.method || 'GET',
      headers: options.headers || {},
    };
    const req = http.request(reqOptions, (res) => {
      let data = '';
      res.on('data', (chunk) => (data += chunk));
      res.on('end', () => {
        let parsed;
        try { parsed = JSON.parse(data); } catch { parsed = data; }
        resolve({ status: res.statusCode || 0, body: parsed });
      });
    });
    req.on('error', reject);
    req.setTimeout(12000, () => { req.destroy(); reject(new Error('request timeout')); });
    if (options.body) req.write(JSON.stringify(options.body));
    req.end();
  });
}

function getValidToken() {
  return process.env.RELAY_API_TOKEN || null;
}

async function registerTestNode(address, publicKey) {
  return request('/api/salvi/inter-cube/crs/test-register', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: { address, publicKey },
  });
}

describe('Relay HTTP Proxy — authentication and error contract', () => {

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
    expect(Object.keys(res401.body).sort()).toEqual(['error', 'hint']);

    const token = getValidToken();
    if (token) {
      const res503 = await request('/api/salvi/inter-cube/slots', {
        headers: { Authorization: 'Bearer ' + token },
      });
      expect(res503.status).toBe(503);
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
});

describe('Relay HTTP Proxy — fan-out behavior (mock daemon injection)', () => {
  const WebSocket = require('ws');
  let mockDaemons = [];
  const RELAY_WS_URL = (process.env.TEST_BASE_URL || 'http://localhost:5000').replace('http', 'ws') + '/ws/relay';

  async function connectMockDaemon(addr) {
    const publicKey = crypto.randomBytes(32).toString('hex');
    const regRes = await registerTestNode(addr, publicKey);
    if (regRes.status !== 200) throw new Error('Failed to register test node: ' + JSON.stringify(regRes.body));

    return new Promise((resolve, reject) => {
      const ws = new WebSocket(RELAY_WS_URL);
      let resolved = false;
      ws.on('message', (data) => {
        try {
          const msg = JSON.parse(data.toString());
          if (msg.type === 'challenge') {
            ws.send(JSON.stringify({ type: 'auth', address: addr, publicKey }));
          }
          if (msg.type === 'auth_ok' && !resolved) {
            resolved = true;
            setTimeout(() => resolve(ws), 100);
          }
          if (msg.type === 'auth_fail') {
            reject(new Error('Auth failed for mock daemon ' + addr + ': ' + JSON.stringify(msg)));
          }
        } catch {}
      });
      ws.on('error', (err) => { if (!resolved) reject(err); });
      setTimeout(() => { if (!resolved) { resolved = true; reject(new Error('Mock daemon connect timeout for ' + addr)); } }, 5000);
    });
  }

  afterEach(async () => {
    for (const ws of mockDaemons) {
      if (ws.readyState === WebSocket.OPEN) ws.close();
    }
    mockDaemons = [];
    await new Promise(r => setTimeout(r, 500));
  });

  test('504 when all connected daemons time out (no proxy response)', async () => {
    const token = process.env.RELAY_API_TOKEN;
    if (!token) { console.log('RELAY_API_TOKEN not set — skipping'); return; }

    const ws = await connectMockDaemon('1.1.1');
    mockDaemons.push(ws);

    const statusRes = await request('/api/salvi/inter-cube/relay/status');
    expect(statusRes.body.connectedNodes).toBeGreaterThanOrEqual(1);

    const res = await request('/api/salvi/inter-cube/slots', {
      headers: { Authorization: 'Bearer ' + token },
    });

    expect(res.status).toBe(504);
    expect(res.body).toEqual({
      error: 'All daemons timed out',
      hint: 'Daemons are connected but not responding. Check daemon logs on the host machine.',
    });
    expect(Object.keys(res.body).sort()).toEqual(['error', 'hint']);
  }, 15000);

  test('200 with node data when daemon responds to proxy request', async () => {
    const token = process.env.RELAY_API_TOKEN;
    if (!token) { console.log('RELAY_API_TOKEN not set — skipping'); return; }

    const ws = await connectMockDaemon('1.1.2');
    mockDaemons.push(ws);

    ws.on('message', (data) => {
      try {
        const msg = JSON.parse(data.toString());
        if (msg.msgType === 'http_proxy_req') {
          const payload = JSON.parse(msg.payload);
          ws.send(JSON.stringify({
            type: 'relay',
            to: '__relay_server__',
            msgType: 'http_proxy_res',
            payload: JSON.stringify({
              request_id: payload.request_id,
              status: 200,
              body: JSON.stringify({
                node_id: 1,
                slots: [{ address: '1.1.1', service: 'Gateway', status: 'active', port: 8181 }],
                summary: { total: 27, occupied: 1 },
              }),
            }),
          }));
        }
      } catch {}
    });

    const res = await request('/api/salvi/inter-cube/slots', {
      headers: { Authorization: 'Bearer ' + token },
    });

    expect(res.status).toBe(200);
    expect(res.body.cluster).toBeDefined();
    expect(res.body.cluster.total_nodes).toBe(1);
    expect(res.body.cluster.responding_nodes).toBe(1);
    expect(res.body.cluster.nodes.length).toBe(1);
    expect(res.body.cluster.nodes[0].status).toBe('ok');
    expect(res.body.cluster.nodes[0].node_id_num).toBe(1);
  }, 15000);

  test('200 with error node when daemon returns non-2xx HTTP status', async () => {
    const token = process.env.RELAY_API_TOKEN;
    if (!token) { console.log('RELAY_API_TOKEN not set — skipping'); return; }

    const ws = await connectMockDaemon('1.1.3');
    mockDaemons.push(ws);

    ws.on('message', (data) => {
      try {
        const msg = JSON.parse(data.toString());
        if (msg.msgType === 'http_proxy_req') {
          const payload = JSON.parse(msg.payload);
          ws.send(JSON.stringify({
            type: 'relay',
            to: '__relay_server__',
            msgType: 'http_proxy_res',
            payload: JSON.stringify({
              request_id: payload.request_id,
              status: 500,
              body: '{"error":"internal"}',
            }),
          }));
        }
      } catch {}
    });

    const res = await request('/api/salvi/inter-cube/slots', {
      headers: { Authorization: 'Bearer ' + token },
    });

    expect(res.status).toBe(200);
    expect(res.body.cluster).toBeDefined();
    expect(res.body.cluster.nodes[0].status).toBe('error');
    expect(res.body.cluster.nodes[0].error).toContain('500');
  }, 15000);

  test('200 partial response: 1 daemon responds, 1 times out', async () => {
    const token = process.env.RELAY_API_TOKEN;
    if (!token) { console.log('RELAY_API_TOKEN not set — skipping'); return; }

    const ws1 = await connectMockDaemon('2.1.1');
    mockDaemons.push(ws1);
    ws1.on('message', (data) => {
      try {
        const msg = JSON.parse(data.toString());
        if (msg.msgType === 'http_proxy_req') {
          const payload = JSON.parse(msg.payload);
          ws1.send(JSON.stringify({
            type: 'relay',
            to: '__relay_server__',
            msgType: 'http_proxy_res',
            payload: JSON.stringify({
              request_id: payload.request_id,
              status: 200,
              body: JSON.stringify({
                node_id: 2,
                slots: [{ address: '2.1.1', service: 'CRS', status: 'active', port: 8182 }],
                summary: { total: 27, occupied: 1 },
              }),
            }),
          }));
        }
      } catch {}
    });

    const ws2 = await connectMockDaemon('3.1.1');
    mockDaemons.push(ws2);

    const res = await request('/api/salvi/inter-cube/slots', {
      headers: { Authorization: 'Bearer ' + token },
    });

    expect(res.status).toBe(200);
    expect(res.body.cluster).toBeDefined();
    expect(res.body.cluster.total_nodes).toBe(2);
    expect(res.body.cluster.responding_nodes).toBe(1);
    expect(res.body.cluster.nodes.length).toBe(2);

    const okNode = res.body.cluster.nodes.find(n => n.status === 'ok');
    const timeoutNode = res.body.cluster.nodes.find(n => n.status === 'timeout');
    expect(okNode).toBeDefined();
    expect(timeoutNode).toBeDefined();
    expect(okNode.node_id_num).toBe(2);
  }, 15000);

  test('concurrent-cap 429 when 6+ inbound requests in flight', async () => {
    const token = process.env.RELAY_API_TOKEN;
    if (!token) { console.log('RELAY_API_TOKEN not set — skipping'); return; }

    const ws = await connectMockDaemon('1.2.1');
    mockDaemons.push(ws);

    const requests = [];
    for (let i = 0; i < 7; i++) {
      requests.push(request('/api/salvi/inter-cube/slots', {
        headers: { Authorization: 'Bearer ' + token },
      }));
    }
    const results = await Promise.all(requests);
    const statuses = results.map(r => r.status);

    const got429 = statuses.filter(s => s === 429).length;
    expect(got429).toBeGreaterThanOrEqual(1);

    const the429 = results.find(r => r.status === 429);
    if (the429) {
      expect(the429.body).toEqual({
        error: 'Too many concurrent requests',
        hint: 'Wait a few seconds and retry. Check for duplicate monitor instances.',
      });
    }
  }, 20000);
});

describe('Relay HTTP Proxy — rate limiting (run last)', () => {

  test('route-specific rate limiter returns 429 after 30 requests', async () => {
    const results = [];
    for (let i = 0; i < 35; i++) {
      const res = await request('/api/salvi/inter-cube/slots');
      results.push(res.status);
      if (res.status === 429) {
        expect(res.body).toEqual({
          error: 'Too many concurrent requests',
          hint: 'Wait a few seconds and retry. Check for duplicate monitor instances.',
        });
        break;
      }
    }
    expect(results).toContain(429);
  }, 30000);

  test('429 from rate limiter uses standardized { error, hint } contract', async () => {
    for (let i = 0; i < 35; i++) {
      const res = await request('/api/salvi/inter-cube/slots');
      if (res.status === 429) {
        expect(Object.keys(res.body).sort()).toEqual(['error', 'hint']);
        expect(typeof res.body.error).toBe('string');
        expect(typeof res.body.hint).toBe('string');
        return;
      }
    }
    throw new Error('Expected 429 but never received it');
  }, 30000);
});
