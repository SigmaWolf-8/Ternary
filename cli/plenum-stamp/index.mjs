#!/usr/bin/env node
/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
 * Patent(s) Pending.
 *
 * plenum-stamp — PlenumNET RFC 3161 TSA CLI
 *
 * Zero-dependency CLI for signing and verifying files with
 * PlenumNET's Time-Stamping Authority. Uses only Node.js built-ins.
 *
 * Usage:
 *   plenum-stamp sign <file>          Hash and timestamp a file
 *   plenum-stamp verify <file>        Verify a file's timestamp
 *   plenum-stamp info <file.tsp>      Display token metadata
 *   plenum-stamp cert                 Download TSA certificate
 *
 * See LICENSE in the repository root for full terms.
 */

import { createHash } from "node:crypto";
import { readFile, writeFile, stat, access } from "node:fs/promises";
import { basename, resolve, extname } from "node:path";
import { request as httpsRequest } from "node:https";
import { request as httpRequest } from "node:http";

const VERSION = "1.0.0";
const DEFAULT_ENDPOINT = "https://plenumnet.replit.app";
const TSP_EXTENSION = ".tsp";
const TSP_META_EXTENSION = ".tsp.json";

function parseArgs(argv) {
  const args = { command: null, file: null, flags: {} };
  let i = 2;
  while (i < argv.length) {
    const a = argv[i];
    if (a === "--endpoint" || a === "-e") {
      args.flags.endpoint = argv[++i];
    } else if (a === "--token" || a === "-t") {
      args.flags.token = argv[++i];
    } else if (a === "--format" || a === "-f") {
      args.flags.format = argv[++i];
    } else if (a === "--algorithm" || a === "-a") {
      args.flags.algorithm = argv[++i];
    } else if (a === "--policy" || a === "-p") {
      args.flags.policy = argv[++i];
    } else if (a === "--output" || a === "-o") {
      args.flags.output = argv[++i];
    } else if (a === "--help" || a === "-h") {
      args.flags.help = true;
    } else if (a === "--version" || a === "-v") {
      args.flags.version = true;
    } else if (a === "--compact") {
      args.flags.compact = true;
    } else if (a === "--calendars") {
      args.flags.calendars = argv[++i];
    } else if (!args.command) {
      args.command = a;
    } else if (!args.file) {
      args.file = a;
    }
    i++;
  }
  return args;
}

function getEndpoint(flags) {
  return (flags.endpoint || process.env.PLENUM_ENDPOINT || DEFAULT_ENDPOINT).replace(/\/$/, "");
}

function getToken(flags) {
  return flags.token || process.env.PLENUM_API_TOKEN || null;
}

function fetch(url, options = {}) {
  return new Promise((resolve, reject) => {
    const parsed = new URL(url);
    const fn = parsed.protocol === "https:" ? httpsRequest : httpRequest;
    const req = fn(url, {
      method: options.method || "GET",
      headers: options.headers || {},
      timeout: 30000,
    }, (res) => {
      const chunks = [];
      res.on("data", (c) => chunks.push(c));
      res.on("end", () => {
        const body = Buffer.concat(chunks);
        resolve({ status: res.statusCode, headers: res.headers, body });
      });
    });
    req.on("error", reject);
    req.on("timeout", () => { req.destroy(); reject(new Error("Request timed out")); });
    if (options.body) req.write(options.body);
    req.end();
  });
}

async function hashFile(filePath, algorithm = "sha256") {
  const data = await readFile(filePath);
  return createHash(algorithm).update(data).digest("hex");
}

function formatBytes(bytes) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function formatTime(isoOrGeneralized) {
  if (!isoOrGeneralized) return "unknown";
  const s = String(isoOrGeneralized);
  const m = s.match(/^(\d{4})(\d{2})(\d{2})(\d{2})(\d{2})(\d{2})(?:\.(\d+))?Z?$/);
  if (m) {
    const frac = m[7] ? `.${m[7]}` : "";
    return `${m[1]}-${m[2]}-${m[3]} ${m[4]}:${m[5]}:${m[6]}${frac} UTC`;
  }
  try { return new Date(s).toISOString(); }
  catch { return s; }
}

function formatAccuracy(acc) {
  if (!acc) return null;
  if (typeof acc === "string") return acc;
  const parts = [];
  if (acc.seconds) parts.push(`${acc.seconds}s`);
  if (acc.millis) parts.push(`${acc.millis}ms`);
  if (acc.micros) parts.push(`${acc.micros}\u00B5s`);
  return parts.length ? `\u00B1${parts.join(" ")}` : "\u00B10";
}

