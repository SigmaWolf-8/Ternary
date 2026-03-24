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

import { encryptToken, decryptToken } from "./crypto-utils";
import { phaseSplit, phaseRecombine } from "./salvi-core/phase-encryption";
import type { EncryptedPhaseData } from "./salvi-core/phase-encryption";
import { 
  type User, type UpsertUser,
  type DemoSession, type InsertDemoSession,
  type BinaryStorage, type InsertBinaryStorage,
  type TernaryStorage, type InsertTernaryStorage,
  type CompressionBenchmark, type InsertCompressionBenchmark,
  type FileUpload, type InsertFileUpload,
  type CompressionHistory, type InsertCompressionHistory,
  type Whitepaper, type InsertWhitepaper,
  type DeveloperSignup, type InsertDeveloperSignup,
  type CompressedDocument, type InsertCompressedDocument,
  type DataSubjectRequest, type InsertDataSubjectRequest,
  type CrsRelayNode, type InsertCrsRelayNode,
  type DeploymentRecord, type InsertDeploymentRecord,
  users, demoSessions, binaryStorage, ternaryStorage, compressionBenchmarks,
  fileUploads, compressionHistory, whitepapers, developerSignups, compressedDocuments,
  dataSubjectRequests, crsRelayNodes, deploymentRecords
} from "@shared/schema";
import { db } from "./db";
import { eq, desc, lt, inArray } from "drizzle-orm";

function bigIntSafeStringify(obj: unknown): string {
  return JSON.stringify(obj, (_key, value) =>
    typeof value === 'bigint' ? value.toString() + 'n' : value
  );
}

function bigIntSafeParse(str: string): unknown {
  return JSON.parse(str, (_key, value) => {
    if (typeof value === 'string' && /^\d+n$/.test(value)) {
      return BigInt(value.slice(0, -1));
    }
    return value;
  });
}

export function phaseEncryptFields(fields: Record<string, unknown>): string {
  const json = bigIntSafeStringify(fields);
  const encrypted = phaseSplit(json, 'performance');
  return bigIntSafeStringify(encrypted);
}

export function phaseDecryptFields(encryptedStr: string | null): Record<string, unknown> | null {
  if (!encryptedStr) return null;
  try {
    const parsed = bigIntSafeParse(encryptedStr) as EncryptedPhaseData;
    const result = phaseRecombine(parsed);
    if (!result.success || !result.data) return null;
    return bigIntSafeParse(result.data) as Record<string, unknown>;
  } catch {
    return null;
  }
}

export interface IStorage {
  getUser(id: string): Promise<User | undefined>;
  getUserByEmail(email: string): Promise<User | undefined>;
  createUser(user: UpsertUser): Promise<User>;
  updateUserGithubToken(id: string, token: string): Promise<User | undefined>;
  getDecryptedGithubToken(id: string): Promise<string | null>;
  setUserAdmin(id: string, isAdmin: boolean): Promise<User | undefined>;
  
  createDemoSession(session: InsertDemoSession): Promise<DemoSession>;
  getDemoSession(sessionId: string): Promise<DemoSession | undefined>;
  
  createBinaryStorage(data: InsertBinaryStorage): Promise<BinaryStorage>;
  getBinaryStorage(sessionId: string): Promise<BinaryStorage[]>;
  
  createTernaryStorage(data: InsertTernaryStorage): Promise<TernaryStorage>;
  getTernaryStorage(sessionId: string): Promise<TernaryStorage[]>;
  
  createCompressionBenchmark(data: InsertCompressionBenchmark): Promise<CompressionBenchmark>;
  getCompressionBenchmarks(sessionId: string): Promise<CompressionBenchmark[]>;
  getRecentBenchmarks(limit: number): Promise<CompressionBenchmark[]>;
  
  createFileUpload(data: InsertFileUpload): Promise<FileUpload>;
  getFileUpload(sessionId: string): Promise<FileUpload | undefined>;
  getAllFileUploads(): Promise<FileUpload[]>;
  
