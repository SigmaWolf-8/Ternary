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
      ? `Update ${path.basename(repoPath)} — Phase 1 security deliverables`
      : `Add ${path.basename(repoPath)} — Phase 1 security deliverables`,
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

async function main() {
  console.log("=".repeat(60));
  console.log("  PUSH TO GITHUB — Phase 1 Security Deliverables");
  console.log("  Repository:", `${REPO_OWNER}/${REPO_NAME}`);
  console.log("  Branch:", BRANCH);
  console.log("  Date:", new Date().toISOString());
  console.log("=".repeat(60));
  console.log();

  const docsDir = path.resolve(__dirname, "../docs/security");
  const scriptsDir = path.resolve(__dirname, "../scripts");

  const filesToPush: { local: string; remote: string }[] = [];

  const securityFiles = fs.readdirSync(docsDir);
  for (const f of securityFiles) {
    const fullPath = path.join(docsDir, f);
    if (fs.statSync(fullPath).isFile()) {
      filesToPush.push({ local: fullPath, remote: `docs/security/${f}` });
    }
  }

  const scriptFiles = [
    "smoke-test-security.ts",
    "smoke-test-http.ts",
    "load-test-security.ts",
  ];
  for (const f of scriptFiles) {
    const fullPath = path.join(scriptsDir, f);
    if (fs.existsSync(fullPath)) {
      filesToPush.push({ local: fullPath, remote: `scripts/${f}` });
    }
  }

  console.log(`  Files to push: ${filesToPush.length}`);
  console.log();

  let ok = 0;
  let fail = 0;

  for (const file of filesToPush) {
    const success = await pushFile(file.local, file.remote);
    if (success) ok++;
    else fail++;
    await new Promise((r) => setTimeout(r, 500));
  }

  console.log();
  console.log("=".repeat(60));
  console.log(`  RESULTS: ${ok} pushed, ${fail} failed`);
  console.log("=".repeat(60));

  if (fail > 0) process.exit(1);
}

main().catch((err) => {
  console.error("Fatal:", err);
  process.exit(1);
});
