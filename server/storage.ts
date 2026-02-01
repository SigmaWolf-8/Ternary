import { 
  type User, type InsertUser,
  type DemoSession, type InsertDemoSession,
  type BinaryStorage, type InsertBinaryStorage,
  type TernaryStorage, type InsertTernaryStorage,
  type CompressionBenchmark, type InsertCompressionBenchmark,
  users, demoSessions, binaryStorage, ternaryStorage, compressionBenchmarks
} from "@shared/schema";
import { db } from "./db";
import { eq } from "drizzle-orm";

export interface IStorage {
  getUser(id: string): Promise<User | undefined>;
  getUserByUsername(username: string): Promise<User | undefined>;
  createUser(user: InsertUser): Promise<User>;
  
  createDemoSession(session: InsertDemoSession): Promise<DemoSession>;
  getDemoSession(sessionId: string): Promise<DemoSession | undefined>;
  
  createBinaryStorage(data: InsertBinaryStorage): Promise<BinaryStorage>;
  getBinaryStorage(sessionId: string): Promise<BinaryStorage[]>;
  
  createTernaryStorage(data: InsertTernaryStorage): Promise<TernaryStorage>;
  getTernaryStorage(sessionId: string): Promise<TernaryStorage[]>;
  
  createCompressionBenchmark(data: InsertCompressionBenchmark): Promise<CompressionBenchmark>;
  getCompressionBenchmarks(sessionId: string): Promise<CompressionBenchmark[]>;
  getRecentBenchmarks(limit: number): Promise<CompressionBenchmark[]>;
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
}

export const storage = new DatabaseStorage();
