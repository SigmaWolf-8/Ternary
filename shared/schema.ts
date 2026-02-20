/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL
 * All Rights Reserved.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * Unauthorized copying, modification, distribution, or use of this file,
 * via any medium, is strictly prohibited without the prior written
 * permission of Capomastro Holdings Ltd.
 *
 * See LICENSE in the repository root for full terms.
 */

import { sql } from "drizzle-orm";
import { pgTable, text, varchar, integer, timestamp, real, jsonb, serial, boolean } from "drizzle-orm/pg-core";
import { createInsertSchema } from "drizzle-zod";
import { z } from "zod";

export * from "./models/auth";

export const demoSessions = pgTable("demo_sessions", {
  id: serial("id").primaryKey(),
  sessionId: varchar("session_id").notNull().unique(),
  datasetName: varchar("dataset_name").notNull(),
  createdAt: timestamp("created_at").defaultNow().notNull(),
});

export const binaryStorage = pgTable("binary_storage", {
  id: serial("id").primaryKey(),
  sessionId: varchar("session_id").notNull(),
  dataType: varchar("data_type").notNull(),
  rawData: jsonb("raw_data").notNull(),
  sizeBytes: integer("size_bytes").notNull(),
  rowCount: integer("row_count").notNull(),
  createdAt: timestamp("created_at").defaultNow().notNull(),
});

export const ternaryStorage = pgTable("ternary_storage", {
  id: serial("id").primaryKey(),
  sessionId: varchar("session_id").notNull(),
  dataType: varchar("data_type").notNull(),
  compressedData: text("compressed_data").notNull(),
  originalSizeBytes: integer("original_size_bytes").notNull(),
  compressedSizeBytes: integer("compressed_size_bytes").notNull(),
  compressionRatio: real("compression_ratio").notNull(),
  rowCount: integer("row_count").notNull(),
  createdAt: timestamp("created_at").defaultNow().notNull(),
});

export const compressionBenchmarks = pgTable("compression_benchmarks", {
  id: serial("id").primaryKey(),
  sessionId: varchar("session_id").notNull(),
  datasetName: varchar("dataset_name").notNull(),
  binaryStorageId: integer("binary_storage_id").notNull(),
  ternaryStorageId: integer("ternary_storage_id").notNull(),
  binarySizeBytes: integer("binary_size_bytes").notNull(),
  ternarySizeBytes: integer("ternary_size_bytes").notNull(),
  savingsPercent: real("savings_percent").notNull(),
  processingTimeMs: integer("processing_time_ms").notNull(),
  createdAt: timestamp("created_at").defaultNow().notNull(),
});

export const fileUploads = pgTable("file_uploads", {
  id: serial("id").primaryKey(),
  sessionId: varchar("session_id").notNull(),
  fileName: varchar("file_name").notNull(),
  fileType: varchar("file_type").notNull(),
  originalSizeBytes: integer("original_size_bytes").notNull(),
  rowCount: integer("row_count").notNull(),
  createdAt: timestamp("created_at").defaultNow().notNull(),
});

export const compressionHistory = pgTable("compression_history", {
  id: serial("id").primaryKey(),
  sessionId: varchar("session_id").notNull(),
  datasetName: varchar("dataset_name").notNull(),
  sourceType: varchar("source_type").notNull(),
  binarySizeBytes: integer("binary_size_bytes").notNull(),
  ternarySizeBytes: integer("ternary_size_bytes").notNull(),
  savingsPercent: real("savings_percent").notNull(),
  rowCount: integer("row_count").notNull(),
  processingTimeMs: integer("processing_time_ms").notNull(),
  createdAt: timestamp("created_at").defaultNow().notNull(),
});

export const whitepapers = pgTable("whitepapers", {
  id: serial("id").primaryKey(),
  version: varchar("version").notNull(),
  title: varchar("title").notNull(),
  content: text("content").notNull(),
  summary: text("summary"),
  author: varchar("author"),
  isActive: integer("is_active").default(1).notNull(),
  createdAt: timestamp("created_at").defaultNow().notNull(),
  updatedAt: timestamp("updated_at").defaultNow().notNull(),
});

