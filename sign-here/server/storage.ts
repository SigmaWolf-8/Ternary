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
  wbsTags,
  envelopeWbsTags,
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
  type WbsTag,
  type InsertWbsTag,
  type EnvelopeWbsTag,
} from "@shared/schema";
import { encryptField, decryptField, encryptJson, decryptJson } from "./services/fieldCrypto";

function encryptTenant(t: InsertTenant): InsertTenant {
  return { ...t, name: encryptField(t.name) };
}
function decryptTenant<T extends Tenant>(t: T): T {
  return { ...t, name: decryptField(t.name) };
}

function encryptUserData(u: any): any {
  return {
    ...u,
    ...(u.username !== undefined && { username: encryptField(u.username) }),
    ...(u.email !== undefined && { email: encryptField(u.email) }),
    ...(u.password !== undefined && { password: encryptField(u.password) }),
  };
}
function decryptUser<T extends User>(u: T): T {
  return { ...u, username: decryptField(u.username), email: decryptField(u.email) };
}

function encryptEnvelopeData(e: any): any {
  return {
    ...e,
    ...(e.title !== undefined && { title: encryptField(e.title) }),
    ...(e.description !== undefined && e.description !== null && { description: encryptField(e.description) }),
    ...(e.zkProof !== undefined && e.zkProof !== null && { zkProof: encryptField(e.zkProof) }),
  };
}
function decryptEnvelope<T extends Envelope>(e: T): T {
  return {
    ...e,
    title: decryptField(e.title),
    description: e.description ? decryptField(e.description) : e.description,
    zkProof: e.zkProof ? decryptField(e.zkProof) : e.zkProof,
  };
}

function encryptRecipientData(r: any): any {
  return {
    ...r,
    ...(r.name !== undefined && { name: encryptField(r.name) }),
    ...(r.email !== undefined && { email: encryptField(r.email) }),
  };
}
function decryptRecipient<T extends Recipient>(r: T): T {
  return { ...r, name: decryptField(r.name), email: decryptField(r.email) };
}

function encryptFieldData(f: any): any {
  return {
    ...f,
    ...(f.value !== undefined && f.value !== null && { value: encryptField(f.value) }),
    ...(f.label !== undefined && f.label !== null && { label: encryptField(f.label) }),
    ...(f.dependsOnValue !== undefined && f.dependsOnValue !== null && { dependsOnValue: encryptField(f.dependsOnValue) }),
  };
}
function decryptFieldRow<T extends Field>(f: T): T {
  return {
    ...f,
    value: f.value ? decryptField(f.value) : f.value,
    label: f.label ? decryptField(f.label) : f.label,
    dependsOnValue: f.dependsOnValue ? decryptField(f.dependsOnValue) : f.dependsOnValue,
  };
}

function encryptAuditData(a: any): any {
  const result: any = { ...a };
  if (a.actorName !== undefined) result.actorName = encryptField(a.actorName);
  if (a.details !== undefined && a.details !== null) result.details = encryptField(a.details);
  if (a.metadata !== undefined && a.metadata !== null) {
    result.metadata = { _enc: encryptField(JSON.stringify(a.metadata)) };
  }
  return result;
}
function decryptAudit<T extends AuditLog>(a: T): T {
  const meta = a.metadata as any;
  let decryptedMeta = meta;
  if (meta && typeof meta === "object" && meta._enc) {
    try {
      decryptedMeta = JSON.parse(decryptField(meta._enc));
    } catch {
      decryptedMeta = meta;
    }
  } else if (typeof meta === "string" && meta.startsWith("fenc:")) {
    try {
      decryptedMeta = JSON.parse(decryptField(meta));
    } catch {
      decryptedMeta = meta;
    }
  }
  return {
    ...a,
    actorName: decryptField(a.actorName),
    details: a.details ? decryptField(a.details) : a.details,
    metadata: decryptedMeta,
  };
}

function encryptTemplateData(t: any): any {
  return {
    ...t,
    ...(t.name !== undefined && { name: encryptField(t.name) }),
    ...(t.description !== undefined && t.description !== null && { description: encryptField(t.description) }),
  };
}
function decryptTemplate<T extends Template>(t: T): T {
  return {
    ...t,
    name: decryptField(t.name),
    description: t.description ? decryptField(t.description) : t.description,
  };
}

function encryptWbsTagData(t: any): any {
  return {
    ...t,
    ...(t.name !== undefined && { name: encryptField(t.name) }),
  };
}
function decryptWbsTag<T extends WbsTag>(t: T): T {
  return { ...t, name: decryptField(t.name) };
}

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

  getWbsTags(tenantId?: string): Promise<WbsTag[]>;
  getWbsTag(id: string): Promise<WbsTag | undefined>;
  createWbsTag(tag: InsertWbsTag): Promise<WbsTag>;
  updateWbsTag(id: string, data: Partial<InsertWbsTag>): Promise<WbsTag | undefined>;
  deleteWbsTag(id: string): Promise<void>;

  getEnvelopeWbsTags(envelopeId: string): Promise<EnvelopeWbsTag[]>;
  setEnvelopeWbsTags(envelopeId: string, tagIds: string[]): Promise<EnvelopeWbsTag[]>;
  getAllEnvelopeWbsTags(): Promise<EnvelopeWbsTag[]>;
}

