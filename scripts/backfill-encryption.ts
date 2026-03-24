/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * One-time backfill: encrypt existing plaintext rows with Phase Encryption v3.
 * Run via: npx tsx scripts/backfill-encryption.ts
 *
 * Handles BigInt fields gracefully by converting them to strings before JSON serialization.
 * Processes in batches of 200 to avoid memory pressure on large tables.
 */

import { db } from "../server/db";
import { phaseEncryptFields } from "../server/storage";
import {
  binaryStorage, ternaryStorage, whitepapers, apiKeys, apiKeyLogs,
  apiKeyAuditEvents, developerSignups, compressedDocuments, agentArrayReports,
  dataSubjectRequests, securityAuditLog, threatModelEntries, coherenceLogs,
  crsRelayNodes, deploymentRecords,
} from "../shared/schema";
import { eq, isNull, sql } from "drizzle-orm";

const BATCH_SIZE = 200;

function sanitizeForJson(obj: Record<string, unknown>): Record<string, unknown> {
  const result: Record<string, unknown> = {};
  for (const [k, v] of Object.entries(obj)) {
    if (typeof v === "bigint") {
      result[k] = v.toString();
    } else if (v && typeof v === "object" && !Array.isArray(v)) {
      result[k] = sanitizeForJson(v as Record<string, unknown>);
    } else {
      result[k] = v;
    }
  }
  return result;
}

async function backfillTable(
  tableName: string,
  table: any,
  idCol: any,
  getFields: (row: any) => Record<string, unknown>,
) {
  let totalEncrypted = 0;
  let batch = 0;

  while (true) {
    const rows = await db.select().from(table)
      .where(isNull(table.encryptedFields))
      .limit(BATCH_SIZE);

    if (rows.length === 0) break;

    batch++;
    let batchCount = 0;

    for (const row of rows) {
      try {
        const rawFields = getFields(row);
        const fields = sanitizeForJson(rawFields);
        const encrypted = phaseEncryptFields(fields);
        await db.update(table).set({ encryptedFields: encrypted }).where(eq(idCol, row.id));
        batchCount++;
      } catch (err: any) {
        console.error(`  ${tableName} row ${row.id}: ${err.message}`);
      }
    }

    totalEncrypted += batchCount;
    console.log(`  ${tableName} batch ${batch}: ${batchCount}/${rows.length} encrypted (total: ${totalEncrypted})`);
  }

  console.log(`  ${tableName}: ${totalEncrypted} rows encrypted`);
  return totalEncrypted;
}

async function main() {
  console.log("Phase Encryption v3 — Backfill Migration");
  console.log("=========================================\n");

  let total = 0;

  total += await backfillTable("binary_storage", binaryStorage, binaryStorage.id,
    (r) => ({ rawData: r.rawData }));

  total += await backfillTable("ternary_storage", ternaryStorage, ternaryStorage.id,
    (r) => ({ compressedData: r.compressedData }));

  total += await backfillTable("whitepapers", whitepapers, whitepapers.id,
    (r) => ({ content: r.content, summary: r.summary, author: r.author }));

  total += await backfillTable("api_keys", apiKeys, apiKeys.id,
    (r) => ({ name: r.name, owner: r.owner, scopes: r.scopes, entityName: r.entityName, project: r.project, department: r.department, tags: r.tags, notes: r.notes }));

  total += await backfillTable("api_key_logs", apiKeyLogs, apiKeyLogs.id,
    (r) => ({ ipAddress: r.ipAddress, endpoint: r.endpoint }));

  total += await backfillTable("api_key_audit_events", apiKeyAuditEvents, apiKeyAuditEvents.id,
    (r) => ({ actorId: r.actorId, actorEmail: r.actorEmail, details: r.details, ipAddress: r.ipAddress }));

  total += await backfillTable("developer_signups", developerSignups, developerSignups.id,
    (r) => ({ email: r.email, name: r.name, company: r.company, interest: r.interest }));

  total += await backfillTable("compressed_documents", compressedDocuments, compressedDocuments.id,
    (r) => ({ content: r.content }));

  total += await backfillTable("agent_array_reports", agentArrayReports, agentArrayReports.id,
    (r) => ({ prompt: r.prompt, unifiedReport: r.unifiedReport, translations: r.translations, executiveSummary: r.executiveSummary, layer2Sections: r.layer2Sections }));

  total += await backfillTable("data_subject_requests", dataSubjectRequests, dataSubjectRequests.id,
    (r) => ({ responseData: r.responseData }));

  total += await backfillTable("security_audit_log", securityAuditLog, securityAuditLog.id,
    (r) => ({ actor: r.actor, description: r.description, evidence: r.evidence || null, ipAddress: r.ipAddress, userId: r.userId }));

  total += await backfillTable("threat_model_entries", threatModelEntries, threatModelEntries.id,
    (r) => ({ description: r.description, controls: r.controls, notes: r.notes, attackVector: r.attackVector }));

  total += await backfillTable("coherence_logs", coherenceLogs, coherenceLogs.id,
    (r) => ({ subIndices: r.subIndices, moduleOutputs: r.moduleOutputs }));

  total += await backfillTable("crs_relay_nodes", crsRelayNodes, crsRelayNodes.id,
    (r) => ({ endpoint: r.endpoint, tlDsaPk: r.tlDsaPk }));

  total += await backfillTable("deployment_records", deploymentRecords, deploymentRecords.id,
    (r) => ({ hostname: r.hostname, ip: r.ip, daemons: r.daemons, binaryPath: r.binaryPath, logDir: r.logDir, identityBase: r.identityBase, deployer: r.deployer }));

  console.log(`\nDone — ${total} total rows encrypted.`);
  process.exit(0);
}

main().catch((err) => {
  console.error("Backfill failed:", err);
  process.exit(1);
});