async function commandSign(file, flags) {
  const endpoint = getEndpoint(flags);
  const token = getToken(flags);
  const algorithm = flags.algorithm || "sha256";
  const format = flags.format || "text";
  const filePath = resolve(file);

  await access(filePath);
  const fileInfo = await stat(filePath);

  process.stderr.write(`Hashing ${basename(filePath)} (${formatBytes(fileInfo.size)})...\n`);
  const hash = await hashFile(filePath, algorithm);
  process.stderr.write(`${algorithm.toUpperCase()}: ${hash}\n`);

  const body = JSON.stringify({
    hash,
    algorithm,
    ...(flags.policy ? { policy: flags.policy } : {}),
    ...(flags.calendars ? { calendars: flags.calendars === "*" ? ["*"] : flags.calendars.split(",") } : {}),
    ...(flags.compact ? { compact: true } : {}),
  });

  const headers = { "Content-Type": "application/json" };
  if (token) headers["Authorization"] = `Bearer ${token}`;

  process.stderr.write(`Requesting timestamp from ${endpoint}...\n`);

  const res = await fetch(`${endpoint}/api/tsa/timestamp/json`, {
    method: "POST",
    headers,
    body,
  });

  if (res.status !== 200) {
    const errBody = res.body.toString("utf-8");
    process.stderr.write(`Error ${res.status}: ${errBody}\n`);
    process.exit(1);
  }

  const result = JSON.parse(res.body.toString("utf-8"));

  if (!result.success || !result.token) {
    process.stderr.write(`TSA error: ${result.error || "no token returned"}\n`);
    process.exit(1);
  }

  const tokenBuf = Buffer.from(result.token, "base64");
  const tspPath = flags.output || `${filePath}${TSP_EXTENSION}`;
  await writeFile(tspPath, tokenBuf);

  const meta = {
    version: 1,
    file: basename(filePath),
    fileSize: fileInfo.size,
    hash,
    algorithm,
    serialNumber: result.serialNumber,
    genTime: result.genTime,
    policyOid: result.policyOid,
    policyName: result.policyName,
    accuracy: result.accuracy,
    endpoint,
    stampedAt: new Date().toISOString(),
  };
  await writeFile(`${tspPath}.json`, JSON.stringify(meta, null, 2));

  if (format === "json") {
    process.stdout.write(JSON.stringify({
      success: true,
      file: basename(filePath),
      hash,
      algorithm,
      serialNumber: result.serialNumber,
      genTime: result.genTime,
      policy: result.policyName,
      tokenFile: basename(tspPath),
      tokenSize: tokenBuf.length,
    }, null, 2) + "\n");
  } else {
    process.stdout.write("\n");
    process.stdout.write(`  Stamped successfully.\n`);
    process.stdout.write(`  Serial:   ${result.serialNumber}\n`);
    process.stdout.write(`  Time:     ${formatTime(result.genTime)}\n`);
    process.stdout.write(`  Policy:   ${result.policyName || result.policyOid}\n`);
    process.stdout.write(`  Token:    ${basename(tspPath)} (${formatBytes(tokenBuf.length)})\n`);
    process.stdout.write(`  Metadata: ${basename(tspPath)}.json\n`);
    process.stdout.write("\n");
  }
}

