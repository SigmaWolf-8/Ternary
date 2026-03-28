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
import { tis27Hash } from "@/lib/tis27-hash";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  TerminalSquare, Plus, X, Maximize2, Monitor, Layers, Server, LogIn,
  RefreshCw, Activity, FileText, Upload, Download, Cpu, HardDrive,
  Clock, Shield, AlertTriangle, CheckCircle, XCircle, Play,
  Square, Eye, Settings, Bot,
} from "lucide-react";
import { PLATFORM } from "@shared/constants";
import {
  type TelemetryMessage,
  type ExecResultMessage,
  type OpsErrorMessage,
  type TailDataMessage,
  type OpsResponseMessage,
  type ChunkAckMessage,
  type ChunkCompleteMessage,
  type FilePushAckMessage,
  type FileDataMessage,
  type ModelSwapResultMessage,
  type OpsErrorCode,
  NINJAEXEC_SIGN_URL,
  OPS_ERROR_DISPLAY_MESSAGES,
  OPS_STATUS_COLORS,
  ENGINE_STATUS_COLORS,
} from "@shared/ops-protocol";

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

interface CrsNode {
  address: string;
  connected: boolean;
  endpoint: string | null;
}

interface OpsTimelineEntry {
  id: string;
  timestamp: number;
  type: string;
  nodeId: string;
  requestId: string;
  status: "success" | "error" | "pending" | "timeout" | "warning";
  summary: string;
  detail?: string;
}

interface NodeTelemetry {
  node_id: string;
  cpu_pct: number;
  ram_pct: number;
  ram_used_mb: number;
  ram_total_mb: number;
  disk_pct: number;
  disk_used_gb: number;
  disk_total_gb: number;
  gpu_pct: number | null;
  gpu_name: string | null;
  gpu_vram_used_mb: number | null;
  gpu_vram_total_mb: number | null;
  process_uptime_seconds: number;
  active_model: string | null;
  llm_engine_status: string;
  os_version: string;
  timestamp: string;
}

type OpsTab = "terminal" | "telemetry" | "logs" | "exec" | "files" | "models" | "timeline";

const CRS_BASE_URL = "https://plenumnet.replit.app";

