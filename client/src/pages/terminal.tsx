// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — PROPRIETARY AND CONFIDENTIAL
// See LICENSE in the repository root for full terms.

import { useEffect, useRef, useState, useCallback } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { TerminalSquare, Plus, X, Maximize2, Monitor, Layers, Server, LogIn } from "lucide-react";
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

const ARRAY3_NODES = [
  { id: "1", label: "Node 1", address: "111.111.111.111.1" },
  { id: "2", label: "Node 2", address: "111.111.111.111.2" },
  { id: "3", label: "Node 3", address: "111.111.111.111.3" },
];

export default function TerminalPage() {
  const termRef = useRef<HTMLDivElement>(null);
  const terminalRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const [connected, setConnected] = useState(false);
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [sessions, setSessions] = useState<SessionInfo[]>([]);
  const [selectedNode, setSelectedNode] = useState("1");
  const [clusterMode, setClusterMode] = useState(false);
  const [clusterResults, setClusterResults] = useState<ClusterResult[]>([]);
  const [clusterCommand, setClusterCommand] = useState("");
  const [clusterPending, setClusterPending] = useState(false);
  const reconnectTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const authErrorRef = useRef(false);
  const sessionIdRef = useRef<string | null>(null);

  const [authError, setAuthError] = useState(false);

  const connectWebSocket = useCallback(async (sid?: string) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) return;
    if (authErrorRef.current) return;

    let token: string;
    try {
      const resp = await fetch("/api/terminal/token", { method: "POST", credentials: "include" });
      if (!resp.ok) {
        if (resp.status === 401) {
          setAuthError(true);
          authErrorRef.current = true;
        }
        setConnected(false);
        return;
      }
      const data = await resp.json();
      token = data.token;
      setAuthError(false);
      authErrorRef.current = false;
    } catch {
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
      setTimeout(() => terminalRef.current?.focus(), 100);
    };

    ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);
        switch (msg.type) {
          case "output":
            terminalRef.current?.write(msg.data);
            break;
          case "session_created":
            sessionIdRef.current = msg.sessionId;
            setSessionId(msg.sessionId);
            break;
          case "session_list":
            setSessions(msg.sessions || []);
            break;
          case "session_attached":
            sessionIdRef.current = msg.sessionId;
            setSessionId(msg.sessionId);
            break;
          case "session_ended":
            sessionIdRef.current = null;
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
        reconnectTimer.current = setTimeout(() => connectWebSocket(sessionIdRef.current || undefined), 5000);
      }
    };

    ws.onerror = () => {
      ws.close();
    };
  }, []);

  useEffect(() => {
    if (!termRef.current) return;

    const term = new Terminal({
      cursorBlink: true,
      fontSize: 16,
      fontFamily: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace",
      theme: {
        background: "#1a1410",
        foreground: "#d4c8bc",
        cursor: "#c49a6c",
        selectionBackground: "#3d2e22",
        black: "#1a1410",
        red: "#f87171",
        green: "#4ade80",
        yellow: "#fbbf24",
        blue: "#60a5fa",
        magenta: "#c084fc",
        cyan: "#22d3ee",
        white: "#e2e8f0",
        brightBlack: "#7a6b5d",
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
  }, []);

  const handleNewSession = () => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({ type: "new_session" }));
    }
  };

  const handleSwitchSession = (sid: string) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({ type: "switch_session", sessionId: sid }));
    }
  };

  const handleDestroySession = (sid: string) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({ type: "destroy_session", sessionId: sid }));
    }
  };

  const handleClusterCommand = () => {
    if (!clusterCommand.trim()) return;
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      setClusterPending(true);
      setClusterResults([]);
      wsRef.current.send(JSON.stringify({ type: "cluster_exec", command: clusterCommand }));
    }
  };

  const handleFit = () => {
    try { fitAddonRef.current?.fit(); } catch {}
  };

  const handleListSessions = () => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({ type: "list_sessions" }));
    }
  };

  const currentNode = ARRAY3_NODES.find(n => n.id === selectedNode) || ARRAY3_NODES[0];

  return (
    <div className="min-h-screen bg-background flex flex-col" data-testid="terminal-page">
      <div className="px-4 py-3 border-b border-border">
        <div className="max-w-[1400px] mx-auto">
          <div className="flex flex-wrap items-center gap-4 justify-between">
            <div className="flex items-center gap-3">
              <TerminalSquare className="w-5 h-5 text-primary" />
              <h1 className="text-lg font-semibold" data-testid="text-terminal-title">
                PlenumNode Terminal
              </h1>
              <Badge
                variant={connected ? "secondary" : "destructive"}
                className={connected ? "bg-green-500/20 text-green-300 border-green-500/30" : ""}
                data-testid="connection-status"
              >
                {connected ? "Connected" : authError ? "Auth Required" : "Disconnected"}
              </Badge>
              {sessionId && (
                <Badge variant="outline" className="text-xs" data-testid="session-id">
                  Session: {sessionId.slice(0, 8)}
                </Badge>
              )}
            </div>

            <div className="flex items-center gap-2">
              <Select value={selectedNode} onValueChange={setSelectedNode}>
                <SelectTrigger className="w-[200px] h-8 text-xs" data-testid="node-selector">
                  <SelectValue placeholder="Select node" />
                </SelectTrigger>
                <SelectContent>
                  {ARRAY3_NODES.map(node => (
                    <SelectItem key={node.id} value={node.id} data-testid={`node-option-${node.id}`}>
                      <span className="font-medium">{node.label}</span>
                      <span className="text-muted-foreground ml-2">{node.address}</span>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>

              <Button
                size="sm"
                variant={clusterMode ? "default" : "outline"}
                className="h-8 text-xs"
                onClick={() => { setClusterMode(!clusterMode); handleListSessions(); }}
                data-testid="toggle-cluster-mode"
              >
                <Layers className="w-3.5 h-3.5 mr-1" />
                Cluster
              </Button>
              <Button
                size="sm"
                variant="outline"
                className="h-8 text-xs"
                onClick={handleNewSession}
                disabled={authError}
                data-testid="new-session-btn"
              >
                <Plus className="w-3.5 h-3.5 mr-1" />
                New
              </Button>
              <Button
                size="sm"
                variant="outline"
                className="h-8 w-8 p-0"
                onClick={handleFit}
                data-testid="fit-terminal-btn"
              >
                <Maximize2 className="w-3.5 h-3.5" />
              </Button>
            </div>
          </div>
        </div>
      </div>

      {authError && (
        <div className="max-w-[1400px] mx-auto px-4 pt-4 w-full">
          <Card className="p-4 border-amber-500/30 bg-amber-500/5" data-testid="auth-error">
            <div className="flex items-center gap-3">
              <LogIn className="w-4 h-4 text-amber-400" />
              <span className="text-sm">Sign in to access the terminal.</span>
              <a href="/api/login">
                <Button size="sm" className="h-7 text-xs" data-testid="login-btn">
                  <LogIn className="w-3 h-3 mr-1" />
                  Sign In
                </Button>
              </a>
            </div>
          </Card>
        </div>
      )}

      {clusterMode && (
        <div className="max-w-[1400px] mx-auto px-4 pt-3 w-full">
          <Card className="p-3" data-testid="cluster-panel">
            <div className="flex items-center gap-2 mb-2">
              <Server className="w-3.5 h-3.5 text-primary" />
              <span className="text-xs font-medium">Array3 Cluster Shell</span>
              <span className="text-xs text-muted-foreground">— Fan-out to all {ARRAY3_NODES.length} nodes</span>
            </div>
            <div className="flex gap-2">
              <input
                type="text"
                value={clusterCommand}
                onChange={(e) => setClusterCommand(e.target.value)}
                onKeyDown={(e) => { if (e.key === "Enter") handleClusterCommand(); }}
                placeholder="Command to execute on all nodes..."
                className="flex-1 px-3 py-1.5 rounded-md bg-muted border border-border text-foreground text-xs font-mono focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
                data-testid="cluster-command-input"
              />
              <Button
                size="sm"
                className="h-7 text-xs"
                onClick={handleClusterCommand}
                disabled={clusterPending || !clusterCommand.trim()}
                data-testid="cluster-exec-btn"
              >
                {clusterPending ? "Running..." : "Execute"}
              </Button>
            </div>
            {clusterResults.length > 0 && (
              <div className="space-y-1.5 mt-2" data-testid="cluster-results">
                {clusterResults.map((r, i) => (
                  <div key={i} className="rounded bg-muted/50 border border-border p-2">
                    <div className="flex items-center gap-2 mb-0.5">
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
        </div>
      )}

      {sessions.length > 1 && (
        <div className="max-w-[1400px] mx-auto px-4 pt-2 w-full">
          <div className="flex gap-1 overflow-x-auto" data-testid="session-tabs">
            {sessions.map((s) => (
              <div
                key={s.id}
                className={`flex items-center gap-1 px-2 py-1 rounded text-xs cursor-pointer transition-colors ${
                  s.id === sessionId
                    ? "bg-primary/10 text-primary border border-primary/20"
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
        </div>
      )}

      <div className="flex-1 max-w-[1400px] mx-auto px-4 py-3 w-full">
        <div
          className="w-full h-full rounded-md overflow-hidden border border-border"
          style={{ minHeight: "400px" }}
        >
          <div
            ref={termRef}
            className="w-full h-full"
            style={{ minHeight: "400px" }}
            data-testid="terminal-container"
            onClick={() => terminalRef.current?.focus()}
          />
        </div>
      </div>
    </div>
  );
}