async function commandVerify(file, flags) {
  const endpoint = getEndpoint(flags);
  const token = getToken(flags);
  const format = flags.format || "text";
  const filePath = resolve(file);
  let tspPath;
  let originalFilePath = null;

  if (extname(filePath) === TSP_EXTENSION) {
    tspPath = filePath;
  } else {
    tspPath = `${filePath}${TSP_EXTENSION}`;
    originalFilePath = filePath;
  }

  await access(tspPath);
  const tokenBuf = await readFile(tspPath);
  const tokenBase64 = tokenBuf.toString("base64");

  process.stderr.write(`Verifying ${basename(tspPath)} (${formatBytes(tokenBuf.length)})...\n`);

  const headers = { "Content-Type": "application/json" };
  if (token) headers["Authorization"] = `Bearer ${token}`;

  const res = await fetch(`${endpoint}/api/tsa/verify`, {
    method: "POST",
    headers,
    body: JSON.stringify({ token: tokenBase64 }),
  });

  if (res.status !== 200) {
    const errBody = res.body.toString("utf-8");
    process.stderr.write(`Error ${res.status}: ${errBody}\n`);
    process.exit(1);
  }

  const result = JSON.parse(res.body.toString("utf-8"));

  let metaCheck = null;
  try {
    const metaBuf = await readFile(`${tspPath}.json`);
    metaCheck = JSON.parse(metaBuf.toString("utf-8"));
  } catch {}

  let hashMatch = null;
  if (originalFilePath && metaCheck) {
    try {
      const currentHash = await hashFile(originalFilePath, metaCheck.algorithm || "sha256");
      hashMatch = currentHash === metaCheck.hash;
      if (!hashMatch) {
        process.stderr.write(`WARNING: File hash mismatch — file may have been modified since stamping.\n`);
        process.stderr.write(`  Expected: ${metaCheck.hash}\n`);
        process.stderr.write(`  Current:  ${currentHash}\n`);
      }
    } catch {}
  }

  if (format === "json") {
    process.stdout.write(JSON.stringify({
      valid: result.valid,
      serialNumber: result.serialNumber,
      genTime: result.genTime,
      policy: result.policyOid,
      accuracy: result.accuracy,
      ...(hashMatch !== null ? { hashMatch } : {}),
      ...(metaCheck ? {
        originalFile: metaCheck.file,
        originalHash: metaCheck.hash,
        hashAlgorithm: metaCheck.algorithm,
      } : {}),
    }, null, 2) + "\n");
  } else {
    process.stdout.write("\n");
    if (result.valid && hashMatch !== false) {
      process.stdout.write(`  VALID timestamp token.\n`);
    } else if (result.valid && hashMatch === false) {
      process.stdout.write(`  VALID token, but FILE MODIFIED since stamping.\n`);
    } else {
      process.stdout.write(`  INVALID — verification failed.\n`);
    }
    process.stdout.write(`  Serial:   ${result.serialNumber}\n`);
    const verifyTime = result.genTime || (metaCheck && metaCheck.genTime);
    if (verifyTime) process.stdout.write(`  Time:     ${formatTime(verifyTime)}\n`);
    process.stdout.write(`  Policy:   ${result.policyOid}\n`);
    const accStr = formatAccuracy(result.accuracy);
    if (accStr) {
      process.stdout.write(`  Accuracy: ${accStr}\n`);
    }
    if (metaCheck) {
      process.stdout.write(`  File:     ${metaCheck.file} (${metaCheck.algorithm}: ${metaCheck.hash})\n`);
    }
    if (hashMatch !== null) {
      process.stdout.write(`  Integrity: ${hashMatch ? "file unchanged" : "FILE MODIFIED"}\n`);
    }
    process.stdout.write("\n");
  }

  if (!result.valid) process.exit(1);
  if (hashMatch === false) process.exit(2);
}

async function commandInfo(file, flags) {
  const format = flags.format || "text";
  const filePath = resolve(file);

  let tspPath = filePath;
  if (extname(filePath) !== TSP_EXTENSION) {
    tspPath = `${filePath}${TSP_EXTENSION}`;
  }

  await access(tspPath);
  const tokenBuf = await readFile(tspPath);

  let meta = null;
  try {
    const metaBuf = await readFile(`${tspPath}.json`);
    meta = JSON.parse(metaBuf.toString("utf-8"));
  } catch {}

  const info = {
    tokenFile: basename(tspPath),
    tokenSize: tokenBuf.length,
    tokenSizeHuman: formatBytes(tokenBuf.length),
    ...(meta || {}),
  };

  if (format === "json") {
    process.stdout.write(JSON.stringify(info, null, 2) + "\n");
  } else {
    process.stdout.write("\n");
    process.stdout.write(`  Token:    ${info.tokenFile} (${info.tokenSizeHuman})\n`);
    if (meta) {
      process.stdout.write(`  File:     ${meta.file} (${formatBytes(meta.fileSize)})\n`);
      process.stdout.write(`  Hash:     ${meta.algorithm}:${meta.hash}\n`);
      process.stdout.write(`  Serial:   ${meta.serialNumber}\n`);
      process.stdout.write(`  Time:     ${formatTime(meta.genTime)}\n`);
      process.stdout.write(`  Policy:   ${meta.policyName || meta.policyOid}\n`);
      const accStr = formatAccuracy(meta.accuracy);
      if (accStr) process.stdout.write(`  Accuracy: ${accStr}\n`);
      process.stdout.write(`  Endpoint: ${meta.endpoint}\n`);
      process.stdout.write(`  Stamped:  ${meta.stampedAt}\n`);
    } else {
      process.stdout.write(`  No metadata file found (${basename(tspPath)}.json)\n`);
    }
    process.stdout.write("\n");
  }
}

