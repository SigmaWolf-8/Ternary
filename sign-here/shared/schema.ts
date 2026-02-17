import { sql } from "drizzle-orm";
import { pgTable, text, varchar, integer, timestamp, jsonb, boolean } from "drizzle-orm/pg-core";
import { relations } from "drizzle-orm";
import { createInsertSchema } from "drizzle-zod";
import { z } from "zod";

export const tenants = pgTable("tenants", {
  id: varchar("id").primaryKey().default(sql`gen_random_uuid()`),
  name: text("name").notNull(),
  createdAt: timestamp("created_at").defaultNow(),
});

export const users = pgTable("users", {
  id: varchar("id").primaryKey().default(sql`gen_random_uuid()`),
  tenantId: varchar("tenant_id"),
  username: text("username").notNull().unique(),
  email: text("email").notNull().default(""),
  password: text("password").notNull(),
  role: text("role").notNull().default("signer"),
});

export const envelopes = pgTable("envelopes", {
  id: varchar("id").primaryKey().default(sql`gen_random_uuid()`),
  tenantId: varchar("tenant_id"),
  plenumDocId: text("plenum_doc_id"),
  title: text("title").notNull(),
  description: text("description"),
  status: text("status").notNull().default("draft"),
  pdfData: text("pdf_data"),
  pageCount: integer("page_count").notNull().default(1),
  zkProof: text("zk_proof"),
  createdAt: timestamp("created_at").defaultNow().notNull(),
  updatedAt: timestamp("updated_at").defaultNow().notNull(),
});

export const recipients = pgTable("recipients", {
  id: varchar("id").primaryKey().default(sql`gen_random_uuid()`),
  envelopeId: varchar("envelope_id").notNull(),
  name: text("name").notNull(),
  email: text("email").notNull(),
  role: text("role").notNull().default("signer"),
  status: text("status").notNull().default("pending"),
  signedAt: timestamp("signed_at"),
  sortOrder: integer("sort_order").notNull().default(0),
});

export const fields = pgTable("fields", {
  id: varchar("id").primaryKey().default(sql`gen_random_uuid()`),
  envelopeId: varchar("envelope_id").notNull(),
  recipientId: varchar("recipient_id"),
  type: text("type").notNull(),
  label: text("label"),
  page: integer("page").notNull().default(1),
  x: integer("x").notNull(),
  y: integer("y").notNull(),
  width: integer("width").notNull(),
  height: integer("height").notNull(),
  value: text("value"),
  required: boolean("required").notNull().default(true),
  dependsOnFieldId: varchar("depends_on_field_id"),
  dependsOnValue: text("depends_on_value"),
});

export const auditLogs = pgTable("audit_logs", {
  id: varchar("id").primaryKey().default(sql`gen_random_uuid()`),
  envelopeId: varchar("envelope_id").notNull(),
  tenantId: varchar("tenant_id"),
  action: text("action").notNull(),
  actorName: text("actor_name").notNull(),
  details: text("details"),
  hpTpTimestamp: text("hptp_timestamp"),
  metadata: jsonb("metadata"),
  createdAt: timestamp("created_at").defaultNow().notNull(),
});

export const templates = pgTable("templates", {
  id: varchar("id").primaryKey().default(sql`gen_random_uuid()`),
  tenantId: varchar("tenant_id"),
  name: text("name").notNull(),
  description: text("description"),
  category: text("category"),
  tags: jsonb("tags").$type<string[]>().default([]),
  fieldDefs: jsonb("field_defs").$type<Array<{ type: string; label: string; page: number; x: number; y: number; width: number; height: number; required: boolean }>>().default([]),
  isPublic: boolean("is_public").notNull().default(false),
  forkedFromId: varchar("forked_from_id"),
  sourceEnvelopeId: varchar("source_envelope_id"),
  createdAt: timestamp("created_at").defaultNow().notNull(),
  updatedAt: timestamp("updated_at").defaultNow().notNull(),
});

export const templatesRelations = relations(templates, ({ one }) => ({
  tenant: one(tenants, { fields: [templates.tenantId], references: [tenants.id] }),
}));

export const envelopesRelations = relations(envelopes, ({ one, many }) => ({
  tenant: one(tenants, { fields: [envelopes.tenantId], references: [tenants.id] }),
  audits: many(auditLogs),
  recipients: many(recipients),
  fieldsList: many(fields),
}));

export const auditLogsRelations = relations(auditLogs, ({ one }) => ({
  envelope: one(envelopes, { fields: [auditLogs.envelopeId], references: [envelopes.id] }),
  tenant: one(tenants, { fields: [auditLogs.tenantId], references: [tenants.id] }),
}));

export const recipientsRelations = relations(recipients, ({ one }) => ({
  envelope: one(envelopes, { fields: [recipients.envelopeId], references: [envelopes.id] }),
}));

export const fieldsRelations = relations(fields, ({ one }) => ({
  envelope: one(envelopes, { fields: [fields.envelopeId], references: [envelopes.id] }),
}));

export const insertTenantSchema = createInsertSchema(tenants).omit({ id: true, createdAt: true });

export const insertUserSchema = createInsertSchema(users).pick({
  username: true,
  password: true,
});

export const insertEnvelopeSchema = createInsertSchema(envelopes).omit({
  id: true,
  createdAt: true,
  updatedAt: true,
});

export const insertRecipientSchema = createInsertSchema(recipients).omit({
  id: true,
  signedAt: true,
});

export const insertFieldSchema = createInsertSchema(fields).omit({
  id: true,
});

export const insertAuditLogSchema = createInsertSchema(auditLogs).omit({
  id: true,
  createdAt: true,
});

export const insertTemplateSchema = createInsertSchema(templates).omit({
  id: true,
  createdAt: true,
  updatedAt: true,
});

export type InsertTenant = z.infer<typeof insertTenantSchema>;
export type Tenant = typeof tenants.$inferSelect;
export type InsertUser = z.infer<typeof insertUserSchema>;
export type User = typeof users.$inferSelect;
export type Envelope = typeof envelopes.$inferSelect;
export type InsertEnvelope = z.infer<typeof insertEnvelopeSchema>;
export type Recipient = typeof recipients.$inferSelect;
export type InsertRecipient = z.infer<typeof insertRecipientSchema>;
export type Field = typeof fields.$inferSelect;
export type InsertField = z.infer<typeof insertFieldSchema>;
export type AuditLog = typeof auditLogs.$inferSelect;
export type InsertAuditLog = z.infer<typeof insertAuditLogSchema>;
export type Template = typeof templates.$inferSelect;
export type InsertTemplate = z.infer<typeof insertTemplateSchema>;