export const apiKeys = pgTable("api_keys", {
  id: varchar("id").primaryKey().default(sql`gen_random_uuid()`),
  keyPrefix: varchar("key_prefix", { length: 12 }).notNull(),
  keyHash: text("key_hash").notNull().unique(),
  name: varchar("name", { length: 255 }).notNull(),
  owner: varchar("owner", { length: 255 }).notNull(),
  scopes: jsonb("scopes").notNull().$type<string[]>(),
  isActive: boolean("is_active").default(true).notNull(),
  expiresAt: timestamp("expires_at"),
  revokedAt: timestamp("revoked_at"),
  lastUsedAt: timestamp("last_used_at"),
  usageCount: integer("usage_count").default(0).notNull(),
  rotationScheduledAt: timestamp("rotation_scheduled_at"),
  previousKeyId: varchar("previous_key_id", { length: 255 }),
  rateLimitTier: varchar("rate_limit_tier", { length: 20 }).default("research").notNull(),
  rateLimitRpm: integer("rate_limit_rpm").default(100).notNull(),
  entityType: varchar("entity_type", { length: 50 }),
  entityName: varchar("entity_name", { length: 255 }),
  project: varchar("project", { length: 255 }),
  department: varchar("department", { length: 255 }),
  tags: jsonb("tags").$type<string[]>().default([]),
  notes: text("notes"),
  createdAt: timestamp("created_at").defaultNow().notNull(),
});

export const apiKeyLogs = pgTable("api_key_logs", {
  id: serial("id").primaryKey(),
  keyId: varchar("key_id").notNull(),
  endpoint: varchar("endpoint", { length: 512 }).notNull(),
  method: varchar("method", { length: 10 }).notNull(),
  statusCode: integer("status_code"),
  ipAddress: varchar("ip_address", { length: 45 }),
  createdAt: timestamp("created_at").defaultNow().notNull(),
});

export const apiKeyAuditEvents = pgTable("api_key_audit_events", {
  id: serial("id").primaryKey(),
  keyId: varchar("key_id").notNull(),
  eventType: varchar("event_type", { length: 50 }).notNull(),
  actorId: varchar("actor_id", { length: 255 }).notNull(),
  actorEmail: varchar("actor_email", { length: 255 }),
  details: jsonb("details").$type<Record<string, unknown>>(),
  ipAddress: varchar("ip_address", { length: 45 }),
  createdAt: timestamp("created_at").defaultNow().notNull(),
});

export const insertApiKeySchema = createInsertSchema(apiKeys).omit({ id: true, createdAt: true, revokedAt: true, lastUsedAt: true, usageCount: true, rotationScheduledAt: true, previousKeyId: true });
export const insertApiKeyLogSchema = createInsertSchema(apiKeyLogs).omit({ id: true, createdAt: true });
export const insertApiKeyAuditEventSchema = createInsertSchema(apiKeyAuditEvents).omit({ id: true, createdAt: true });
export type InsertApiKey = z.infer<typeof insertApiKeySchema>;
export type ApiKey = typeof apiKeys.$inferSelect;
export type InsertApiKeyLog = z.infer<typeof insertApiKeyLogSchema>;
export type ApiKeyLog = typeof apiKeyLogs.$inferSelect;
export type ApiKeyAuditEvent = typeof apiKeyAuditEvents.$inferSelect;

export const insertDemoSessionSchema = createInsertSchema(demoSessions).omit({ id: true, createdAt: true });
export const insertBinaryStorageSchema = createInsertSchema(binaryStorage).omit({ id: true, createdAt: true });
export const insertTernaryStorageSchema = createInsertSchema(ternaryStorage).omit({ id: true, createdAt: true });
export const insertCompressionBenchmarkSchema = createInsertSchema(compressionBenchmarks).omit({ id: true, createdAt: true });
export const insertFileUploadSchema = createInsertSchema(fileUploads).omit({ id: true, createdAt: true });
export const insertCompressionHistorySchema = createInsertSchema(compressionHistory).omit({ id: true, createdAt: true });
export const developerSignups = pgTable("developer_signups", {
  id: serial("id").primaryKey(),
  email: varchar("email").notNull().unique(),
  name: varchar("name"),
  company: varchar("company"),
  interest: varchar("interest"),
  createdAt: timestamp("created_at").defaultNow().notNull(),
});

export const insertWhitepaperSchema = createInsertSchema(whitepapers).omit({ id: true, createdAt: true, updatedAt: true });
export const insertDeveloperSignupSchema = createInsertSchema(developerSignups).omit({ id: true, createdAt: true });

export const compressedDocuments = pgTable("compressed_documents", {
  id: serial("id").primaryKey(),
  title: varchar("title").notNull(),
  content: text("content").notNull(),
  isCompressed: integer("is_compressed").default(0).notNull(),
  isEncrypted: integer("is_encrypted").default(0).notNull(),
  encryptionMode: varchar("encryption_mode"),
  originalSizeBytes: integer("original_size_bytes"),
  storedSizeBytes: integer("stored_size_bytes"),
  compressionRatio: real("compression_ratio"),
  createdAt: timestamp("created_at").defaultNow().notNull(),
});

export const insertCompressedDocumentSchema = createInsertSchema(compressedDocuments).omit({ id: true, createdAt: true });

