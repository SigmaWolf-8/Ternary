import { eq, desc, or, sql } from "drizzle-orm";
import { db } from "./db";
import {
  tenants,
  users,
  envelopes,
  recipients,
  fields,
  auditLogs,
  templates,
  type Tenant,
  type InsertTenant,
  type User,
  type InsertUser,
  type Envelope,
  type InsertEnvelope,
  type Recipient,
  type InsertRecipient,
  type Field,
  type InsertField,
  type AuditLog,
  type InsertAuditLog,
  type Template,
  type InsertTemplate,
} from "@shared/schema";

export interface IStorage {
  createTenant(tenant: InsertTenant): Promise<Tenant>;
  getTenant(id: string): Promise<Tenant | undefined>;
  getAllTenants(): Promise<Tenant[]>;

  getUser(id: string): Promise<User | undefined>;
  getUserByUsername(username: string): Promise<User | undefined>;
  createUser(user: InsertUser): Promise<User>;
  getAllUsers(): Promise<User[]>;
  updateUser(id: string, data: Partial<User>): Promise<User | undefined>;
  deleteUser(id: string): Promise<void>;

  getEnvelopes(): Promise<Envelope[]>;
  getEnvelope(id: string): Promise<Envelope | undefined>;
  createEnvelope(envelope: InsertEnvelope): Promise<Envelope>;
  updateEnvelope(id: string, data: Partial<InsertEnvelope>): Promise<Envelope | undefined>;
  deleteEnvelope(id: string): Promise<void>;

  getRecipientsByEnvelope(envelopeId: string): Promise<Recipient[]>;
  getRecipient(id: string): Promise<Recipient | undefined>;
  createRecipient(recipient: InsertRecipient): Promise<Recipient>;
  updateRecipient(id: string, data: Partial<Recipient>): Promise<Recipient | undefined>;
  deleteRecipient(id: string): Promise<void>;

  getFieldsByEnvelope(envelopeId: string): Promise<Field[]>;
  createField(field: InsertField): Promise<Field>;
  updateField(id: string, data: Partial<Field>): Promise<Field | undefined>;
  deleteFieldsByEnvelope(envelopeId: string): Promise<void>;

  getAuditLogsByEnvelope(envelopeId: string): Promise<AuditLog[]>;
  createAuditLog(log: InsertAuditLog): Promise<AuditLog>;

  getTemplates(tenantId?: string): Promise<Template[]>;
  getTemplate(id: string): Promise<Template | undefined>;
  createTemplate(template: InsertTemplate): Promise<Template>;
  updateTemplate(id: string, data: Partial<InsertTemplate>): Promise<Template | undefined>;
  deleteTemplate(id: string): Promise<void>;
}

export class DatabaseStorage implements IStorage {
  async createTenant(tenant: InsertTenant): Promise<Tenant> {
    const [created] = await db.insert(tenants).values(tenant).returning();
    return created;
  }

  async getTenant(id: string): Promise<Tenant | undefined> {
    const [tenant] = await db.select().from(tenants).where(eq(tenants.id, id));
    return tenant;
  }

  async getAllTenants(): Promise<Tenant[]> {
    return db.select().from(tenants).orderBy(desc(tenants.createdAt));
  }

  async getUser(id: string): Promise<User | undefined> {
    const [user] = await db.select().from(users).where(eq(users.id, id));
    return user;
  }

  async getUserByUsername(username: string): Promise<User | undefined> {
    const [user] = await db.select().from(users).where(eq(users.username, username));
    return user;
  }

  async createUser(insertUser: InsertUser): Promise<User> {
    const [user] = await db.insert(users).values(insertUser).returning();
    return user;
  }

  async getAllUsers(): Promise<User[]> {
    return db.select().from(users);
  }

  async updateUser(id: string, data: Partial<User>): Promise<User | undefined> {
    const [updated] = await db.update(users).set(data).where(eq(users.id, id)).returning();
    return updated;
  }

  async deleteUser(id: string): Promise<void> {
    await db.delete(users).where(eq(users.id, id));
  }

  async getEnvelopes(): Promise<Envelope[]> {
    const rows = await db.select({
      id: envelopes.id,
      tenantId: envelopes.tenantId,
      plenumDocId: envelopes.plenumDocId,
      title: envelopes.title,
      description: envelopes.description,
      status: envelopes.status,
      pdfData: sql<string | null>`CASE WHEN ${envelopes.pdfData} IS NOT NULL THEN 'has_pdf' ELSE NULL END`.as("pdf_data"),
      pageCount: envelopes.pageCount,
      zkProof: envelopes.zkProof,
      createdAt: envelopes.createdAt,
      updatedAt: envelopes.updatedAt,
    }).from(envelopes).orderBy(desc(envelopes.createdAt));
    return rows as Envelope[];
  }