  createCompressionHistory(data: InsertCompressionHistory): Promise<CompressionHistory>;
  getCompressionHistory(limit: number): Promise<CompressionHistory[]>;
  
  createWhitepaper(data: InsertWhitepaper): Promise<Whitepaper>;
  getWhitepaper(id: number): Promise<Whitepaper | undefined>;
  getActiveWhitepaper(): Promise<Whitepaper | undefined>;
  getAllWhitepapers(): Promise<Whitepaper[]>;
  updateWhitepaper(id: number, data: Partial<InsertWhitepaper>): Promise<Whitepaper | undefined>;

  createDeveloperSignup(data: InsertDeveloperSignup): Promise<DeveloperSignup>;
  getDeveloperSignupByEmail(email: string): Promise<DeveloperSignup | undefined>;
  getDeveloperSignupCount(): Promise<number>;
  getAllDeveloperSignups(): Promise<DeveloperSignup[]>;
  deleteDeveloperSignup(id: number): Promise<void>;

  createCompressedDocument(data: InsertCompressedDocument): Promise<CompressedDocument>;
  getCompressedDocument(id: number): Promise<CompressedDocument | undefined>;
  getAllCompressedDocuments(): Promise<CompressedDocument[]>;
  deleteCompressedDocument(id: number): Promise<void>;

  createDataSubjectRequest(data: InsertDataSubjectRequest): Promise<DataSubjectRequest>;
  getDataSubjectRequests(userId: string): Promise<DataSubjectRequest[]>;
  updateDataSubjectRequest(id: number, status: string, responseData?: unknown): Promise<DataSubjectRequest | undefined>;
  getUserData(userId: string): Promise<Record<string, unknown>>;
  deleteUserData(userId: string): Promise<void>;

  upsertCrsRelayNode(publicKey: string, address: string, endpoint: string, tlDsaPk?: string): Promise<CrsRelayNode>;
  getCrsRelayNodeByPublicKey(publicKey: string): Promise<CrsRelayNode | undefined>;
  getAllCrsRelayNodes(): Promise<CrsRelayNode[]>;
  deleteStaleCrsRelayNodes(maxAgeMs: number): Promise<number>;
  deleteCrsRelayNodesByAddresses(addresses: string[]): Promise<number>;
  deleteCrsRelayNode(publicKey: string): Promise<void>;

  createDeploymentRecord(data: InsertDeploymentRecord): Promise<DeploymentRecord>;
  getAllDeploymentRecords(): Promise<DeploymentRecord[]>;
  getDeploymentsByHostname(hostname: string): Promise<DeploymentRecord[]>;
}

export class DatabaseStorage implements IStorage {
  async getUser(id: string): Promise<User | undefined> {
    const [user] = await db.select().from(users).where(eq(users.id, id));
    return user;
  }

  async getUserByEmail(email: string): Promise<User | undefined> {
    const [user] = await db.select().from(users).where(eq(users.email, email));
    return user;
  }

  async createUser(insertUser: UpsertUser): Promise<User> {
    const [user] = await db.insert(users).values(insertUser).returning();
    return user;
  }

  async updateUserGithubToken(id: string, token: string): Promise<User | undefined> {
    const encrypted = encryptToken(token);
    const [user] = await db.update(users).set({ githubToken: encrypted }).where(eq(users.id, id)).returning();
    return user;
  }

  async getDecryptedGithubToken(id: string): Promise<string | null> {
    const [user] = await db.select().from(users).where(eq(users.id, id));
    if (!user?.githubToken) return null;
    return decryptToken(user.githubToken);
  }

  async setUserAdmin(id: string, isAdmin: boolean): Promise<User | undefined> {
    const [user] = await db.update(users).set({ isAdmin }).where(eq(users.id, id)).returning();
    return user;
  }

  async createDemoSession(session: InsertDemoSession): Promise<DemoSession> {
    const [result] = await db.insert(demoSessions).values(session).returning();
    return result;
  }

  async getDemoSession(sessionId: string): Promise<DemoSession | undefined> {
    const [session] = await db.select().from(demoSessions).where(eq(demoSessions.sessionId, sessionId));
    return session;
  }

