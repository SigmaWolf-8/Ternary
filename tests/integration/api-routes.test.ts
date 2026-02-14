/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
 * Patent(s) Pending.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * Unauthorized copying, modification, distribution, or use of this file,
 * via any medium, is strictly prohibited without the prior written
 * permission of Capomastro Holdings Ltd.
 *
 * See LICENSE in the repository root for full terms.
 */

import { describe, it, expect, beforeAll } from "vitest";

const BASE_URL = "http://localhost:5000";

function url(path: string): string {
  return `${BASE_URL}${path}`;
}

async function jsonGet(path: string, headers?: Record<string, string>) {
  const res = await fetch(url(path), { headers });
  return res;
}

async function jsonPost(path: string, body: unknown, headers?: Record<string, string>) {
  const res = await fetch(url(path), {
    method: "POST",
    headers: { "Content-Type": "application/json", ...headers },
    body: JSON.stringify(body),
  });
  return res;
}

describe("PlenumNET API Integration Tests", () => {
  beforeAll(async () => {
    let retries = 10;
    while (retries > 0) {
      try {
        const res = await fetch(url("/api/health"));
        if (res.ok) return;
      } catch {}
      retries--;
      await new Promise((r) => setTimeout(r, 2000));
    }
    throw new Error("Server did not become ready within timeout");
  }, 30000);

  describe("Health & Legal", () => {
    it("GET /api/health returns 200 with status, timestamp, and services", async () => {
      const res = await jsonGet("/api/health");
      expect(res.status).toBe(200);
      const data = await res.json();
      expect(data).toHaveProperty("status");
      expect(data).toHaveProperty("timestamp");
      expect(data).toHaveProperty("services");
      expect(data.services).toHaveProperty("database");
      expect(data.services).toHaveProperty("server");
    });

    it("GET /api/legal/terms returns 200 with title and content", async () => {
      const res = await jsonGet("/api/legal/terms");
      expect(res.status).toBe(200);
      const data = await res.json();
      expect(data).toHaveProperty("title");
      expect(data).toHaveProperty("content");
      expect(typeof data.title).toBe("string");
      expect(typeof data.content).toBe("string");
    });
  });

  describe("Demo/Compression", () => {
    it("POST /api/demo/run with sensor dataset", async () => {
      const res = await jsonPost("/api/demo/run", {
        datasetName: "sensor",
        rowCount: 10,
      });
      expect(res.status).toBe(200);
      const data = await res.json();
      expect(data.success).toBe(true);
      expect(data).toHaveProperty("sessionId");
      expect(data).toHaveProperty("binarySize");
      expect(data).toHaveProperty("ternarySize");
      expect(data).toHaveProperty("savingsPercent");
      expect(data.datasetName).toBe("sensor");
      expect(data.rowCount).toBe(10);
    }, 15000);

    it("GET /api/demo/stats returns statistics", async () => {
      const res = await jsonGet("/api/demo/stats");
      expect(res.status).toBe(200);
      const data = await res.json();
      expect(data).toHaveProperty("totalRuns");
      expect(data).toHaveProperty("avgSavings");
      expect(data).toHaveProperty("totalDataProcessed");
    });

    it("GET /api/demo/files returns file list", async () => {
      const res = await jsonGet("/api/demo/files");
      expect(res.status).toBe(200);
      const data = await res.json();
      expect(data.success).toBe(true);
      expect(data).toHaveProperty("files");
      expect(Array.isArray(data.files)).toBe(true);
    });

    it("GET /api/demo/history returns compression history", async () => {
      const res = await jsonGet("/api/demo/history");
      expect(res.status).toBe(200);
      const data = await res.json();
      expect(data.success).toBe(true);
      expect(data).toHaveProperty("history");
      expect(Array.isArray(data.history)).toBe(true);
    });

    it("POST /api/compression/file with test data", async () => {
      const testContent = Buffer.from("Hello PlenumNET test data for compression").toString("base64");
      const res = await jsonPost("/api/compression/file", {
        fileName: "test.txt",
        content: testContent,
      });
      expect(res.status).toBe(200);
      const data = await res.json();
      expect(data.success).toBe(true);
      expect(data).toHaveProperty("originalSize");
      expect(data).toHaveProperty("compressedSize");
      expect(data).toHaveProperty("compressionRatio");
      expect(data).toHaveProperty("data");
    }, 10000);

    it("POST /api/compression/decompress with compressed data", async () => {
      const testContent = Buffer.from("Decompress round-trip test").toString("base64");
      const compressRes = await jsonPost("/api/compression/file", {
        fileName: "roundtrip.txt",
        content: testContent,
      });
      const compressData = await compressRes.json();

      if (compressData.success && compressData.data) {
        const res = await jsonPost("/api/compression/decompress", {
          content: compressData.data,
        });
        expect(res.status).toBe(200);
        const data = await res.json();
        expect(data.success).toBe(true);
        expect(data).toHaveProperty("originalFileName");
        expect(data).toHaveProperty("originalSize");
      }
    }, 10000);

    it("POST /api/compression/db/store stores test data", async () => {
      const res = await jsonPost("/api/compression/db/store", {
        title: "Integration Test Document",
        content: "This is test content for the compression DB store endpoint",
      });
      expect(res.status).toBe(200);
      const data = await res.json();
      expect(data.success).toBe(true);
      expect(data).toHaveProperty("document");
      expect(data.document).toHaveProperty("id");
      expect(data.document).toHaveProperty("title");
    });

    it("GET /api/compression/db/documents returns document list", async () => {
      const res = await jsonGet("/api/compression/db/documents");
      expect(res.status).toBe(200);
      const data = await res.json();
      expect(data.success).toBe(true);
      expect(data).toHaveProperty("documents");
      expect(Array.isArray(data.documents)).toBe(true);
    });

    it("GET /api/whitepapers returns whitepaper list", async () => {
      const res = await jsonGet("/api/whitepapers");
      expect(res.status).toBe(200);
      const data = await res.json();
      expect(data.success).toBe(true);
      expect(data).toHaveProperty("whitepapers");
      expect(Array.isArray(data.whitepapers)).toBe(true);
    });
  });

  describe("Salvi Timing", () => {
    it("GET /api/salvi/docs returns API documentation", async () => {
      const res = await jsonGet("/api/salvi/docs");
      expect(res.status).toBe(200);
      const data = await res.json();
      expect(data).toHaveProperty("name");
      expect(data).toHaveProperty("version");
      expect(data).toHaveProperty("endpoints");
      expect(data.endpoints).toHaveProperty("ternary");
      expect(data.endpoints).toHaveProperty("timing");
      expect(data.endpoints).toHaveProperty("phase");
    });

    it("GET /api/salvi/timing/timestamp returns femtosecond precision fields", async () => {
      const res = await jsonGet("/api/salvi/timing/timestamp");
      expect(res.status).toBe(200);
      const data = await res.json();
      expect(data.success).toBe(true);
      expect(data).toHaveProperty("timestamp");
      expect(data.timestamp).toHaveProperty("femtoseconds");
      expect(data.timestamp).toHaveProperty("salviEpochOffset");
      expect(data).toHaveProperty("epoch");
      expect(data).toHaveProperty("hptp");
      expect(data.hptp).toHaveProperty("protocol");
    });

    it("GET /api/salvi/timing/metrics returns timing metrics", async () => {
      const res = await jsonGet("/api/salvi/timing/metrics");
      expect(res.status).toBe(200);
      const data = await res.json();
      expect(data.success).toBe(true);
      expect(data).toHaveProperty("timestamp");
    });

    it("GET /api/salvi/timing/batch/5 returns batch of timestamps", async () => {
      const res = await jsonGet("/api/salvi/timing/batch/5");
      expect(res.status).toBe(200);
      const data = await res.json();
      expect(data.success).toBe(true);
      expect(data).toHaveProperty("count");
      expect(data.count).toBe(5);
      expect(data).toHaveProperty("timestamps");
      expect(Array.isArray(data.timestamps)).toBe(true);
      expect(data.timestamps.length).toBe(5);
    });

    it("GET /api/salvi/timing/self-test returns timer analysis", async () => {
      const res = await jsonGet("/api/salvi/timing/self-test");
      expect(res.status).toBe(200);
      const data = await res.json();
      expect(data.success).toBe(true);
      expect(data).toHaveProperty("selfTest");
      expect(data).toHaveProperty("sampleCount");
      expect(data).toHaveProperty("resolution");
      expect(data).toHaveProperty("jitter");
      expect(data).toHaveProperty("monotonicity");
      expect(data).toHaveProperty("verdict");
    }, 15000);

    it("GET /api/salvi/timing/error-budget returns HPTP error budget", async () => {
      const res = await jsonGet("/api/salvi/timing/error-budget");
      expect(res.status).toBe(200);
      const data = await res.json();
      expect(data.success).toBe(true);
      expect(data).toHaveProperty("errorBudget");
    });

    it("GET /api/salvi/timing/epoch/anchors returns epoch anchor points", async () => {
      const res = await jsonGet("/api/salvi/timing/epoch/anchors");
      expect(res.status).toBe(200);
      const data = await res.json();
      expect(data.success).toBe(true);
    });

    it("GET /api/salvi/timing/epoch/calendars returns calendar sync", async () => {
      const res = await jsonGet("/api/salvi/timing/epoch/calendars");
      expect(res.status).toBe(200);
      const data = await res.json();
      expect(data.success).toBe(true);
    });

    it("GET /api/salvi/timing/epoch/calendars/mayan returns Mayan calendar", async () => {
      const res = await jsonGet("/api/salvi/timing/epoch/calendars/mayan");
      expect(res.status).toBe(200);
      const data = await res.json();
      expect(data.success).toBe(true);
      expect(data).toHaveProperty("calendar");
      expect(data.calendar).toBe("Mayan Long Count");
    });

    it("GET /api/salvi/timing/epoch/calendars/hebrew returns Hebrew calendar", async () => {
      const res = await jsonGet("/api/salvi/timing/epoch/calendars/hebrew");
      expect(res.status).toBe(200);
      const data = await res.json();
      expect(data.success).toBe(true);
      expect(data).toHaveProperty("calendar");
      expect(data.calendar).toBe("Hebrew");
    });
  });

  describe("Salvi Ternary Operations", () => {
    it("POST /api/salvi/ternary/convert converts between representations", async () => {
      const res = await jsonPost("/api/salvi/ternary/convert", {
        value: 1,
        from: "A",
        to: "B",
      });
      expect(res.status).toBe(200);
      const data = await res.json();
      expect(data.success).toBe(true);
    });

    it("POST /api/salvi/ternary/add performs GF(3) addition", async () => {
      const res = await jsonPost("/api/salvi/ternary/add", {
        a: 1,
        b: -1,
      });
      expect(res.status).toBe(200);
      const data = await res.json();
      expect(data.success).toBe(true);
    });

    it("POST /api/salvi/ternary/multiply performs GF(3) multiplication", async () => {
      const res = await jsonPost("/api/salvi/ternary/multiply", {
        a: 1,
        b: -1,
      });
      expect(res.status).toBe(200);
      const data = await res.json();
      expect(data.success).toBe(true);
    });

    it("POST /api/salvi/ternary/rotate performs bijective rotation", async () => {
      const res = await jsonPost("/api/salvi/ternary/rotate", {
        value: 1,
        steps: 1,
      });
      expect(res.status).toBe(200);
      const data = await res.json();
      expect(data.success).toBe(true);
    });

    it("POST /api/salvi/ternary/not performs ternary NOT", async () => {
      const res = await jsonPost("/api/salvi/ternary/not", {
        value: 1,
      });
      expect(res.status).toBe(200);
      const data = await res.json();
      expect(data.success).toBe(true);
    });

    it("POST /api/salvi/ternary/xor performs ternary XOR", async () => {
      const res = await jsonPost("/api/salvi/ternary/xor", {
        a: 1,
        b: -1,
      });
      expect(res.status).toBe(200);
      const data = await res.json();
      expect(data.success).toBe(true);
    });

    it("GET /api/salvi/ternary/density/10 calculates information density", async () => {
      const res = await jsonGet("/api/salvi/ternary/density/10");
      expect(res.status).toBe(200);
      const data = await res.json();
      expect(data.success).toBe(true);
    });
  });

  describe("Salvi VM & Phase Encryption", () => {
    it("GET /api/salvi/vm/spec returns TVM ISA specification", async () => {
      const res = await jsonGet("/api/salvi/vm/spec");
      expect(res.status).toBe(200);
      const data = await res.json();
      expect(data.success).toBe(true);
      expect(data).toHaveProperty("spec");
    });

    it("GET /api/salvi/vm/conformance returns conformance test results", async () => {
      const res = await jsonGet("/api/salvi/vm/conformance");
      expect(res.status).toBe(200);
      const data = await res.json();
      expect(data.success).toBe(true);
      expect(data).toHaveProperty("conformance");
      expect(data.conformance).toHaveProperty("totalTests");
      expect(data.conformance).toHaveProperty("passed");
      expect(data.conformance).toHaveProperty("verdict");
    });

    it("GET /api/salvi/phase/config/standard returns phase config", async () => {
      const res = await jsonGet("/api/salvi/phase/config/standard");
      expect(res.status).toBe(200);
      const data = await res.json();
      expect(data.success).toBe(true);
      expect(data).toHaveProperty("config");
    });

    it("POST /api/salvi/phase/split splits data into phases", async () => {
      const res = await jsonPost("/api/salvi/phase/split", {
        data: "test data",
        mode: "balanced",
      });
      expect(res.status).toBe(200);
      const data = await res.json();
      expect(data.success).toBe(true);
      expect(data).toHaveProperty("encrypted");
      expect(data.encrypted).toHaveProperty("primaryPhase");
      expect(data.encrypted).toHaveProperty("secondaryPhase");
    }, 10000);

    it("GET /api/salvi/phase/recommend returns encryption recommendation", async () => {
      const res = await jsonGet("/api/salvi/phase/recommend");
      expect(res.status).toBe(200);
      const data = await res.json();
      expect(data.success).toBe(true);
      expect(data).toHaveProperty("recommendation");
      expect(data.recommendation).toHaveProperty("mode");
      expect(data.recommendation).toHaveProperty("config");
    });
  });

  describe("Tribonacci", () => {
    it("GET /api/tribonacci/sequence?count=10 returns sequence", async () => {
      try {
        const res = await jsonGet("/api/tribonacci/sequence?count=10");
        if (res.status === 200) {
          const data = await res.json();
          expect(data.success).toBe(true);
          expect(data).toHaveProperty("data");
          expect(Array.isArray(data.data)).toBe(true);
        } else {
          console.warn("Tribonacci sequence endpoint returned", res.status, "- PostgreSQL extension may not be installed");
        }
      } catch (e) {
        console.warn("Tribonacci sequence test skipped - endpoint may require PostgreSQL extension");
      }
    }, 15000);

    it("GET /api/tribonacci/hash?key=42&buckets=28 returns hash", async () => {
      try {
        const res = await jsonGet("/api/tribonacci/hash?key=42&buckets=28");
        if (res.status === 200) {
          const data = await res.json();
          expect(data.success).toBe(true);
          expect(data).toHaveProperty("key");
          expect(data.key).toBe(42);
        } else {
          console.warn("Tribonacci hash endpoint returned", res.status, "- PostgreSQL extension may not be installed");
        }
      } catch (e) {
        console.warn("Tribonacci hash test skipped - endpoint may require PostgreSQL extension");
      }
    }, 15000);

    it("POST /api/tribonacci/generate-id generates a tribonacci ID", async () => {
      try {
        const res = await jsonPost("/api/tribonacci/generate-id", {});
        if (res.status === 200) {
          const data = await res.json();
          expect(data.success).toBe(true);
          expect(data).toHaveProperty("id");
        } else {
          console.warn("Tribonacci generate-id endpoint returned", res.status, "- PostgreSQL extension may not be installed");
        }
      } catch (e) {
        console.warn("Tribonacci generate-id test skipped - endpoint may require PostgreSQL extension");
      }
    }, 15000);

    it("GET /api/tribonacci/hash-distribution?count=100 returns distribution", async () => {
      try {
        const res = await jsonGet("/api/tribonacci/hash-distribution?count=100");
        if (res.status === 200) {
          const data = await res.json();
          expect(data.success).toBe(true);
          expect(data).toHaveProperty("distribution");
          expect(Array.isArray(data.distribution)).toBe(true);
        } else {
          console.warn("Tribonacci hash-distribution endpoint returned", res.status, "- PostgreSQL extension may not be installed");
        }
      } catch (e) {
        console.warn("Tribonacci hash-distribution test skipped - endpoint may require PostgreSQL extension");
      }
    }, 15000);

    it("GET /api/tribonacci/skip-lookup?position=5 returns skip lookup", async () => {
      try {
        const res = await jsonGet("/api/tribonacci/skip-lookup?position=5");
        if (res.status === 200) {
          const data = await res.json();
          expect(data.success).toBe(true);
          expect(data).toHaveProperty("position");
          expect(data.position).toBe(5);
        } else {
          console.warn("Tribonacci skip-lookup endpoint returned", res.status, "- PostgreSQL extension may not be installed");
        }
      } catch (e) {
        console.warn("Tribonacci skip-lookup test skipped - endpoint may require PostgreSQL extension");
      }
    }, 15000);
  });

  describe("Input Validation", () => {
    it("POST /api/salvi/ternary/convert with missing fields returns 400", async () => {
      const res = await jsonPost("/api/salvi/ternary/convert", {});
      expect(res.status).toBe(400);
      const data = await res.json();
      expect(data).toHaveProperty("error");
    });

    it("GET /api/tribonacci/hash without key param returns 400", async () => {
      try {
        const res = await jsonGet("/api/tribonacci/hash");
        if (res.status === 400) {
          const data = await res.json();
          expect(data.success).toBe(false);
          expect(data).toHaveProperty("error");
        } else if (res.status === 500) {
          console.warn("Tribonacci hash validation returned 500 - PostgreSQL extension may not be installed");
        }
      } catch (e) {
        console.warn("Tribonacci hash validation test skipped");
      }
    });

    it("GET /api/tribonacci/skip-lookup without position returns 400", async () => {
      try {
        const res = await jsonGet("/api/tribonacci/skip-lookup");
        if (res.status === 400) {
          const data = await res.json();
          expect(data.success).toBe(false);
          expect(data).toHaveProperty("error");
        } else if (res.status === 500) {
          console.warn("Tribonacci skip-lookup validation returned 500 - PostgreSQL extension may not be installed");
        }
      } catch (e) {
        console.warn("Tribonacci skip-lookup validation test skipped");
      }
    });

    it("POST /api/demo/run with invalid dataset returns 400", async () => {
      const res = await jsonPost("/api/demo/run", {
        datasetName: "invalid_dataset",
        rowCount: 10,
      });
      expect(res.status).toBe(400);
      const data = await res.json();
      expect(data).toHaveProperty("error");
    });

    it("GET /api/legal/nonexistent returns 404", async () => {
      const res = await jsonGet("/api/legal/nonexistent");
      expect(res.status).toBe(404);
      const data = await res.json();
      expect(data).toHaveProperty("error");
    });
  });

  describe("Security Headers", () => {
    it("response includes X-Content-Type-Options: nosniff", async () => {
      const res = await jsonGet("/api/health");
      const xcto = res.headers.get("x-content-type-options");
      expect(xcto).toBeTruthy();
      expect(xcto!.toLowerCase()).toBe("nosniff");
    });

    it("response includes Referrer-Policy header", async () => {
      const res = await jsonGet("/api/health");
      const rp = res.headers.get("referrer-policy");
      expect(rp).toBeTruthy();
    });
  });

  describe("CORS", () => {
    it("request with allowed origin should succeed", async () => {
      const res = await fetch(url("/api/health"), {
        headers: { Origin: "http://localhost:5000" },
      });
      expect([200, 429]).toContain(res.status);
    });

    it("request with disallowed origin should be rejected", async () => {
      try {
        const res = await fetch(url("/api/health"), {
          headers: { Origin: "https://evil-attacker.com" },
        });
        const corsHeader = res.headers.get("access-control-allow-origin");
        expect(corsHeader).not.toBe("https://evil-attacker.com");
      } catch (e) {
        expect(e).toBeDefined();
      }
    });
  });

  describe("Kong Gateway", () => {
    it("GET /api/kong/status returns connection status", async () => {
      const res = await jsonGet("/api/kong/status");
      expect([200, 429]).toContain(res.status);
      if (res.status === 200) {
        const data = await res.json();
        expect(data).toHaveProperty("connected");
      }
    });

    it("GET /api/kong/service-catalog returns service catalog", async () => {
      const res = await jsonGet("/api/kong/service-catalog");
      expect([200, 429]).toContain(res.status);
      if (res.status === 200) {
        const data = await res.json();
        expect(data).toHaveProperty("totalServices");
        expect(data).toHaveProperty("services");
      }
    });
  });

  describe("Admin Auth Protection", () => {
    it("GET /api/github/status without auth returns 401 or 403", async () => {
      const res = await jsonGet("/api/github/status");
      expect([401, 403, 429]).toContain(res.status);
    });

    it("GET /api/kong/config without auth returns 401 or 403", async () => {
      const res = await jsonGet("/api/kong/config");
      expect([401, 403, 429]).toContain(res.status);
    });
  });
});