export class DatabaseStorage implements IStorage {
  async createTenant(tenant: InsertTenant): Promise<Tenant> {
    const [created] = await db.insert(tenants).values(encryptTenant(tenant)).returning();
    return decryptTenant(created);
  }

  async getTenant(id: string): Promise<Tenant | undefined> {
    const [tenant] = await db.select().from(tenants).where(eq(tenants.id, id));
    return tenant ? decryptTenant(tenant) : undefined;
  }

  async getAllTenants(): Promise<Tenant[]> {
    const rows = await db.select().from(tenants).orderBy(desc(tenants.createdAt));
    return rows.map(decryptTenant);
  }

  async getUser(id: string): Promise<User | undefined> {
    const [user] = await db.select().from(users).where(eq(users.id, id));
    return user ? decryptUser(user) : undefined;
  }

  async getUserByUsername(username: string): Promise<User | undefined> {
    const rows = await db.select().from(users);
    const decrypted = rows.map(decryptUser);
    return decrypted.find((u) => u.username === username);
  }

  async createUser(insertUser: InsertUser): Promise<User> {
    const [user] = await db.insert(users).values(encryptUserData(insertUser)).returning();
    return decryptUser(user);
  }

  async getAllUsers(): Promise<User[]> {
    const rows = await db.select().from(users);
    return rows.map(decryptUser);
  }