export const agentArrayReports = pgTable("agent_array_reports", {
  id: serial("id").primaryKey(),
  prompt: text("prompt").notNull(),
  tribonacciHash: varchar("tribonacci_hash").notNull(),
  unifiedReport: text("unified_report").notNull(),
  translations: jsonb("translations").notNull(),
  executiveSummary: jsonb("executive_summary"),
  layer2Sections: jsonb("layer2_sections"),
  agentCount: integer("agent_count").notNull(),
  successCount: integer("success_count").notNull(),
  totalDurationMs: integer("total_duration_ms").notNull(),
  createdAt: timestamp("created_at").defaultNow().notNull(),
});

export const insertAgentArrayReportSchema = createInsertSchema(agentArrayReports).omit({ id: true, createdAt: true });

export const dataSubjectRequests = pgTable("data_subject_requests", {
  id: serial("id").primaryKey(),
  userId: varchar("user_id").notNull(),
  requestType: varchar("request_type").notNull(),
  status: varchar("status").notNull().default("pending"),
  requestedAt: timestamp("requested_at").defaultNow().notNull(),
  completedAt: timestamp("completed_at"),
  responseData: jsonb("response_data"),
});

export const insertDataSubjectRequestSchema = createInsertSchema(dataSubjectRequests).omit({ id: true, requestedAt: true, completedAt: true });
export type InsertDataSubjectRequest = z.infer<typeof insertDataSubjectRequestSchema>;
export type DataSubjectRequest = typeof dataSubjectRequests.$inferSelect;

export type InsertAgentArrayReport = z.infer<typeof insertAgentArrayReportSchema>;
export type AgentArrayReport = typeof agentArrayReports.$inferSelect;

export type InsertDemoSession = z.infer<typeof insertDemoSessionSchema>;
export type DemoSession = typeof demoSessions.$inferSelect;
export type InsertBinaryStorage = z.infer<typeof insertBinaryStorageSchema>;
export type BinaryStorage = typeof binaryStorage.$inferSelect;
export type InsertTernaryStorage = z.infer<typeof insertTernaryStorageSchema>;
export type TernaryStorage = typeof ternaryStorage.$inferSelect;
export type InsertCompressionBenchmark = z.infer<typeof insertCompressionBenchmarkSchema>;
export type CompressionBenchmark = typeof compressionBenchmarks.$inferSelect;
export type InsertFileUpload = z.infer<typeof insertFileUploadSchema>;
export type FileUpload = typeof fileUploads.$inferSelect;
export type InsertCompressionHistory = z.infer<typeof insertCompressionHistorySchema>;
export type CompressionHistory = typeof compressionHistory.$inferSelect;
export type InsertWhitepaper = z.infer<typeof insertWhitepaperSchema>;
export type Whitepaper = typeof whitepapers.$inferSelect;
export type InsertDeveloperSignup = z.infer<typeof insertDeveloperSignupSchema>;
export type DeveloperSignup = typeof developerSignups.$inferSelect;
export type InsertCompressedDocument = z.infer<typeof insertCompressedDocumentSchema>;
export type CompressedDocument = typeof compressedDocuments.$inferSelect;

export const securityAuditLog = pgTable("security_audit_log", {
  id: serial("id").primaryKey(),
  severity: varchar("severity", { length: 20 }).notNull(),
  category: varchar("category", { length: 50 }).notNull(),
  eventType: varchar("event_type", { length: 100 }).notNull(),
  actor: varchar("actor", { length: 255 }),
  description: text("description").notNull(),
  affectedComponent: varchar("affected_component", { length: 255 }),
  evidence: jsonb("evidence").$type<Record<string, unknown>>(),
  resolutionStatus: varchar("resolution_status", { length: 20 }).default("unresolved").notNull(),
  resolutionNotes: text("resolution_notes"),
  resolvedBy: varchar("resolved_by", { length: 255 }),
  resolvedAt: timestamp("resolved_at"),
  ipAddress: varchar("ip_address", { length: 45 }),
  userId: varchar("user_id", { length: 255 }),
  createdAt: timestamp("created_at").defaultNow().notNull(),
});

export const hptpAnomalyEvents = pgTable("hptp_anomaly_events", {
  id: serial("id").primaryKey(),
  anomalyType: varchar("anomaly_type", { length: 50 }).notNull(),
  severityScore: real("severity_score").notNull(),
  thresholdValue: real("threshold_value").notNull(),
  observedValue: real("observed_value").notNull(),
  variancePercentage: real("variance_percentage"),
  fallbackChain: jsonb("fallback_chain").$type<Record<string, unknown>>().notNull(),
  activeTier: varchar("active_tier", { length: 20 }).notNull(),
  escalationTriggered: boolean("escalation_triggered").default(false).notNull(),
  escalationTimestamp: timestamp("escalation_timestamp"),
  auditLogId: integer("audit_log_id"),
  resolved: boolean("resolved").default(false).notNull(),
  resolvedAt: timestamp("resolved_at"),
  createdAt: timestamp("created_at").defaultNow().notNull(),
});