  async createBinaryStorage(data: InsertBinaryStorage): Promise<BinaryStorage> {
    const encryptedFields = phaseEncryptFields({ rawData: data.rawData });
    const [result] = await db.insert(binaryStorage).values({ ...data, encryptedFields }).returning();
    return result;
  }

  async getBinaryStorage(sessionId: string): Promise<BinaryStorage[]> {
    const rows = await db.select().from(binaryStorage).where(eq(binaryStorage.sessionId, sessionId));
    return rows.map(r => {
      const dec = phaseDecryptFields(r.encryptedFields);
      if (dec?.rawData !== undefined) r.rawData = dec.rawData;
      return r;
    });
  }

  async createTernaryStorage(data: InsertTernaryStorage): Promise<TernaryStorage> {
    const encryptedFields = phaseEncryptFields({ compressedData: data.compressedData });
    const [result] = await db.insert(ternaryStorage).values({ ...data, encryptedFields }).returning();
    return result;
  }

  async getTernaryStorage(sessionId: string): Promise<TernaryStorage[]> {
    const rows = await db.select().from(ternaryStorage).where(eq(ternaryStorage.sessionId, sessionId));
    return rows.map(r => {
      const dec = phaseDecryptFields(r.encryptedFields);
      if (dec?.compressedData !== undefined) r.compressedData = dec.compressedData as string;
      return r;
    });
  }

  async createCompressionBenchmark(data: InsertCompressionBenchmark): Promise<CompressionBenchmark> {
    const [result] = await db.insert(compressionBenchmarks).values(data).returning();
    return result;
  }

  async getCompressionBenchmarks(sessionId: string): Promise<CompressionBenchmark[]> {
    return await db.select().from(compressionBenchmarks).where(eq(compressionBenchmarks.sessionId, sessionId));
  }

  async getRecentBenchmarks(limit: number): Promise<CompressionBenchmark[]> {
    return await db.select().from(compressionBenchmarks).limit(limit);
  }

  async createFileUpload(data: InsertFileUpload): Promise<FileUpload> {
    const [result] = await db.insert(fileUploads).values(data).returning();
    return result;
  }

  async getFileUpload(sessionId: string): Promise<FileUpload | undefined> {
    const [upload] = await db.select().from(fileUploads).where(eq(fileUploads.sessionId, sessionId));
    return upload;
  }

  async getAllFileUploads(): Promise<FileUpload[]> {
    return await db.select().from(fileUploads).orderBy(desc(fileUploads.createdAt));
  }

  async createCompressionHistory(data: InsertCompressionHistory): Promise<CompressionHistory> {
    const [result] = await db.insert(compressionHistory).values(data).returning();
    return result;
  }

  async getCompressionHistory(limit: number): Promise<CompressionHistory[]> {
    return await db.select().from(compressionHistory).orderBy(desc(compressionHistory.createdAt)).limit(limit);
  }

  private _decryptWhitepaper(wp: Whitepaper): Whitepaper {
    const dec = phaseDecryptFields(wp.encryptedFields);
    if (dec) {
      if (dec.content !== undefined) wp.content = dec.content as string;
      if (dec.summary !== undefined) wp.summary = dec.summary as string | null;
      if (dec.author !== undefined) wp.author = dec.author as string | null;
    }
    return wp;
  }

  async createWhitepaper(data: InsertWhitepaper): Promise<Whitepaper> {
    const encryptedFields = phaseEncryptFields({ content: data.content, summary: data.summary, author: data.author });
    const [result] = await db.insert(whitepapers).values({ ...data, encryptedFields }).returning();
    return result;
  }

  async getWhitepaper(id: number): Promise<Whitepaper | undefined> {
    const [wp] = await db.select().from(whitepapers).where(eq(whitepapers.id, id));
    return wp ? this._decryptWhitepaper(wp) : undefined;
  }

  async getActiveWhitepaper(): Promise<Whitepaper | undefined> {
    const [wp] = await db.select().from(whitepapers).where(eq(whitepapers.isActive, 1)).orderBy(desc(whitepapers.createdAt)).limit(1);
    return wp ? this._decryptWhitepaper(wp) : undefined;
  }

