import * as fs from "fs";
import * as path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const GITHUB_TOKEN = process.env.GITHUB_TOKEN;
const REPO_OWNER = "SigmaWolf-8";
const REPO_NAME = "Ternary";
const BRANCH = "main";

if (!GITHUB_TOKEN) {
  console.error("GITHUB_TOKEN not set");
  process.exit(1);
}

const headers = {
  Authorization: `token ${GITHUB_TOKEN}`,
  Accept: "application/vnd.github.v3+json",
  "Content-Type": "application/json",
  "User-Agent": "PlenumNET-CI",
};

async function getFileSha(filePath: string): Promise<string | null> {
  const url = `https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/contents/${filePath}?ref=${BRANCH}`;
  const res = await fetch(url, { headers });
  if (res.status === 200) {
    const data: any = await res.json();
    return data.sha;
  }
  return null;
}

async function pushFile(localPath: string, repoPath: string): Promise<boolean> {
  const content = fs.readFileSync(localPath);
  const base64Content = content.toString("base64");
  const sha = await getFileSha(repoPath);

  const body: any = {
    message: sha
      ? `Update ${path.basename(repoPath)} — PQTI service integration`
      : `Add ${path.basename(repoPath)} — PQTI post-quantum signing service`,
    content: base64Content,
    branch: BRANCH,
  };

  if (sha) {
    body.sha = sha;
  }

  const url = `https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/contents/${repoPath}`;
  const res = await fetch(url, {
    method: "PUT",
    headers,
    body: JSON.stringify(body),
  });

  if (res.status === 200 || res.status === 201) {
    console.log(`  OK  ${repoPath} (${sha ? "updated" : "created"})`);
    return true;
  } else {
    const err: any = await res.json();
    console.error(`  FAIL  ${repoPath}: ${res.status} — ${err.message || JSON.stringify(err)}`);
    return false;
  }
}

function collectFiles(dir: string, remotePrefix: string): { local: string; remote: string }[] {
  const files: { local: string; remote: string }[] = [];
  if (!fs.existsSync(dir)) return files;
  
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isFile()) {
      files.push({ local: fullPath, remote: `${remotePrefix}/${entry.name}` });
    } else if (entry.isDirectory()) {
      files.push(...collectFiles(fullPath, `${remotePrefix}/${entry.name}`));
    }
  }
  return files;
}

async function main() {
  console.log("=".repeat(60));
  console.log("  PUSH TO GITHUB — PQTI Post-Quantum Signing Service");
  console.log("  Repository:", `${REPO_OWNER}/${REPO_NAME}`);
  console.log("  Branch:", BRANCH);
  console.log("  Date:", new Date().toISOString());
  console.log("=".repeat(60));
  console.log();

  const root = path.resolve(__dirname, "..");
  const filesToPush: { local: string; remote: string }[] = [];

  const pqtiServiceFiles = collectFiles(
    path.join(root, "services/pqti-service/src"),
    "services/pqti-service/src"
  );
  filesToPush.push(...pqtiServiceFiles);

  const pqtiCargoToml = path.join(root, "services/pqti-service/Cargo.toml");
  if (fs.existsSync(pqtiCargoToml)) {
    filesToPush.push({ local: pqtiCargoToml, remote: "services/pqti-service/Cargo.toml" });
  }

  const pqtiTestScript = path.join(root, "services/pqti-service/test_api.sh");
  if (fs.existsSync(pqtiTestScript)) {
    filesToPush.push({ local: pqtiTestScript, remote: "services/pqti-service/test_api.sh" });
  }

  const kernelCryptoFiles = [
    "src/kernel/src/crypto/tl_dsa.rs",
    "src/kernel/src/crypto/tl_kem.rs",
  ];
  for (const f of kernelCryptoFiles) {
    const fullPath = path.join(root, f);
    if (fs.existsSync(fullPath)) {
      filesToPush.push({ local: fullPath, remote: f });
    }
  }

  const xplenumStub = path.join(root, "src/kernel/src/arch/xplenum_stub.rs");
  if (fs.existsSync(xplenumStub)) {
    filesToPush.push({ local: xplenumStub, remote: "src/kernel/src/arch/xplenum_stub.rs" });
  }

  const kongConfig = path.join(root, "kong/kong.yaml");
  if (fs.existsSync(kongConfig)) {
    filesToPush.push({ local: kongConfig, remote: "kong/kong.yaml" });
  }

  const proxyRoute = path.join(root, "server/routes/pqti.ts");
  if (fs.existsSync(proxyRoute)) {
    filesToPush.push({ local: proxyRoute, remote: "server/routes/pqti.ts" });
  }

  console.log(`  Files to push: ${filesToPush.length}`);
  console.log();
  for (const f of filesToPush) {
    console.log(`    ${f.remote}`);
  }
  console.log();

  let ok = 0;
  let fail = 0;

  for (const file of filesToPush) {
    const success = await pushFile(file.local, file.remote);
    if (success) ok++;
    else fail++;
    await new Promise((r) => setTimeout(r, 600));
  }

  console.log();
  console.log("=".repeat(60));
  console.log(`  RESULTS: ${ok} pushed, ${fail} failed, ${filesToPush.length} total`);
  console.log("=".repeat(60));

  if (fail > 0) process.exit(1);
}

main().catch((err) => {
  console.error("Fatal:", err);
  process.exit(1);
});