export const threatModelEntries = pgTable("threat_model_entries", {
  id: serial("id").primaryKey(),
  threatId: varchar("threat_id", { length: 50 }).notNull().unique(),
  threatName: varchar("threat_name", { length: 255 }).notNull(),
  description: text("description"),
  category: varchar("category", { length: 50 }).notNull(),
  attackVector: varchar("attack_vector", { length: 100 }),
  likelihood: varchar("likelihood", { length: 20 }).notNull(),
  impact: varchar("impact", { length: 20 }).notNull(),
  riskScore: real("risk_score").notNull(),
  mitigationStatus: varchar("mitigation_status", { length: 20 }).notNull(),
  controls: jsonb("controls").$type<Array<{ controlId: string; controlName: string; status: string; evidence?: string }>>(),
  residualRisk: real("residual_risk"),
  notes: text("notes"),
  createdBy: varchar("created_by", { length: 255 }),
  updatedBy: varchar("updated_by", { length: 255 }),
  createdAt: timestamp("created_at").defaultNow().notNull(),
  updatedAt: timestamp("updated_at").defaultNow().notNull(),
});

export const implementationStatus = pgTable("implementation_status", {
  id: serial("id").primaryKey(),
  componentName: varchar("component_name", { length: 255 }).notNull().unique(),
  category: varchar("category", { length: 50 }).notNull(),
  status: varchar("status", { length: 20 }).notNull(),
  completionPercentage: integer("completion_percentage").default(0).notNull(),
  description: text("description"),
  locTotal: integer("loc_total"),
  locTested: integer("loc_tested"),
  testCount: integer("test_count"),
  proofCount: integer("proof_count"),
  proofCoveragePercentage: real("proof_coverage_percentage"),
  githubPath: varchar("github_path", { length: 255 }),
  responsibleTeam: varchar("responsible_team", { length: 100 }),
  milestoneDate: varchar("milestone_date", { length: 20 }),
  summaryLine: text("summary_line"),
  externalAuditStatus: varchar("external_audit_status", { length: 50 }),
  externalAuditor: varchar("external_auditor", { length: 100 }),
  lastUpdated: timestamp("last_updated").defaultNow().notNull(),
  createdAt: timestamp("created_at").defaultNow().notNull(),
  updatedAt: timestamp("updated_at").defaultNow().notNull(),
});

export const coherenceLogs = pgTable("coherence_logs", {
  id: serial("id").primaryKey(),
  logId: varchar("log_id", { length: 64 }).notNull().unique(),
  cvp: real("cvp").notNull(),
  subIndices: jsonb("sub_indices").notNull().$type<Record<string, { value: number; source: string }>>(),
  moduleOutputs: jsonb("module_outputs").$type<Record<string, unknown>>(),
  phaseAdvance: jsonb("phase_advance").$type<Record<string, unknown>>(),
  governorStatus: varchar("governor_status", { length: 50 }),
  sourceTimestamp: timestamp("source_timestamp"),
  createdAt: timestamp("created_at").defaultNow().notNull(),
});

export const insertCoherenceLogSchema = createInsertSchema(coherenceLogs).omit({ id: true, createdAt: true });
export type InsertCoherenceLog = z.infer<typeof insertCoherenceLogSchema>;
export type CoherenceLog = typeof coherenceLogs.$inferSelect;

export const insertSecurityAuditLogSchema = createInsertSchema(securityAuditLog).omit({ id: true, createdAt: true });
export const insertHptpAnomalyEventSchema = createInsertSchema(hptpAnomalyEvents).omit({ id: true, createdAt: true });
export const insertThreatModelEntrySchema = createInsertSchema(threatModelEntries).omit({ id: true, createdAt: true, updatedAt: true });
export const insertImplementationStatusSchema = createInsertSchema(implementationStatus).omit({ id: true, createdAt: true, updatedAt: true, lastUpdated: true });

export type SecurityAuditLogEntry = typeof securityAuditLog.$inferSelect;
export type InsertSecurityAuditLog = z.infer<typeof insertSecurityAuditLogSchema>;
export type HptpAnomalyEvent = typeof hptpAnomalyEvents.$inferSelect;
export type InsertHptpAnomalyEvent = z.infer<typeof insertHptpAnomalyEventSchema>;
export type ThreatModelEntry = typeof threatModelEntries.$inferSelect;
export type InsertThreatModelEntry = z.infer<typeof insertThreatModelEntrySchema>;
export type ImplementationStatusEntry = typeof implementationStatus.$inferSelect;
export type InsertImplementationStatus = z.infer<typeof insertImplementationStatusSchema>;