  async updateUser(id: string, data: Partial<User>): Promise<User | undefined> {
    const [updated] = await db.update(users).set(encryptUserData(data)).where(eq(users.id, id)).returning();
    return updated ? decryptUser(updated) : undefined;
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
      wbsTagId: envelopes.wbsTagId,
      createdAt: envelopes.createdAt,
      updatedAt: envelopes.updatedAt,
    }).from(envelopes).orderBy(desc(envelopes.createdAt));
    return (rows as Envelope[]).map(decryptEnvelope);
  }

  async getEnvelope(id: string): Promise<Envelope | undefined> {
    const [envelope] = await db.select().from(envelopes).where(eq(envelopes.id, id));
    return envelope ? decryptEnvelope(envelope) : undefined;
  }

  async createEnvelope(envelope: InsertEnvelope): Promise<Envelope> {
    const [created] = await db.insert(envelopes).values(encryptEnvelopeData(envelope)).returning();
    return decryptEnvelope(created);
  }

  async updateEnvelope(id: string, data: Partial<InsertEnvelope>): Promise<Envelope | undefined> {
    const [updated] = await db
      .update(envelopes)
      .set({ ...encryptEnvelopeData(data), updatedAt: new Date() })
      .where(eq(envelopes.id, id))
      .returning();
    return updated ? decryptEnvelope(updated) : undefined;
  }

  async deleteEnvelope(id: string): Promise<void> {
    await db.delete(fields).where(eq(fields.envelopeId, id));
    await db.delete(recipients).where(eq(recipients.envelopeId, id));
    await db.delete(auditLogs).where(eq(auditLogs.envelopeId, id));
    await db.delete(envelopes).where(eq(envelopes.id, id));
  }

  async getRecipientsByEnvelope(envelopeId: string): Promise<Recipient[]> {
    const rows = await db.select().from(recipients).where(eq(recipients.envelopeId, envelopeId));
    return rows.map(decryptRecipient);
  }

  async getRecipient(id: string): Promise<Recipient | undefined> {
    const [recipient] = await db.select().from(recipients).where(eq(recipients.id, id));
    return recipient ? decryptRecipient(recipient) : undefined;
  }

  async createRecipient(recipient: InsertRecipient): Promise<Recipient> {
    const [created] = await db.insert(recipients).values(encryptRecipientData(recipient)).returning();
    return decryptRecipient(created);
  }

  async updateRecipient(id: string, data: Partial<Recipient>): Promise<Recipient | undefined> {
    const [updated] = await db
      .update(recipients)
      .set(encryptRecipientData(data))
      .where(eq(recipients.id, id))
      .returning();
    return updated ? decryptRecipient(updated) : undefined;
  }

  async deleteRecipient(id: string): Promise<void> {
    await db.delete(recipients).where(eq(recipients.id, id));
  }

  async getFieldsByEnvelope(envelopeId: string): Promise<Field[]> {
    const rows = await db.select().from(fields).where(eq(fields.envelopeId, envelopeId));
    return rows.map(decryptFieldRow);
  }

  async createField(field: InsertField): Promise<Field> {
    const [created] = await db.insert(fields).values(encryptFieldData(field)).returning();
    return decryptFieldRow(created);
  }

  async updateField(id: string, data: Partial<Field>): Promise<Field | undefined> {
    const [updated] = await db
      .update(fields)
      .set(encryptFieldData(data))
      .where(eq(fields.id, id))
      .returning();
    return updated ? decryptFieldRow(updated) : undefined;
  }

  async deleteFieldsByEnvelope(envelopeId: string): Promise<void> {
    await db.delete(fields).where(eq(fields.envelopeId, envelopeId));
  }

  async getAuditLogsByEnvelope(envelopeId: string): Promise<AuditLog[]> {
    const rows = await db
      .select()
      .from(auditLogs)
      .where(eq(auditLogs.envelopeId, envelopeId))
      .orderBy(desc(auditLogs.createdAt));
    return rows.map(decryptAudit);
  }

  async createAuditLog(log: InsertAuditLog): Promise<AuditLog> {
    const [created] = await db.insert(auditLogs).values(encryptAuditData(log)).returning();
    return decryptAudit(created);
  }

  async getTemplates(tenantId?: string): Promise<Template[]> {
    let rows: Template[];
    if (tenantId) {
      rows = await db.select().from(templates)
        .where(or(eq(templates.isPublic, true), eq(templates.tenantId, tenantId)))
        .orderBy(desc(templates.createdAt));
    } else {
      rows = await db.select().from(templates).orderBy(desc(templates.createdAt));
    }
    return rows.map(decryptTemplate);
  }

  async getTemplate(id: string): Promise<Template | undefined> {
    const [template] = await db.select().from(templates).where(eq(templates.id, id));
    return template ? decryptTemplate(template) : undefined;
  }

  async createTemplate(template: InsertTemplate): Promise<Template> {
    const [created] = await db.insert(templates).values(encryptTemplateData(template) as any).returning();
    return decryptTemplate(created);
  }

  async updateTemplate(id: string, data: Partial<InsertTemplate>): Promise<Template | undefined> {
    const [updated] = await db
      .update(templates)
      .set({ ...encryptTemplateData(data), updatedAt: new Date() } as any)
      .where(eq(templates.id, id))
      .returning();
    return updated ? decryptTemplate(updated) : undefined;
  }

  async deleteTemplate(id: string): Promise<void> {
    await db.delete(templates).where(eq(templates.id, id));
  }

  async getWbsTags(tenantId?: string): Promise<WbsTag[]> {
    let rows: WbsTag[];
    if (tenantId) {
      rows = await db.select().from(wbsTags)
        .where(or(eq(wbsTags.tenantId, tenantId), sql`${wbsTags.tenantId} IS NULL`))
        .orderBy(wbsTags.sortOrder);
    } else {
      rows = await db.select().from(wbsTags).orderBy(wbsTags.sortOrder);
    }
    return rows.map(decryptWbsTag);
  }

  async getWbsTag(id: string): Promise<WbsTag | undefined> {
    const [tag] = await db.select().from(wbsTags).where(eq(wbsTags.id, id));
    return tag ? decryptWbsTag(tag) : undefined;
  }

  async createWbsTag(tag: InsertWbsTag): Promise<WbsTag> {
    const [created] = await db.insert(wbsTags).values(encryptWbsTagData(tag)).returning();
    return decryptWbsTag(created);
  }

  async updateWbsTag(id: string, data: Partial<InsertWbsTag>): Promise<WbsTag | undefined> {
    const [updated] = await db
      .update(wbsTags)
      .set(encryptWbsTagData(data))
      .where(eq(wbsTags.id, id))
      .returning();
    return updated ? decryptWbsTag(updated) : undefined;
  }

  async deleteWbsTag(id: string): Promise<void> {
    await db.delete(envelopeWbsTags).where(eq(envelopeWbsTags.wbsTagId, id));
    await db.delete(wbsTags).where(eq(wbsTags.id, id));
  }

  async getEnvelopeWbsTags(envelopeId: string): Promise<EnvelopeWbsTag[]> {
    return db.select().from(envelopeWbsTags).where(eq(envelopeWbsTags.envelopeId, envelopeId));
  }

  async setEnvelopeWbsTags(envelopeId: string, tagIds: string[]): Promise<EnvelopeWbsTag[]> {
    await db.delete(envelopeWbsTags).where(eq(envelopeWbsTags.envelopeId, envelopeId));
    if (tagIds.length === 0) return [];
    const rows = tagIds.map((wbsTagId) => ({ envelopeId, wbsTagId }));
    return db.insert(envelopeWbsTags).values(rows).returning();
  }

  async getAllEnvelopeWbsTags(): Promise<EnvelopeWbsTag[]> {
    return db.select().from(envelopeWbsTags);
  }
}

export const storage = new DatabaseStorage();
