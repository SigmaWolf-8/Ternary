// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division

/**
 * PlenumNET Launcher — slide-up panel for managing PlenumNET applications.
 *
 * z-index: 9999. Host app toast/notification systems should render at z-index >= 10000.
 *
 * @module LauncherPanel
 */

import { useState, useEffect, useRef, useCallback, useMemo, createContext, useContext } from "react";

type PanelState = "CLOSED" | "OPENING" | "OPEN" | "MINIMIZED" | "CLOSING";
type DaemonState = "DISCONNECTED" | "HEALTH_CHECK" | "CONNECTING" | "CONNECTED" | "RECONNECTING" | "FAILED";

interface LauncherContextValue {
  panelState: PanelState;
  setPanelState: (s: PanelState) => void;
  togglePanel: () => void;
  isActive: boolean;
  widgetMode: boolean;
}

const LauncherContext = createContext<LauncherContextValue>({
  panelState: "CLOSED",
  setPanelState: () => {},
  togglePanel: () => {},
  isActive: false,
  widgetMode: false,
});

export function useLauncher() {
  return useContext(LauncherContext);
}

const CUBE_API_HOST = typeof window !== "undefined" && import.meta.env?.VITE_CUBE_API_HOST
  ? import.meta.env.VITE_CUBE_API_HOST
  : "localhost";
const CUBE_API_PORT = typeof window !== "undefined" && import.meta.env?.VITE_CUBE_API_PORT
  ? import.meta.env.VITE_CUBE_API_PORT
  : "11124";

function isLocalhostHost(host: string) {
  return host === "localhost" || host === "127.0.0.1" || host === "::1";
}

function usesSecureTransport() {
  if (isLocalhostHost(CUBE_API_HOST)) return false;
  return typeof window !== "undefined" && window.location.protocol === "https:";
}

const DAEMON_HTTP = `${usesSecureTransport() ? "https" : "http"}://${CUBE_API_HOST}:${CUBE_API_PORT}`;

function getDaemonWsUrl() {
  const protocol = usesSecureTransport() ? "wss:" : "ws:";
  return `${protocol}//${CUBE_API_HOST}:${CUBE_API_PORT}/ws/relay`;
}

const VALID_MSG_TYPES = new Set(["chat", "telemetry", "product-status", "auth_ok", "challenge", "ops-error", "pong"]);
const REP_C_RE = /^[1-3]{13}$/;
const MAX_MSG_SIZE = 64 * 1024;

interface RelayEnvelope {
  type: string;
  msgType: string;
  from?: string;
  payload: string;
  ts: number;
}

function validateInboundMessage(raw: string): RelayEnvelope | null {
  if (raw.length > MAX_MSG_SIZE) {
    console.warn(`[launcher] Message exceeds 64KB limit: ${raw.length} bytes`);
    return null;
  }

  let msg: Record<string, unknown>;
  try {
    msg = JSON.parse(raw);
  } catch {
    console.warn(`[launcher] Invalid JSON from WS: ${raw.slice(0, 100)}`);
    return null;
  }

  if (typeof msg !== "object" || msg === null) {
    console.warn(`[launcher] Non-object message from WS`);
    return null;
  }

  if (typeof msg.type !== "string") {
    console.warn(`[launcher] Type mismatch: expected string for 'type', got ${typeof msg.type}`);
    return null;
  }

  const msgType = (msg.msgType ?? msg.relay_msg_type) as string | undefined;
  if (typeof msgType !== "string" || !VALID_MSG_TYPES.has(msgType)) {
    console.warn(`[launcher] Unknown msgType: ${msgType}`);
    return null;
  }

  const fromField = typeof msg.from === "string" ? msg.from : undefined;
  if (fromField !== undefined && !REP_C_RE.test(fromField)) {
    console.warn(`[launcher] Invalid Rep C in 'from': ${fromField}`);
  }

  const payload = typeof msg.payload === "string" ? msg.payload : undefined;
  const ts = typeof msg.ts === "number" ? msg.ts : Date.now();

  return { type: msg.type as string, msgType, from: fromField, payload: payload ?? "", ts };
}

function useDaemonConnection(active: boolean) {
  const [state, setState] = useState<DaemonState>("DISCONNECTED");
  const [reconnectCountdown, setReconnectCountdown] = useState<number | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const abortRef = useRef<AbortController | null>(null);
  const reconnectTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const heartbeatRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const pongTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const countdownRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const backoffRef = useRef(1000);
  const startTimeRef = useRef<number>(0);
  const activeRef = useRef(false);
  const awaitingPongRef = useRef(false);
  const failedToastShownRef = useRef(false);
  const listenersRef = useRef<Map<string, Set<(env: RelayEnvelope) => void>>>(new Map());

  const cleanup = useCallback(() => {
    activeRef.current = false;
    if (wsRef.current) {
      try { wsRef.current.close(); } catch (e) { console.warn("[launcher] WS close error:", e); }
      wsRef.current = null;
    }
    if (abortRef.current) {
      abortRef.current.abort();
      abortRef.current = null;
    }
    if (reconnectTimerRef.current) {
      clearTimeout(reconnectTimerRef.current);
      reconnectTimerRef.current = null;
    }
    if (heartbeatRef.current) {
      clearInterval(heartbeatRef.current);
      heartbeatRef.current = null;
    }
    if (pongTimeoutRef.current) {
      clearTimeout(pongTimeoutRef.current);
      pongTimeoutRef.current = null;
    }
    if (countdownRef.current) {
      clearInterval(countdownRef.current);
      countdownRef.current = null;
    }
    setReconnectCountdown(null);
    awaitingPongRef.current = false;
    failedToastShownRef.current = false;
    backoffRef.current = 1000;
    setState("DISCONNECTED");
  }, []);

  const subscribe = useCallback((msgType: string, handler: (env: RelayEnvelope) => void) => {
    if (!listenersRef.current.has(msgType)) {
      listenersRef.current.set(msgType, new Set());
    }
    listenersRef.current.get(msgType)!.add(handler);
    return () => {
      listenersRef.current.get(msgType)?.delete(handler);
    };
  }, []);

  const wsSend = useCallback((data: object) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(data));
    }
  }, []);

  const connectWs = useCallback(() => {
    setState("CONNECTING");
    const url = getDaemonWsUrl();
    try {
      const ws = new WebSocket(url);
      wsRef.current = ws;

      ws.onopen = () => {
        console.log(`[launcher] WS connected: ${url}`);
        setState("CONNECTED");
        setReconnectCountdown(null);
        backoffRef.current = 1000;
        awaitingPongRef.current = false;

        heartbeatRef.current = setInterval(() => {
          if (ws.readyState === WebSocket.OPEN) {
            try {
              if (pongTimeoutRef.current) {
                clearTimeout(pongTimeoutRef.current);
                pongTimeoutRef.current = null;
              }
              ws.send(JSON.stringify({ type: "ping", ts: Date.now() }));
              awaitingPongRef.current = true;
              pongTimeoutRef.current = setTimeout(() => {
                if (awaitingPongRef.current && activeRef.current) {
                  console.warn(`[launcher] No pong received within 10s, reconnecting`);
                  awaitingPongRef.current = false;
                  if (heartbeatRef.current) {
                    clearInterval(heartbeatRef.current);
                    heartbeatRef.current = null;
                  }
                  try { ws.close(); } catch (e) { console.warn("[launcher] WS close on pong timeout:", e); }
                }
              }, 10000);
            } catch (e) {
              console.warn("[launcher] Heartbeat send error:", e);
            }
          }
        }, 30000);
      };

      ws.onmessage = (event) => {
        const raw = typeof event.data === "string" ? event.data : "";
        try {
          const peek = JSON.parse(raw);
          if (peek?.type === "pong" || peek?.msgType === "pong") {
            awaitingPongRef.current = false;
            if (pongTimeoutRef.current) {
              clearTimeout(pongTimeoutRef.current);
              pongTimeoutRef.current = null;
            }
            return;
          }
        } catch (e) {
          console.debug("[launcher] Non-JSON WS message, treating as relay envelope");
        }
        const env = validateInboundMessage(raw);
        if (!env) return;
        const handlers = listenersRef.current.get(env.msgType);
        if (handlers) {
          handlers.forEach((h) => h(env));
        }
      };

      ws.onerror = () => {
        console.warn(`[launcher] WS error: ${url}`);
      };

      ws.onclose = (event) => {
        console.log(`[launcher] WS closed: code=${event.code} reason=${event.reason}`);
        if (heartbeatRef.current) {
          clearInterval(heartbeatRef.current);
          heartbeatRef.current = null;
        }
        if (pongTimeoutRef.current) {
          clearTimeout(pongTimeoutRef.current);
          pongTimeoutRef.current = null;
        }
        awaitingPongRef.current = false;
        wsRef.current = null;
        if (activeRef.current) {
          scheduleReconnect();
        }
      };
    } catch {
      console.warn(`[launcher] WS connection refused: ${url}`);
      if (activeRef.current) {
        scheduleReconnect();
      }
    }
  }, []);

  const scheduleReconnect = useCallback(() => {
    if (!activeRef.current) return;
    const elapsed = Date.now() - startTimeRef.current;
    if (elapsed > 5 * 60 * 1000) {
      setState("FAILED");
      setReconnectCountdown(null);
      if (!failedToastShownRef.current) {
        failedToastShownRef.current = true;
        try {
          const event = new CustomEvent("launcher:connection-failed", {
            detail: { message: "PlenumNET daemon unreachable after 5 minutes. Check that Inter-Cube is running." },
          });
          window.dispatchEvent(event);
        } catch (e) {
          console.warn("[launcher] Failed to dispatch connection-failed event:", e);
        }
      }
      console.warn(`[launcher] Reconnection exhausted after 5 minutes`);
      return;
    }

    setState("RECONNECTING");
    const jitter = 1 + (Math.random() * 0.4 - 0.2);
    const delay = Math.min(backoffRef.current * jitter, 30000);
    backoffRef.current = Math.min(backoffRef.current * 2, 30000);

    const delaySec = Math.ceil(delay / 1000);
    setReconnectCountdown(delaySec);
    if (countdownRef.current) clearInterval(countdownRef.current);
    let remaining = delaySec;
    countdownRef.current = setInterval(() => {
      remaining--;
      if (remaining <= 0) {
        if (countdownRef.current) clearInterval(countdownRef.current);
        countdownRef.current = null;
        setReconnectCountdown(null);
      } else {
        setReconnectCountdown(remaining);
      }
    }, 1000);

    reconnectTimerRef.current = setTimeout(() => {
      setReconnectCountdown(null);
      doHealthCheck();
    }, delay);
  }, []);

  const doHealthCheck = useCallback(() => {
    if (!activeRef.current) return;
    setState("HEALTH_CHECK");
    const controller = new AbortController();
    abortRef.current = controller;

    const timeoutId = setTimeout(() => controller.abort(), 5000);

    fetch(`${DAEMON_HTTP}/health`, { signal: controller.signal })
      .then((res) => {
        clearTimeout(timeoutId);
        if (!activeRef.current) return;
        if (res.ok) {
          connectWs();
        } else {
          scheduleReconnect();
        }
      })
      .catch((err) => {
        clearTimeout(timeoutId);
        if (!activeRef.current) return;
        if (err.name === "TypeError") {
          console.warn(`[launcher] CORS error: /health`);
        } else if (err.name !== "AbortError") {
          console.warn(`[launcher] REST timeout: /health after 5000ms`);
        }
        if (err.name !== "AbortError") scheduleReconnect();
      });
  }, [connectWs, scheduleReconnect]);

  const retry = useCallback(() => {
    if (reconnectTimerRef.current) {
      clearTimeout(reconnectTimerRef.current);
      reconnectTimerRef.current = null;
    }
    if (countdownRef.current) {
      clearInterval(countdownRef.current);
      countdownRef.current = null;
    }
    setReconnectCountdown(null);
    if (abortRef.current) {
      abortRef.current.abort();
      abortRef.current = null;
    }
    startTimeRef.current = Date.now();
    backoffRef.current = 1000;
    doHealthCheck();
  }, [doHealthCheck]);

  useEffect(() => {
    if (active) {
      activeRef.current = true;
      startTimeRef.current = Date.now();
      doHealthCheck();
    } else {
      cleanup();
    }
    return cleanup;
  }, [active]);

  return { state, subscribe, wsSend, retry, reconnectCountdown };
}