  async getAllWhitepapers(): Promise<Whitepaper[]> {
    const rows = await db.select().from(whitepapers).orderBy(desc(whitepapers.createdAt));
    return rows.map(r => this._decryptWhitepaper(r));
  }

  async updateWhitepaper(id: number, data: Partial<InsertWhitepaper>): Promise<Whitepaper | undefined> {
    const setData: Record<string, unknown> = { ...data };
    if (data.content !== undefined || data.summary !== undefined || data.author !== undefined) {
      const existing = await this.getWhitepaper(id);
      if (existing) {
        setData.encryptedFields = phaseEncryptFields({
          content: data.content ?? existing.content,
          summary: data.summary ?? existing.summary,
          author: data.author ?? existing.author,
        });
      }
    }
    const [result] = await db.update(whitepapers).set(setData).where(eq(whitepapers.id, id)).returning();
    return result ? this._decryptWhitepaper(result) : undefined;
  }

  private _decryptDeveloperSignup(row: DeveloperSignup): DeveloperSignup {
    const dec = phaseDecryptFields(row.encryptedFields);
    if (dec) {
      if (dec.email !== undefined) row.email = dec.email as string;
      if (dec.name !== undefined) row.name = dec.name as string | null;
      if (dec.company !== undefined) row.company = dec.company as string | null;
      if (dec.interest !== undefined) row.interest = dec.interest as string | null;
    }
    return row;
  }

  async createDeveloperSignup(data: InsertDeveloperSignup): Promise<DeveloperSignup> {
    const encryptedFields = phaseEncryptFields({ email: data.email, name: data.name, company: data.company, interest: data.interest });
    const [result] = await db.insert(developerSignups).values({ ...data, encryptedFields }).returning();
    return result;
  }

  async getDeveloperSignupByEmail(email: string): Promise<DeveloperSignup | undefined> {
    const [signup] = await db.select().from(developerSignups).where(eq(developerSignups.email, email));
    return signup ? this._decryptDeveloperSignup(signup) : undefined;
  }

  async getDeveloperSignupCount(): Promise<number> {
    const result = await db.select().from(developerSignups);
    return result.length;
  }

  async getAllDeveloperSignups(): Promise<DeveloperSignup[]> {
    const rows = await db.select().from(developerSignups).orderBy(developerSignups.createdAt);
    return rows.map(r => this._decryptDeveloperSignup(r));
  }

  async deleteDeveloperSignup(id: number): Promise<void> {
    await db.delete(developerSignups).where(eq(developerSignups.id, id));
  }

  private _decryptCompressedDocument(row: CompressedDocument): CompressedDocument {
    const dec = phaseDecryptFields(row.encryptedFields);
    if (dec?.content !== undefined) row.content = dec.content as string;
    return row;
  }

  async createCompressedDocument(data: InsertCompressedDocument): Promise<CompressedDocument> {
    const encryptedFields = phaseEncryptFields({ content: data.content });
    const [result] = await db.insert(compressedDocuments).values({ ...data, encryptedFields }).returning();
    return result;
  }

  async getCompressedDocument(id: number): Promise<CompressedDocument | undefined> {
    const [doc] = await db.select().from(compressedDocuments).where(eq(compressedDocuments.id, id));
    return doc ? this._decryptCompressedDocument(doc) : undefined;
  }

  async getAllCompressedDocuments(): Promise<CompressedDocument[]> {
    const rows = await db.select().from(compressedDocuments).orderBy(desc(compressedDocuments.createdAt));
    return rows.map(r => this._decryptCompressedDocument(r));
  }

  async deleteCompressedDocument(id: number): Promise<void> {
    await db.delete(compressedDocuments).where(eq(compressedDocuments.id, id));
  }

  private _decryptDataSubjectRequest(row: DataSubjectRequest): DataSubjectRequest {
    const dec = phaseDecryptFields(row.encryptedFields);
    if (dec?.responseData !== undefined) row.responseData = dec.responseData;
    return row;
  }

