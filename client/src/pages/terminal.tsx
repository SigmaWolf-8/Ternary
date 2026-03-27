// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — PROPRIETARY AND CONFIDENTIAL
// See LICENSE in the repository root for full terms.

import { useEffect, useRef, useState, useCallback } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import { Link } from "wouter";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { TerminalSquare, Plus, X, Maximize2, Monitor, Layers, Server, ChevronRight, LogIn } from "lucide-react";
import { PLATFORM } from "@shared/constants";

interface SessionInfo {
  id: string;
  createdAt: number;
  lastActivity: number;
  cols: number;
  rows: number;
}

interface ClusterResult {
  nodeId: string;
  address: string;
  output: string;
  error?: string;
  exitCode: number | null;
}

export default function TerminalPage() {
  const termRef = useRef<HTMLDivElement>(null);
  const terminalRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const [connected, setConnected] = useState(false);
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [sessions, setSessions] = useState<SessionInfo[]>([]);
  const [clusterMode, setClusterMode] = useState(false);
  const [clusterResults, setClusterResults] = useState<ClusterResult[]>([]);
  const [clusterCommand, setClusterCommand] = useState("");
  const [clusterPending, setClusterPending] = useState(false);
  const reconnectTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const authErrorRef = useRef(false);

  const [authError, setAuthError] = useState(false);

  const connectWebSocket = useCallback(async (sid?: string) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) return;
    if (authErrorRef.current) return;

    let token: string;
    try {
      const resp = await fetch("/api/terminal/token", { method: "POST", credentials: "include" });
      if (!resp.ok) {
        setAuthError(true);
        authErrorRef.current = true;
        setConnected(false);
        return;
      }
      const data = await resp.json();
      token = data.token;
      setAuthError(false);
      authErrorRef.current = false;
    } catch {
      setAuthError(true);
      authErrorRef.current = true;
      setConnected(false);
      return;
    }

    const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    const params = new URLSearchParams({ token });
    if (sid) params.set("session", sid);
    const url = `${protocol}//${window.location.host}/ws/terminal?${params.toString()}`;

    const ws = new WebSocket(url);
    wsRef.current = ws;

    ws.onopen = () => {
      setConnected(true);
      if (reconnectTimer.current) {
        clearTimeout(reconnectTimer.current);
        reconnectTimer.current = null;
      }
    };

    ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);
        switch (msg.type) {
          case "output":
            terminalRef.current?.write(msg.data);
            break;
          case "session_created":
            setSessionId(msg.sessionId);
            break;
          case "session_list":
            setSessions(msg.sessions || []);
            break;
          case "session_attached":
            setSessionId(msg.sessionId);
            break;
          case "session_ended":
            setSessionId(null);
            terminalRef.current?.writeln("\r\n\x1b[38;2;255;100;100m[Session ended]\x1b[0m");
            break;
          case "cluster_result":
            setClusterResults(msg.results || []);
            setClusterPending(false);
            break;
          case "error":
            terminalRef.current?.writeln(`\r\n\x1b[38;2;255;100;100m[Error: ${msg.message}]\x1b[0m`);
            break;
        }
      } catch {
        terminalRef.current?.write(event.data);
      }
    };

    ws.onclose = () => {
      setConnected(false);
      if (!authErrorRef.current) {
        reconnectTimer.current = setTimeout(() => connectWebSocket(sessionId || undefined), 5000);
      }
    };

    ws.onerror = () => {
      ws.close();
    };
  }, [sessionId]);

  useEffect(() => {
    if (!termRef.current) return;

    const term = new Terminal({
      cursorBlink: true,
      fontSize: 14,
      fontFamily: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace",
      theme: {
        background: "#0f172a",
        foreground: "#cbd5e1",
        cursor: "#6495ed",
        selectionBackground: "#334155",
        black: "#0f172a",
        red: "#f87171",
        green: "#4ade80",
        yellow: "#fbbf24",
        blue: "#60a5fa",
        magenta: "#c084fc",
        cyan: "#22d3ee",
        white: "#e2e8f0",
        brightBlack: "#64748b",
        brightRed: "#fb7185",
        brightGreen: "#86efac",
        brightYellow: "#fcd34d",
        brightBlue: "#93c5fd",
        brightMagenta: "#d8b4fe",
        brightCyan: "#67e8f9",
        brightWhite: "#f8fafc",
      },
      allowProposedApi: true,
      scrollback: 5000,
    });

    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.open(termRef.current);

    setTimeout(() => {
      try { fitAddon.fit(); } catch {}
    }, 50);

    terminalRef.current = term;
    fitAddonRef.current = fitAddon;

    term.onData((data) => {
      if (wsRef.current?.readyState === WebSocket.OPEN) {
        wsRef.current.send(JSON.stringify({ type: "input", data }));
      }
    });

    term.onResize(({ cols, rows }) => {
      if (wsRef.current?.readyState === WebSocket.OPEN) {
        wsRef.current.send(JSON.stringify({ type: "resize", cols, rows }));
      }
    });

    const handleResize = () => {
      try { fitAddon.fit(); } catch {}
    };
    window.addEventListener("resize", handleResize);

    connectWebSocket();

    return () => {
      window.removeEventListener("resize", handleResize);
      term.dispose();
      if (wsRef.current) {
        wsRef.current.close();
        wsRef.current = null;
      }
      if (reconnectTimer.current) {
        clearTimeout(reconnectTimer.current);
      }
    };
  }, [connectWebSocket]);

  const handleNewSession = () => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({ type: "new_session" }));
    }
  };

  const handleSwitchSession = (sid: string) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({ type: "attach", sessionId: sid }));
      terminalRef.current?.clear();
    }
  };

  const handleDestroySession = (sid: string) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({ type: "destroy", sessionId: sid }));
    }
  };

  const handleClusterCommand = () => {
    if (!clusterCommand.trim() || !wsRef.current || wsRef.current.readyState !== WebSocket.OPEN) return;
    setClusterPending(true);
    setClusterResults([]);
    wsRef.current.send(JSON.stringify({ type: "cluster_exec", command: clusterCommand }));
  };

  const handleFit = () => {
    try { fitAddonRef.current?.fit(); } catch {}
  };

  const handleListSessions = () => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({ type: "list_sessions" }));
    }
  };

  return (
    <div className="min-h-screen bg-background" data-testid="terminal-page">
      <div className="bg-gradient-to-b from-slate-900 to-slate-800 dark:from-slate-950 dark:to-slate-900 text-white py-8 px-4">
        <div className="max-w-[1400px] mx-auto">
          <div className="flex items-center gap-2 text-sm text-slate-400 mb-4" data-testid="text-breadcrumb">
            <Link href="/" className="hover:text-white transition-colors">
              Home
            </Link>
            <ChevronRight className="w-3 h-3" />
            <span>Node Terminal</span>
          </div>

          <div className="flex flex-wrap items-start gap-4 justify-between">
            <div>
              <div className="flex items-center gap-3 mb-2">
                <TerminalSquare className="w-8 h-8 text-blue-400" />
                <h1 className="text-3xl font-bold tracking-tight" data-testid="text-terminal-title">
                  PlenumNode Terminal
                </h1>
              </div>
              <p className="text-slate-300 max-w-2xl text-lg" data-testid="text-terminal-subtitle">
                Interactive shell access to the PlenumNET node. Manage sessions, execute commands,
                and fan out operations across the Array3 cluster via Inter-Cube tunnels.
              </p>
            </div>
            <div className="flex flex-wrap items-center gap-2">
              <Badge
                variant={connected ? "secondary" : "destructive"}
                className={connected ? "bg-green-500/20 text-green-300 border-green-500/30" : ""}
                data-testid="connection-status"
              >
                {connected ? "Connected" : authError ? "Auth Required" : "Disconnected"}
              </Badge>
              {sessionId && (
                <Badge variant="secondary" className="bg-blue-500/20 text-blue-300 border-blue-500/30" data-testid="session-id">
                  Session: {sessionId.slice(0, 8)}
                </Badge>
              )}
            </div>
          </div>
        </div>
      </div>

      <div className="max-w-[1400px] mx-auto px-4 py-6">
        {authError && (
          <Card className="mb-6 p-6 border-amber-500/30 bg-amber-500/5" data-testid="auth-error">
            <div className="flex items-start gap-4">
              <div className="p-2 rounded-md bg-amber-500/10">
                <LogIn className="w-5 h-5 text-amber-400" />
              </div>
              <div>
                <h3 className="font-semibold text-sm mb-1">Authentication Required</h3>
                <p className="text-sm text-muted-foreground mb-3">
                  Sign in to access the PlenumNode terminal. The terminal provides interactive shell access
                  and requires authenticated sessions for security.
                </p>
                <a href="/api/login">
                  <Button size="sm" data-testid="login-btn">
                    <LogIn className="w-4 h-4 mr-2" />
                    Sign In
                  </Button>
                </a>
              </div>
            </div>
          </Card>
        )}

        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-2">
            <Button
              size="sm"
              variant={clusterMode ? "default" : "outline"}
              onClick={() => { setClusterMode(!clusterMode); handleListSessions(); }}
              data-testid="toggle-cluster-mode"
            >
              <Layers className="w-4 h-4 mr-1" />
              Cluster Shell
            </Button>
            <Button
              size="sm"
              variant="outline"
              onClick={handleNewSession}
              disabled={authError}
              data-testid="new-session-btn"
            >
              <Plus className="w-4 h-4 mr-1" />
              New Session
            </Button>
            <Button
              size="sm"
              variant="outline"
              onClick={handleFit}
              data-testid="fit-terminal-btn"
            >
              <Maximize2 className="w-4 h-4" />
            </Button>
          </div>
        </div>

        {clusterMode && (
          <Card className="mb-4 p-4" data-testid="cluster-panel">
            <div className="flex items-center gap-2 mb-3">
              <Server className="w-4 h-4 text-primary" />
              <span className="text-sm font-medium">Array3 Cluster Shell</span>
              <span className="text-xs text-muted-foreground">Fan-out command to all nodes via Inter-Cube tunnels</span>
            </div>
            <div className="flex gap-2 mb-3">
              <input
                type="text"
                value={clusterCommand}
                onChange={(e) => setClusterCommand(e.target.value)}
                onKeyDown={(e) => { if (e.key === "Enter") handleClusterCommand(); }}
                placeholder="Enter command to execute on all nodes..."
                className="flex-1 px-3 py-2 rounded-md bg-muted border border-border text-foreground text-sm font-mono focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
                data-testid="cluster-command-input"
              />
              <Button
                size="sm"
                onClick={handleClusterCommand}
                disabled={clusterPending || !clusterCommand.trim()}
                data-testid="cluster-exec-btn"
              >
                {clusterPending ? "Running..." : "Execute"}
              </Button>
            </div>
            {clusterResults.length > 0 && (
              <div className="space-y-2" data-testid="cluster-results">
                {clusterResults.map((r, i) => (
                  <div key={i} className="rounded-md bg-muted/50 border border-border p-3">
                    <div className="flex items-center gap-2 mb-1">
                      <Monitor className="w-3 h-3 text-primary" />
                      <span className="text-xs font-medium text-primary">Node {r.nodeId}</span>
                      <span className="text-xs text-muted-foreground">{r.address}</span>
                      {r.exitCode !== null && (
                        <Badge variant={r.exitCode === 0 ? "default" : "destructive"} className="text-xs h-4">
                          exit {r.exitCode}
                        </Badge>
                      )}
                    </div>
                    <pre className="text-xs text-muted-foreground font-mono whitespace-pre-wrap">{r.output || r.error || "(no output)"}</pre>
                  </div>
                ))}
              </div>
            )}
          </Card>
        )}

        {sessions.length > 1 && (
          <div className="flex gap-1 mb-2 overflow-x-auto" data-testid="session-tabs">
            {sessions.map((s) => (
              <div
                key={s.id}
                className={`flex items-center gap-1 px-3 py-1.5 rounded-t text-xs cursor-pointer transition-colors ${
                  s.id === sessionId
                    ? "bg-slate-900 text-white border border-b-0 border-border"
                    : "bg-muted text-muted-foreground hover:text-foreground"
                }`}
                onClick={() => handleSwitchSession(s.id)}
                data-testid={`session-tab-${s.id.slice(0, 8)}`}
              >
                <TerminalSquare className="w-3 h-3" />
                <span>{s.id.slice(0, 8)}</span>
                <button
                  onClick={(e) => { e.stopPropagation(); handleDestroySession(s.id); }}
                  className="ml-1 hover:text-destructive"
                  data-testid={`destroy-session-${s.id.slice(0, 8)}`}
                >
                  <X className="w-3 h-3" />
                </button>
              </div>
            ))}
          </div>
        )}

        <Card className="overflow-hidden border-slate-700 dark:border-slate-800">
          <div className="flex items-center justify-between px-4 py-2 bg-slate-800 dark:bg-slate-900 border-b border-slate-700">
            <div className="flex items-center gap-2">
              <div className="flex gap-1.5">
                <div className="w-3 h-3 rounded-full bg-red-500/80" />
                <div className="w-3 h-3 rounded-full bg-yellow-500/80" />
                <div className="w-3 h-3 rounded-full bg-green-500/80" />
              </div>
              <span className="text-xs text-slate-400 ml-2 font-mono">
                salvi@plenumnode ~ {sessionId ? `session:${sessionId.slice(0, 8)}` : "ready"}
              </span>
            </div>
            <div className="flex items-center gap-2">
              {connected && (
                <span className="flex items-center gap-1.5 text-xs text-green-400">
                  <span className="w-1.5 h-1.5 rounded-full bg-green-400 animate-pulse" />
                  live
                </span>
              )}
            </div>
          </div>
          <div
            ref={termRef}
            className="w-full"
            style={{ height: clusterMode ? "calc(100vh - 520px)" : "calc(100vh - 360px)", minHeight: "300px" }}
            data-testid="terminal-container"
          />
        </Card>
      </div>
    </div>
  );
}