function daemonFetch(endpoint: string, options?: RequestInit & { externalSignal?: AbortSignal }): Promise<Response> {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), 5000);
  if (options?.externalSignal) {
    options.externalSignal.addEventListener("abort", () => controller.abort(), { once: true });
  }
  const { externalSignal: _, ...fetchOptions } = options ?? {};
  return fetch(`${DAEMON_HTTP}${endpoint}`, {
    ...fetchOptions,
    signal: controller.signal,
  }).finally(() => clearTimeout(timeoutId));
}

interface Product {
  id: string;
  name: string;
  icon: string;
  description: string;
  version: string;
  status: "running" | "stopped" | "available" | "installing";
  type?: string;
  arch?: string;
  size?: string;
  updateAvailable?: boolean;
  latestVersion?: string;
  progress?: number;
  progressLabel?: string;
  error?: string;
}

interface ResourceEntry {
  label: string;
  value: number;
  detail: string;
  cores: string | null;
}

interface ModelEntry {
  name: string;
  engine: string;
  mem: string;
  status: string;
}

interface FtsNeighbor {
  address: string;
  status: string;
  lastSeen?: string;
}

interface ConStats {
  tunnelsUp: number;
  tunnelsTotal: number;
  pqKeysDeived: number;
  overlayStatus: string;
}

interface SystemPayload {
  resources?: ResourceEntry[];
  models?: ModelEntry[];
  integrity?: string;
  ftsNeighbors?: FtsNeighbor[];
  conStats?: ConStats;
}

interface NetworkNode {
  node: string;
  addr: string;
  role: string;
  latency: string;
}

interface NetworkInterface {
  name: string;
  ip: string;
  ipv6: string;
  speed: string;
  status: string;
}

interface ClusterHealthPayload {
  arrayName?: string;
  nodeCount?: number;
  latencyMs?: number;
  arch?: string;
  installPath?: string;
  repC?: string;
}

interface NetworkPayload {
  topologyStatus?: string;
  nodes?: NetworkNode[];
  interfaces?: NetworkInterface[];
  bandwidth?: { up: string; down: string };
  peers?: string;
  wsConnections?: string;
  clusterHealth?: ClusterHealthPayload;
}

interface TelemetryPayload {
  system?: Partial<SystemPayload>;
  network?: Partial<NetworkPayload>;
}

interface ProductStatusPayload {
  id?: string;
  status?: Product["status"];
  progress?: number;
  progressLabel?: string;
  error?: string;
}

const CUT = 16;
function chamfer(inset = 0) {
  const c = CUT - inset;
  return `polygon(${c}px 0%, calc(100% - ${c}px) 0%, 100% ${c}px, 100% calc(100% - ${c}px), calc(100% - ${c}px) 100%, ${c}px 100%, 0% calc(100% - ${c}px), 0% ${c}px)`;
}

function miniChamfer(c = 5) {
  return `polygon(${c}px 0%, calc(100% - ${c}px) 0%, 100% ${c}px, 100% calc(100% - ${c}px), calc(100% - ${c}px) 100%, ${c}px 100%, 0% calc(100% - ${c}px), 0% ${c}px)`;
}

function CopyDetailsButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);
  return (
    <button
      data-testid="button-copy-details"
      onClick={() => {
        navigator.clipboard.writeText(text).then(() => {
          setCopied(true);
          setTimeout(() => setCopied(false), 2000);
        });
      }}
      style={{
        background: "none",
        border: `1px solid var(--launcher-border)`,
        color: "var(--launcher-accent)",
        fontSize: 9,
        fontFamily: "var(--launcher-font-mono)",
        padding: "2px 6px",
        cursor: "pointer",
        marginLeft: 6,
      }}
    >
      {copied ? "Copied!" : "[Copy Details]"}
    </button>
  );
}

function StatusDot({ state, label, pulse }: { state: string; label: string; pulse?: boolean }) {
  const colorMap: Record<string, string> = {
    connected: "var(--launcher-accent)",
    disconnected: "var(--launcher-text-faint)",
    connecting: "var(--launcher-text-slate)",
    error: "var(--launcher-destructive)",
    reconnecting: "var(--launcher-warning)",
  };
  const c = colorMap[state] || "var(--launcher-text-faint)";
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 5 }}>
      <div
        role={state === "connected" || state === "reconnecting" ? "status" : undefined}
        style={{
          width: 5, height: 5, borderRadius: "50%", background: c,
          boxShadow: state === "connected" ? `0 0 5px ${c}` : "none",
          animation: pulse ? "launcherPulse 2s infinite" : "none",
        }}
      />
      <span style={{ fontSize: 9, fontFamily: "var(--launcher-font-mono)", fontWeight: 500, color: c }}>{label}</span>
    </div>
  );
}

function Tag({ children }: { children: React.ReactNode }) {
  return (
    <span style={{ fontSize: 8, fontFamily: "var(--launcher-font-mono)", color: "var(--launcher-text-faint)", padding: "1px 5px", background: "var(--launcher-iron)" }}>
      {children}
    </span>
  );
}

function Btn({ label, c, onClick, ld, testId }: { label: string; c: string; onClick?: () => void; ld?: boolean; testId?: string }) {
  const [h, setH] = useState(false);
  return (
    <button
      data-testid={testId}
      onClick={onClick}
      onMouseEnter={() => setH(true)}
      onMouseLeave={() => setH(false)}
      disabled={ld}
      style={{
        padding: "2px 8px", fontSize: 9, fontWeight: 600, fontFamily: "var(--launcher-font-body)", cursor: ld ? "wait" : "pointer",
        border: `1px solid ${h ? c : c + "40"}`, background: h ? c + "12" : "transparent",
        color: ld ? "var(--launcher-text-faint)" : c, transition: "all 0.12s", clipPath: miniChamfer(3),
      }}
    >
      {ld ? "···" : label}
    </button>
  );
}

function ProductCard({ product, onAction, actionLoading }: { product: Product; onAction: (id: string, action: string) => void; actionLoading: string | null }) {
  const [hov, setHov] = useState(false);
  const isR = product.status === "running";
  const isS = product.status === "stopped";
  const isA = product.status === "available";
  const isI = product.status === "installing";
  const sL = isR ? "Running" : isS ? "Stopped" : isI ? "Installing..." : "Available";
  const sS = isR ? "connected" : isI ? "connecting" : "disconnected";

  return (
    <div
      data-testid={`card-product-${product.id}`}
      onMouseEnter={() => setHov(true)}
      onMouseLeave={() => setHov(false)}
      style={{
        background: hov ? "var(--launcher-bg-surface)" : "transparent",
        border: `1px solid ${hov ? "var(--launcher-border)" : "transparent"}`,
        padding: "10px 12px", transition: "all 0.12s", clipPath: miniChamfer(6),
      }}
    >
      {product.error && (
        <div role="alert" style={{ padding: "4px 8px", marginBottom: 6, background: "var(--launcher-destructive)", color: "var(--launcher-text-heading)", fontSize: 10, fontFamily: "var(--launcher-font-body)" }}>
          {product.name}: {product.error}
        </div>
      )}
      <div style={{ display: "flex", alignItems: "center", gap: 10, marginBottom: 5 }}>
        <span style={{ fontSize: 15, width: 22, textAlign: "center", flexShrink: 0 }}>{product.icon}</span>
        <div style={{ flex: 1 }}>
          <div style={{ display: "flex", alignItems: "baseline", gap: 7 }}>
            <span style={{ fontSize: 12, fontWeight: 600, color: "var(--launcher-text-heading)", fontFamily: "var(--launcher-font-body)" }}>{product.name}</span>
            <span style={{ fontSize: 9, fontFamily: "var(--launcher-font-mono)", color: "var(--launcher-text-faint)" }}>{product.version}</span>
          </div>
          <div style={{ fontSize: 10, color: "var(--launcher-text-label)", fontFamily: "var(--launcher-font-body)", marginTop: 1, lineHeight: 1.3 }}>{product.description}</div>
        </div>
        <StatusDot state={sS} label={sL} pulse={isI} />
      </div>
      <div style={{ display: "flex", gap: 5, marginLeft: 32, marginBottom: isI ? 6 : 2, flexWrap: "wrap" }}>
        {product.arch && <Tag>{product.arch}</Tag>}
        {product.type && <Tag>{product.type}</Tag>}
        {product.size && <Tag>{product.size}</Tag>}
      </div>
      {isI && (
        <div style={{ marginLeft: 32, marginTop: 4 }}>
          <div style={{ height: 2, background: "var(--launcher-iron)", overflow: "hidden" }}>
            <div style={{ width: `${product.progress || 0}%`, height: "100%", background: "linear-gradient(90deg, var(--launcher-accent), var(--launcher-accent-hover))", transition: "width 0.3s" }} />
          </div>
          <div style={{ fontSize: 9, fontFamily: "var(--launcher-font-mono)", color: "var(--launcher-text-faint)", marginTop: 2 }}>{product.progressLabel}</div>
        </div>
      )}
      {!isI && (hov || product.updateAvailable) && (
        <div style={{ display: "flex", gap: 5, marginLeft: 32, marginTop: 4 }}>
          {isA && <Btn label="Install" c="var(--launcher-accent)" onClick={() => onAction(product.id, "install")} ld={actionLoading === product.id} testId={`button-install-${product.id}`} />}
          {product.updateAvailable && <Btn label={`Update → ${product.latestVersion}`} c="var(--launcher-accent-hover)" onClick={() => onAction(product.id, "update")} ld={actionLoading === product.id} testId={`button-update-${product.id}`} />}
          {isR && (
            <>
              <Btn label="Stop" c="var(--launcher-text-slate)" onClick={() => onAction(product.id, "stop")} testId={`button-stop-${product.id}`} />
              <Btn label="Restart" c="var(--launcher-text-slate)" onClick={() => onAction(product.id, "restart")} testId={`button-restart-${product.id}`} />
            </>
          )}
          {isS && <Btn label="Start" c="var(--launcher-accent)" onClick={() => onAction(product.id, "start")} testId={`button-start-${product.id}`} />}
          {(isR || isS) && <Btn label="Logs" c="var(--launcher-iron)" onClick={() => onAction(product.id, "logs")} testId={`button-logs-${product.id}`} />}
        </div>
      )}
    </div>
  );
}