  async createDataSubjectRequest(data: InsertDataSubjectRequest): Promise<DataSubjectRequest> {
    const encryptedFields = phaseEncryptFields({ responseData: data.responseData });
    const [result] = await db.insert(dataSubjectRequests).values({ ...data, encryptedFields }).returning();
    return result;
  }

  async getDataSubjectRequests(userId: string): Promise<DataSubjectRequest[]> {
    const rows = await db.select().from(dataSubjectRequests).where(eq(dataSubjectRequests.userId, userId)).orderBy(desc(dataSubjectRequests.requestedAt));
    return rows.map(r => this._decryptDataSubjectRequest(r));
  }

  async updateDataSubjectRequest(id: number, status: string, responseData?: unknown): Promise<DataSubjectRequest | undefined> {
    const updateData: Record<string, unknown> = { status, completedAt: new Date() };
    if (responseData !== undefined) {
      updateData.responseData = responseData;
      updateData.encryptedFields = phaseEncryptFields({ responseData });
    }
    const [result] = await db.update(dataSubjectRequests).set(updateData).where(eq(dataSubjectRequests.id, id)).returning();
    return result ? this._decryptDataSubjectRequest(result) : undefined;
  }

  async getUserData(userId: string): Promise<Record<string, unknown>> {
    const [user] = await db.select().from(users).where(eq(users.id, userId));
    if (!user) return {};
    const { githubToken, ...safeUser } = user;
    const signupRows = await db.select().from(developerSignups).where(eq(developerSignups.email, user.email || ""));
    const decryptedSignups = signupRows.map(r => this._decryptDeveloperSignup(r));
    const dsrRows = await db.select().from(dataSubjectRequests).where(eq(dataSubjectRequests.userId, userId));
    const decryptedDsr = dsrRows.map(r => this._decryptDataSubjectRequest(r));
    return {
      account: safeUser,
      developerSignups: decryptedSignups,
      dataSubjectRequests: decryptedDsr,
      exportDate: new Date().toISOString(),
    };
  }

  async deleteUserData(userId: string): Promise<void> {
    const [user] = await db.select().from(users).where(eq(users.id, userId));
    if (user?.email) {
      await db.delete(developerSignups).where(eq(developerSignups.email, user.email));
    }
    await db.delete(dataSubjectRequests).where(eq(dataSubjectRequests.userId, userId));
    await db.delete(users).where(eq(users.id, userId));
  }

  private _decryptCrsRelayNode(row: CrsRelayNode): CrsRelayNode {
    const dec = phaseDecryptFields(row.encryptedFields);
    if (dec) {
      if (dec.endpoint !== undefined) row.endpoint = dec.endpoint as string;
      if (dec.tlDsaPk !== undefined) row.tlDsaPk = dec.tlDsaPk as string | null;
    }
    return row;
  }

  async upsertCrsRelayNode(publicKey: string, address: string, endpoint: string, tlDsaPk?: string): Promise<CrsRelayNode> {
    const phaseData = phaseSplit(publicKey, 'performance');
    const publicKeyEncrypted = JSON.stringify(phaseData);
    const existing = await db.select().from(crsRelayNodes).where(eq(crsRelayNodes.publicKey, publicKey));
    if (existing.length > 0) {
      const existingNode = existing[0];
      const mergedTlDsaPk = tlDsaPk || existingNode.tlDsaPk || null;
      const encryptedFields = phaseEncryptFields({ endpoint, tlDsaPk: mergedTlDsaPk });
      const setFields: any = { address, endpoint, publicKeyEncrypted, encryptedFields, lastSeen: new Date(), updatedAt: new Date() };
      if (tlDsaPk) setFields.tlDsaPk = tlDsaPk;
      const [updated] = await db.update(crsRelayNodes)
        .set(setFields)
        .where(eq(crsRelayNodes.publicKey, publicKey))
        .returning();
      return updated;
    }
    const encryptedFields = phaseEncryptFields({ endpoint, tlDsaPk: tlDsaPk || null });
    const [node] = await db.insert(crsRelayNodes)
      .values({ publicKey, publicKeyEncrypted, address, endpoint, lastSeen: new Date(), tlDsaPk: tlDsaPk || null, encryptedFields })
      .returning();
    return node;
  }

