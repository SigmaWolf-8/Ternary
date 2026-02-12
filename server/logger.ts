/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 */

type LogLevel = "debug" | "info" | "warn" | "error";

const LOG_LEVELS: Record<LogLevel, number> = {
  debug: 0,
  info: 1,
  warn: 2,
  error: 3,
};

const currentLevel: LogLevel = (process.env.LOG_LEVEL as LogLevel) || "info";

function shouldLog(level: LogLevel): boolean {
  return LOG_LEVELS[level] >= LOG_LEVELS[currentLevel];
}

function formatMessage(level: LogLevel, module: string, message: string, meta?: Record<string, unknown>): string {
  const timestamp = new Date().toISOString();
  const metaStr = meta ? ` ${JSON.stringify(meta)}` : "";
  return `[${timestamp}] [${level.toUpperCase()}] [${module}] ${message}${metaStr}`;
}

export function createLogger(module: string) {
  return {
    debug(message: string, meta?: Record<string, unknown>) {
      if (shouldLog("debug")) console.debug(formatMessage("debug", module, message, meta));
    },
    info(message: string, meta?: Record<string, unknown>) {
      if (shouldLog("info")) console.info(formatMessage("info", module, message, meta));
    },
    warn(message: string, meta?: Record<string, unknown>) {
      if (shouldLog("warn")) console.warn(formatMessage("warn", module, message, meta));
    },
    error(message: string, error?: unknown, meta?: Record<string, unknown>) {
      if (shouldLog("error")) {
        const errMeta = { ...meta };
        if (error instanceof Error) {
          errMeta.errorName = error.name;
          errMeta.errorMessage = error.message;
          if (process.env.NODE_ENV === "development") {
            errMeta.stack = error.stack;
          }
        } else if (error !== undefined) {
          errMeta.errorRaw = String(error);
        }
        console.error(formatMessage("error", module, message, errMeta));
      }
    },
  };
}

export function toErrorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  return String(error);
}