function YodaChat({ connected, wsSend, subscribe }: { connected: boolean; wsSend: (data: object) => void; subscribe: (msgType: string, handler: (env: RelayEnvelope) => void) => () => void }) {
  const [msg, setMsg] = useState("");
  const [hist, setHist] = useState<Array<{ from: string; text: string }>>([]);
  const [sending, setSending] = useState(false);
  const endRef = useRef<HTMLDivElement>(null);

  useEffect(() => { endRef.current?.scrollIntoView({ behavior: "smooth" }); }, [hist]);

  useEffect(() => {
    const unsub = subscribe("chat", (env) => {
      let text: string;
      try {
        const parsed = JSON.parse(env.payload);
        text = parsed?.message || parsed?.text || env.payload;
      } catch {
        text = env.payload;
      }
      setHist((h) => [...h, { from: "yoda", text }]);
      setSending(false);
    });
    return unsub;
  }, [subscribe]);

  const sendMsg = useCallback(() => {
    if (!msg.trim() || !connected) return;
    const text = msg.trim();
    setHist((h) => [...h, { from: "user", text }]);
    wsSend({
      type: "relay",
      msgType: "chat",
      payload: JSON.stringify({ context: "PlenumNET-YODA-CHAT-v1", message: text }),
      ts: Date.now(),
    });
    setMsg("");
    setSending(true);
  }, [msg, connected, wsSend]);

  return (
    <div style={{ display: "flex", flexDirection: "column", flex: 1, minHeight: 0 }}>
      <div style={{ flex: 1, overflowY: "auto", padding: "4px 0", display: "flex", flexDirection: "column", gap: 6 }}>
        {!connected && (
          <div style={{ fontSize: 11, color: "var(--launcher-text-label)", fontFamily: "var(--launcher-font-body)", textAlign: "center", padding: 20 }}>
            AI assistant offline — check daemon connection
          </div>
        )}
        {hist.map((m, i) => (
          <div key={i} style={{ fontSize: 11, lineHeight: 1.6, fontFamily: m.from === "yoda" ? "var(--launcher-font-mono)" : "var(--launcher-font-body)", color: m.from === "yoda" ? "var(--launcher-text-body)" : "var(--launcher-text-nav)" }}>
            {m.from === "yoda" && <span style={{ color: "var(--launcher-accent)", fontWeight: 700, marginRight: 5, fontSize: 10 }}>[YODA]</span>}
            {m.from === "user" && <span style={{ color: "var(--launcher-text-faint)", marginRight: 3 }}>›</span>}
            {m.text}
          </div>
        ))}
        {sending && (
          <div style={{ fontSize: 11, fontFamily: "var(--launcher-font-mono)", color: "var(--launcher-text-faint)" }}>
            <span style={{ color: "var(--launcher-accent)", fontWeight: 700, fontSize: 10 }}>[YODA]</span>{" "}
            <span style={{ animation: "launcherPulse 1s infinite" }}>thinking...</span>
          </div>
        )}
        <div ref={endRef} />
      </div>
      <div style={{ display: "flex", gap: 6, flexShrink: 0, marginTop: 6 }}>
        <input
          data-testid="input-yoda-message"
          value={msg}
          onChange={(e) => setMsg(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && sendMsg()}
          placeholder={connected ? "Message YODA..." : "Offline"}
          disabled={!connected}
          aria-label="Message YODA assistant"
          style={{
            flex: 1, padding: "6px 10px", border: "1px solid var(--launcher-border)", background: "var(--launcher-bg-surface)",
            color: "var(--launcher-text-nav)", fontSize: 11, fontFamily: "var(--launcher-font-mono)", outline: "none", clipPath: miniChamfer(4),
          }}
        />
        <button
          data-testid="button-yoda-send"
          onClick={sendMsg}
          disabled={!connected || !msg.trim()}
          style={{
            padding: "6px 12px", border: "1px solid var(--launcher-accent-dim)", background: "transparent",
            color: connected ? "var(--launcher-accent)" : "var(--launcher-text-faint)", fontSize: 10, fontWeight: 700,
            fontFamily: "var(--launcher-font-body)", cursor: connected ? "pointer" : "not-allowed", clipPath: miniChamfer(4),
          }}
        >
          SEND
        </button>
      </div>
    </div>
  );
}

function GearIcon({ size = 13, color }: { size?: number; color: string }) {
  return (
    <svg width={size} height={size} viewBox="0 0 16 16" fill="none" stroke={color} strokeWidth="1.3" strokeLinecap="round">
      <circle cx="8" cy="8" r="2.2" />
      <path d="M8 1.5v1.5M8 13v1.5M1.5 8H3M13 8h1.5M3.4 3.4l1.06 1.06M11.54 11.54l1.06 1.06M3.4 12.6l1.06-1.06M11.54 4.46l1.06-1.06" />
    </svg>
  );
}

function LeatherGrain({ seed = 1 }: { seed?: number }) {
  const ref = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const c = ref.current;
    if (!c) return;
    const w = (c.width = 128);
    const h = (c.height = 128);
    const ctx = c.getContext("2d");
    if (!ctx) return;
    const img = ctx.createImageData(w, h);
    let s = seed * 9301 + 49297;
    for (let i = 0; i < img.data.length; i += 4) {
      s = (s * 9301 + 49297) % 233280;
      const v = (s / 233280) * 255;
      img.data[i] = v;
      img.data[i + 1] = v;
      img.data[i + 2] = v;
      img.data[i + 3] = 32;
    }
    ctx.putImageData(img, 0, 0);
  }, [seed]);
  return <canvas ref={ref} style={{ position: "absolute", inset: 0, width: "100%", height: "100%", pointerEvents: "none", zIndex: 5, mixBlendMode: "overlay" }} />;
}

function HeaderCanvas() {
  const ref = useRef<HTMLCanvasElement>(null);
  const pts = useRef<Array<{ x: number; y: number; vx: number; vy: number; r: number; o: number }>>([]);
  useEffect(() => {
    const c = ref.current;
    if (!c) return;
    const ctx = c.getContext("2d");
    if (!ctx) return;
    const w = (c.width = 460);
    const h = (c.height = 80);
    if (!pts.current.length) {
      pts.current = Array.from({ length: 14 }, () => ({
        x: Math.random() * w, y: Math.random() * h,
        vx: (Math.random() - 0.5) * 0.2, vy: (Math.random() - 0.5) * 0.12,
        r: Math.random() * 1 + 0.3, o: Math.random() * 0.25 + 0.05,
      }));
    }
    let raf: number;
    const draw = () => {
      ctx.clearRect(0, 0, w, h);
      const p = pts.current;
      for (let i = 0; i < p.length; i++) {
        for (let j = i + 1; j < p.length; j++) {
          const dx = p[i].x - p[j].x;
          const dy = p[i].y - p[j].y;
          const d = Math.sqrt(dx * dx + dy * dy);
          if (d < 70) {
            ctx.beginPath();
            ctx.moveTo(p[i].x, p[i].y);
            ctx.lineTo(p[j].x, p[j].y);
            ctx.strokeStyle = `rgba(74,158,245,${(1 - d / 70) * 0.08})`;
            ctx.lineWidth = 0.5;
            ctx.stroke();
          }
        }
      }
      p.forEach((pt) => {
        ctx.beginPath();
        ctx.arc(pt.x, pt.y, pt.r, 0, Math.PI * 2);
        ctx.fillStyle = `rgba(74,158,245,${pt.o})`;
        ctx.fill();
        pt.x += pt.vx;
        pt.y += pt.vy;
        if (pt.x < 0 || pt.x > w) pt.vx *= -1;
        if (pt.y < 0 || pt.y > h) pt.vy *= -1;
      });
      raf = requestAnimationFrame(draw);
    };
    draw();
    return () => cancelAnimationFrame(raf);
  }, []);
  return <canvas ref={ref} style={{ position: "absolute", inset: 0, width: "100%", height: "100%", pointerEvents: "none" }} />;
}

const TABS = [
  { id: "yoda", label: "Assistant" },
  { id: "apps", label: "Apps" },
  { id: "local", label: "System" },
  { id: "net", label: "Network" },
  { id: "apis", label: "APIs" },
] as const;

type TabId = (typeof TABS)[number]["id"];

