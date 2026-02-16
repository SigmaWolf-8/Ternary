import { sql } from "drizzle-orm";
import { pgTable, text, varchar, integer, timestamp, jsonb, boolean } from "drizzle-orm/pg-core";
import { createInsertSchema } from "drizzle-zod";
import { z } from "zod";

export const users = pgTable("users", {
  id: varchar("id").primaryKey().default(sql`gen_random_uuid()`),
  username: text("username").notNull().unique(),
  password: text("password").notNull(),
});

export const envelopes = pgTable("envelopes", {
  id: varchar("id").primaryKey().default(sql`gen_random_uuid()`),
  title: text("title").notNull(),
  description: text("description"),
  status: text("status").notNull().default("draft"),
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
});

export const auditLogs = pgTable("audit_logs", {
  id: varchar("id").primaryKey().default(sql`gen_random_uuid()`),
  envelopeId: varchar("envelope_id").notNull(),
  action: text("action").notNull(),
  actorName: text("actor_name").notNull(),
  details: text("details"),
  createdAt: timestamp("created_at").defaultNow().notNull(),
});

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
