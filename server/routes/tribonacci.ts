/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 */

import type { Express } from "express";
import { pool } from "../db";
import { createLogger } from "../logger";

const log = createLogger("tribonacci-routes");

export function registerTribonacciRoutes(app: Express) {
  app.get("/api/tribonacci/hook", async (_req, res) => {
    try {
      const result = await pool.query("SELECT * FROM plenumnet.demonstrate_hook()");
      res.json({ success: true, data: result.rows });
    } catch (error) {
      log.error("Hook demo failed", error);
      res.status(500).json({ success: false, error: "Failed to run hook demo" });
    }
  });

  app.get("/api/tribonacci/permutation", async (_req, res) => {
    try {
      const result = await pool.query("SELECT * FROM plenumnet.verify_permutation()");
      res.json({ success: true, data: result.rows });
    } catch (error) {
      log.error("Permutation verification failed", error);
      res.status(500).json({ success: false, error: "Failed to verify permutation" });
    }
  });

  app.get("/api/tribonacci/coverage", async (req, res) => {
    try {
      const numTerms = Math.min(Math.max(parseInt(String(req.query.terms)) || 200, 10), 500);
      const result = await pool.query("SELECT * FROM plenumnet.verify_28fold_coverage($1)", [numTerms]);
      res.json({ success: true, data: result.rows });
    } catch (error) {
      log.error("Coverage verification failed", error);
      res.status(500).json({ success: false, error: "Failed to verify coverage" });
    }
  });

  app.get("/api/tribonacci/hash", async (req, res) => {
    try {
      const key = parseInt(String(req.query.key));
      if (isNaN(key)) {
        return res.status(400).json({ success: false, error: "key parameter required (integer)" });
      }
      const buckets = Math.min(Math.max(parseInt(String(req.query.buckets)) || 28, 2), 1024);

      const result = await pool.query(
        "SELECT plenumnet.trad_hash_28($1) as shard_28, plenumnet.tribonacci_hash($1, $2) as trib_hash",
        [key, buckets]
      );
      res.json({ success: true, key, buckets, ...result.rows[0] });
    } catch (error) {
      log.error("Hash computation failed", error);
      res.status(500).json({ success: false, error: "Failed to compute hash" });
    }
  });

  app.get("/api/tribonacci/sequence", async (req, res) => {
    try {
      const count = Math.min(Math.max(parseInt(String(req.query.count)) || 20, 1), 60);
      const result = await pool.query("SELECT * FROM plenumnet.tribonacci_table($1)", [count]);
      res.json({ success: true, data: result.rows });
    } catch (error) {
      log.error("Sequence generation failed", error);
      res.status(500).json({ success: false, error: "Failed to generate sequence" });
    }
  });

  app.post("/api/tribonacci/generate-id", async (_req, res) => {
    try {
      const result = await pool.query("SELECT plenumnet.generate_trib_id() as id");
      res.json({ success: true, id: result.rows[0].id });
    } catch (error) {
      log.error("ID generation failed", error);
      res.status(500).json({ success: false, error: "Failed to generate ID" });
    }
  });

  app.get("/api/tribonacci/next-worker", async (_req, res) => {
    try {
      const result = await pool.query("SELECT plenumnet.next_worker('demo') as worker_id");
      res.json({ success: true, worker_id: result.rows[0].worker_id });
    } catch (error) {
      log.error("Next worker failed", error);
      res.status(500).json({ success: false, error: "Failed to get next worker" });
    }
  });

  app.get("/api/tribonacci/skip-lookup", async (req, res) => {
    try {
      const pos = parseInt(String(req.query.position));
      if (isNaN(pos) || pos < 0) {
        return res.status(400).json({ success: false, error: "position parameter required (non-negative integer)" });
      }
      const result = await pool.query("SELECT * FROM plenumnet.skip_lookup($1)", [pos]);
      res.json({ success: true, position: pos, jumps: result.rows });
    } catch (error) {
      log.error("Skip lookup failed", error);
      res.status(500).json({ success: false, error: "Failed to perform skip lookup" });
    }
  });

  app.get("/api/tribonacci/hash-distribution", async (req, res) => {
    try {
      const count = Math.min(Math.max(parseInt(String(req.query.count)) || 1000, 100), 10000);
      const result = await pool.query(`
        SELECT
          plenumnet.trad_hash_28(n) as shard,
          COUNT(*) as key_count
        FROM generate_series(1, $1) AS n
        GROUP BY shard
        ORDER BY shard
      `, [count]);
      res.json({ success: true, count, distribution: result.rows });
    } catch (error) {
      log.error("Hash distribution failed", error);
      res.status(500).json({ success: false, error: "Failed to compute distribution" });
    }
  });
}
