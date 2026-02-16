import { eq, desc } from "drizzle-orm";
import { db } from "./db";
import {
  users,
  envelopes,
  recipients,
  fields,
  auditLogs,
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
} from "@shared/schema";

export interface IStorage {
  getUser(id: string): Promise<User | undefined>;
  getUserByUsername(username: string): Promise<User | undefined>;
  createUser(user: InsertUser): Promise<User>;

  getEnvelopes(): Promise<Envelope[]>;
  getEnvelope(id: string): Promise<Envelope | undefined>;
  createEnvelope(envelope: InsertEnvelope): Promise<Envelope>;
  updateEnvelope(id: string, data: Partial<InsertEnvelope>): Promise<Envelope | undefined>;
  deleteEnvelope(id: string): Promise<void>;

  getRecipientsByEnvelope(envelopeId: string): Promise<Recipient[]>;
  getRecipient(id: string): Promise<Recipient | undefined>;
  createRecipient(recipient: InsertRecipient): Promise<Recipient>;
  updateRecipient(id: string, data: Partial<Recipient>): Promise<Recipient | undefined>;

  getFieldsByEnvelope(envelopeId: string): Promise<Field[]>;
  createField(field: InsertField): Promise<Field>;
  updateField(id: string, data: Partial<Field>): Promise<Field | undefined>;
  deleteFieldsByEnvelope(envelopeId: string): Promise<void>;

  getAuditLogsByEnvelope(envelopeId: string): Promise<AuditLog[]>;
  createAuditLog(log: InsertAuditLog): Promise<AuditLog>;
}

export class DatabaseStorage implements IStorage {
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

  async getEnvelopes(): Promise<Envelope[]> {
    return db.select().from(envelopes).orderBy(desc(envelopes.createdAt));
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
}

export const storage = new DatabaseStorage();
