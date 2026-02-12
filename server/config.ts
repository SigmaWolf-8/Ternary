/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 */

export const config = {
  port: parseInt(process.env.PORT || "5000", 10),
  nodeEnv: process.env.NODE_ENV || "development",
  isDev: (process.env.NODE_ENV || "development") === "development",
  isProd: process.env.NODE_ENV === "production",

  database: {
    url: process.env.DATABASE_URL || "",
  },

  session: {
    secret: process.env.SESSION_SECRET || process.env.REPL_ID || "dev-session-secret",
  },

  github: {
    token: process.env.GITHUB_TOKEN || "",
  },

  kong: {
    apiBase: "https://us.api.konghq.com/v2",
    token: process.env.KONG_KONNECT_TOKEN || "",
  },

  cors: {
    allowedOrigins: [
      /\.replit\.dev$/,
      /\.repl\.co$/,
      /\.replit\.app$/,
      /^https?:\/\/localhost/,
    ],
  },

  rateLimits: {
    global: { windowMs: 60_000, max: 100 },
    auth: { windowMs: 60_000, max: 20 },
    githubToken: { windowMs: 60_000, max: 10 },
    computation: { windowMs: 60_000, max: 50 },
  },

  inputBounds: {
    maxPageSize: 1000,
    maxTritCount: 1000,
    maxDataLength: 10_000,
    maxBatchCount: 100,
    maxRowCount: 10_000,
  },
} as const;