function LauncherPanelInner() {
  const { panelState, setPanelState, widgetMode } = useLauncher();
  const [tab, setTab] = useState<TabId>("yoda");
  const [settingsOpen, setSettingsOpen] = useState(false);
  const apiKeyRef = useRef("");
  const apiKeyInputRef = useRef<HTMLInputElement>(null);
  const [apiKeyStored, setApiKeyStored] = useState(false);
  const [apiKeyValid, setApiKeyValid] = useState<boolean | null>(null);
  const [apiKeyMsg, setApiKeyMsg] = useState<React.ReactNode>("");
  const [actionLoading, setActionLoading] = useState<string | null>(null);
  const [products, setProducts] = useState<Product[]>([]);
  const [filter, setFilter] = useState("all");
  const [systemData, setSystemData] = useState<SystemPayload | null>(null);
  const [networkData, setNetworkData] = useState<NetworkPayload | null>(null);
  const [clusterHealth, setClusterHealth] = useState<ClusterHealthPayload | null>(null);
  const [failedToast, setFailedToast] = useState(false);
  const [restErrors, setRestErrors] = useState<Record<string, string>>({});
  const restAbortRef = useRef<AbortController | null>(null);
  const panelRef = useRef<HTMLDivElement>(null);
  const touchStartY = useRef<number | null>(null);
  const tabRefs = useRef<Map<string, HTMLButtonElement | null>>(new Map());

  const isAnimating = panelState === "OPENING" || panelState === "CLOSING";
  const isVisible = panelState !== "CLOSED";

  const shouldConnect = panelState === "OPEN" || panelState === "OPENING" || panelState === "MINIMIZED";
  const daemon = useDaemonConnection(shouldConnect);
  const isOn = daemon.state === "CONNECTED";

  useEffect(() => {
    const stored = sessionStorage.getItem("plenumnet-api-key");
    if (stored) {
      apiKeyRef.current = stored;
      setApiKeyStored(true);
      validateKeyWithDaemon(stored);
    }
  }, []);

  useEffect(() => {
    if (!isOn) {
      if (restAbortRef.current) {
        restAbortRef.current.abort();
        restAbortRef.current = null;
      }
      return;
    }
    const ac = new AbortController();
    restAbortRef.current = ac;
    const sig = ac.signal;
    setRestErrors({});

    const trackError = (endpoint: string, err: Error) => {
      if (err.name !== "AbortError") {
        const detail = err.name === "TypeError"
          ? `CORS blocked: ${DAEMON_HTTP}${endpoint} — Origin ${window.location.origin} rejected`
          : `${endpoint}: ${err.message}`;
        console.warn(`[launcher] REST ${endpoint}: ${err.message}`);
        setRestErrors((prev) => ({ ...prev, [endpoint]: detail }));
      }
    };

    daemonFetch("/api/salvi/inter-cube/node/info", { externalSignal: sig })
      .then((res) => {
        if (res.ok) return res.json();
        throw new Error(`HTTP ${res.status}`);
      })
      .then((data: { products?: Product[] }) => {
        if (data?.products && Array.isArray(data.products)) {
          setProducts(data.products);
        }
      })
      .catch((err) => trackError("/node/info", err));

    daemonFetch("/api/salvi/inter-cube/relay/cluster-health", { externalSignal: sig })
      .then((res) => {
        if (res.ok) return res.json();
        throw new Error(`HTTP ${res.status}`);
      })
      .then((data: SystemPayload & ClusterHealthPayload) => {
        setSystemData(data);
        setClusterHealth({
          arrayName: data.arrayName,
          nodeCount: data.nodeCount,
          latencyMs: data.latencyMs,
          arch: data.arch,
          installPath: data.installPath,
          repC: data.repC,
        });
        setNetworkData((prev) => prev ? { ...prev, clusterHealth: data } : { clusterHealth: data });
      })
      .catch((err) => {
        trackError("/cluster-health", err);
        if (err.name !== "AbortError") {
          setSystemData(null);
          setClusterHealth(null);
        }
      });

    daemonFetch("/api/salvi/inter-cube/topology", { externalSignal: sig })
      .then((res) => {
        if (res.ok) return res.json();
        throw new Error(`HTTP ${res.status}`);
      })
      .then((data: NetworkPayload) => setNetworkData(data))
      .catch((err) => {
        trackError("/topology", err);
        if (err.name !== "AbortError") setNetworkData(null);
      });

    daemonFetch("/api/salvi/inter-cube/fts/status", { externalSignal: sig })
      .then((res) => {
        if (res.ok) return res.json();
        throw new Error(`HTTP ${res.status}`);
      })
      .then((data: FtsNeighbor[]) => setSystemData((prev) => prev ? { ...prev, ftsNeighbors: data } : { ftsNeighbors: data }))
      .catch((err) => trackError("/fts/status", err));

    daemonFetch("/api/salvi/inter-cube/con/stats", { externalSignal: sig })
      .then((res) => {
        if (res.ok) return res.json();
        throw new Error(`HTTP ${res.status}`);
      })
      .then((data: ConStats) => setSystemData((prev) => prev ? { ...prev, conStats: data } : { conStats: data }))
      .catch((err) => trackError("/con/stats", err));

    return () => {
      ac.abort();
      restAbortRef.current = null;
    };
  }, [isOn]);

  useEffect(() => {
    if (!shouldConnect) return;
    const unsub = daemon.subscribe("telemetry", (env) => {
      try {
        const data: TelemetryPayload = JSON.parse(env.payload);
        if (data.system) setSystemData((prev) => prev ? { ...prev, ...data.system } : data.system as SystemPayload);
        if (data.network) setNetworkData((prev) => prev ? { ...prev, ...data.network } : data.network as NetworkPayload);
      } catch {
        console.warn(`[launcher] Invalid telemetry payload`);
      }
    });
    return unsub;
  }, [shouldConnect, daemon.subscribe]);

  useEffect(() => {
    const handler = (e: Event) => {
      const detail = (e as CustomEvent).detail;
      setFailedToast(true);
      console.warn(`[launcher] FAILED toast: ${detail?.message}`);
      setTimeout(() => setFailedToast(false), 8000);
    };
    window.addEventListener("launcher:connection-failed", handler);
    return () => window.removeEventListener("launcher:connection-failed", handler);
  }, []);

  useEffect(() => {
    if (!shouldConnect) return;
    const unsub = daemon.subscribe("product-status", (env) => {
      try {
        const data: ProductStatusPayload = JSON.parse(env.payload);
        if (data.id) {
          setProducts((ps) =>
            ps.map((p) =>
              p.id === data.id
                ? { ...p, status: data.status ?? p.status, progress: data.progress, progressLabel: data.progressLabel, error: data.error }
                : p
            )
          );
        }
      } catch {
        console.warn(`[launcher] Invalid product-status payload`);
      }
    });
    return unsub;
  }, [shouldConnect, daemon.subscribe]);

  const validateKeyWithDaemon = useCallback((key: string) => {
    setApiKeyValid(null);
    setApiKeyMsg("");
    apiKeyRef.current = key;
    daemonFetch("/api/salvi/inter-cube/validate-key", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ key }),
    })
      .then((res) => {
        if (res.status === 200) {
          setApiKeyValid(true);
          setApiKeyMsg("");
          setApiKeyStored(true);
          if (apiKeyInputRef.current) apiKeyInputRef.current.value = "";
          sessionStorage.setItem("plenumnet-api-key", key);
        } else if (res.status === 401) {
          setApiKeyValid(false);
          setApiKeyMsg("Invalid API key. Please check your key and try again.");
          apiKeyRef.current = "";
          setApiKeyStored(false);
          sessionStorage.removeItem("plenumnet-api-key");
          console.warn(`[launcher] Key validation failed: 401`);
        } else if (res.status === 404) {
          setApiKeyValid(true);
          setApiKeyMsg("Key validation is not available on this daemon version.");
          setApiKeyStored(true);
          if (apiKeyInputRef.current) apiKeyInputRef.current.value = "";
          sessionStorage.setItem("plenumnet-api-key", key);
          console.warn(`[launcher] Key validation endpoint not found: 404`);
        } else {
          setApiKeyValid(false);
          const errDetail = `HTTP ${res.status} from /api/salvi/inter-cube/validate-key`;
          setApiKeyMsg(<>The daemon encountered an error. Try restarting it. <CopyDetailsButton text={errDetail} /></>);
          apiKeyRef.current = "";
          setApiKeyStored(false);
          sessionStorage.removeItem("plenumnet-api-key");
          console.warn(`[launcher] 500: /api/salvi/inter-cube/validate-key`);
        }
      })
      .catch((err) => {
        if (err.name === "TypeError") {
          const errDetail = `CORS error: ${DAEMON_HTTP}/api/salvi/inter-cube/validate-key — Origin ${window.location.origin} blocked`;
          setApiKeyMsg(<>Browser security prevented the connection. <CopyDetailsButton text={errDetail} /></>);
          console.warn(`[launcher] CORS error: /api/salvi/inter-cube/validate-key`);
        } else {
          setApiKeyMsg("Daemon offline — cannot validate your key right now.");
          console.warn(`[launcher] Key validation error: ${err.message}`);
        }
        setApiKeyValid(false);
      });
  }, []);

  const handleAction = useCallback(
    (id: string, action: string) => {
      if (action === "install") {
        if (!apiKeyValid || !apiKeyRef.current) {
          setSettingsOpen(true);
          return;
        }
        setActionLoading(id);
        setProducts((ps) => ps.map((p) => (p.id === id ? { ...p, status: "installing" as const, progress: 0, progressLabel: "Requesting install..." } : p)));
        daemonFetch(`/api/salvi/inter-cube/node/install`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ productId: id }),
        })
          .then((res) => {
            if (!res.ok) {
              setProducts((ps) => ps.map((p) => (p.id === id ? { ...p, status: "available" as const, progress: undefined, progressLabel: undefined, error: `Installation failed (HTTP ${res.status})` } : p)));
              setActionLoading(null);
            }
          })
          .catch((err) => {
            console.warn(`[launcher] Install failed: ${id} — ${err.message}`);
            setProducts((ps) => ps.map((p) => (p.id === id ? { ...p, status: "available" as const, progress: undefined, progressLabel: undefined, error: err.message } : p)));
            setActionLoading(null);
          });
      } else if (action === "start" || action === "stop" || action === "restart") {
        setActionLoading(id);
        daemonFetch(`/api/salvi/inter-cube/node/${action}`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ productId: id }),
        })
          .then((res) => {
            if (res.ok) {
              return res.json().then((data: Record<string, unknown>) => {
                const newStatus: Product["status"] = typeof data.status === "string" ? data.status as Product["status"] : (action === "stop" ? "stopped" : "running");
                setProducts((ps) => ps.map((p) => (p.id === id ? { ...p, status: newStatus } : p)));
              });
            }
            setProducts((ps) => ps.map((p) => (p.id === id ? { ...p, error: `Action '${action}' failed (HTTP ${res.status})` } : p)));
          })
          .catch((err) => {
            console.warn(`[launcher] ${action} failed: ${id} — ${err.message}`);
            setProducts((ps) => ps.map((p) => (p.id === id ? { ...p, error: err.message } : p)));
          })
          .finally(() => setActionLoading(null));
      }
    },
    [apiKeyValid]
  );

  const installed = products.filter((p) => ["running", "stopped", "installing"].includes(p.status));
  const available = products.filter((p) => p.status === "available");
  const filtered = filter === "all" ? products : filter === "installed" ? installed : available;

  const handleAnimationEnd = useCallback(() => {
    if (panelState === "OPENING") setPanelState("OPEN");
    if (panelState === "CLOSING") setPanelState("CLOSED");
  }, [panelState, setPanelState]);

  useEffect(() => {
    if (!isVisible) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setPanelState("CLOSING");
      } else if (e.key === "m" || e.key === "M") {
        const tag = (e.target as HTMLElement)?.tagName;
        if (tag !== "INPUT" && tag !== "TEXTAREA") {
          setPanelState("MINIMIZED");
        }
      } else if (e.key === "s" || e.key === "S") {
        const tag = (e.target as HTMLElement)?.tagName;
        if (tag !== "INPUT" && tag !== "TEXTAREA") {
          setSettingsOpen((prev) => !prev);
        }
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [isVisible, setPanelState]);

  const handleTouchStart = useCallback((e: React.TouchEvent) => {
    touchStartY.current = e.touches[0].clientY;
  }, []);

  const handleTouchEnd = useCallback((e: React.TouchEvent) => {
    if (touchStartY.current !== null) {
      const diff = e.changedTouches[0].clientY - touchStartY.current;
      if (diff > 50) {
        setPanelState("CLOSING");
      }
      touchStartY.current = null;
    }
  }, [setPanelState]);

  const tabIndex = TABS.findIndex((t) => t.id === tab);

  const handleTabKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "ArrowRight") {
        e.preventDefault();
        const next = (tabIndex + 1) % TABS.length;
        setTab(TABS[next].id);
        tabRefs.current.get(TABS[next].id)?.focus();
      } else if (e.key === "ArrowLeft") {
        e.preventDefault();
        const prev = (tabIndex - 1 + TABS.length) % TABS.length;
        setTab(TABS[prev].id);
        tabRefs.current.get(TABS[prev].id)?.focus();
      }
    },
    [tabIndex]
  );

  useEffect(() => {
    const prefersReduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    const noAnims = document.documentElement.classList.contains("no-animations");
    if ((prefersReduced || noAnims) && panelState === "OPENING") {
      setPanelState("OPEN");
    }
    if ((prefersReduced || noAnims) && panelState === "CLOSING") {
      setPanelState("CLOSED");
    }
  }, [panelState, setPanelState]);

  if (!isVisible) return null;

  const animationName = panelState === "OPENING" ? "launcherSlideFromTray" : panelState === "CLOSING" ? "launcherSlideToTray" : "none";
  const animationDuration = panelState === "OPENING" ? "1.2s" : "0.3s";

  const connectionStatusLabel =
    daemon.state === "CONNECTED"
      ? "Connected"
      : daemon.state === "HEALTH_CHECK"
        ? "Connecting to daemon..."
        : daemon.state === "CONNECTING"
          ? "Establishing secure channel..."
          : daemon.state === "RECONNECTING"
            ? daemon.reconnectCountdown !== null
              ? `Reconnecting in ${daemon.reconnectCountdown}s...`
              : "Reconnecting..."
            : daemon.state === "FAILED"
              ? "Connection failed"
              : "Daemon offline";

  if (panelState === "MINIMIZED") {
    return (
      <>
        <LauncherStyles />
        <div
          data-testid="launcher-minimized-bar"
          data-launcher-theme="dark"
          onClick={() => setPanelState("OPEN")}
          style={{
            position: "fixed", bottom: 0, right: 16, zIndex: 9999,
            width: "min(450px, calc(100vw - 32px))",
            background: "var(--launcher-bg-primary)",
            border: "1px solid var(--launcher-border)",
            borderBottom: "none",
            padding: "6px 16px",
            display: "flex", justifyContent: "space-between", alignItems: "center",
            cursor: "pointer",
            fontSize: 10, fontFamily: "var(--launcher-font-mono)", color: "var(--launcher-text-faint)",
          }}
        >
          <span>PlenumNET Launcher</span>
          <span>{TABS.find((t) => t.id === tab)?.label}</span>
          <div role="status" aria-live="polite">
            <StatusDot state={daemon.state === "CONNECTED" ? "connected" : daemon.state === "RECONNECTING" ? "reconnecting" : "disconnected"} label={connectionStatusLabel} />
          </div>
        </div>
      </>
    );
  }

  return (
    <>
      <LauncherStyles />
      <div
        ref={panelRef}
        data-testid="launcher-panel"
        data-launcher-theme="dark"
        role="dialog"
        aria-label="PlenumNET Launcher"
        style={widgetMode ? {
          position: "absolute",
          inset: 0,
          zIndex: 9999,
          display: "flex",
          flexDirection: "column",
          pointerEvents: "auto",
        } : {
          position: "fixed",
          bottom: 0,
          right: 16,
          zIndex: 9999,
          width: "min(450px, calc(100vw - 32px))",
          maxHeight: "70vh",
          display: "flex",
          flexDirection: "column",
          animation: animationName !== "none" ? `${animationName} ${animationDuration} cubic-bezier(0.25, 0.1, 0.25, 1) both` : "none",
          pointerEvents: isAnimating ? "none" : "auto",
          filter: "drop-shadow(0 6px 28px var(--launcher-shadow-overlay)) drop-shadow(0 2px 4px var(--launcher-shadow-subtle))",
        }}
        onAnimationEnd={handleAnimationEnd}
      >
        <div style={{ clipPath: chamfer(0), background: `linear-gradient(160deg, var(--launcher-bg-outer-frame) 0%, var(--launcher-bg-inner-frame) 50%, var(--launcher-bg-primary) 100%)`, padding: 2, position: "relative", ...(widgetMode ? { flex: 1, display: "flex", flexDirection: "column" as const } : {}) }}>
          <div style={{ position: "absolute", inset: 0, pointerEvents: "none", zIndex: 10, clipPath: chamfer(0), background: `linear-gradient(180deg, var(--launcher-highlight) 0%, var(--launcher-highlight-subtle) 15%, transparent 40%)` }} />
          <div style={{ clipPath: chamfer(2), background: `linear-gradient(145deg, var(--launcher-bg-primary) 0%, var(--launcher-bg-inner-frame) 30%, var(--launcher-bg-surface) 100%)`, padding: 8, position: "relative", overflow: "hidden", ...(widgetMode ? { flex: 1, display: "flex", flexDirection: "column" as const } : {}) }}>
            <LeatherGrain seed={3} />
            <div style={{ clipPath: chamfer(5), background: `linear-gradient(145deg, var(--launcher-bg-chamfer) 0%, var(--launcher-bg-panel) 50%, var(--launcher-bg-surface) 100%)`, padding: 2, position: "relative", overflow: "hidden", zIndex: 6, ...(widgetMode ? { flex: 1, display: "flex", flexDirection: "column" as const } : {}) }}>
              <LeatherGrain seed={7} />
              <div style={{ clipPath: chamfer(6), overflow: "hidden", display: "flex", flexDirection: "column", background: "var(--launcher-bg-primary)", position: "relative", zIndex: 6, ...(widgetMode ? { flex: 1 } : {}) }}>

                {/* Header */}
                <div
                  style={{ position: "relative", background: `linear-gradient(180deg, var(--launcher-header-bg) 0%, var(--launcher-header-mid) 60%, var(--launcher-header-edge) 100%)`, padding: "10px 22px 8px", flexShrink: 0 }}
                  onTouchStart={handleTouchStart}
                  onTouchEnd={handleTouchEnd}
                >
                  <HeaderCanvas />
                  <div style={{ position: "absolute", inset: 0, pointerEvents: "none", zIndex: 5, background: `linear-gradient(180deg, var(--launcher-highlight-edge) 0%, var(--launcher-highlight-subtle) 25%, transparent 55%)` }} />
                  <div style={{ position: "absolute", top: 6, right: 8, zIndex: 6, display: "flex", gap: 4 }}>
                    <button
                      data-testid="button-launcher-minimize"
                      onClick={() => setPanelState("MINIMIZED")}
                      aria-label="Minimize panel (M)"
                      title="Minimize (M)"
                      style={{ background: "none", border: "none", cursor: "pointer", padding: 4, color: "var(--launcher-header-sub)", fontSize: 14, lineHeight: 1 }}
                    >
                      ─
                    </button>
                    <button
                      data-testid="button-launcher-settings"
                      onClick={() => setSettingsOpen(!settingsOpen)}
                      aria-label="Toggle settings (S)"
                      title="Settings (S)"
                      style={{ background: "none", border: "none", cursor: "pointer", padding: 4, opacity: settingsOpen ? 1 : 0.5, transition: "opacity 0.15s" }}
                    >
                      <GearIcon size={14} color={settingsOpen ? "var(--launcher-accent)" : "var(--launcher-header-sub)"} />
                    </button>
                    <button
                      data-testid="button-launcher-close"
                      onClick={() => setPanelState("CLOSING")}
                      aria-label="Close (Esc)"
                      title="Close (Esc)"
                      style={{ background: "none", border: "none", cursor: "pointer", padding: 4, color: "var(--launcher-header-sub)", fontSize: 14, lineHeight: 1 }}
                    >
                      ✕
                    </button>
                  </div>
                  <div style={{ position: "relative", zIndex: 1, display: "flex", flexDirection: "column", alignItems: "center" }}>
                    <img src="data:image/png;base64,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" alt="Capomastro" style={{ height: 28, marginBottom: 2 }} data-testid="img-capomastro-logo" />
                    <div style={{ fontSize: 9, color: "var(--launcher-header-sub)", fontFamily: "var(--launcher-font-mono)", letterSpacing: "0.12em", marginTop: 5, textTransform: "uppercase" }}>
                      Applied Physics Division
                    </div>
                  </div>
                  {isOn && (
                    <div style={{ marginTop: 6, padding: "4px 10px", background: "var(--launcher-header-info-bg)", border: "1px solid var(--launcher-header-info-border)", display: "flex", justifyContent: "space-between", fontSize: 9, fontFamily: "var(--launcher-font-mono)", color: "var(--launcher-header-sub)", clipPath: miniChamfer(3) }}>
                      <span>{clusterHealth?.arrayName ?? "—"}</span>
                      <span style={{ color: "var(--launcher-accent)", fontWeight: 600 }}>◈ {clusterHealth?.nodeCount != null ? `${clusterHealth.nodeCount}-node relay` : "relay"}</span>
                      <span>{getDaemonWsUrl()}</span>
                      <span>{clusterHealth?.latencyMs != null ? `${clusterHealth.latencyMs}ms` : "—"}</span>
                    </div>
                  )}
                </div>

                {/* Ledge */}
                <div style={{ position: "relative", height: 8, background: `linear-gradient(180deg, var(--launcher-ledge-top) 0%, var(--launcher-bg-primary) 100%)`, flexShrink: 0 }}>
                  <div style={{ position: "absolute", top: 0, left: 0, right: 0, height: 6, background: "linear-gradient(180deg, rgba(0,0,0,0.3) 0%, rgba(0,0,0,0.06) 70%, transparent 100%)" }} />
                </div>

                {/* Dark Body */}
                <div style={{ flex: 1, background: "var(--launcher-bg-primary)", display: "flex", flexDirection: "column", overflow: "hidden", position: "relative", minHeight: 400 }}>
                  {/* FAILED toast notification */}
                  {failedToast && (
                    <div
                      role="alert"
                      data-testid="toast-connection-failed"
                      style={{
                        position: "absolute", top: 8, left: 16, right: 16, zIndex: 100,
                        padding: "10px 14px", background: "var(--launcher-destructive)", color: "var(--launcher-text-heading)",
                        fontSize: 11, fontFamily: "var(--launcher-font-body)", display: "flex", alignItems: "center", gap: 8,
                        clipPath: miniChamfer(6), boxShadow: "0 4px 12px rgba(0,0,0,0.5)",
                      }}
                    >
                      <span style={{ flex: 1 }}>PlenumNET daemon unreachable after 5 minutes. Check that Inter-Cube is running.</span>
                      <CopyDetailsButton text={`Connection FAILED after 5min. Daemon: ${DAEMON_HTTP}. WS: ${getDaemonWsUrl()}. Health: /health. Last state: ${daemon.state}`} />
                      <button
                        data-testid="button-dismiss-toast"
                        onClick={() => setFailedToast(false)}
                        style={{ background: "none", border: "none", color: "var(--launcher-text-heading)", cursor: "pointer", fontSize: 14, padding: 2 }}
                      >
                        ✕
                      </button>
                    </div>
                  )}

                  {/* Connection Status Banner */}
                  {daemon.state !== "CONNECTED" && (
                    <div
                      role="status"
                      aria-live="polite"
                      data-testid="status-daemon-connection"
                      style={{ padding: "6px 16px", background: daemon.state === "FAILED" ? "var(--launcher-destructive)" : "var(--launcher-bg-surface)", display: "flex", alignItems: "center", justifyContent: "center", gap: 8, fontSize: 11, fontFamily: "var(--launcher-font-body)", color: daemon.state === "FAILED" ? "var(--launcher-text-heading)" : "var(--launcher-text-label)", flexWrap: "wrap" }}
                    >
                      {(daemon.state === "HEALTH_CHECK" || daemon.state === "CONNECTING") && (
                        <div style={{ width: 12, height: 12, border: "2px solid var(--launcher-accent)", borderTopColor: "transparent", borderRadius: "50%", animation: "launcherSpin 0.8s linear infinite" }} />
                      )}
                      {daemon.state === "RECONNECTING" && (
                        <div style={{ width: 6, height: 6, borderRadius: "50%", background: "var(--launcher-warning)", animation: "launcherPulse 2s infinite" }} />
                      )}
                      <span>{connectionStatusLabel}</span>
                      {daemon.state === "FAILED" && (
                        <>
                          <button data-testid="button-daemon-retry" onClick={daemon.retry} style={{ background: "var(--launcher-text-heading)", color: "var(--launcher-bg-primary)", border: "none", padding: "2px 8px", fontSize: 10, fontWeight: 600, cursor: "pointer", marginLeft: 8 }}>
                            Retry
                          </button>
                          <CopyDetailsButton text={`Connection FAILED. Daemon: ${DAEMON_HTTP}. WS: ${getDaemonWsUrl()}. Exhausted reconnection after 5 minutes.`} />
                        </>
                      )}
                      {daemon.state === "DISCONNECTED" && (
                        <span style={{ fontSize: 9 }}>Ensure PlenumNET Inter-Cube is running.</span>
                      )}
                    </div>
                  )}

                  {/* Settings overlay */}
                  {settingsOpen && (
                    <div style={{ position: "absolute", inset: 0, zIndex: 50, background: "var(--launcher-bg-primary)", backdropFilter: "blur(6px)", padding: "18px 20px", overflowY: "auto", display: "flex", flexDirection: "column", gap: 16 }}>
                      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                        <span style={{ fontSize: 12, fontWeight: 700, color: "var(--launcher-text-heading)" }}>Settings</span>
                        <Btn label="Close" c="var(--launcher-text-faint)" onClick={() => setSettingsOpen(false)} testId="button-settings-close" />
                      </div>
                      <div>
                        <div style={{ fontSize: 9, fontWeight: 600, color: "var(--launcher-text-label)", marginBottom: 6, letterSpacing: "0.06em", textTransform: "uppercase", fontFamily: "var(--launcher-font-mono)" }}>API Key</div>
                        <div style={{ display: "flex", gap: 6 }}>
                          <input
                            ref={apiKeyInputRef}
                            data-testid="input-api-key"
                            type="password"
                            defaultValue=""
                            onChange={(e) => {
                              apiKeyRef.current = e.target.value;
                              setApiKeyStored(false);
                              setApiKeyValid(null);
                              setApiKeyMsg("");
                            }}
                            onKeyDown={(e) => e.key === "Enter" && apiKeyRef.current && validateKeyWithDaemon(apiKeyRef.current)}
                            placeholder={apiKeyStored ? "••••••••••••••••" : "PlenumNET API key"}
                            aria-label="API key"
                            style={{ flex: 1, padding: "6px 10px", border: `1px solid ${apiKeyValid === false ? "var(--launcher-destructive)" : "var(--launcher-border)"}`, background: "var(--launcher-bg-surface)", color: "var(--launcher-text-nav)", fontSize: 11, fontFamily: "var(--launcher-font-mono)", outline: "none", clipPath: miniChamfer(4) }}
                          />
                          <Btn label="Validate" c="var(--launcher-accent)" onClick={() => apiKeyRef.current && validateKeyWithDaemon(apiKeyRef.current)} testId="button-validate-key" />
                        </div>
                        {apiKeyValid === true && <div style={{ fontSize: 9, color: "var(--launcher-success)", marginTop: 4, fontFamily: "var(--launcher-font-mono)" }}>✓ Validated</div>}
                        {apiKeyValid === false && <div style={{ fontSize: 9, color: "var(--launcher-destructive)", marginTop: 4, fontFamily: "var(--launcher-font-mono)" }}>✗ {apiKeyMsg || "Invalid"}</div>}
                        {apiKeyMsg && apiKeyValid !== false && <div style={{ fontSize: 9, color: "var(--launcher-text-label)", marginTop: 4, fontFamily: "var(--launcher-font-mono)" }}>{apiKeyMsg}</div>}
                        <div style={{ fontSize: 9, color: "var(--launcher-text-faint)", marginTop: 8, fontFamily: "var(--launcher-font-body)", lineHeight: 1.5 }}>
                          API keys are created when you first install PlenumNET. You can find them in your Cloud Console or contact your administrator for help.
                        </div>
                      </div>
                      <div style={{ height: 1, background: "var(--launcher-border)" }} />
                      <div>
                        <div style={{ fontSize: 9, fontWeight: 600, color: "var(--launcher-text-label)", marginBottom: 6, letterSpacing: "0.06em", textTransform: "uppercase", fontFamily: "var(--launcher-font-mono)" }}>Daemon</div>
                        <div style={{ padding: "8px 10px", background: "var(--launcher-bg-surface)", border: "1px solid var(--launcher-border)", clipPath: miniChamfer(4) }}>
                          {([
                            ["Endpoint", isOn ? getDaemonWsUrl() : "—", isOn ? "var(--launcher-accent)" : "var(--launcher-text-faint)"],
                            ["Array", isOn ? (clusterHealth?.arrayName ? `${clusterHealth.arrayName} (${clusterHealth.nodeCount ?? "?"} nodes)` : "querying...") : "—", isOn ? "var(--launcher-accent)" : "var(--launcher-text-faint)"],
                            ["Rep C", isOn ? (clusterHealth?.repC ?? "querying...") : "—", "var(--launcher-text-slate)"],
                            ["Arch", isOn ? (clusterHealth?.arch ?? "querying...") : "—", "var(--launcher-text-body)"],
                            ["Path", isOn ? (clusterHealth?.installPath ?? "querying...") : "—", "var(--launcher-text-slate)"],
                          ] as [string, string, string][]).map(([l, v, c]) => (
                            <div key={l} style={{ display: "flex", justifyContent: "space-between", marginBottom: 4, fontSize: 10 }}>
                              <span style={{ color: "var(--launcher-text-label)" }}>{l}</span>
                              <span data-testid={`text-daemon-${l.toLowerCase().replace(/\s/g, "-")}`} style={{ color: c, fontFamily: "var(--launcher-font-mono)", fontWeight: 500 }}>{v}</span>
                            </div>
                          ))}
                        </div>
                      </div>
                    </div>
                  )}

                  {/* Tabs */}
                  <div
                    role="tablist"
                    aria-label="Launcher tabs"
                    style={{
                      display: "flex", padding: "0 16px", flexShrink: 0,
                      background: `linear-gradient(180deg, var(--launcher-bg-deep) 0%, var(--launcher-bg-primary) 50%, var(--launcher-bg-deep) 100%)`,
                      boxShadow: "var(--launcher-tab-groove-shadow)",
                    }}
                    onKeyDown={handleTabKeyDown}
                  >
                    {TABS.map((t) => (
                      <button
                        key={t.id}
                        ref={(el) => tabRefs.current.set(t.id, el)}
                        role="tab"
                        id={`launcher-tab-${t.id}`}
                        aria-selected={tab === t.id}
                        aria-controls={`launcher-tabpanel-${t.id}`}
                        tabIndex={tab === t.id ? 0 : -1}
                        data-testid={`tab-${t.id}`}
                        onClick={() => setTab(t.id)}
                        style={{
                          padding: "9px 10px", border: "none", background: tab === t.id ? "var(--launcher-accent-active-bg)" : "transparent",
                          color: tab === t.id ? "var(--launcher-accent)" : "var(--launcher-text-faint)",
                          fontSize: 13, fontWeight: 500,
                          fontFamily: "var(--launcher-font-body)", cursor: "pointer",
                          borderBottom: tab === t.id ? "3px solid var(--launcher-accent)" : "3px solid transparent",
                          transition: "all 0.12s",
                          outline: "none",
                        }}
                      >
                        {t.label}
                        {t.id === "apps" && products.length > 0 && (
                          <span style={{ marginLeft: 4, fontSize: 9, fontFamily: "var(--launcher-font-mono)", opacity: 0.5 }}>{products.length}</span>
                        )}
                      </button>
                    ))}
                  </div>

                  {/* Tab Content */}
                  <div style={{ flex: 1, overflowY: "auto", padding: "10px 14px 14px", display: "flex", flexDirection: "column" }}>
                    {/* Assistant Tab */}
                    <div role="tabpanel" id="launcher-tabpanel-yoda" aria-labelledby="launcher-tab-yoda" hidden={tab !== "yoda"} style={{ display: tab === "yoda" ? "flex" : "none", flexDirection: "column", flex: 1 }}>
                      <div style={{ display: "flex", alignItems: "center", gap: 6, marginBottom: 4 }}>
                        <span style={{ color: "var(--launcher-accent)", fontSize: 12 }}>◉</span>
                        <span style={{ fontSize: 11, fontWeight: 600, color: "var(--launcher-text-heading)" }}>Conversational Relay</span>
                        <span style={{ fontSize: 8, fontFamily: "var(--launcher-font-mono)", color: "var(--launcher-text-faint)", marginLeft: "auto" }}>TL-DSA</span>
                      </div>
                      <div style={{ height: 1, background: "var(--launcher-border)", marginBottom: 6 }} />
                      <YodaChat connected={isOn} wsSend={daemon.wsSend} subscribe={daemon.subscribe} />
                    </div>

                    {/* Apps Tab */}
                    <div role="tabpanel" id="launcher-tabpanel-apps" aria-labelledby="launcher-tab-apps" hidden={tab !== "apps"} style={{ display: tab === "apps" ? "block" : "none" }}>
                      {!isOn ? (
                        <div style={{ textAlign: "center", padding: "44px 20px" }}>
                          <div style={{ fontSize: 28, marginBottom: 12, opacity: 0.15 }}>◈</div>
                          <div style={{ fontSize: 12, color: "var(--launcher-text-label)" }}>
                            Connecting to daemon...
                            <CopyDetailsButton text={`Apps tab offline. Daemon at ${DAEMON_HTTP} unreachable. State: ${daemon.state}`} />
                          </div>
                        </div>
                      ) : (
                        <>
                          {restErrors["/node/info"] && (
                            <div role="alert" style={{ padding: "6px 10px", marginBottom: 6, background: "var(--launcher-bg-surface)", border: "1px solid var(--launcher-border)", clipPath: miniChamfer(4), fontSize: 9, color: "var(--launcher-text-label)", display: "flex", alignItems: "center", gap: 6, flexWrap: "wrap" }}>
                              <span>Product list could not be loaded.</span>
                              <CopyDetailsButton text={restErrors["/node/info"]} />
                            </div>
                          )}
                          <div style={{ display: "flex", gap: 5, marginBottom: 8 }}>
                            {[
                              { id: "all", label: "All", c: products.length },
                              { id: "installed", label: "Installed", c: installed.length },
                              { id: "available", label: "Available", c: available.length },
                            ].map((f) => (
                              <button
                                key={f.id}
                                data-testid={`button-filter-${f.id}`}
                                onClick={() => setFilter(f.id)}
                                style={{
                                  padding: "2px 8px", fontSize: 9, fontWeight: 600, fontFamily: "var(--launcher-font-body)", cursor: "pointer",
                                  border: `1px solid ${filter === f.id ? "var(--launcher-accent-dim)" : "transparent"}`,
                                  background: filter === f.id ? "var(--launcher-accent-active-subtle)" : "transparent",
                                  color: filter === f.id ? "var(--launcher-accent)" : "var(--launcher-text-faint)",
                                }}
                              >
                                {f.label} <span style={{ fontFamily: "var(--launcher-font-mono)" }}>{f.c}</span>
                              </button>
                            ))}
                          </div>
                          <div style={{ display: "flex", flexDirection: "column", gap: 1 }}>
                            {filtered.map((p) => (
                              <ProductCard key={p.id} product={p} onAction={handleAction} actionLoading={actionLoading} />
                            ))}
                          </div>
                          {available.length > 0 && !apiKeyValid && (
                            <div style={{ marginTop: 8, padding: "7px 10px", background: "var(--launcher-bg-surface)", fontSize: 10, color: "var(--launcher-text-label)", clipPath: miniChamfer(4) }}>
                              <span style={{ color: "var(--launcher-accent)", fontWeight: 600 }}>API key required</span> —{" "}
                              <button
                                data-testid="button-open-settings"
                                onClick={() => setSettingsOpen(true)}
                                style={{ background: "none", border: "none", color: "var(--launcher-accent-hover)", cursor: "pointer", fontFamily: "var(--launcher-font-body)", fontSize: 10, fontWeight: 600, padding: 0, textDecoration: "underline" }}
                              >
                                open settings ⚙
                              </button>
                            </div>
                          )}
                        </>
                      )}
                    </div>

                    {/* System Tab */}
                    <div role="tabpanel" id="launcher-tabpanel-local" aria-labelledby="launcher-tab-local" hidden={tab !== "local"} style={{ display: tab === "local" ? "flex" : "none", flexDirection: "column", gap: 6 }}>
                      <div style={{ fontSize: 11, fontWeight: 600, color: "var(--launcher-text-heading)", marginBottom: 2 }}>System Resources</div>
                      {!isOn ? (
                        <div style={{ textAlign: "center", padding: "20px", fontSize: 11, color: "var(--launcher-text-label)" }}>
                          Awaiting system data — daemon offline
                          <CopyDetailsButton text={`System tab offline. Daemon at ${DAEMON_HTTP} unreachable. State: ${daemon.state}`} />
                        </div>
                      ) : (
                        <>
                          {(restErrors["/cluster-health"] || restErrors["/fts/status"] || restErrors["/con/stats"]) && (
                            <div role="alert" style={{ padding: "6px 10px", background: "var(--launcher-bg-surface)", border: "1px solid var(--launcher-border)", clipPath: miniChamfer(4), fontSize: 9, color: "var(--launcher-text-label)", display: "flex", alignItems: "center", gap: 6, flexWrap: "wrap" }}>
                              <span>Some system endpoints failed to respond.</span>
                              <CopyDetailsButton text={Object.entries(restErrors).filter(([k]) => ["/cluster-health", "/fts/status", "/con/stats"].includes(k)).map(([k, v]) => `${k}: ${v}`).join("\n")} />
                            </div>
                          )}
                          {(systemData?.resources ?? [
                            { label: "CPU", value: 0, detail: "Awaiting /cluster-health...", cores: "—" },
                            { label: "RAM", value: 0, detail: "Awaiting /cluster-health...", cores: null },
                            { label: "GPU", value: 0, detail: "Awaiting /cluster-health...", cores: "—" },
                            { label: "Disk", value: 0, detail: "Awaiting /cluster-health...", cores: "—" },
                          ] as ResourceEntry[]).map((r) => (
                            <div key={r.label} data-testid={`status-resource-${r.label.toLowerCase()}`} style={{ padding: "8px 10px", background: "var(--launcher-bg-panel)", border: "1px solid var(--launcher-border)", clipPath: miniChamfer(5) }}>
                              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 4 }}>
                                <span style={{ fontSize: 10, fontWeight: 700, color: "var(--launcher-text-heading)" }}>{r.label}</span>
                                <span style={{ fontSize: 9, fontFamily: "var(--launcher-font-mono)", color: "var(--launcher-text-faint)" }}>{r.cores}</span>
                              </div>
                              <div style={{ height: 4, background: "var(--launcher-bg-surface)", borderRadius: 2, overflow: "hidden", marginBottom: 4 }}>
                                <div style={{ height: "100%", width: `${r.value ?? 0}%`, background: (r.value ?? 0) > 80 ? "var(--launcher-destructive)" : "var(--launcher-accent)", borderRadius: 2, transition: "width 0.5s" }} />
                              </div>
                              <div style={{ display: "flex", justifyContent: "space-between" }}>
                                <span style={{ fontSize: 9, fontFamily: "var(--launcher-font-mono)", color: "var(--launcher-text-body)" }}>{r.detail}</span>
                                <span style={{ fontSize: 9, fontFamily: "var(--launcher-font-mono)", color: (r.value ?? 0) > 80 ? "var(--launcher-destructive)" : "var(--launcher-accent)" }}>{r.value ?? 0}%</span>
                              </div>
                            </div>
                          ))}

                          <div style={{ fontSize: 11, fontWeight: 600, color: "var(--launcher-text-heading)", marginTop: 4, marginBottom: 2 }}>AI Models Active</div>
                          {(systemData?.models ?? [] as ModelEntry[]).map((m) => (
                            <div key={m.name} style={{ padding: "6px 10px", background: "var(--launcher-bg-panel)", border: "1px solid var(--launcher-border)", clipPath: miniChamfer(4), display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                              <div>
                                <div style={{ fontSize: 10, fontWeight: 600, color: "var(--launcher-text-heading)" }}>{m.name}</div>
                                <div style={{ fontSize: 9, fontFamily: "var(--launcher-font-mono)", color: "var(--launcher-text-faint)" }}>{m.engine} · {m.mem}</div>
                              </div>
                              <StatusDot state={m.status === "running" ? "connected" : "disconnected"} label={m.status} />
                            </div>
                          ))}

                          <div style={{ padding: "6px 10px", background: "var(--launcher-bg-surface)", clipPath: miniChamfer(4), fontSize: 9, fontFamily: "var(--launcher-font-mono)", color: "var(--launcher-text-faint)", display: "flex", flexWrap: "wrap", gap: "2px 16px" }}>
                            <span>{systemData?.integrity ?? "Integrity check pending"}</span>
                          </div>

                          <div style={{ fontSize: 11, fontWeight: 600, color: "var(--launcher-text-heading)", marginTop: 4, marginBottom: 2 }}>Fault Tolerance (FTS)</div>
                          <div data-testid="status-fts" style={{ padding: "6px 10px", background: "var(--launcher-bg-panel)", border: "1px solid var(--launcher-border)", clipPath: miniChamfer(4), fontSize: 9, fontFamily: "var(--launcher-font-mono)", color: "var(--launcher-text-body)", display: "flex", flexWrap: "wrap", gap: "2px 12px" }}>
                            {systemData?.ftsNeighbors ? (
                              <>
                                <span>{systemData.ftsNeighbors.filter((n) => n.status === "up").length} up</span>
                                <span>{systemData.ftsNeighbors.filter((n) => n.status === "suspect").length} suspect</span>
                                <span>{systemData.ftsNeighbors.filter((n) => n.status === "down").length} down</span>
                              </>
                            ) : (
                              <span style={{ color: "var(--launcher-text-faint)" }}>Awaiting /fts/status...</span>
                            )}
                          </div>

                          <div style={{ fontSize: 11, fontWeight: 600, color: "var(--launcher-text-heading)", marginTop: 4, marginBottom: 2 }}>Overlay Network (CON)</div>
                          <div data-testid="status-con" style={{ padding: "6px 10px", background: "var(--launcher-bg-panel)", border: "1px solid var(--launcher-border)", clipPath: miniChamfer(4), fontSize: 9, fontFamily: "var(--launcher-font-mono)", color: "var(--launcher-text-body)", display: "flex", flexWrap: "wrap", gap: "2px 12px" }}>
                            {systemData?.conStats ? (
                              <>
                                <span>Tunnels: {systemData.conStats.tunnelsUp}/{systemData.conStats.tunnelsTotal}</span>
                                <span>PQ Keys: {systemData.conStats.pqKeysDeived}</span>
                                <span>Status: {systemData.conStats.overlayStatus}</span>
                              </>
                            ) : (
                              <span style={{ color: "var(--launcher-text-faint)" }}>Awaiting /con/stats...</span>
                            )}
                          </div>
                        </>
                      )}
                    </div>

                    {/* Network Tab */}
                    <div role="tabpanel" id="launcher-tabpanel-net" aria-labelledby="launcher-tab-net" hidden={tab !== "net"} style={{ display: tab === "net" ? "flex" : "none", flexDirection: "column", gap: 6 }}>
                      <div style={{ fontSize: 11, fontWeight: 600, color: "var(--launcher-text-heading)", marginBottom: 2 }}>PlenumLAN Network</div>
                      {!isOn ? (
                        <div style={{ textAlign: "center", padding: "20px", fontSize: 11, color: "var(--launcher-text-label)" }}>
                          Network data unavailable — daemon offline
                          <CopyDetailsButton text={`Network tab offline. Daemon at ${DAEMON_HTTP} unreachable. State: ${daemon.state}. Checked /topology endpoint.`} />
                          <div style={{ marginTop: 8 }}>
                            <button
                              data-testid="button-check-settings"
                              onClick={() => setSettingsOpen(true)}
                              style={{ background: "none", border: "none", color: "var(--launcher-accent-hover)", cursor: "pointer", fontFamily: "var(--launcher-font-body)", fontSize: 11, textDecoration: "underline", padding: 0 }}
                            >
                              Check Settings
                            </button>
                          </div>
                        </div>
                      ) : (
                        <>
                          {(restErrors["/topology"] || restErrors["/cluster-health"]) && (
                            <div role="alert" style={{ padding: "6px 10px", background: "var(--launcher-bg-surface)", border: "1px solid var(--launcher-border)", clipPath: miniChamfer(4), fontSize: 9, color: "var(--launcher-text-label)", display: "flex", alignItems: "center", gap: 6, flexWrap: "wrap" }}>
                              <span>Network endpoints failed to respond.</span>
                              <CopyDetailsButton text={[restErrors["/topology"], restErrors["/cluster-health"]].filter(Boolean).join("\n")} />
                            </div>
                          )}
                          <div style={{ padding: "8px 10px", background: "var(--launcher-bg-panel)", border: "1px solid var(--launcher-border)", clipPath: miniChamfer(5) }}>
                            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 6 }}>
                              <span style={{ fontSize: 10, fontWeight: 700, color: "var(--launcher-text-heading)" }}>{networkData?.clusterHealth?.arrayName ?? "Array"} Relay Topology</span>
                              <StatusDot state="connected" label={networkData?.topologyStatus ?? "nominal"} />
                            </div>
                            {(networkData?.nodes ?? [] as NetworkNode[]).map((n) => (
                              <div key={n.node} data-testid={`text-node-${n.node.toLowerCase()}`} style={{ display: "flex", justifyContent: "space-between", alignItems: "center", padding: "4px 0", borderTop: "1px solid var(--launcher-border)" }}>
                                <div>
                                  <div style={{ fontSize: 10, fontWeight: 600, color: "var(--launcher-text-nav)" }}>{n.node}</div>
                                  <div style={{ fontSize: 8, fontFamily: "var(--launcher-font-mono)", color: "var(--launcher-accent)" }}>{n.addr}</div>
                                </div>
                                <div style={{ textAlign: "right" }}>
                                  <div style={{ fontSize: 8, fontFamily: "var(--launcher-font-mono)", color: "var(--launcher-text-faint)" }}>{n.role}</div>
                                  <div style={{ fontSize: 8, fontFamily: "var(--launcher-font-mono)", color: "var(--launcher-text-body)" }}>{n.latency}</div>
                                </div>
                              </div>
                            ))}
                          </div>

                          <div style={{ fontSize: 10, fontWeight: 600, color: "var(--launcher-text-heading)", marginTop: 2 }}>Interfaces</div>
                          {(networkData?.interfaces ?? [] as NetworkInterface[]).map((iface) => (
                            <div key={iface.name} style={{ padding: "6px 10px", background: "var(--launcher-bg-panel)", border: "1px solid var(--launcher-border)", clipPath: miniChamfer(4) }}>
                              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 2 }}>
                                <span style={{ fontSize: 10, fontWeight: 600, color: "var(--launcher-text-heading)" }}>{iface.name}</span>
                                <StatusDot state={iface.status === "up" ? "connected" : "disconnected"} label={iface.status} />
                              </div>
                              <div style={{ fontSize: 9, fontFamily: "var(--launcher-font-mono)", color: "var(--launcher-text-body)", display: "flex", gap: 12 }}>
                                <span>IPv4: {iface.ip}</span>
                                <span>IPv6: {iface.ipv6}</span>
                                <span>{iface.speed}</span>
                              </div>
                            </div>
                          ))}

                          <div style={{ padding: "6px 10px", background: "var(--launcher-bg-surface)", clipPath: miniChamfer(4), fontSize: 9, fontFamily: "var(--launcher-font-mono)", color: "var(--launcher-text-faint)", display: "flex", justifyContent: "space-between" }}>
                            <span>↑ {networkData?.bandwidth?.up ?? "—"}</span>
                            <span>↓ {networkData?.bandwidth?.down ?? "—"}</span>
                            <span>Peers: {networkData?.peers ?? "—"}</span>
                            <span>WS: {networkData?.wsConnections ?? "—"}</span>
                          </div>
                        </>
                      )}
                    </div>

                    {/* APIs Tab */}
                    <div role="tabpanel" id="launcher-tabpanel-apis" aria-labelledby="launcher-tab-apis" hidden={tab !== "apis"} style={{ display: tab === "apis" ? "flex" : "none", flexDirection: "column", gap: 8 }}>
                      <div style={{ fontSize: 11, fontWeight: 600, color: "var(--launcher-text-heading)", marginBottom: 2 }}>PlenumNET API Endpoints</div>
                      {[
                        { name: "CRS", url: "plenumnet.replit.app", status: isOn ? "live" : "unknown", desc: "Central Registration Service" },
                        { name: "Relay", url: "ws://localhost:11124", status: isOn ? "live" : "offline", desc: "Daemon gateway — relay + API" },
                        { name: "NinjaExec", url: "localhost:21027", status: isOn ? "live" : "offline", desc: "TL-DSA signer (daemon-internal)" },
                        { name: "YODA", url: "via relay", status: isOn ? "live" : "offline", desc: "Multi-agent intelligence" },
                        { name: "TDNS", url: "localhost:5353", status: "not installed", desc: "Ternary name resolution" },
                        { name: "PPTPro", url: "pptpro.replit.app", status: isOn ? "live" : "unknown", desc: "Bio-signal tonal engine" },
                      ].map((api) => (
                        <div key={api.name} data-testid={`card-api-${api.name.toLowerCase()}`} style={{ padding: "8px 10px", background: "var(--launcher-bg-panel)", border: "1px solid var(--launcher-border)", clipPath: miniChamfer(5) }}>
                          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 3 }}>
                            <span style={{ fontSize: 11, fontWeight: 600, color: "var(--launcher-text-heading)" }}>{api.name}</span>
                            <StatusDot state={api.status === "live" ? "connected" : "disconnected"} label={api.status === "live" ? "live" : api.status === "unknown" ? "unknown" : api.status} />
                          </div>
                          <div style={{ fontSize: 9, fontFamily: "var(--launcher-font-mono)", color: "var(--launcher-accent)", marginBottom: 2 }}>{api.url}</div>
                          <div style={{ fontSize: 9, color: "var(--launcher-text-faint)" }}>{api.desc}</div>
                        </div>
                      ))}
                    </div>
                  </div>

                  {/* Footer */}
                  <div
                    data-testid="launcher-footer"
                    style={{
                      flexShrink: 0,
                      background: `linear-gradient(180deg, var(--launcher-bg-deep) 0%, var(--launcher-bg-primary) 50%, var(--launcher-bg-deep) 100%)`,
                      boxShadow: "var(--launcher-tab-groove-shadow)",
                      padding: "6px 16px",
                      display: "flex", justifyContent: "space-between", fontSize: 8, color: "var(--launcher-text-faint)", fontFamily: "var(--launcher-font-mono)", letterSpacing: "0.04em",
                    }}
                  >
                    <span>LAUNCHER v1.0.0</span>
                    <span>SALVI FRAMEWORK</span>
                    <div role="status" aria-live="polite">
                      <span>{isOn ? "◈ CONNECTED" : connectionStatusLabel.toUpperCase()}</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </>
  );
}

