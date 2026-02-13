import fs from 'fs';
import path from 'path';

const OWNER = 'SigmaWolf-8';
const REPO = 'Ternary';
const BRANCH = 'main';
const TOKEN = process.env.GITHUB_TOKEN;

if (!TOKEN) {
  console.error('GITHUB_TOKEN not set');
  process.exit(1);
}

const headers = {
  'Authorization': `Bearer ${TOKEN}`,
  'Accept': 'application/vnd.github.v3+json',
  'Content-Type': 'application/json',
  'User-Agent': 'Salvi-Framework-Sync'
};

async function getFileSha(filePath) {
  try {
    const res = await fetch(
      `https://api.github.com/repos/${OWNER}/${REPO}/contents/${filePath}?ref=${BRANCH}`,
      { headers }
    );
    if (res.ok) {
      const data = await res.json();
      return data.sha;
    }
    return null;
  } catch {
    return null;
  }
}

async function pushFile(filePath, commitMessage) {
  const localPath = path.resolve(filePath);
  if (!fs.existsSync(localPath)) {
    console.log(`SKIP (not found): ${filePath}`);
    return false;
  }

  const content = fs.readFileSync(localPath);
  const base64Content = content.toString('base64');
  const sha = await getFileSha(filePath);

  const body = {
    message: commitMessage,
    content: base64Content,
    branch: BRANCH
  };
  if (sha) {
    body.sha = sha;
  }

  const res = await fetch(
    `https://api.github.com/repos/${OWNER}/${REPO}/contents/${filePath}`,
    {
      method: 'PUT',
      headers,
      body: JSON.stringify(body)
    }
  );

  if (res.ok) {
    const data = await res.json();
    console.log(`OK: ${filePath} → ${data.commit?.sha?.substring(0, 7) || 'committed'}`);
    return true;
  } else {
    const err = await res.json().catch(() => ({}));
    console.error(`FAIL: ${filePath} → ${res.status} ${err.message || ''}`);
    return false;
  }
}

async function pushBatch(files, commitMessage, delay = 500) {
  let success = 0;
  let fail = 0;
  for (const f of files) {
    const msg = typeof commitMessage === 'function' ? commitMessage(f) : commitMessage;
    const ok = await pushFile(f, msg);
    if (ok) success++;
    else fail++;
    if (delay > 0) await new Promise(r => setTimeout(r, delay));
  }
  console.log(`\nBatch complete: ${success} ok, ${fail} failed\n`);
  return { success, fail };
}

const phase = process.argv[2];

