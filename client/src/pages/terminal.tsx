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
import { TerminalSquare, Plus, X, Maximize2, Monitor, Layers, Server } from "lucide-react";

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

  const [authError, setAuthError] = useState(false);

  const connectWebSocket = useCallback(async (sid?: string) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) return;

    let token: string;
    try {
      const resp = await fetch("/api/terminal/token", { method: "POST", credentials: "include" });
      if (!resp.ok) {
        setAuthError(true);
        setConnected(false);
        return;
      }
      const data = await resp.json();
      token = data.token;
      setAuthError(false);
    } catch {
      setAuthError(true);
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
      reconnectTimer.current = setTimeout(() => connectWebSocket(sessionId || undefined), 5000);
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
        background: "#0a0e14",
        foreground: "#b3b1ad",
        cursor: "#6495ed",
        selectionBackground: "#253340",
        black: "#01060e",
        red: "#ea6c73",
        green: "#91b362",
        yellow: "#f9af4f",
        blue: "#53bdfa",
        magenta: "#fae994",
        cyan: "#90e1c6",
        white: "#c7c7c7",
        brightBlack: "#686868",
        brightRed: "#f07178",
        brightGreen: "#c2d94c",
        brightYellow: "#ffb454",
        brightBlue: "#59c2ff",
        brightMagenta: "#ffee99",
        brightCyan: "#95e6cb",
        brightWhite: "#ffffff",
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
    <div className="min-h-screen bg-[#0a0e14]" data-testid="terminal-page">
      <div className="max-w-[1400px] mx-auto px-4 py-6">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-3">
            <TerminalSquare className="w-6 h-6 text-[#6495ed]" />
            <h1 className="text-xl font-bold text-white">PlenumNode Terminal</h1>
            <Badge
              variant={connected ? "default" : "destructive"}
              className={connected ? "bg-green-600/20 text-green-400 border-green-600/30" : ""}
              data-testid="connection-status"
            >
              {connected ? "Connected" : "Disconnected"}
            </Badge>
            {sessionId && (
              <Badge variant="outline" className="text-[#6495ed] border-[#6495ed]/30" data-testid="session-id">
                Session: {sessionId.slice(0, 8)}
              </Badge>
            )}
          </div>

          <div className="flex items-center gap-2">
            <Button
              size="sm"
              variant={clusterMode ? "default" : "outline"}
              onClick={() => { setClusterMode(!clusterMode); handleListSessions(); }}
              className={clusterMode ? "bg-[#6495ed] hover:bg-[#5a87d8]" : "border-gray-700 text-gray-300 hover:bg-gray-800"}
              data-testid="toggle-cluster-mode"
            >
              <Layers className="w-4 h-4 mr-1" />
              Cluster Shell
            </Button>
            <Button
              size="sm"
              variant="outline"
              onClick={handleNewSession}
              className="border-gray-700 text-gray-300 hover:bg-gray-800"
              data-testid="new-session-btn"
            >
              <Plus className="w-4 h-4 mr-1" />
              New Session
            </Button>
            <Button
              size="sm"
              variant="outline"
              onClick={handleFit}
              className="border-gray-700 text-gray-300 hover:bg-gray-800"
              data-testid="fit-terminal-btn"
            >
              <Maximize2 className="w-4 h-4" />
            </Button>
          </div>
        </div>

        {clusterMode && (
          <div className="mb-4 p-4 rounded-lg border border-[#6495ed]/20 bg-[#0d1117]" data-testid="cluster-panel">
            <div className="flex items-center gap-2 mb-3">
              <Server className="w-4 h-4 text-[#6495ed]" />
              <span className="text-sm font-medium text-white">Array3 Cluster Shell</span>
              <span className="text-xs text-gray-500">Fan-out command to all nodes via Inter-Cube tunnels</span>
            </div>
            <div className="flex gap-2 mb-3">
              <input
                type="text"
                value={clusterCommand}
                onChange={(e) => setClusterCommand(e.target.value)}
                onKeyDown={(e) => { if (e.key === "Enter") handleClusterCommand(); }}
                placeholder="Enter command to execute on all nodes..."
                className="flex-1 px-3 py-2 rounded bg-[#1a1f2e] border border-gray-700 text-white text-sm font-mono focus:border-[#6495ed] focus:outline-none"
                data-testid="cluster-command-input"
              />
              <Button
                size="sm"
                onClick={handleClusterCommand}
                disabled={clusterPending || !clusterCommand.trim()}
                className="bg-[#6495ed] hover:bg-[#5a87d8]"
                data-testid="cluster-exec-btn"
              >
                {clusterPending ? "Running..." : "Execute"}
              </Button>
            </div>
            {clusterResults.length > 0 && (
              <div className="space-y-2" data-testid="cluster-results">
                {clusterResults.map((r, i) => (
                  <div key={i} className="rounded bg-[#0a0e14] border border-gray-800 p-3">
                    <div className="flex items-center gap-2 mb-1">
                      <Monitor className="w-3 h-3 text-[#6495ed]" />
                      <span className="text-xs font-medium text-[#6495ed]">Node {r.nodeId}</span>
                      <span className="text-xs text-gray-600">{r.address}</span>
                      {r.exitCode !== null && (
                        <Badge variant={r.exitCode === 0 ? "default" : "destructive"} className="text-xs h-4">
                          exit {r.exitCode}
                        </Badge>
                      )}
                    </div>
                    <pre className="text-xs text-gray-300 font-mono whitespace-pre-wrap">{r.output || r.error || "(no output)"}</pre>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {sessions.length > 1 && (
          <div className="flex gap-1 mb-2 overflow-x-auto" data-testid="session-tabs">
            {sessions.map((s) => (
              <div
                key={s.id}
                className={`flex items-center gap-1 px-3 py-1.5 rounded-t text-xs cursor-pointer ${
                  s.id === sessionId
                    ? "bg-[#0a0e14] text-white border border-b-0 border-gray-700"
                    : "bg-[#1a1f2e] text-gray-400 hover:text-white"
                }`}
                onClick={() => handleSwitchSession(s.id)}
                data-testid={`session-tab-${s.id.slice(0, 8)}`}
              >
                <TerminalSquare className="w-3 h-3" />
                <span>{s.id.slice(0, 8)}</span>
                <button
                  onClick={(e) => { e.stopPropagation(); handleDestroySession(s.id); }}
                  className="ml-1 hover:text-red-400"
                  data-testid={`destroy-session-${s.id.slice(0, 8)}`}
                >
                  <X className="w-3 h-3" />
                </button>
              </div>
            ))}
          </div>
        )}

        {authError && (
          <div className="mb-4 p-4 rounded-lg border border-yellow-600/30 bg-yellow-900/10" data-testid="auth-error">
            <p className="text-yellow-400 text-sm">Authentication required. Please <a href="/api/login" className="underline text-[#6495ed] hover:text-[#82b1ff]">sign in</a> to access the terminal.</p>
          </div>
        )}

        <div
          ref={termRef}
          className="w-full rounded-lg border border-gray-800 overflow-hidden"
          style={{ height: clusterMode ? "calc(100vh - 400px)" : "calc(100vh - 240px)" }}
          data-testid="terminal-container"
        />
      </div>
    </div>
  );
}
