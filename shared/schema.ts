import { sql } from "drizzle-orm";
import { pgTable, text, varchar, integer, timestamp, real, jsonb, serial } from "drizzle-orm/pg-core";
import { createInsertSchema } from "drizzle-zod";
import { z } from "zod";

export const users = pgTable("users", {
  id: varchar("id").primaryKey().default(sql`gen_random_uuid()`),
  username: text("username").notNull().unique(),
  password: text("password").notNull(),
});

export const insertUserSchema = createInsertSchema(users).pick({
  username: true,
  password: true,
});

export type InsertUser = z.infer<typeof insertUserSchema>;
export type User = typeof users.$inferSelect;

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

export const insertDemoSessionSchema = createInsertSchema(demoSessions).omit({ id: true, createdAt: true });
export const insertBinaryStorageSchema = createInsertSchema(binaryStorage).omit({ id: true, createdAt: true });
export const insertTernaryStorageSchema = createInsertSchema(ternaryStorage).omit({ id: true, createdAt: true });
export const insertCompressionBenchmarkSchema = createInsertSchema(compressionBenchmarks).omit({ id: true, createdAt: true });
export const insertFileUploadSchema = createInsertSchema(fileUploads).omit({ id: true, createdAt: true });
export const insertCompressionHistorySchema = createInsertSchema(compressionHistory).omit({ id: true, createdAt: true });
export const insertWhitepaperSchema = createInsertSchema(whitepapers).omit({ id: true, createdAt: true, updatedAt: true });

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