async function commandCert(flags) {
  const endpoint = getEndpoint(flags);
  const token = getToken(flags);
  const format = flags.format || "text";

  process.stderr.write(`Downloading TSA certificate from ${endpoint}...\n`);

  const headers = {};
  if (token) headers["Authorization"] = `Bearer ${token}`;

  const res = await fetch(`${endpoint}/api/tsa/certificate/download`, { headers });

  if (res.status !== 200) {
    process.stderr.write(`Error ${res.status}: ${res.body.toString("utf-8")}\n`);
    process.exit(1);
  }

  const certPem = res.body.toString("utf-8");
  const outPath = flags.output || "plenumnet-tsa.pem";
  await writeFile(outPath, certPem);

  if (format === "json") {
    process.stdout.write(JSON.stringify({ saved: outPath, size: certPem.length }, null, 2) + "\n");
  } else {
    process.stdout.write(`\n  Certificate saved to ${outPath} (${formatBytes(certPem.length)})\n\n`);
  }
}

function printHelp() {
  process.stdout.write(`
plenum-stamp v${VERSION} — PlenumNET RFC 3161 TSA CLI

USAGE
  plenum-stamp <command> [file] [options]

COMMANDS
  sign <file>          Hash and timestamp a file
  verify <file>        Verify a file's timestamp token
  info <file|.tsp>     Display token metadata
  cert                 Download TSA certificate

OPTIONS
  -e, --endpoint <url>   TSA endpoint (default: ${DEFAULT_ENDPOINT})
                         Also: PLENUM_ENDPOINT env var
  -t, --token <token>    API auth token
                         Also: PLENUM_API_TOKEN env var
  -a, --algorithm <alg>  Hash algorithm (default: sha256)
                         Supported: sha256, sha384, sha512, sha3-256
  -p, --policy <oid>     TSA policy OID
  -f, --format <fmt>     Output format: text (default) or json
  -o, --output <path>    Output file path
  --compact              Use compact calendar encoding
  --calendars <list>     Calendar systems (comma-separated, or * for all)
  -h, --help             Show this help
  -v, --version          Show version

EXAMPLES
  # Sign a release binary
  plenum-stamp sign dist/app-v1.0.0.tar.gz

  # Verify the stamp
  plenum-stamp verify dist/app-v1.0.0.tar.gz

  # Machine-readable output
  plenum-stamp sign README.md --format json

  # Use a custom endpoint
  plenum-stamp sign file.txt --endpoint http://localhost:5000

  # Download the TSA certificate for offline verification
  plenum-stamp cert

ENVIRONMENT
  PLENUM_ENDPOINT      TSA base URL
  PLENUM_API_TOKEN     Bearer token for authentication

FILES
  <file>.tsp           DER-encoded RFC 3161 timestamp token
  <file>.tsp.json      Human-readable metadata (hash, serial, time)

`);
}

async function main() {
  const args = parseArgs(process.argv);

  if (args.flags.version) {
    process.stdout.write(`plenum-stamp v${VERSION}\n`);
    return;
  }

  if (args.flags.help || !args.command) {
    printHelp();
    return;
  }

  try {
    switch (args.command) {
      case "sign":
        if (!args.file) { process.stderr.write("Error: sign requires a file argument\n"); process.exit(1); }
        await commandSign(args.file, args.flags);
        break;
      case "verify":
        if (!args.file) { process.stderr.write("Error: verify requires a file argument\n"); process.exit(1); }
        await commandVerify(args.file, args.flags);
        break;
      case "info":
        if (!args.file) { process.stderr.write("Error: info requires a file argument\n"); process.exit(1); }
        await commandInfo(args.file, args.flags);
        break;
      case "cert":
        await commandCert(args.flags);
        break;
      default:
        process.stderr.write(`Unknown command: ${args.command}\n`);
        printHelp();
        process.exit(1);
    }
  } catch (err) {
    if (err.code === "ENOENT") {
      process.stderr.write(`Error: file not found — ${err.path || args.file}\n`);
    } else {
      process.stderr.write(`Error: ${err.message}\n`);
    }
    process.exit(1);
  }
}

main();