if (phase === 'phase0') {
  console.log('=== PHASE 0: Emergency IP & Repository Remediation ===');
  await pushBatch([
    'src/kernel/Cargo.toml',
    'src/kernel/wasm/Cargo.toml',
    '.gitignore',
    'MANIFEST-FIXES.md'
  ], (f) => {
    if (f.includes('Cargo.toml')) return 'fix(legal): change license from MIT to LicenseRef-Proprietary';
    if (f.includes('.gitignore')) return 'fix(repo): gitignore nested target/ directories with **/target/';
    return 'fix(legal): update manifest fixes tracking';
  });
} else if (phase === 'phase1') {
  console.log('=== PHASE 1: License Header Completion ===');
  const salviCore = [
    'server/salvi-core/unified-metadata-schema.ts',
    'server/salvi-core/error-handling.ts',
    'server/salvi-core/payment-listener-api.ts',
    'server/salvi-core/timing-service.ts',
    'server/salvi-core/sfk-operations-api.ts',
    'server/salvi-core/blockchain-integrations.ts'
  ];
  const libternary = [
    'libternary/src/index.ts',
    'libternary/src/ternary-types.ts',
    'libternary/src/ternary-operations.ts',
    'libternary/src/phase-encryption.ts',
    'libternary/src/femtosecond-timing.ts',
    'libternary/src/tribonacci.ts',
    'libternary/tests/tribonacci.test.ts',
    'libternary/tests/ternary-operations.test.ts'
  ];
  const sharedConfig = [
    'shared/tribonacci-constants.ts',
    'script/build.ts',
    'contracts/oracle-bridge/src/main.ts'
  ];
  const rootDocs = [
    'CODE-OF-CONDUCT.md',
    'SECURITY.md'
  ];

  await pushBatch(salviCore, 'fix(legal): add proprietary copyright headers to salvi-core files');
  await pushBatch(libternary, 'fix(legal): add proprietary copyright headers to libternary files');
  await pushBatch(sharedConfig, 'fix(legal): add proprietary copyright headers to shared/config files');
  await pushBatch(rootDocs, (f) => {
    if (f === 'CODE-OF-CONDUCT.md') return 'docs: add Code of Conduct';
    return 'docs: add SECURITY.md to repository root';
  });
} else if (phase === 'phase2') {
  console.log('=== PHASE 2: Security Hardening ===');
  await pushBatch([
    'package.json',
    'package-lock.json',
    'server/routes/middleware.ts',
    'server/crypto-utils.ts'
  ], (f) => {
    if (f === 'package.json' || f === 'package-lock.json') return 'fix(security): add rate-limit, CORS, helmet dependencies';
    if (f.includes('middleware')) return 'fix(security): add auth and admin middleware with rate limiting';
    return 'fix(security): add AES-256-GCM token encryption utility';
  });
} else if (phase === 'phase3') {
  console.log('=== PHASE 3: Code Architecture Improvements ===');
  await pushBatch([
    'server/routes.ts',
    'server/routes/github.ts',
    'server/routes/kong.ts',
    'server/routes/salvi.ts',
    'server/routes/middleware.ts',
    'server/logger.ts',
    'server/config.ts'
  ], (f) => {
    if (f === 'server/routes.ts') return 'refactor(arch): decompose monolithic routes (3732→893 lines)';
    if (f.includes('routes/')) return `refactor(arch): extract ${path.basename(f, '.ts')} route module`;
    if (f.includes('logger')) return 'refactor(arch): add structured Winston logger';
    return 'refactor(arch): add centralized environment configuration';
  });
} else if (phase === 'phase4') {
  console.log('=== PHASE 4: Test Coverage Expansion ===');
  await pushBatch([
    'vitest.config.ts',
    'tests/ternary-operations.test.ts',
    'tests/phase-encryption.test.ts',
    'tests/calendar-sync.test.ts',
    '.github/workflows/test-typescript.yml'
  ], (f) => {
    if (f.includes('vitest')) return 'test: add Vitest configuration';
    if (f.includes('ternary')) return 'test(ternary): add 50 GF(3) arithmetic KAT tests';
    if (f.includes('phase')) return 'test(crypto): add 25 phase encryption round-trip tests';
    if (f.includes('calendar')) return 'test(calendar): add 11 calendar synchronization tests';
    return 'ci: add TypeScript test workflow';
  });
} else if (phase === 'phase5') {
  console.log('=== PHASE 5: Legal Enhancement & Documentation ===');
  await pushBatch([
    'CHANGELOG.md',
    'LICENSING-AUDIT-REPORT.md',
    'DATA-PROCESSING-AGREEMENT.md',
    'replit.md'
  ], (f) => {
    if (f === 'CHANGELOG.md') return 'docs: add comprehensive changelog';
    if (f === 'LICENSING-AUDIT-REPORT.md') return 'docs: update licensing audit report with follow-up';
    if (f === 'DATA-PROCESSING-AGREEMENT.md') return 'docs: add Data Processing Agreement template';
    return 'docs: update replit.md with current architecture';
  });
} else if (phase === 'rust-headers') {
  console.log('=== Rust Kernel Copyright Headers ===');
  const rustFiles = [];
  const findRust = (dir) => {
    if (!fs.existsSync(dir)) return;
    const entries = fs.readdirSync(dir, { withFileTypes: true });
    for (const e of entries) {
      const full = path.join(dir, e.name);
      if (e.isDirectory() && e.name !== 'target') findRust(full);
      else if (e.isFile() && e.name.endsWith('.rs')) rustFiles.push(full);
    }
  };
  findRust('src/kernel/src');
  console.log(`Found ${rustFiles.length} Rust files`);
  await pushBatch(rustFiles, 'fix(legal): add copyright headers to kernel source files', 300);
} else if (phase === 'client') {
  console.log('=== Client-Side Changes ===');
  const clientFiles = [];
  const findClient = (dir) => {
    if (!fs.existsSync(dir)) return;
    const entries = fs.readdirSync(dir, { withFileTypes: true });
    for (const e of entries) {
      const full = path.join(dir, e.name);
      if (e.isDirectory() && !['node_modules', '.git', 'dist'].includes(e.name)) findClient(full);
      else if (e.isFile() && (e.name.endsWith('.tsx') || e.name.endsWith('.ts') || e.name.endsWith('.css'))) {
        clientFiles.push(full);
      }
    }
  };
  findClient('client/src');
  console.log(`Found ${clientFiles.length} client files`);
  await pushBatch(clientFiles, 'feat(frontend): update client with API demos and remediation UI', 300);
} else if (phase === 'server-index') {
  console.log('=== Server Index & Storage ===');
  await pushBatch([
    'server/index.ts',
    'server/storage.ts',
    'server/vite.ts',
    'shared/schema.ts'
  ], (f) => `refactor: update ${path.basename(f)} with security and architecture changes`);
} else {
  console.log('Usage: node scripts/github-sync.mjs <phase>');
  console.log('Phases: phase0, phase1, phase2, phase3, phase4, phase5, rust-headers, client, server-index');
}