  async getCrsRelayNodeByPublicKey(publicKey: string): Promise<CrsRelayNode | undefined> {
    const [node] = await db.select().from(crsRelayNodes).where(eq(crsRelayNodes.publicKey, publicKey));
    return node ? this._decryptCrsRelayNode(node) : undefined;
  }

  async getAllCrsRelayNodes(): Promise<CrsRelayNode[]> {
    const rows = await db.select().from(crsRelayNodes);
    return rows.map(r => this._decryptCrsRelayNode(r));
  }

  async deleteStaleCrsRelayNodes(maxAgeMs: number): Promise<number> {
    const cutoff = new Date(Date.now() - maxAgeMs);
    const deleted = await db.delete(crsRelayNodes).where(lt(crsRelayNodes.lastSeen, cutoff)).returning();
    return deleted.length;
  }

  async deleteCrsRelayNodesByAddresses(addresses: string[]): Promise<number> {
    if (addresses.length === 0) return 0;
    const deleted = await db.delete(crsRelayNodes).where(inArray(crsRelayNodes.address, addresses)).returning();
    return deleted.length;
  }

  async deleteCrsRelayNode(publicKey: string): Promise<void> {
    await db.delete(crsRelayNodes).where(eq(crsRelayNodes.publicKey, publicKey));
  }

  private _decryptDeploymentRecord(row: DeploymentRecord): DeploymentRecord {
    const dec = phaseDecryptFields(row.encryptedFields);
    if (dec) {
      if (dec.hostname !== undefined) row.hostname = dec.hostname as string;
      if (dec.ip !== undefined) row.ip = dec.ip as string;
      if (dec.daemons !== undefined) row.daemons = dec.daemons as any;
      if (dec.binaryPath !== undefined) row.binaryPath = dec.binaryPath as string | null;
      if (dec.logDir !== undefined) row.logDir = dec.logDir as string | null;
      if (dec.identityBase !== undefined) row.identityBase = dec.identityBase as string | null;
      if (dec.deployer !== undefined) row.deployer = dec.deployer as string | null;
    }
    return row;
  }

  async createDeploymentRecord(data: InsertDeploymentRecord): Promise<DeploymentRecord> {
    const encryptedFields = phaseEncryptFields({
      hostname: data.hostname, ip: data.ip, daemons: data.daemons, binaryPath: data.binaryPath,
      logDir: data.logDir, identityBase: data.identityBase, deployer: data.deployer,
    });
    const [record] = await db.insert(deploymentRecords)
      .values({ ...data, encryptedFields })
      .onConflictDoUpdate({
        target: deploymentRecords.hostname,
        set: {
          ip: data.ip,
          architecture: data.architecture,
          daemonCount: data.daemonCount,
          daemons: data.daemons,
          crsUrl: data.crsUrl,
          crsAddress: data.crsAddress,
          binaryPath: data.binaryPath,
          binarySizeMB: data.binarySizeMB,
          logDir: data.logDir,
          identityBase: data.identityBase,
          deployer: data.deployer,
          deployedAt: data.deployedAt,
          encryptedFields,
        },
      })
      .returning();
    return record;
  }

  async getAllDeploymentRecords(): Promise<DeploymentRecord[]> {
    const rows = await db.select().from(deploymentRecords).orderBy(desc(deploymentRecords.createdAt));
    return rows.map(r => this._decryptDeploymentRecord(r));
  }

  async getDeploymentsByHostname(hostname: string): Promise<DeploymentRecord[]> {
    const rows = await db.select().from(deploymentRecords).where(eq(deploymentRecords.hostname, hostname)).orderBy(desc(deploymentRecords.createdAt));
    return rows.map(r => this._decryptDeploymentRecord(r));
  }
}

export const storage = new DatabaseStorage();