function formatUptime(seconds: number): string {
  const d = Math.floor(seconds / 86400);
  const h = Math.floor((seconds % 86400) / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  if (d > 0) return `${d}d ${h}h ${m}m`;
  if (h > 0) return `${h}h ${m}m`;
  return `${m}m`;
}

function MetricBar({ label, value, max, unit, color }: {
  label: string; value: number; max: number; unit: string; color: string;
}) {
  const pct = max > 0 ? Math.min((value / max) * 100, 100) : 0;
  return (
    <div className="space-y-1">
      <div className="flex justify-between text-xs">
        <span className="text-muted-foreground">{label}</span>
        <span className="font-mono">{value.toFixed(1)}{unit} / {max.toFixed(1)}{unit}</span>
      </div>
      <div className="h-2 bg-muted rounded-full overflow-hidden">
        <div
          className="h-full rounded-full transition-all duration-500"
          style={{ width: `${pct}%`, backgroundColor: color }}
          data-testid={`metric-bar-${label.toLowerCase().replace(/\s/g, '-')}`}
        />
      </div>
    </div>
  );
}

function ApprovalGate({ script, onApprove, onReject, signing }: {
  script: string;
  onApprove: () => void;
  onReject: () => void;
  signing?: boolean;
}) {
  return (
    <Card className="p-4 border-amber-500/30 bg-amber-500/5" data-testid="approval-gate">
      <div className="flex items-center gap-2 mb-3">
        <Shield className="w-4 h-4 text-amber-400" />
        <span className="text-sm font-medium text-amber-300">Approval Required</span>
      </div>
      <div className="bg-muted/50 rounded p-3 mb-3 max-h-40 overflow-auto">
        <pre className="text-xs font-mono text-foreground whitespace-pre-wrap" data-testid="approval-script">{script}</pre>
      </div>
      <p className="text-xs text-muted-foreground mb-3">
        This script will be signed with your NinjaExec key and executed on the target node.
        Review carefully before approving.
      </p>
      {signing ? (
        <div className="flex items-center gap-2 py-1" data-testid="signing-indicator">
          <RefreshCw className="w-3.5 h-3.5 text-violet-400 animate-spin" />
          <span className="text-xs text-violet-300">Requesting signature from NinjaExec…</span>
        </div>
      ) : (
        <div className="flex gap-2">
          <Button
            size="sm"
            className="h-7 text-xs bg-green-600 hover:bg-green-700"
            onClick={onApprove}
            data-testid="approve-exec-btn"
          >
            <CheckCircle className="w-3 h-3 mr-1" />
            Approve & Sign
          </Button>
          <Button
            size="sm"
            variant="destructive"
            className="h-7 text-xs"
            onClick={onReject}
            data-testid="reject-exec-btn"
          >
            <XCircle className="w-3 h-3 mr-1" />
            Reject
          </Button>
        </div>
      )}
    </Card>
  );
}

export default function TerminalPage() {
  const termRef = useRef<HTMLDivElement>(null);
  const terminalRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const [connected, setConnected] = useState(false);
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [sessions, setSessions] = useState<SessionInfo[]>([]);
  const [selectedNode, setSelectedNode] = useState("local");
  const [clusterMode, setClusterMode] = useState(false);
  const [clusterResults, setClusterResults] = useState<ClusterResult[]>([]);
  const [clusterCommand, setClusterCommand] = useState("");
  const [clusterPending, setClusterPending] = useState(false);
  const reconnectTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const authErrorRef = useRef(false);
  const sessionIdRef = useRef<string | null>(null);

  const [authError, setAuthError] = useState(false);
  const [crsNodes, setCrsNodes] = useState<CrsNode[]>([]);
  const [loadingNodes, setLoadingNodes] = useState(true);

  const [activeTab, setActiveTab] = useState<OpsTab>("terminal");
  const [nodeTelemetry, setNodeTelemetry] = useState<Map<string, NodeTelemetry>>(new Map());
  const [opsTimeline, setOpsTimeline] = useState<OpsTimelineEntry[]>([]);
  const [opsEnabled, setOpsEnabled] = useState(false);

  const [execScript, setExecScript] = useState("");
  const [execTarget, setExecTarget] = useState("");
  const [execPending, setExecPending] = useState(false);
  const [approvalSigning, setApprovalSigning] = useState(false);
  const [execResults, setExecResults] = useState<ExecResultMessage[]>([]);
  const [pendingApproval, setPendingApproval] = useState<string | null>(null);
  const [aiProposals, setAiProposals] = useState<Array<{ id: string; script: string; rationale: string; targetNode: string; proposedAt: string }>>([]);

  const [tailPath, setTailPath] = useState("");
  const [tailLines, setTailLines] = useState(50);
  const [tailFollow, setTailFollow] = useState(false);
  const [activeTailId, setActiveTailId] = useState<string | null>(null);
  const [tailOutput, setTailOutput] = useState<string[]>([]);

  const addTimelineEntry = useCallback((entry: Omit<OpsTimelineEntry, "id" | "timestamp">) => {
    setOpsTimeline(prev => [{
      ...entry,
      id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      timestamp: Date.now(),
    }, ...prev].slice(0, 200));
  }, []);

  const fetchCrsNodes = useCallback(async () => {
    setLoadingNodes(true);
    try {
      const resp = await fetch(`${CRS_BASE_URL}/api/salvi/inter-cube/relay/status`);
      if (resp.ok) {
        const data = await resp.json();
        setCrsNodes(data.nodes || []);
      }
    } catch {
      setCrsNodes([]);
    } finally {
      setLoadingNodes(false);
    }
  }, []);

  const fetchOpsStatus = useCallback(async () => {
    try {
      const resp = await fetch("/api/ops/status");
      if (resp.ok) {
        const data = await resp.json();
        setOpsEnabled(data.ops_enabled || false);
        if (data.nodes) {
          const newTelemetry = new Map<string, NodeTelemetry>();
          for (const node of data.nodes) {
            if (node.last_telemetry) {
              newTelemetry.set(node.node_id, node.last_telemetry);
            }
          }
          setNodeTelemetry(newTelemetry);
        }
      }
    } catch (e) {
      console.error("[ops] Telemetry fetch failed:", e);
    }
  }, []);

  useEffect(() => {
    fetchCrsNodes();
    fetchOpsStatus();
    const nodesInterval = setInterval(fetchCrsNodes, 30000);
    const opsInterval = setInterval(fetchOpsStatus, 15000);
    return () => { clearInterval(nodesInterval); clearInterval(opsInterval); };
  }, [fetchCrsNodes, fetchOpsStatus]);

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
          case "remote_connected":
            terminalRef.current?.reset();
            terminalRef.current?.writeln(`\x1b[38;2;96;165;250m[Connected to remote node ${msg.address}]\x1b[0m\r\n`);
            break;
          case "local_connected":
            terminalRef.current?.reset();
            terminalRef.current?.writeln(`\x1b[38;2;96;165;250m[Connected to local CRS node]\x1b[0m\r\n`);
            break;
          case "error":
            terminalRef.current?.writeln(`\r\n\x1b[38;2;255;100;100m[Error: ${msg.message}]\x1b[0m`);
            break;
          case "ops_message":
            if (msg.data && typeof msg.data === "object" && typeof msg.data.type === "string") {
              handleOpsMessage(msg.data as OpsResponseMessage);
            }
            break;
          case "propose-exec":
            if (msg.proposed_script && msg.target_node_id) {
              setAiProposals(prev => [{
                id: msg.proposal_id || `ai-${Date.now()}`,
                script: msg.proposed_script,
                rationale: msg.rationale || "AI-generated script",
                targetNode: msg.target_node_id,
                proposedAt: msg.proposed_at || new Date().toISOString(),
              }, ...prev].slice(0, 20));
              addTimelineEntry({
                type: "exec",
                nodeId: msg.target_node_id,
                requestId: msg.proposal_id || `ai-${Date.now()}`,
                status: "pending",
                summary: `AI proposed script for review: ${msg.rationale || "pending approval"}`,
              });
              setActiveTab("exec");
            }
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

  const handleOpsMessage = useCallback((data: OpsResponseMessage) => {
    if (!data || !data.type) return;

    switch (data.type) {
      case "exec-result": {
        const result = data as ExecResultMessage;
        setExecResults(prev => [result, ...prev].slice(0, 50));
        setExecPending(false);
        addTimelineEntry({
          type: "exec-result",
          nodeId: result.node_id,
          requestId: result.request_id,
          status: result.timed_out ? "timeout" : result.exit_code === 0 ? "success" : "error",
          summary: `Script ${result.timed_out ? "timed out" : `exited ${result.exit_code}`} (${result.duration_ms}ms)`,
          detail: result.stdout || result.stderr,
        });
        break;
      }
      case "tail-data": {
        const tail = data as TailDataMessage;
        setTailOutput(prev => [...prev, tail.data].slice(-1000));
        break;
      }
      case "telemetry": {
        const telem = data;
        const snapshot: NodeTelemetry = {
          node_id: telem.node_id,
          cpu_pct: telem.cpu_pct,
          ram_pct: telem.ram_pct,
          ram_used_mb: telem.ram_used_mb,
          ram_total_mb: telem.ram_total_mb,
          disk_pct: telem.disk_pct,
          disk_used_gb: telem.disk_used_gb,
          disk_total_gb: telem.disk_total_gb,
          gpu_pct: telem.gpu_pct,
          gpu_name: telem.gpu_name,
          gpu_vram_used_mb: telem.gpu_vram_used_mb,
          gpu_vram_total_mb: telem.gpu_vram_total_mb,
          process_uptime_seconds: telem.process_uptime_seconds,
          active_model: telem.active_model,
          llm_engine_status: telem.llm_engine_status,
          os_version: telem.os_version,
          timestamp: new Date().toISOString(),
        };
        setNodeTelemetry(prev => {
          const next = new Map(prev);
          next.set(snapshot.node_id, snapshot);
          return next;
        });
        break;
      }
      case "ops-error": {
        const err = data as OpsErrorMessage;
        const displayMsg = OPS_ERROR_DISPLAY_MESSAGES[err.error_code as OpsErrorCode] || err.message;
        addTimelineEntry({
          type: "ops-error",
          nodeId: err.node_id,
          requestId: err.request_id,
          status: "error",
          summary: displayMsg,
          detail: err.error_code !== err.message ? `${err.error_code}: ${err.message}` : undefined,
        });
        break;
      }
      case "file-push-ack": {
        const ack = data;
        addTimelineEntry({
          type: "file-push-ack",
          nodeId: ack.node_id,
          requestId: ack.request_id,
          status: "success",
          summary: `File pushed: ${ack.file_path} (${ack.bytes_written} bytes)`,
        });
        break;
      }
      case "file-data": {
        const fileResp = data;
        setFilePullResult({
          path: fileResp.file_path,
          data: fileResp.data_base64,
          size: fileResp.size_bytes,
          hash: fileResp.tis27_hash,
        });
        addTimelineEntry({
          type: "file-data",
          nodeId: fileResp.node_id,
          requestId: fileResp.request_id,
          status: "success",
          summary: `File received: ${fileResp.file_path} (${fileResp.size_bytes} bytes)`,
        });
        break;
      }
      case "chunk-ack": {
        const chunkAck = data;
        if (chunkAck.chunk_index === -1 && chunkAck.success !== false && chunkAck.transfer_id) {
          setChunkTransfer(prev => prev ? { ...prev, transferId: chunkAck.transfer_id, initAckReceived: true } : prev);
        } else if (chunkAck.success !== false && chunkTransferRef.current?.active) {
          setChunkTransfer(prev => prev ? { ...prev, sentChunks: (chunkAck.chunk_index ?? prev.sentChunks) + 1 } : prev);
        }
        if (chunkAck.success === false) {
          setChunkTransfer(prev => prev ? { ...prev, status: "error", active: false, errorMessage: chunkAck.error_message || "Chunk rejected" } : prev);
          addTimelineEntry({
            type: "chunk-ack",
            nodeId: chunkAck.node_id,
            requestId: chunkAck.request_id,
            status: "error",
            summary: `Chunk ${chunkAck.chunk_index ?? "?"} of transfer ${chunkAck.transfer_id}: failed`,
          });
        }
        break;
      }
      case "chunk-complete": {
        const complete = data;
        setChunkTransfer(prev => prev ? {
          ...prev,
          status: complete.success ? "complete" : "error",
          active: false,
          sentChunks: complete.success ? prev.totalChunks : prev.sentChunks,
          errorMessage: complete.success ? undefined : (complete.error_message || "Transfer failed"),
        } : prev);
        addTimelineEntry({
          type: "chunk-complete",
          nodeId: complete.node_id,
          requestId: complete.request_id,
          status: complete.success === false ? "error" : "success",
          summary: `Transfer ${complete.transfer_id}: ${complete.success ? `completed (${complete.total_bytes} bytes, hash verified: ${complete.tis27_hash_verified})` : complete.error_message || "failed"}`,
        });
        break;
      }
      case "model-swap-result": {
        const swap = data;
        const engineStatus = swap.engine_status;
        const isCritical = ["recovery_failed"].includes(engineStatus);
        const isRecovered = ["running_rollback", "running_restarted"].includes(engineStatus);
        let swapSummary: string;
        if (swap.success) {
          swapSummary = `Model swapped to ${swap.new_model} — engine: ${engineStatus}`;
        } else if (isCritical) {
          swapSummary = `CRITICAL: Model swap failed — engine ${engineStatus}. ${swap.error_message || ""} ` +
            `${swap.rollback_performed ? "(rollback attempted)" : "(no rollback)"} — manual intervention required`;
        } else if (isRecovered) {
          swapSummary = `Model swap failed but original model restored — engine: ${engineStatus}` +
            `${swap.rollback_verified ? " (verified)" : " (unverified)"}`;
        } else {
          swapSummary = `Model swap failed: ${swap.error_message || "unknown error"}` +
            `${swap.rollback_performed ? " (rollback performed)" : ""} — engine: ${engineStatus}`;
        }
        addTimelineEntry({
          type: "model-swap",
          nodeId: swap.node_id,
          requestId: swap.request_id,
          status: isCritical ? "error" : swap.success ? "success" : isRecovered ? "warning" : "error",
          summary: swapSummary,
        });
        break;
      }
    }
  }, [addTimelineEntry]);

  useEffect(() => {
    if (!termRef.current) return;

    const term = new Terminal({
      cursorBlink: true,
      fontSize: 18,
      fontFamily: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace",
      theme: {
        background: "#0a0a0f",
        foreground: "#e2e8f0",
        cursor: "#60a5fa",
        selectionBackground: "#1e293b",
        black: "#0a0a0f",
        red: "#f87171",
        green: "#60a5fa",
        yellow: "#fbbf24",
        blue: "#60a5fa",
        magenta: "#c084fc",
        cyan: "#22d3ee",
        white: "#f1f5f9",
        brightBlack: "#64748b",
        brightRed: "#fb7185",
        brightGreen: "#93c5fd",
        brightYellow: "#fcd34d",
        brightBlue: "#93c5fd",
        brightMagenta: "#d8b4fe",
        brightCyan: "#67e8f9",
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
  }, []);

  useEffect(() => {
    if (wsRef.current?.readyState !== WebSocket.OPEN) return;
    if (selectedNode === "local") {
      wsRef.current.send(JSON.stringify({ type: "connect_local" }));
    } else {
      terminalRef.current?.reset();
      terminalRef.current?.writeln(`\x1b[38;2;96;165;250m[Connecting to ${selectedNode}...]\x1b[0m`);
      terminalRef.current?.writeln(`\x1b[38;2;150;150;150m[Remote shell requires daemon recompile — input will work once PTY is available on the target node]\x1b[0m\r\n`);
      wsRef.current.send(JSON.stringify({ type: "connect_remote", address: selectedNode }));
    }
  }, [selectedNode]);

  const handleNewSession = () => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({ type: "new_session" }));
    }
  };

  const handleSwitchSession = (sid: string) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({ type: "attach", sessionId: sid }));
    }
  };

  const handleDestroySession = (sid: string) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({ type: "destroy", sessionId: sid }));
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

  const handleFullscreen = () => {
    const el = document.querySelector('[data-testid="terminal-page"]') as HTMLElement;
    if (!el) return;
    if (document.fullscreenElement) {
      document.exitFullscreen().catch(() => {});
    } else {
      el.requestFullscreen().catch(() => {});
    }
    setTimeout(() => { try { fitAddonRef.current?.fit(); } catch {} }, 200);
  };

  const handleListSessions = () => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({ type: "list_sessions" }));
    }
  };

  const handleSubmitExec = () => {
    if (!execScript.trim() || !execTarget) return;
    setPendingApproval(execScript);
  };

  const handleApproveExec = async () => {
    if (!pendingApproval || !execTarget) return;
    setApprovalSigning(true);
    const requestId = `exec-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;

    const canonicalMsg = {
      type: "exec",
      node_id: execTarget,
      request_id: requestId,
      script: pendingApproval,
      language: "powershell",
    };

    const signed = await signOpsRequest(canonicalMsg, requestId);
    setApprovalSigning(false);
    if (!signed) {
      addTimelineEntry({
        type: "exec",
        nodeId: execTarget,
        requestId,
        status: "error",
        summary: "Signature required — execution blocked",
        detail: "NinjaExec signing agent unavailable or did not return a valid signature. No unsigned operations are permitted.",
      });
      return;
    }

    const execMsg = {
      ...canonicalMsg,
      signature: signed.signature,
      operator_fingerprint: signed.fingerprint,
    };

    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(execMsg));
      setExecPending(true);
      addTimelineEntry({
        type: "exec",
        nodeId: execTarget,
        requestId,
        status: "pending",
        summary: `Executing signed script on ${execTarget}`,
        detail: pendingApproval,
      });
    }
    setPendingApproval(null);
  };

  const handleRejectExec = () => {
    setPendingApproval(null);
  };

  const canonicalStringify = (obj: Record<string, unknown>): string => {
    const sorted: Record<string, unknown> = {};
    for (const key of Object.keys(obj).sort()) {
      if (key !== "signature" && key !== "operator_fingerprint") {
        sorted[key] = obj[key];
      }
    }
    return JSON.stringify(sorted);
  };

  const signOpsRequest = async (payload: Record<string, unknown>, requestId: string): Promise<{ signature: string; fingerprint: string } | null> => {
    try {
      const canonical = canonicalStringify(payload);
      const signResp = await fetch(NINJAEXEC_SIGN_URL, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ payload: canonical, request_id: requestId }),
      });
      if (signResp.ok) {
        const signData = await signResp.json();
        if (signData.signature && signData.fingerprint) {
          return { signature: signData.signature, fingerprint: signData.fingerprint };
        }
      }
    } catch (e) {
      console.error("[ops] NinjaExec signing failed:", e);
      addTimelineEntry({ type: "exec", nodeId: "", requestId, status: "error", summary: `Signing failed: ${e instanceof Error ? e.message : "NinjaExec unreachable"}` });
    }
    return null;
  };

  const handleTailRequest = async () => {
    if (!tailPath.trim() || !execTarget) return;
    const requestId = `tail-${Date.now()}`;
    const baseMsg = {
      type: "tail",
      node_id: execTarget,
      request_id: requestId,
      file_path: tailPath,
      lines: tailLines,
      follow: tailFollow,
    };
    const signed = await signOpsRequest(baseMsg, requestId);
    if (!signed) {
      addTimelineEntry({
        type: "tail",
        nodeId: execTarget,
        requestId,
        status: "error",
        summary: "Signing failed — tail request blocked",
        detail: "NinjaExec signing agent unavailable. All ops require valid TL-DSA signatures.",
      });
      return;
    }
    const tailMsg = { ...baseMsg, signature: signed.signature, operator_fingerprint: signed.fingerprint };
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(tailMsg));
      setTailOutput([]);
      if (tailFollow) {
        setActiveTailId(requestId);
      }
    }
  };

  const handleTailStop = async () => {
    if (!activeTailId || !execTarget) return;
    const requestId = `tail-stop-${Date.now()}`;
    const baseMsg = {
      type: "tail-stop",
      node_id: execTarget,
      request_id: requestId,
      original_request_id: activeTailId,
    };
    const signed = await signOpsRequest(baseMsg, requestId);
    if (!signed) return;
    const stopMsg = { ...baseMsg, signature: signed.signature, operator_fingerprint: signed.fingerprint };
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(stopMsg));
    }
    setActiveTailId(null);
  };

  const handleToggleOps = async () => {
    try {
      const resp = await fetch(`/api/ops/${opsEnabled ? "disable" : "enable"}`, { method: "POST" });
      if (resp.ok) {
        const data = await resp.json();
        setOpsEnabled(data.ops_enabled);
      }
    } catch (e) {
      console.error("[ops] Failed to toggle ops channel:", e);
    }
  };

  const allNodes = [
    { id: "local", label: "CRS (Local)", address: "this-node", connected: true },
    ...crsNodes.map(n => ({
      id: n.address,
      label: `Node ${n.address}`,
      address: n.address,
      connected: n.connected,
    })),
  ];

  const connectedNodes = crsNodes.filter(n => n.connected);

  const [filePushPath, setFilePushPath] = useState("");
  const [filePullPath, setFilePullPath] = useState("");
  const [filePullResult, setFilePullResult] = useState<{ path: string; data: string; size: number; hash: string } | null>(null);
  const [modelPath, setModelPath] = useState("");
  const [modelName, setModelName] = useState("");
  const [previousModelPath, setPreviousModelPath] = useState("");
  const [modelEnginePort, setModelEnginePort] = useState(8080);
  const [chunkTransfer, setChunkTransfer] = useState<{
    active: boolean;
    transferId: string;
    fileName: string;
    totalChunks: number;
    sentChunks: number;
    totalBytes: number;
    status: "uploading" | "complete" | "error" | "cancelled";
    errorMessage?: string;
    initAckReceived?: boolean;
  } | null>(null);
  const chunkTransferRef = useRef(chunkTransfer);
  chunkTransferRef.current = chunkTransfer;

  const tabs: { id: OpsTab; label: string; icon: typeof TerminalSquare }[] = [
    { id: "terminal", label: "Terminal", icon: TerminalSquare },
    { id: "exec", label: "Exec", icon: Play },
    { id: "telemetry", label: "Telemetry", icon: Activity },
    { id: "logs", label: "Logs", icon: FileText },
    { id: "files", label: "Files", icon: HardDrive },
    { id: "models", label: "Models", icon: Cpu },
    { id: "timeline", label: "Timeline", icon: Clock },
  ];

  return (
    <div className="h-screen bg-background flex flex-col overflow-hidden" data-testid="terminal-page">
      <div className="px-4 py-2 border-b border-border flex-shrink-0">
        <div className="flex flex-wrap items-center gap-3 justify-between">
          <div className="flex items-center gap-3">
            <TerminalSquare className="w-5 h-5 text-primary" />
            <h1 className="text-lg font-semibold" data-testid="text-terminal-title">
              PlenumNode Ops Console
            </h1>
            <Badge
              variant={connected ? "secondary" : "destructive"}
              className={connected ? "bg-blue-500/20 text-blue-300 border-blue-500/30" : ""}
              data-testid="connection-status"
            >
              {connected ? "Connected" : authError ? "Auth Required" : "Disconnected"}
            </Badge>
            <Badge
              variant="outline"
              className={`text-xs cursor-pointer ${opsEnabled ? "border-green-500/30 text-green-400" : "border-gray-500/30 text-gray-400"}`}
              onClick={handleToggleOps}
              data-testid="ops-enabled-badge"
            >
              <Settings className="w-3 h-3 mr-1" />
              Ops {opsEnabled ? "ON" : "OFF"}
            </Badge>
            {sessionId && (
              <Badge variant="outline" className="text-xs" data-testid="session-id">
                Session: {sessionId.slice(0, 8)}
              </Badge>
            )}
          </div>

          <div className="flex items-center gap-2">
            <Select value={selectedNode} onValueChange={setSelectedNode}>
              <SelectTrigger className="w-[280px] h-8 text-xs" data-testid="node-selector">
                <SelectValue placeholder="Select node" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="local" data-testid="node-option-local">
                  <span className="font-medium">CRS (Local)</span>
                </SelectItem>
                {crsNodes.map(node => (
                  <SelectItem key={node.address} value={node.address} data-testid={`node-option-${node.address}`}>
                    <span className="flex items-center gap-2">
                      <span
                        className={`inline-block w-2.5 h-2.5 rounded-full ${node.connected ? "bg-blue-400" : "bg-red-500"}`}
                        style={node.connected ? { boxShadow: "0 0 6px 2px rgba(96, 165, 250, 0.7)" } : {}}
                      />
                      <span className="font-mono font-medium">{node.address}</span>
                      {node.endpoint && <span className="text-muted-foreground">{node.endpoint}</span>}
                    </span>
                  </SelectItem>
                ))}
                {loadingNodes && crsNodes.length === 0 && (
                  <SelectItem value="_loading" disabled>Loading nodes...</SelectItem>
                )}
              </SelectContent>
            </Select>

            <Button
              size="sm"
              variant="ghost"
              className="h-8 w-8 p-0"
              onClick={() => { fetchCrsNodes(); fetchOpsStatus(); }}
              title="Refresh node list"
              data-testid="refresh-nodes-btn"
            >
              <RefreshCw className="w-3.5 h-3.5" />
            </Button>

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
              onClick={handleFullscreen}
              data-testid="fit-terminal-btn"
            >
              <Maximize2 className="w-3.5 h-3.5" />
            </Button>
          </div>
        </div>
      </div>

      {authError && (
        <div className="px-4 pt-3 flex-shrink-0">
          <Card className="p-3 border-amber-500/30 bg-amber-500/5" data-testid="auth-error">
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

      {crsNodes.length > 0 && (
        <div className="px-4 pt-2 flex-shrink-0">
          <div className="flex gap-2 flex-wrap" data-testid="node-status-bar">
            {crsNodes.map(node => (
              <Badge
                key={node.address}
                variant="outline"
                className={`text-xs font-mono cursor-pointer ${
                  node.connected
                    ? "border-blue-500/30 text-blue-400"
                    : "border-red-500/30 text-red-400"
                } ${selectedNode === node.address ? "bg-primary/10 border-primary" : ""}`}
                onClick={() => setSelectedNode(node.address)}
                data-testid={`node-badge-${node.address}`}
              >
                <span
                  className={`inline-block w-2.5 h-2.5 rounded-full mr-1.5 ${node.connected ? "bg-blue-400" : "bg-red-500"}`}
                  style={node.connected ? { boxShadow: "0 0 6px 2px rgba(96, 165, 250, 0.7)" } : {}}
                />
                {node.address} {node.endpoint || ""}
              </Badge>
            ))}
          </div>
        </div>
      )}

      <div className="px-4 pt-2 flex-shrink-0">
        <div className="flex gap-1 border-b border-border" data-testid="ops-tabs">
          {tabs.map(tab => (
            <button
              key={tab.id}
              onClick={() => {
                setActiveTab(tab.id);
                if (tab.id === "terminal") {
                  setTimeout(() => { try { fitAddonRef.current?.fit(); } catch {} }, 100);
                }
              }}
              className={`flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium border-b-2 transition-colors ${
                activeTab === tab.id
                  ? "border-primary text-primary"
                  : "border-transparent text-muted-foreground hover:text-foreground hover:border-border"
              }`}
              data-testid={`tab-${tab.id}`}
            >
              <tab.icon className="w-3.5 h-3.5" />
              {tab.label}
            </button>
          ))}
        </div>
      </div>

      {clusterMode && activeTab === "terminal" && (
        <div className="px-4 pt-2 flex-shrink-0">
          <Card className="p-3" data-testid="cluster-panel">
            <div className="flex items-center gap-2 mb-2">
              <Server className="w-3.5 h-3.5 text-primary" />
              <span className="text-xs font-medium">Array3 Cluster Shell</span>
              <span className="text-xs text-muted-foreground">
                — Fan-out to {connectedNodes.length} connected node(s)
              </span>
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
                      <span className="text-xs font-mono font-medium text-primary">{r.nodeId}</span>
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

      {sessions.length > 1 && activeTab === "terminal" && (
        <div className="px-4 pt-2 flex-shrink-0">
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

      <div className="flex-1 px-4 py-2 min-h-0">
        {activeTab === "terminal" && (
          <div className="w-full h-full rounded-md overflow-hidden border border-border">
            <div
              ref={termRef}
              className="w-full h-full p-3"
              style={{ backgroundColor: "#0a0a0f" }}
              data-testid="terminal-container"
              onClick={() => terminalRef.current?.focus()}
            />
          </div>
        )}

        {activeTab === "exec" && (
          <div className="w-full h-full overflow-auto space-y-3" data-testid="exec-panel">
            {aiProposals.length > 0 && (
              <Card className="p-4 border-blue-500/30 bg-blue-500/5" data-testid="ai-proposals-panel">
                <div className="flex items-center gap-2 mb-3">
                  <Bot className="w-4 h-4 text-blue-400" />
                  <span className="text-sm font-medium text-blue-300">AI-Proposed Scripts</span>
                  <Badge variant="outline" className="text-xs text-blue-400 border-blue-400/30">
                    Requires Operator Approval
                  </Badge>
                </div>
                <div className="space-y-2">
                  {aiProposals.map((proposal) => (
                    <div key={proposal.id} className="rounded border border-blue-500/20 bg-muted/30 p-3" data-testid={`ai-proposal-${proposal.id}`}>
                      <p className="text-xs text-blue-300 mb-2">{proposal.rationale}</p>
                      <pre className="text-xs font-mono bg-muted/50 rounded p-2 mb-2 max-h-24 overflow-auto whitespace-pre-wrap" data-testid={`ai-proposal-script-${proposal.id}`}>{proposal.script}</pre>
                      <div className="flex items-center gap-2">
                        <Button
                          size="sm"
                          className="h-6 text-xs bg-green-600 hover:bg-green-700"
                          onClick={() => {
                            setExecScript(proposal.script);
                            setExecTarget(proposal.targetNode);
                            setPendingApproval(proposal.script);
                            setAiProposals(prev => prev.filter(p => p.id !== proposal.id));
                          }}
                          data-testid={`approve-ai-proposal-${proposal.id}`}
                        >
                          <CheckCircle className="w-3 h-3 mr-1" />
                          Review & Approve
                        </Button>
                        <Button
                          size="sm"
                          variant="destructive"
                          className="h-6 text-xs"
                          onClick={() => {
                            setAiProposals(prev => prev.filter(p => p.id !== proposal.id));
                            addTimelineEntry({ type: "exec", nodeId: proposal.targetNode, requestId: proposal.id, status: "error", summary: "AI proposal rejected by operator" });
                          }}
                          data-testid={`reject-ai-proposal-${proposal.id}`}
                        >
                          <XCircle className="w-3 h-3 mr-1" />
                          Reject
                        </Button>
                        <span className="text-xs text-muted-foreground ml-auto">Target: {proposal.targetNode}</span>
                      </div>
                    </div>
                  ))}
                </div>
              </Card>
            )}
            <Card className="p-4">
              <div className="flex items-center gap-2 mb-3">
                <Play className="w-4 h-4 text-primary" />
                <span className="text-sm font-medium">Remote Script Execution</span>
                <Badge variant="outline" className="text-xs">
                  <Shield className="w-3 h-3 mr-1" />
                  TL-DSA Signed
                </Badge>
              </div>
              <div className="space-y-3">
                <div>
                  <label className="text-xs text-muted-foreground mb-1 block">Target Node</label>
                  <Select value={execTarget} onValueChange={setExecTarget}>
                    <SelectTrigger className="h-8 text-xs" data-testid="exec-target-select">
                      <SelectValue placeholder="Select target node" />
                    </SelectTrigger>
                    <SelectContent>
                      {connectedNodes.map(node => (
                        <SelectItem key={node.address} value={node.address}>
                          <span className="font-mono">{node.address}</span>
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <div>
                  <label className="text-xs text-muted-foreground mb-1 block">Script (PowerShell)</label>
                  <textarea
                    value={execScript}
                    onChange={(e) => setExecScript(e.target.value)}
                    placeholder="Get-Process | Select-Object -First 10"
                    className="w-full h-32 px-3 py-2 rounded-md bg-muted border border-border text-foreground text-xs font-mono focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary resize-none"
                    data-testid="exec-script-input"
                  />
                </div>
                <Button
                  size="sm"
                  className="h-8 text-xs"
                  onClick={handleSubmitExec}
                  disabled={execPending || !execScript.trim() || !execTarget}
                  data-testid="exec-submit-btn"
                >
                  <Play className="w-3.5 h-3.5 mr-1" />
                  {execPending ? "Executing..." : "Submit for Approval"}
                </Button>
              </div>
            </Card>

            {pendingApproval && (
              <ApprovalGate
                script={pendingApproval}
                onApprove={handleApproveExec}
                onReject={handleRejectExec}
                signing={approvalSigning}
              />
            )}

            {execResults.length > 0 && (
              <Card className="p-4">
                <div className="flex items-center gap-2 mb-3">
                  <FileText className="w-4 h-4 text-primary" />
                  <span className="text-sm font-medium">Execution Results</span>
                </div>
                <div className="space-y-2">
                  {execResults.map((r, i) => (
                    <div key={i} className="rounded bg-muted/50 border border-border p-3" data-testid={`exec-result-${i}`}>
                      <div className="flex items-center gap-2 mb-2">
                        <Badge variant={r.exit_code === 0 ? "default" : "destructive"} className="text-xs">
                          exit {r.exit_code}
                        </Badge>
                        <span className="text-xs text-muted-foreground font-mono">{r.node_id}</span>
                        <span className="text-xs text-muted-foreground">{r.duration_ms}ms</span>
                        {r.timed_out && <Badge variant="destructive" className="text-xs">Timed Out</Badge>}
                      </div>
                      {r.stdout && (
                        <pre className="text-xs font-mono whitespace-pre-wrap text-foreground bg-background/50 rounded p-2 max-h-40 overflow-auto">{r.stdout}</pre>
                      )}
                      {r.stderr && (
                        <pre className="text-xs font-mono whitespace-pre-wrap text-red-400 bg-red-500/5 rounded p-2 mt-1 max-h-20 overflow-auto">{r.stderr}</pre>
                      )}
                    </div>
                  ))}
                </div>
              </Card>
            )}
          </div>
        )}

        {activeTab === "telemetry" && (
          <div className="w-full h-full overflow-auto space-y-3" data-testid="telemetry-panel">
            {nodeTelemetry.size === 0 ? (
              <Card className="p-8 text-center">
                <Activity className="w-8 h-8 text-muted-foreground mx-auto mb-3" />
                <p className="text-sm text-muted-foreground">No telemetry data available yet.</p>
                <p className="text-xs text-muted-foreground mt-1">
                  Telemetry heartbeats arrive every 60 seconds from connected nodes.
                </p>
              </Card>
            ) : (
              Array.from(nodeTelemetry.entries()).map(([nodeId, telem]) => (
                <Card key={nodeId} className="p-4" data-testid={`telemetry-card-${nodeId}`}>
                  <div className="flex items-center gap-2 mb-3">
                    <Server className="w-4 h-4 text-primary" />
                    <span className="text-sm font-medium font-mono">{nodeId}</span>
                    <Badge variant="outline" className="text-xs">
                      <Clock className="w-3 h-3 mr-1" />
                      {formatUptime(telem.process_uptime_seconds)}
                    </Badge>
                    <Badge variant="outline" className="text-xs">{telem.os_version}</Badge>
                    <Badge
                      variant="outline"
                      className={`text-xs ${(() => {
                        const ec = ENGINE_STATUS_COLORS[telem.llm_engine_status as keyof typeof ENGINE_STATUS_COLORS];
                        return ec ? `border-current/30 ${ec.tailwind}` : "border-gray-500/30 text-gray-400";
                      })()}`}
                      title={(() => {
                        const ec = ENGINE_STATUS_COLORS[telem.llm_engine_status as keyof typeof ENGINE_STATUS_COLORS];
                        return ec ? `${ec.label} (${ec.hex})` : telem.llm_engine_status;
                      })()}
                    >
                      {telem.llm_engine_status === "swapping" && <RefreshCw className="w-3 h-3 mr-1 animate-spin" />}
                      LLM: {ENGINE_STATUS_COLORS[telem.llm_engine_status as keyof typeof ENGINE_STATUS_COLORS]?.label || telem.llm_engine_status}
                    </Badge>
                  </div>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div className="space-y-3">
                      <MetricBar label="CPU" value={telem.cpu_pct} max={100} unit="%" color="#60a5fa" />
                      <MetricBar label="RAM" value={telem.ram_used_mb} max={telem.ram_total_mb} unit=" MB" color="#c084fc" />
                      <MetricBar label="Disk" value={telem.disk_used_gb} max={telem.disk_total_gb} unit=" GB" color="#fbbf24" />
                    </div>
                    <div className="space-y-3">
                      {telem.gpu_name && (
                        <>
                          <div className="flex items-center gap-2">
                            <Cpu className="w-3.5 h-3.5 text-muted-foreground" />
                            <span className="text-xs">{telem.gpu_name}</span>
                          </div>
                          <MetricBar
                            label="GPU"
                            value={telem.gpu_pct || 0}
                            max={100}
                            unit="%"
                            color="#22d3ee"
                          />
                          {telem.gpu_vram_used_mb != null && telem.gpu_vram_total_mb != null && (
                            <MetricBar
                              label="VRAM"
                              value={telem.gpu_vram_used_mb}
                              max={telem.gpu_vram_total_mb}
                              unit=" MB"
                              color="#22d3ee"
                            />
                          )}
                        </>
                      )}
                      {telem.active_model && (
                        <div className="flex items-center gap-2">
                          <HardDrive className="w-3.5 h-3.5 text-muted-foreground" />
                          <span className="text-xs">Model: {telem.active_model}</span>
                        </div>
                      )}
                      <div className="text-xs text-muted-foreground">
                        Last updated: {new Date(telem.timestamp).toLocaleTimeString()}
                      </div>
                    </div>
                  </div>
                </Card>
              ))
            )}
          </div>
        )}

        {activeTab === "logs" && (
          <div className="w-full h-full overflow-auto space-y-3" data-testid="logs-panel">
            <Card className="p-4">
              <div className="flex items-center gap-2 mb-3">
                <Eye className="w-4 h-4 text-primary" />
                <span className="text-sm font-medium">Remote Log Tail</span>
              </div>
              <div className="flex gap-2 mb-3">
                <Select value={execTarget} onValueChange={setExecTarget}>
                  <SelectTrigger className="w-[200px] h-8 text-xs" data-testid="tail-target-select">
                    <SelectValue placeholder="Target node" />
                  </SelectTrigger>
                  <SelectContent>
                    {connectedNodes.map(node => (
                      <SelectItem key={node.address} value={node.address}>
                        <span className="font-mono">{node.address}</span>
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <input
                  type="text"
                  value={tailPath}
                  onChange={(e) => setTailPath(e.target.value)}
                  placeholder=".plenumnet/logs/daemon.log"
                  className="flex-1 px-3 py-1.5 rounded-md bg-muted border border-border text-foreground text-xs font-mono focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
                  data-testid="tail-path-input"
                />
                <input
                  type="number"
                  value={tailLines}
                  onChange={(e) => setTailLines(parseInt(e.target.value) || 50)}
                  className="w-20 px-2 py-1.5 rounded-md bg-muted border border-border text-foreground text-xs font-mono focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
                  data-testid="tail-lines-input"
                  min={1}
                  max={500}
                />
                <label className="flex items-center gap-1.5 text-xs cursor-pointer" data-testid="tail-follow-toggle">
                  <input
                    type="checkbox"
                    checked={tailFollow}
                    onChange={(e) => setTailFollow(e.target.checked)}
                    className="w-3.5 h-3.5"
                  />
                  Follow
                </label>
                <Button
                  size="sm"
                  className="h-8 text-xs"
                  onClick={handleTailRequest}
                  disabled={!tailPath.trim() || !execTarget}
                  data-testid="tail-submit-btn"
                >
                  <FileText className="w-3.5 h-3.5 mr-1" />
                  Tail
                </Button>
                {activeTailId && (
                  <Button
                    size="sm"
                    variant="destructive"
                    className="h-8 text-xs"
                    onClick={handleTailStop}
                    data-testid="tail-stop-btn"
                  >
                    Stop
                  </Button>
                )}
              </div>
              <div className="rounded bg-muted/50 border border-border p-3 max-h-[60vh] overflow-auto font-mono text-xs">
                {tailOutput.length > 0 ? (
                  <pre className="whitespace-pre-wrap" data-testid="tail-output">{tailOutput.join("\n")}</pre>
                ) : (
                  <span className="text-muted-foreground" data-testid="tail-empty">No log output. Select a node and file path to begin tailing.</span>
                )}
              </div>
            </Card>
          </div>
        )}

        {activeTab === "files" && (
          <div className="w-full h-full overflow-auto space-y-3" data-testid="files-panel">
            <Card className="p-4">
              <div className="flex items-center gap-2 mb-3">
                <Upload className="w-4 h-4 text-primary" />
                <span className="text-sm font-medium">Push File to Node</span>
              </div>
              <div className="flex gap-2 mb-3">
                <Select value={execTarget} onValueChange={setExecTarget}>
                  <SelectTrigger className="w-[200px] h-8 text-xs" data-testid="file-push-target-select">
                    <SelectValue placeholder="Target node" />
                  </SelectTrigger>
                  <SelectContent>
                    {connectedNodes.map(node => (
                      <SelectItem key={node.address} value={node.address}>
                        <span className="font-mono">{node.address}</span>
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <input
                  type="text"
                  value={filePushPath}
                  onChange={(e) => setFilePushPath(e.target.value)}
                  placeholder=".plenumnet/configs/remote.json"
                  className="flex-1 px-3 py-1.5 rounded-md bg-muted border border-border text-foreground text-xs font-mono focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
                  data-testid="file-push-path-input"
                />
              </div>
              <div className="flex gap-2 mb-3">
                <input
                  type="file"
                  className="flex-1 text-xs"
                  data-testid="file-push-file-input"
                  onChange={async (e) => {
                    const file = e.target.files?.[0];
                    if (!file || !execTarget || !filePushPath.trim()) return;
                    const requestId = `push-${Date.now()}`;
                    const reader = new FileReader();
                    reader.onload = async () => {
                      const base64 = (reader.result as string).split(",")[1] || "";
                      const pushMsg = {
                        type: "file-push" as const,
                        node_id: execTarget,
                        request_id: requestId,
                        file_path: filePushPath,
                        data_base64: base64,
                        size_bytes: file.size,
                        overwrite: true,
                      };
                      const signed = await signOpsRequest(pushMsg, requestId);
                      if (!signed) {
                        addTimelineEntry({ type: "file-push", nodeId: execTarget, requestId, status: "error", summary: "Signing failed — file push blocked" });
                        return;
                      }
                      if (wsRef.current?.readyState === WebSocket.OPEN) {
                        wsRef.current.send(JSON.stringify({ ...pushMsg, signature: signed.signature, operator_fingerprint: signed.fingerprint }));
                        addTimelineEntry({ type: "file-push", nodeId: execTarget, requestId, status: "pending", summary: `Pushing ${file.name} → ${filePushPath}` });
                      }
                    };
                    reader.readAsDataURL(file);
                  }}
                />
              </div>
              <p className="text-xs text-muted-foreground">Max 5 MB. Whitelisted paths: .plenumnet/ops/, .plenumnet/logs/, .plenumnet/configs/, .plenumnet/transfers/, .plenumnet/models/</p>
            </Card>

            <Card className="p-4">
              <div className="flex items-center gap-2 mb-3">
                <Upload className="w-4 h-4 text-primary" />
                <span className="text-sm font-medium">Chunked GGUF Transfer (&gt;5 MB)</span>
              </div>
              <div className="flex gap-2 mb-3">
                <Select value={execTarget} onValueChange={setExecTarget}>
                  <SelectTrigger className="w-[200px] h-8 text-xs" data-testid="chunk-target-select">
                    <SelectValue placeholder="Target node" />
                  </SelectTrigger>
                  <SelectContent>
                    {connectedNodes.map(node => (
                      <SelectItem key={node.address} value={node.address}>
                        <span className="font-mono">{node.address}</span>
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <input
                  type="file"
                  accept=".gguf"
                  className="flex-1 text-xs"
                  data-testid="chunk-file-input"
                  disabled={chunkTransfer?.active || false}
                  onChange={async (e) => {
                    const file = e.target.files?.[0];
                    if (!file || !execTarget) return;
                    const CHUNK_SIZE = 512 * 1024;
                    const totalChunks = Math.ceil(file.size / CHUNK_SIZE);
                    const requestId = `chunk-${Date.now()}`;
                    const destPath = `.plenumnet/models/${file.name}`;

                    const fileBytes = new Uint8Array(await file.arrayBuffer());
                    let fullFileB64 = "";
                    const FB64_BLOCK = 8192;
                    for (let j = 0; j < fileBytes.length; j += FB64_BLOCK) {
                      fullFileB64 += String.fromCharCode(...fileBytes.subarray(j, Math.min(j + FB64_BLOCK, fileBytes.length)));
                    }
                    fullFileB64 = btoa(fullFileB64);
                    const fullHashHex = tis27Hash(fullFileB64);

                    const initMsg = {
                      type: "chunk-init" as const,
                      node_id: execTarget,
                      request_id: requestId,
                      file_path: destPath,
                      total_size_bytes: file.size,
                      chunk_count: totalChunks,
                      chunk_size_bytes: CHUNK_SIZE,
                      tis27_hash_full: fullHashHex,
                    };
                    const signed = await signOpsRequest(initMsg, requestId);
                    if (!signed) {
                      addTimelineEntry({ type: "chunk-init", nodeId: execTarget, requestId, status: "error", summary: "Signing failed — chunked transfer blocked" });
                      return;
                    }
                    if (wsRef.current?.readyState !== WebSocket.OPEN) return;

                    setChunkTransfer({
                      active: true,
                      transferId: requestId,
                      fileName: file.name,
                      totalChunks,
                      sentChunks: 0,
                      totalBytes: file.size,
                      status: "uploading",
                    });

                    wsRef.current.send(JSON.stringify({ ...initMsg, signature: signed.signature, operator_fingerprint: signed.fingerprint }));
                    addTimelineEntry({ type: "chunk-init", nodeId: execTarget, requestId, status: "pending", summary: `Starting chunked transfer: ${file.name} (${totalChunks} chunks, ${(file.size / 1048576).toFixed(1)} MB)` });

                    const waitForInitAck = () => new Promise<boolean>((resolve) => {
                      const maxWait = setTimeout(() => resolve(false), 10000);
                      const check = () => {
                        const ct = chunkTransferRef.current;
                        if (!ct?.active) { clearTimeout(maxWait); resolve(false); return; }
                        if (ct.initAckReceived) { clearTimeout(maxWait); resolve(true); return; }
                        setTimeout(check, 100);
                      };
                      setTimeout(check, 100);
                    });
                    const initAckReceived = await waitForInitAck();
                    if (!initAckReceived) {
                      setChunkTransfer(prev => prev ? { ...prev, status: "error", active: false, errorMessage: "Init-ack timeout or cancelled" } : prev);
                      return;
                    }

                    let currentTransferId = requestId;
                    const waitForChunkAck = (chunkIndex: number): Promise<boolean> => new Promise((resolve) => {
                      const ACK_TIMEOUT = 30000;
                      const maxWait = setTimeout(() => resolve(false), ACK_TIMEOUT);
                      const checkAck = () => {
                        const ct = chunkTransferRef.current;
                        if (!ct?.active) { clearTimeout(maxWait); resolve(false); return; }
                        if (ct.sentChunks > chunkIndex) { clearTimeout(maxWait); resolve(true); return; }
                        setTimeout(checkAck, 50);
                      };
                      setTimeout(checkAck, 50);
                    });

                    for (let i = 0; i < totalChunks; i++) {
                      if (!chunkTransferRef.current?.active) break;
                      currentTransferId = chunkTransferRef.current?.transferId || requestId;
                      const start = i * CHUNK_SIZE;
                      const end = Math.min(start + CHUNK_SIZE, file.size);
                      const chunkBytes = fileBytes.slice(start, end);
                      let chunkB64 = "";
                      const ENCODE_BLOCK = 8192;
                      for (let j = 0; j < chunkBytes.length; j += ENCODE_BLOCK) {
                        chunkB64 += String.fromCharCode(...chunkBytes.subarray(j, Math.min(j + ENCODE_BLOCK, chunkBytes.length)));
                      }
                      chunkB64 = btoa(chunkB64);
                      const chunkHashValue = tis27Hash(chunkB64);
                      const chunkMsg = {
                        type: "chunk-data" as const,
                        node_id: execTarget,
                        request_id: `${currentTransferId}-c${i}`,
                        transfer_id: currentTransferId,
                        chunk_index: i,
                        data_base64: chunkB64,
                        tis27_hash_chunk: chunkHashValue,
                      };
                      const chunkSigned = await signOpsRequest(chunkMsg, chunkMsg.request_id);
                      if (!chunkSigned) {
                        setChunkTransfer(prev => prev ? { ...prev, status: "error", active: false, errorMessage: "Chunk signing failed" } : prev);
                        break;
                      }
                      if (wsRef.current?.readyState === WebSocket.OPEN) {
                        wsRef.current.send(JSON.stringify({ ...chunkMsg, signature: chunkSigned.signature, operator_fingerprint: chunkSigned.fingerprint }));
                      }
                      const ackReceived = await waitForChunkAck(i);
                      if (!ackReceived) {
                        setChunkTransfer(prev => prev ? { ...prev, status: "error", active: false, errorMessage: `Chunk ${i} ack timeout` } : prev);
                        break;
                      }
                    }

                    if (wsRef.current?.readyState === WebSocket.OPEN && currentTransferId) {
                      const completeReqId = `${currentTransferId}-complete`;
                      const completeMsg = {
                        type: "chunk-complete" as const,
                        node_id: execTarget,
                        request_id: completeReqId,
                        transfer_id: currentTransferId,
                        full_hash: fullHashHex,
                      };
                      const completeSigned = await signOpsRequest(completeMsg, completeReqId);
                      if (completeSigned) {
                        wsRef.current.send(JSON.stringify({ ...completeMsg, signature: completeSigned.signature, operator_fingerprint: completeSigned.fingerprint }));
                      }
                    }
                  }}
                />
              </div>
              {chunkTransfer && (
                <div className="space-y-2" data-testid="chunk-progress">
                  <div className="flex items-center justify-between text-xs">
                    <span className="font-mono">{chunkTransfer.fileName}</span>
                    <span className={chunkTransfer.status === "complete" ? "text-green-400" : chunkTransfer.status === "error" ? "text-red-400" : "text-amber-400"}>
                      {chunkTransfer.status === "uploading" ? `${chunkTransfer.sentChunks}/${chunkTransfer.totalChunks} chunks` :
                       chunkTransfer.status === "complete" ? "Transfer complete" :
                       chunkTransfer.status === "cancelled" ? "Cancelled" :
                       `Error: ${chunkTransfer.errorMessage || "unknown"}`}
                    </span>
                  </div>
                  <div className="w-full bg-muted rounded-full h-2">
                    <div
                      className={`h-2 rounded-full transition-all ${chunkTransfer.status === "complete" ? "bg-green-500" : chunkTransfer.status === "error" ? "bg-red-500" : "bg-primary"}`}
                      style={{ width: `${chunkTransfer.totalChunks > 0 ? (chunkTransfer.sentChunks / chunkTransfer.totalChunks) * 100 : 0}%` }}
                      data-testid="chunk-progress-bar"
                    />
                  </div>
                  <div className="flex items-center justify-between text-xs text-muted-foreground">
                    <span>{(chunkTransfer.totalBytes / 1048576).toFixed(1)} MB total</span>
                    <span>{Math.round((chunkTransfer.sentChunks / Math.max(chunkTransfer.totalChunks, 1)) * 100)}%</span>
                  </div>
                  {chunkTransfer.active && (
                    <Button
                      size="sm"
                      variant="destructive"
                      className="h-7 text-xs"
                      onClick={async () => {
                        setChunkTransfer(prev => prev ? { ...prev, active: false, status: "cancelled" } : prev);
                        if (wsRef.current?.readyState === WebSocket.OPEN) {
                          const cancelReqId = `cancel-${Date.now()}`;
                          const cancelMsg = {
                            type: "transfer-cancel" as const,
                            node_id: execTarget,
                            request_id: cancelReqId,
                            transfer_id: chunkTransfer.transferId,
                          };
                          const signed = await signOpsRequest(cancelMsg, cancelReqId);
                          if (signed) {
                            wsRef.current.send(JSON.stringify({ ...cancelMsg, signature: signed.signature, operator_fingerprint: signed.fingerprint }));
                          }
                        }
                      }}
                      data-testid="chunk-cancel-btn"
                    >
                      Cancel Transfer
                    </Button>
                  )}
                </div>
              )}
              <p className="text-xs text-muted-foreground mt-2">For GGUF models &gt;5 MB. Uses 512 KB chunks with hash verification and resume support.</p>
            </Card>

            <Card className="p-4">
              <div className="flex items-center gap-2 mb-3">
                <Download className="w-4 h-4 text-primary" />
                <span className="text-sm font-medium">Pull File from Node</span>
              </div>
              <div className="flex gap-2 mb-3">
                <Select value={execTarget} onValueChange={setExecTarget}>
                  <SelectTrigger className="w-[200px] h-8 text-xs" data-testid="file-pull-target-select">
                    <SelectValue placeholder="Target node" />
                  </SelectTrigger>
                  <SelectContent>
                    {connectedNodes.map(node => (
                      <SelectItem key={node.address} value={node.address}>
                        <span className="font-mono">{node.address}</span>
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <input
                  type="text"
                  value={filePullPath}
                  onChange={(e) => setFilePullPath(e.target.value)}
                  placeholder=".plenumnet/logs/daemon.log"
                  className="flex-1 px-3 py-1.5 rounded-md bg-muted border border-border text-foreground text-xs font-mono focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
                  data-testid="file-pull-path-input"
                />
                <Button
                  size="sm"
                  className="h-8 text-xs"
                  disabled={!filePullPath.trim() || !execTarget}
                  onClick={async () => {
                    const requestId = `pull-${Date.now()}`;
                    const pullMsg = {
                      type: "file-pull" as const,
                      node_id: execTarget,
                      request_id: requestId,
                      file_path: filePullPath,
                    };
                    const signed = await signOpsRequest(pullMsg, requestId);
                    if (!signed) {
                      addTimelineEntry({ type: "file-pull", nodeId: execTarget, requestId, status: "error", summary: "Signing failed — file pull blocked" });
                      return;
                    }
                    if (wsRef.current?.readyState === WebSocket.OPEN) {
                      wsRef.current.send(JSON.stringify({ ...pullMsg, signature: signed.signature, operator_fingerprint: signed.fingerprint }));
                      addTimelineEntry({ type: "file-pull", nodeId: execTarget, requestId, status: "pending", summary: `Pulling ${filePullPath}` });
                    }
                  }}
                  data-testid="file-pull-btn"
                >
                  <Download className="w-3.5 h-3.5 mr-1" />
                  Pull
                </Button>
              </div>
              {filePullResult && (
                <div className="rounded bg-muted/50 border border-border p-3 max-h-[40vh] overflow-auto" data-testid="file-pull-result">
                  <div className="flex items-center gap-2 mb-2 text-xs text-muted-foreground">
                    <span className="font-mono">{filePullResult.path}</span>
                    <Badge variant="outline" className="text-xs">{filePullResult.size} bytes</Badge>
                    <Badge variant="outline" className="text-xs font-mono">hash: {filePullResult.hash}</Badge>
                  </div>
                  <pre className="text-xs font-mono whitespace-pre-wrap">{atob(filePullResult.data)}</pre>
                </div>
              )}
            </Card>
          </div>
        )}

        {activeTab === "models" && (
          <div className="w-full h-full overflow-auto space-y-3" data-testid="models-panel">
            <Card className="p-4">
              <div className="flex items-center gap-2 mb-3">
                <Cpu className="w-4 h-4 text-primary" />
                <span className="text-sm font-medium">GGUF Model Hot-Swap</span>
              </div>
              <div className="space-y-3">
                <div className="flex gap-2">
                  <Select value={execTarget} onValueChange={setExecTarget}>
                    <SelectTrigger className="w-[200px] h-8 text-xs" data-testid="model-target-select">
                      <SelectValue placeholder="Target node" />
                    </SelectTrigger>
                    <SelectContent>
                      {connectedNodes.map(node => (
                        <SelectItem key={node.address} value={node.address}>
                          <span className="font-mono">{node.address}</span>
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <div className="grid grid-cols-2 gap-2">
                  <div>
                    <label className="text-xs text-muted-foreground mb-1 block">Model Path (on node)</label>
                    <input
                      type="text"
                      value={modelPath}
                      onChange={(e) => setModelPath(e.target.value)}
                      placeholder=".plenumnet/models/model.gguf"
                      className="w-full px-3 py-1.5 rounded-md bg-muted border border-border text-foreground text-xs font-mono focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
                      data-testid="model-path-input"
                    />
                  </div>
                  <div>
                    <label className="text-xs text-muted-foreground mb-1 block">Model Name</label>
                    <input
                      type="text"
                      value={modelName}
                      onChange={(e) => setModelName(e.target.value)}
                      placeholder="my-model-7b-q4"
                      className="w-full px-3 py-1.5 rounded-md bg-muted border border-border text-foreground text-xs font-mono focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
                      data-testid="model-name-input"
                    />
                  </div>
                </div>
                <div className="grid grid-cols-2 gap-2">
                  <div>
                    <label className="text-xs text-muted-foreground mb-1 block">Previous Model Path (for rollback)</label>
                    <input
                      type="text"
                      value={previousModelPath}
                      onChange={(e) => setPreviousModelPath(e.target.value)}
                      placeholder=".plenumnet/models/old-model.gguf"
                      className="w-full px-3 py-1.5 rounded-md bg-muted border border-border text-foreground text-xs font-mono focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
                      data-testid="model-previous-path-input"
                    />
                  </div>
                  <div>
                    <label className="text-xs text-muted-foreground mb-1 block">Engine Port</label>
                    <input
                      type="number"
                      value={modelEnginePort}
                      onChange={(e) => setModelEnginePort(parseInt(e.target.value) || 8080)}
                      className="w-full px-3 py-1.5 rounded-md bg-muted border border-border text-foreground text-xs font-mono focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
                      data-testid="model-port-input"
                    />
                  </div>
                </div>
                <Button
                  size="sm"
                  className="h-8 text-xs"
                  disabled={!modelPath.trim() || !modelName.trim() || !execTarget}
                  onClick={async () => {
                    const requestId = `swap-${Date.now()}`;
                    const swapMsg = {
                      type: "model-swap" as const,
                      node_id: execTarget,
                      request_id: requestId,
                      model_path: modelPath,
                      model_name: modelName,
                      previous_model_path: previousModelPath,
                      engine_port: modelEnginePort,
                    };
                    const signed = await signOpsRequest(swapMsg, requestId);
                    if (!signed) {
                      addTimelineEntry({ type: "model-swap", nodeId: execTarget, requestId, status: "error", summary: "Signing failed — model swap blocked" });
                      return;
                    }
                    if (wsRef.current?.readyState === WebSocket.OPEN) {
                      wsRef.current.send(JSON.stringify({ ...swapMsg, signature: signed.signature, operator_fingerprint: signed.fingerprint }));
                      addTimelineEntry({ type: "model-swap", nodeId: execTarget, requestId, status: "pending", summary: `Swapping model → ${modelName}`, detail: `Path: ${modelPath}, Rollback: ${previousModelPath || "none"}` });
                    }
                  }}
                  data-testid="model-swap-btn"
                >
                  <RefreshCw className="w-3.5 h-3.5 mr-1" />
                  Swap Model
                </Button>
              </div>
              <div className="mt-3 rounded bg-muted/50 border border-border p-3">
                <p className="text-xs text-muted-foreground">
                  <Shield className="w-3 h-3 inline mr-1" />
                  Hot-swap performs: health check → slot erase → model load → /v1/models verification.
                  On failure, rollback to previous model is attempted automatically.
                  Only .gguf files supported. Requires TL-DSA signed operation.
                </p>
              </div>
            </Card>
          </div>
        )}

        {activeTab === "timeline" && (
          <div className="w-full h-full overflow-auto space-y-1" data-testid="timeline-panel">
            {opsTimeline.length === 0 ? (
              <Card className="p-8 text-center">
                <Clock className="w-8 h-8 text-muted-foreground mx-auto mb-3" />
                <p className="text-sm text-muted-foreground">No operations recorded yet.</p>
              </Card>
            ) : (
              opsTimeline.map(entry => (
                <div
                  key={entry.id}
                  className="flex items-start gap-3 px-3 py-2 rounded bg-muted/30 border border-border"
                  data-testid={`timeline-entry-${entry.id}`}
                >
                  <div className="mt-0.5">
                    {entry.status === "success" && <CheckCircle className={`w-4 h-4 ${OPS_STATUS_COLORS.success.tailwind}`} />}
                    {entry.status === "error" && <XCircle className={`w-4 h-4 ${OPS_STATUS_COLORS.error.tailwind}`} />}
                    {entry.status === "warning" && <AlertTriangle className={`w-4 h-4 ${OPS_STATUS_COLORS.warning.tailwind}`} />}
                    {entry.status === "pending" && <Clock className={`w-4 h-4 ${OPS_STATUS_COLORS.pending.tailwind} animate-pulse`} />}
                    {entry.status === "timeout" && <AlertTriangle className={`w-4 h-4 ${OPS_STATUS_COLORS.timeout.tailwind}`} />}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <Badge variant="outline" className="text-xs">{entry.type}</Badge>
                      <span className="text-xs font-mono text-muted-foreground" title={entry.nodeId}>{entry.nodeId}</span>
                      <span className="text-xs text-muted-foreground ml-auto">
                        {new Date(entry.timestamp).toLocaleTimeString()}
                      </span>
                    </div>
                    <p className="text-xs mt-0.5">{entry.summary}</p>
                    {entry.detail && (
                      <p className="text-xs mt-0.5 text-muted-foreground font-mono">{entry.detail}</p>
                    )}
                  </div>
                </div>
              ))
            )}
          </div>
        )}
      </div>
    </div>
  );
}
