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
}

const LauncherContext = createContext<LauncherContextValue>({
  panelState: "CLOSED",
  setPanelState: () => {},
  togglePanel: () => {},
  isActive: false,
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
  const { panelState, setPanelState } = useLauncher();
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
                const newStatus = typeof data.status === "string" ? data.status as ProductStatusPayload["status"] : (action === "stop" ? "stopped" : "running");
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
        style={{
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
        <div style={{ clipPath: chamfer(0), background: `linear-gradient(160deg, var(--launcher-bg-outer-frame) 0%, var(--launcher-bg-inner-frame) 50%, var(--launcher-bg-primary) 100%)`, padding: 2, position: "relative" }}>
          <div style={{ position: "absolute", inset: 0, pointerEvents: "none", zIndex: 10, clipPath: chamfer(0), background: `linear-gradient(180deg, var(--launcher-highlight) 0%, var(--launcher-highlight-subtle) 15%, transparent 40%)` }} />
          <div style={{ clipPath: chamfer(2), background: `linear-gradient(145deg, var(--launcher-bg-primary) 0%, var(--launcher-bg-inner-frame) 30%, var(--launcher-bg-surface) 100%)`, padding: 8, position: "relative", overflow: "hidden" }}>
            <LeatherGrain seed={3} />
            <div style={{ clipPath: chamfer(5), background: `linear-gradient(145deg, var(--launcher-bg-chamfer) 0%, var(--launcher-bg-panel) 50%, var(--launcher-bg-surface) 100%)`, padding: 2, position: "relative", overflow: "hidden", zIndex: 6 }}>
              <LeatherGrain seed={7} />
              <div style={{ clipPath: chamfer(6), overflow: "hidden", display: "flex", flexDirection: "column", background: "var(--launcher-bg-primary)", position: "relative", zIndex: 6 }}>

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
                    <div style={{ fontFamily: "var(--launcher-font-display)", fontSize: 16, fontWeight: 700, color: "var(--launcher-header-text)", letterSpacing: "0.08em" }}>
                      PlenumNET Launcher
                    </div>
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


export function LauncherProvider({ children }: { children: React.ReactNode }) {
  const [panelState, setPanelState] = useState<PanelState>("CLOSED");

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
    () => ({ panelState, setPanelState, togglePanel, isActive }),
    [panelState, setPanelState, togglePanel, isActive]
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