  async getEnvelope(id: string): Promise<Envelope | undefined> {
    const [envelope] = await db.select().from(envelopes).where(eq(envelopes.id, id));
    return envelope;
  }

  async createEnvelope(envelope: InsertEnvelope): Promise<Envelope> {
    const [created] = await db.insert(envelopes).values(envelope).returning();
    return created;
  }

  async updateEnvelope(id: string, data: Partial<InsertEnvelope>): Promise<Envelope | undefined> {
    const [updated] = await db
      .update(envelopes)
      .set({ ...data, updatedAt: new Date() })
      .where(eq(envelopes.id, id))
      .returning();
    return updated;
  }

  async deleteEnvelope(id: string): Promise<void> {
    await db.delete(fields).where(eq(fields.envelopeId, id));
    await db.delete(recipients).where(eq(recipients.envelopeId, id));
    await db.delete(auditLogs).where(eq(auditLogs.envelopeId, id));
    await db.delete(envelopes).where(eq(envelopes.id, id));
  }

  async getRecipientsByEnvelope(envelopeId: string): Promise<Recipient[]> {
    return db.select().from(recipients).where(eq(recipients.envelopeId, envelopeId));
  }

  async getRecipient(id: string): Promise<Recipient | undefined> {
    const [recipient] = await db.select().from(recipients).where(eq(recipients.id, id));
    return recipient;
  }

  async createRecipient(recipient: InsertRecipient): Promise<Recipient> {
    const [created] = await db.insert(recipients).values(recipient).returning();
    return created;
  }

  async updateRecipient(id: string, data: Partial<Recipient>): Promise<Recipient | undefined> {
    const [updated] = await db
      .update(recipients)
      .set(data)
      .where(eq(recipients.id, id))
      .returning();
    return updated;
  }

  async deleteRecipient(id: string): Promise<void> {
    await db.delete(recipients).where(eq(recipients.id, id));
  }

  async getFieldsByEnvelope(envelopeId: string): Promise<Field[]> {
    return db.select().from(fields).where(eq(fields.envelopeId, envelopeId));
  }

  async createField(field: InsertField): Promise<Field> {
    const [created] = await db.insert(fields).values(field).returning();
    return created;
  }

  async updateField(id: string, data: Partial<Field>): Promise<Field | undefined> {
    const [updated] = await db
      .update(fields)
      .set(data)
      .where(eq(fields.id, id))
      .returning();
    return updated;
  }

  async deleteFieldsByEnvelope(envelopeId: string): Promise<void> {
    await db.delete(fields).where(eq(fields.envelopeId, envelopeId));
  }

  async getAuditLogsByEnvelope(envelopeId: string): Promise<AuditLog[]> {
    return db
      .select()
      .from(auditLogs)
      .where(eq(auditLogs.envelopeId, envelopeId))
      .orderBy(desc(auditLogs.createdAt));
  }

  async createAuditLog(log: InsertAuditLog): Promise<AuditLog> {
    const [created] = await db.insert(auditLogs).values(log).returning();
    return created;
  }

  async getTemplates(tenantId?: string): Promise<Template[]> {
    if (tenantId) {
      return db.select().from(templates)
        .where(or(eq(templates.isPublic, true), eq(templates.tenantId, tenantId)))
        .orderBy(desc(templates.createdAt));
    }
    return db.select().from(templates).orderBy(desc(templates.createdAt));
  }

  async getTemplate(id: string): Promise<Template | undefined> {
    const [template] = await db.select().from(templates).where(eq(templates.id, id));
    return template;
  }

  async createTemplate(template: InsertTemplate): Promise<Template> {
    const [created] = await db.insert(templates).values(template as any).returning();
    return created;
  }

  async updateTemplate(id: string, data: Partial<InsertTemplate>): Promise<Template | undefined> {
    const [updated] = await db
      .update(templates)
      .set({ ...data, updatedAt: new Date() } as any)
      .where(eq(templates.id, id))
      .returning();
    return updated;
  }

  async deleteTemplate(id: string): Promise<void> {
    await db.delete(templates).where(eq(templates.id, id));
  }
}

export const storage = new DatabaseStorage();
