// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — PROPRIETARY AND CONFIDENTIAL
// See LICENSE in the repository root for full terms.

import * as pty from "node-pty";
import crypto from "crypto";

export interface TerminalSession {
  id: string;
  ownerId: string;
  ptyProcess: pty.IPty;
  createdAt: number;
  lastActivity: number;
  cols: number;
  rows: number;
  exitHandlerAttached: boolean;
}

const sessions = new Map<string, TerminalSession>();

const MAX_SESSIONS = 10;
const MAX_SESSIONS_PER_USER = 3;
const SESSION_TIMEOUT = 30 * 60 * 1000;

const CLUSTER_COMMAND_ALLOWLIST = [
  /^echo\s/,
  /^hostname$/,
  /^whoami$/,
  /^uname\s/,
  /^date$/,
  /^uptime$/,
  /^df\s/,
  /^free\s/,
  /^cat\s+\/proc\/(cpuinfo|meminfo|version|loadavg)$/,
  /^ls\s/,
  /^pwd$/,
  /^id$/,
  /^ps\s/,
  /^env$/,
  /^printenv$/,
];

export function isClusterCommandAllowed(command: string): boolean {
  const trimmed = command.trim();
  if (trimmed.includes(";") || trimmed.includes("&&") || trimmed.includes("||") || trimmed.includes("|") || trimmed.includes("`") || trimmed.includes("$(") || trimmed.includes("${")) return false;
  return CLUSTER_COMMAND_ALLOWLIST.some(pattern => pattern.test(trimmed));
}

export function createSession(ownerId: string, cols = 80, rows = 24): TerminalSession {
  const userSessions = [...sessions.values()].filter(s => s.ownerId === ownerId);
  if (userSessions.length >= MAX_SESSIONS_PER_USER) {
    const oldest = userSessions.sort((a, b) => a.lastActivity - b.lastActivity)[0];
    if (oldest) destroySession(oldest.id);
  }

  if (sessions.size >= MAX_SESSIONS) {
    const oldest = [...sessions.values()].sort((a, b) => a.lastActivity - b.lastActivity)[0];
    if (oldest) destroySession(oldest.id);
  }

  const id = crypto.randomBytes(8).toString("hex");
  const shell = process.env.SHELL || "/bin/bash";

  const ptyProcess = pty.spawn(shell, ["--norc", "--noprofile"], {
    name: "xterm-256color",
    cols,
    rows,
    cwd: process.env.HOME || "/home/runner",
    env: {
      ...process.env,
      TERM: "xterm-256color",
      COLORTERM: "truecolor",
      PLENUM_TERMINAL: "1",
      PS1: "\\[\\033[38;2;226;232;240m\\]salvi\\[\\033[0m\\]@\\[\\033[38;2;226;232;240m\\]plenumnode\\[\\033[0m\\] \\[\\033[38;2;96;165;250m\\]\\w\\[\\033[0m\\] $ ",
      HISTFILE: "",
    } as Record<string, string>,
  });

  const session: TerminalSession = {
    id,
    ownerId,
    ptyProcess,
    createdAt: Date.now(),
    lastActivity: Date.now(),
    cols,
    rows,
    exitHandlerAttached: false,
  };

  sessions.set(id, session);
  return session;
}

export function getSession(id: string): TerminalSession | undefined {
  return sessions.get(id);
}

export function isSessionOwner(sessionId: string, ownerId: string): boolean {
  const session = sessions.get(sessionId);
  return !!session && session.ownerId === ownerId;
}

export function destroySession(id: string): boolean {
  const session = sessions.get(id);
  if (!session) return false;
  try {
    session.ptyProcess.kill();
  } catch {}
  sessions.delete(id);
  return true;
}

export function listSessions(ownerId: string): Array<{ id: string; createdAt: number; lastActivity: number; cols: number; rows: number }> {
  return [...sessions.values()]
    .filter(s => s.ownerId === ownerId)
    .map(s => ({
      id: s.id,
      createdAt: s.createdAt,
      lastActivity: s.lastActivity,
      cols: s.cols,
      rows: s.rows,
    }));
}

export function resizeSession(id: string, cols: number, rows: number): boolean {
  const session = sessions.get(id);
  if (!session) return false;
  try {
    session.ptyProcess.resize(cols, rows);
    session.cols = cols;
    session.rows = rows;
    session.lastActivity = Date.now();
  } catch {}
  return true;
}

setInterval(() => {
  const now = Date.now();
  for (const [id, session] of sessions.entries()) {
    if (now - session.lastActivity > SESSION_TIMEOUT) {
      console.log(`[terminal] Session ${id} timed out after ${Math.round((now - session.lastActivity) / 1000)}s idle`);
      destroySession(id);
    }
  }
}, 60_000);
