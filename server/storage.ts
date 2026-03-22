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
import { phaseSplit } from "./salvi-core/phase-encryption";
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
  users, demoSessions, binaryStorage, ternaryStorage, compressionBenchmarks,
  fileUploads, compressionHistory, whitepapers, developerSignups, compressedDocuments,
  dataSubjectRequests, crsRelayNodes
} from "@shared/schema";
import { db } from "./db";
import { eq, desc, lt, inArray } from "drizzle-orm";

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

  upsertCrsRelayNode(publicKey: string, address: string, endpoint: string): Promise<CrsRelayNode>;
  getCrsRelayNodeByPublicKey(publicKey: string): Promise<CrsRelayNode | undefined>;
  getAllCrsRelayNodes(): Promise<CrsRelayNode[]>;
  deleteStaleCrsRelayNodes(maxAgeMs: number): Promise<number>;
  deleteCrsRelayNodesByAddresses(addresses: string[]): Promise<number>;
  deleteCrsRelayNode(publicKey: string): Promise<void>;
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
    const [result] = await db.insert(binaryStorage).values(data).returning();
    return result;
  }

  async getBinaryStorage(sessionId: string): Promise<BinaryStorage[]> {
    return await db.select().from(binaryStorage).where(eq(binaryStorage.sessionId, sessionId));
  }

  async createTernaryStorage(data: InsertTernaryStorage): Promise<TernaryStorage> {
    const [result] = await db.insert(ternaryStorage).values(data).returning();
    return result;
  }

  async getTernaryStorage(sessionId: string): Promise<TernaryStorage[]> {
    return await db.select().from(ternaryStorage).where(eq(ternaryStorage.sessionId, sessionId));
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

  async createWhitepaper(data: InsertWhitepaper): Promise<Whitepaper> {
    const [result] = await db.insert(whitepapers).values(data).returning();
    return result;
  }

  async getWhitepaper(id: number): Promise<Whitepaper | undefined> {
    const [wp] = await db.select().from(whitepapers).where(eq(whitepapers.id, id));
    return wp;
  }

  async getActiveWhitepaper(): Promise<Whitepaper | undefined> {
    const [wp] = await db.select().from(whitepapers).where(eq(whitepapers.isActive, 1)).orderBy(desc(whitepapers.createdAt)).limit(1);
    return wp;
  }

  async getAllWhitepapers(): Promise<Whitepaper[]> {
    return await db.select().from(whitepapers).orderBy(desc(whitepapers.createdAt));
  }

  async updateWhitepaper(id: number, data: Partial<InsertWhitepaper>): Promise<Whitepaper | undefined> {
    const [result] = await db.update(whitepapers).set(data).where(eq(whitepapers.id, id)).returning();
    return result;
  }

  async createDeveloperSignup(data: InsertDeveloperSignup): Promise<DeveloperSignup> {
    const [result] = await db.insert(developerSignups).values(data).returning();
    return result;
  }

  async getDeveloperSignupByEmail(email: string): Promise<DeveloperSignup | undefined> {
    const [signup] = await db.select().from(developerSignups).where(eq(developerSignups.email, email));
    return signup;
  }

  async getDeveloperSignupCount(): Promise<number> {
    const result = await db.select().from(developerSignups);
    return result.length;
  }

  async getAllDeveloperSignups(): Promise<DeveloperSignup[]> {
    return await db.select().from(developerSignups).orderBy(developerSignups.createdAt);
  }

  async deleteDeveloperSignup(id: number): Promise<void> {
    await db.delete(developerSignups).where(eq(developerSignups.id, id));
  }

  async createCompressedDocument(data: InsertCompressedDocument): Promise<CompressedDocument> {
    const [result] = await db.insert(compressedDocuments).values(data).returning();
    return result;
  }

  async getCompressedDocument(id: number): Promise<CompressedDocument | undefined> {
    const [doc] = await db.select().from(compressedDocuments).where(eq(compressedDocuments.id, id));
    return doc;
  }

  async getAllCompressedDocuments(): Promise<CompressedDocument[]> {
    return await db.select().from(compressedDocuments).orderBy(desc(compressedDocuments.createdAt));
  }

  async deleteCompressedDocument(id: number): Promise<void> {
    await db.delete(compressedDocuments).where(eq(compressedDocuments.id, id));
  }

  async createDataSubjectRequest(data: InsertDataSubjectRequest): Promise<DataSubjectRequest> {
    const [result] = await db.insert(dataSubjectRequests).values(data).returning();
    return result;
  }

  async getDataSubjectRequests(userId: string): Promise<DataSubjectRequest[]> {
    return await db.select().from(dataSubjectRequests).where(eq(dataSubjectRequests.userId, userId)).orderBy(desc(dataSubjectRequests.requestedAt));
  }

  async updateDataSubjectRequest(id: number, status: string, responseData?: unknown): Promise<DataSubjectRequest | undefined> {
    const updateData: Record<string, unknown> = { status, completedAt: new Date() };
    if (responseData !== undefined) updateData.responseData = responseData;
    const [result] = await db.update(dataSubjectRequests).set(updateData).where(eq(dataSubjectRequests.id, id)).returning();
    return result;
  }

  async getUserData(userId: string): Promise<Record<string, unknown>> {
    const [user] = await db.select().from(users).where(eq(users.id, userId));
    if (!user) return {};
    const { githubToken, ...safeUser } = user;
    const signups = await db.select().from(developerSignups).where(eq(developerSignups.email, user.email || ""));
    const dsrHistory = await db.select().from(dataSubjectRequests).where(eq(dataSubjectRequests.userId, userId));
    return {
      account: safeUser,
      developerSignups: signups,
      dataSubjectRequests: dsrHistory,
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

  async upsertCrsRelayNode(publicKey: string, address: string, endpoint: string): Promise<CrsRelayNode> {
    const phaseData = phaseSplit(publicKey, 'performance');
    const encrypted = JSON.stringify(phaseData);
    const existing = await db.select().from(crsRelayNodes).where(eq(crsRelayNodes.publicKey, publicKey));
    if (existing.length > 0) {
      const [updated] = await db.update(crsRelayNodes)
        .set({ address, endpoint, publicKeyEncrypted: encrypted, lastSeen: new Date(), updatedAt: new Date() })
        .where(eq(crsRelayNodes.publicKey, publicKey))
        .returning();
      return updated;
    }
    const [node] = await db.insert(crsRelayNodes)
      .values({ publicKey, publicKeyEncrypted: encrypted, address, endpoint, lastSeen: new Date() })
      .returning();
    return node;
  }

  async getCrsRelayNodeByPublicKey(publicKey: string): Promise<CrsRelayNode | undefined> {
    const [node] = await db.select().from(crsRelayNodes).where(eq(crsRelayNodes.publicKey, publicKey));
    return node;
  }

  async getAllCrsRelayNodes(): Promise<CrsRelayNode[]> {
    return db.select().from(crsRelayNodes);
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
}

export const storage = new DatabaseStorage();