function LauncherStyles() {
  return (
    <style>{`
      @import url('https://fonts.googleapis.com/css2?family=Orbitron:wght@700;800;900&family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500;600;700&display=swap');

      [data-launcher-theme="dark"] {
        --launcher-bg-primary: #0F0C0A;
        --launcher-bg-panel: #181411;
        --launcher-bg-surface: #1D1915;
        --launcher-bg-secondary: #181411;
        --launcher-bg-deep: #080706;
        --launcher-bg-outer-frame: #2A2724;
        --launcher-bg-inner-frame: #16120E;
        --launcher-bg-chamfer: #0C0A08;
        --launcher-border: #272220;
        --launcher-text-heading: #F0EDE8;
        --launcher-text-nav: #E4DFD5;
        --launcher-text-body: #C9C1B4;
        --launcher-text-label: #998F82;
        --launcher-text-faint: #6B655E;
        --launcher-text-slate: #78828C;
        --launcher-accent: #4A9EF5;
        --launcher-accent-hover: #38BDF8;
        --launcher-accent-dim: rgba(74,158,245,0.3);
        --launcher-accent-glow: rgba(74,158,245,0.08);
        --launcher-accent-active-bg: rgba(74,158,245,0.1);
        --launcher-accent-active-subtle: rgba(74,158,245,0.04);
        --launcher-iron: #3D444B;
        --launcher-destructive: #E5484D;
        --launcher-success: #30A46C;
        --launcher-warning: #D97706;
        --launcher-header-bg: #E4DFD5;
        --launcher-header-mid: #D1CCC6;
        --launcher-header-edge: #C9C1B4;
        --launcher-header-text: #1A1714;
        --launcher-header-sub: #6B655E;
        --launcher-header-info-bg: rgba(0,0,0,0.04);
        --launcher-header-info-border: rgba(0,0,0,0.05);
        --launcher-ledge-top: #B8B0A5;
        --launcher-shadow-overlay: rgba(0,0,0,0.8);
        --launcher-shadow-subtle: rgba(0,0,0,0.6);
        --launcher-highlight: rgba(255,255,255,0.12);
        --launcher-highlight-subtle: rgba(255,255,255,0.04);
        --launcher-highlight-edge: rgba(255,255,255,0.18);
        --launcher-tab-groove-shadow: inset 0 3px 6px rgba(0,0,0,0.7), inset 0 -2px 3px rgba(0,0,0,0.4), 0 1px 0 rgba(255,255,255,0.04);
        --launcher-font-display: 'Orbitron', sans-serif;
        --launcher-font-body: 'Inter', system-ui, sans-serif;
        --launcher-font-mono: 'JetBrains Mono', 'Fira Code', monospace;
      }

      @keyframes launcherSlideFromTray {
        from { transform: translateY(100%) scale(0.95); opacity: 0; }
        to { transform: translateY(0) scale(1); opacity: 1; }
      }

      @keyframes launcherSlideToTray {
        from { transform: translateY(0) scale(1); opacity: 1; }
        to { transform: translateY(100%) scale(0.95); opacity: 0; }
      }

      @keyframes launcherPulse {
        0%, 100% { opacity: 1; }
        50% { opacity: 0.4; }
      }

      @keyframes launcherSpin {
        to { transform: rotate(360deg); }
      }

      @media (prefers-reduced-motion: reduce) {
        [data-launcher-theme="dark"] * {
          animation-duration: 0.01ms !important;
          transition-duration: 0.01ms !important;
        }
      }

      .no-animations [data-launcher-theme="dark"] * {
        animation-duration: 0.01ms !important;
        transition-duration: 0.01ms !important;
      }

      @media (max-width: 480px) {
        [data-testid="launcher-panel"],
        [data-testid="launcher-minimized-bar"] {
          width: 100vw !important;
          right: 0 !important;
        }
      }

      [data-launcher-theme="dark"] ::-webkit-scrollbar { width: 3px; }
      [data-launcher-theme="dark"] ::-webkit-scrollbar-track { background: #0F0C0A; }
      [data-launcher-theme="dark"] ::-webkit-scrollbar-thumb { background: #272220; }

      [data-launcher-theme="dark"] input::placeholder { color: #6B655E; }

      [data-launcher-theme="dark"] button:focus-visible,
      [data-launcher-theme="dark"] input:focus-visible {
        outline: 2px solid #4A9EF5;
        outline-offset: 2px;
      }
    `}</style>
  );
}


export function LauncherProvider({ children, widgetMode = false }: { children: React.ReactNode; widgetMode?: boolean }) {
  const [panelState, setPanelState] = useState<PanelState>(widgetMode ? "OPEN" : "CLOSED");

  const togglePanel = useCallback(() => {
    setPanelState((prev) => {
      if (prev === "CLOSED") return "OPENING";
      if (prev === "OPEN") return "CLOSING";
      if (prev === "MINIMIZED") return "OPEN";
      return prev;
    });
  }, []);

  const isActive = panelState !== "CLOSED";

  const value = useMemo(
    () => ({ panelState, setPanelState, togglePanel, isActive, widgetMode }),
    [panelState, setPanelState, togglePanel, isActive, widgetMode]
  );

  return (
    <LauncherContext.Provider value={value}>
      {children}
      <LauncherPanelInner />
    </LauncherContext.Provider>
  );
}

export default function LauncherPanel() {
  return null;
}
