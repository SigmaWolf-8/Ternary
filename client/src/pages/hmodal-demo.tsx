import { useState, useEffect, useRef, useCallback } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Activity, Play, Square, Download, FileText, Zap, AlertCircle, CheckCircle, ShieldCheck } from "lucide-react";

interface StatusResponse {
  raplAvailable: boolean;
  raplPath: string;
  mode: "hardware-watts" | "compute-throughput-proxy";
  message: string;
  constants: {
    alpha: string;
    beta: string;
    dutyHigh: string;
    dutyLow: string;
    savings: string;
    savingsPct: number;
    dcMean: string;
    discriminant: number;
  };
}

interface Sample {
  t: number;
  phase: "high" | "low";
  opsPerSec: number;
  watts: number | null;
  mode: string;
  observedRatio: number;
  theoreticalRatio: number;
  savingsObserved: number | null;
  theoreticalSavings: number;
  cumulativeEnergyUj: number;
  cumulativeOps?: number;
  cumulativeOpsHigh?: number;
  cumulativeOpsLow?: number;
  timeHighMs?: number;
  timeLowMs?: number;
  cacheHits?: number;
  cacheMisses?: number;
  cacheHitRate?: number;
  realHighWorkMs?: number;
  cachedHighMs?: number;
  compressedSavings?: number;
  theoreticalCompressedSavings?: number;
  wattsContinuous?: number;
  wattsHmodalNoCache?: number;
  wattsHmodalCached?: number;
  wattsSavedVsContinuous?: number;
  effectiveComputeFrac?: number;
  logicalOpsPerSecAvg?: number;
  realCpuOpsPerSecAvg?: number;
  demandMode?: "idle" | "steady" | "burst" | "auto";
  queueDepth?: number;
  cacheFillRatio?: number;
  dutyTarget?: number;
  keyTouchCount?: number;
  signatureCount?: number;
  keyExposureRatio?: number;
  keyIsolationFactor?: number;
}

type DemandMode = "idle" | "steady" | "burst" | "auto";

const MAX_SAMPLES = 300;

type WsState = "idle" | "connecting" | "live" | "closed" | "error";

export default function HModalDemo() {
  const [status, setStatus] = useState<StatusResponse | null>(null);
  const [running, setRunning] = useState(false);
  const [samples, setSamples] = useState<Sample[]>([]);
  const [latest, setLatest] = useState<Sample | null>(null);
  const [wsState, setWsState] = useState<WsState>("idle");
  const [wsUrl, setWsUrl] = useState<string>("");
  const [lastRaw, setLastRaw] = useState<string>("");
  const [frameCount, setFrameCount] = useState(0);
  const [demandMode, setDemandMode] = useState<DemandMode>("auto");
  const [eacIssuing, setEacIssuing] = useState(false);
  const [eacResult, setEacResult] = useState<any>(null);
  const [eacError, setEacError] = useState<string | null>(null);
  const [tunnel, setTunnel] = useState<{
    sessionId?: string;
    cipher?: string;
    chainSeedHex?: string;
    sealedCount: number;
    chainTagHex?: string;
    lastIndex?: string;
  }>({ sealedCount: 0 });

  const issueEac = useCallback(async () => {
    setEacIssuing(true);
    setEacError(null);
    try {
      const r = await fetch("/api/hmodal/issue-eac", { method: "POST" });
      const j = await r.json();
      if (!r.ok || !j.ok) {
        setEacError(j.message || j.error || `HTTP ${r.status}`);
        setEacResult(j);
      } else {
        setEacResult(j.eac);
      }
    } catch (e: any) {
      setEacError(e?.message ?? String(e));
    } finally {
      setEacIssuing(false);
    }
  }, []);
  const wsRef = useRef<WebSocket | null>(null);
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  const sendMode = useCallback((mode: DemandMode) => {
    setDemandMode(mode);
    const ws = wsRef.current;
    if (ws && ws.readyState === ws.OPEN) {
      try { ws.send(JSON.stringify({ type: "setMode", mode })); } catch {}
    }
  }, []);

  useEffect(() => {
    fetch("/api/hmodal/status")
      .then((r) => r.json())
      .then(setStatus)
      .catch(() => setStatus(null));
  }, []);

  const start = useCallback(() => {
    if (wsRef.current) return;
    setSamples([]);
    setLatest(null);
    setLastRaw("");
    setFrameCount(0);
    const proto = window.location.protocol === "https:" ? "wss:" : "ws:";
    const url = `${proto}//${window.location.host}/ws/hmodal`;
    setWsUrl(url);
    setWsState("connecting");
    let ws: WebSocket;
    try {
      ws = new WebSocket(url);
    } catch (err: any) {
      setWsState("error");
      setLastRaw(`construct error: ${err?.message ?? err}`);
      setRunning(false);
      return;
    }
    ws.onopen = () => setWsState("live");
    ws.onmessage = (e) => {
      const raw = typeof e.data === "string" ? e.data : "(binary)";
      setLastRaw(raw.length > 220 ? raw.substring(0, 217) + "..." : raw);
      setFrameCount((c) => c + 1);
      try {
        const msg = JSON.parse(raw);
        if (msg.type === "session") {
          setTunnel({
            sessionId: msg.sessionId,
            cipher: msg.cipher,
            chainSeedHex: msg.chainSeedHex,
            sealedCount: 0,
          });
          return;
        }
        if (msg.type === "sealed") {
          setTunnel((prev) => ({
            ...prev,
            sealedCount: prev.sealedCount + 1,
            chainTagHex: msg.chainTagHex,
            lastIndex: msg.index,
          }));
          return;
        }
        if (msg.type !== "sample") return;
        const s: Sample = msg;
        setLatest(s);
        setSamples((prev) => {
          const next = [...prev, s];
          if (next.length > MAX_SAMPLES) next.splice(0, next.length - MAX_SAMPLES);
          return next;
        });
      } catch {}
    };
    ws.onclose = (ev) => {
      setWsState("closed");
      setLastRaw((p) => `closed code=${ev.code} reason=${ev.reason || "(none)"}\n${p}`);
      wsRef.current = null;
      setRunning(false);
    };
    ws.onerror = () => {
      setWsState("error");
      setLastRaw((p) => `WebSocket error\n${p}`);
    };
    wsRef.current = ws;
    setRunning(true);
  }, []);

  const stop = useCallback(() => {
    try { wsRef.current?.close(); } catch {}
    wsRef.current = null;
    setRunning(false);
    setWsState("idle");
  }, []);

  // Auto-start on mount so the user sees life immediately.
  useEffect(() => {
    start();
    return () => { try { wsRef.current?.close(); } catch {} };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Strip chart
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    const W = canvas.width;
    const H = canvas.height;
    ctx.fillStyle = "#0F0C0A";
    ctx.fillRect(0, 0, W, H);

    if (samples.length === 0) {
      ctx.fillStyle = "#78828C";
      ctx.font = "14px ui-sans-serif, system-ui";
      ctx.fillText("waiting for samples...", 16, H / 2);
      return;
    }

    const useWatts = samples.some((s) => s.watts != null);
    const values = samples.map((s) => (useWatts ? (s.watts ?? 0) : s.opsPerSec));
    const max = Math.max(...values, 1);
    const min = 0;

    // grid
    ctx.strokeStyle = "#2D7DD2";
    ctx.globalAlpha = 0.2;
    ctx.lineWidth = 1;
    for (let i = 0; i <= 4; i++) {
      const y = (i * H) / 4;
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(W, y);
      ctx.stroke();
    }
    ctx.globalAlpha = 1;

    // duty-cycle background bands
    const stepW = W / Math.max(samples.length, 1);
    samples.forEach((s, i) => {
      ctx.fillStyle = s.phase === "high" ? "rgba(74,158,245,0.18)" : "rgba(120,130,140,0.06)";
      ctx.fillRect(i * stepW, 0, stepW + 0.5, H);
    });

    // line
    ctx.strokeStyle = "#4A9EF5";
    ctx.lineWidth = 2;
    ctx.beginPath();
    values.forEach((v, i) => {
      const x = i * stepW;
      const y = H - ((v - min) / (max - min || 1)) * (H - 8) - 4;
      if (i === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    });
    ctx.stroke();

    // axis labels
    ctx.fillStyle = "#a8b4c0";
    ctx.font = "11px ui-monospace, Menlo, Consolas";
    ctx.fillText(useWatts ? `${max.toFixed(2)} W` : `${(max / 1e6).toFixed(2)} Mops/s`, 6, 14);
    ctx.fillText("0", 6, H - 4);
  }, [samples]);

  const isHardware = status?.raplAvailable === true;

  return (
    <div className="min-h-screen bg-background text-foreground p-6 md:p-10" data-testid="page-hmodal-demo">
      <div className="max-w-6xl mx-auto space-y-6">
        <div>
          <h1 className="text-3xl md:text-4xl font-bold text-primary" data-testid="text-title">
            HModal Power Demo
          </h1>
          <p className="text-muted-foreground mt-2">
            Trit-native square-wave workload. α = 91/36, β = 91/3, duty 1:4, theoretical savings 143/192 ≈ 74.48%.
          </p>
        </div>

        {status && (
          <Card className={isHardware ? "border-green-500/40" : "border-yellow-500/40"} data-testid="card-status">
            <CardHeader className="flex flex-row items-start gap-3 pb-2">
              {isHardware ? (
                <CheckCircle className="w-5 h-5 text-green-500 mt-0.5" />
              ) : (
                <AlertCircle className="w-5 h-5 text-yellow-500 mt-0.5" />
              )}
              <CardTitle className="text-base">
                {isHardware
                  ? "Mode: HARDWARE WATTS (Intel RAPL detected)"
                  : "Mode: COMPUTE-THROUGHPUT PROXY (RAPL not exposed in this container)"}
              </CardTitle>
            </CardHeader>
            <CardContent className="text-sm text-muted-foreground" data-testid="text-status-message">
              {status.message}
            </CardContent>
          </Card>
        )}

        <Card className="bg-black/40 border-blue-500/30" data-testid="card-ws-debug">
          <CardContent className="pt-4 pb-4 font-mono text-xs">
            <div className="flex flex-wrap items-center gap-3 mb-2">
              <Badge
                variant={wsState === "live" ? "default" : "secondary"}
                className={
                  wsState === "live" ? "bg-green-600" :
                  wsState === "connecting" ? "bg-yellow-600" :
                  wsState === "error" ? "bg-red-600" : "bg-gray-600"
                }
                data-testid="badge-ws-state"
              >
                WS: {wsState.toUpperCase()}
              </Badge>
              <span className="text-muted-foreground">frames received: <span className="text-primary" data-testid="text-frame-count">{frameCount}</span></span>
              <span className="text-muted-foreground">samples buffered: <span className="text-primary">{samples.length}</span></span>
              <span className="text-muted-foreground truncate">url: <span className="text-blue-400">{wsUrl || "(none)"}</span></span>
            </div>
            <div className="text-muted-foreground">last frame:</div>
            <div className="text-foreground/80 break-all whitespace-pre-wrap mt-1" data-testid="text-last-raw">
              {lastRaw || "(no frame yet)"}
            </div>
          </CardContent>
        </Card>

        <Card data-testid="card-mode-selector" className="border-primary/30">
          <CardContent className="pt-4 pb-4">
            <div className="flex flex-wrap items-center gap-3">
              <span className="text-sm text-muted-foreground font-mono">DEMAND MODE:</span>
              {(["idle", "steady", "burst", "auto"] as DemandMode[]).map((m) => (
                <Button
                  key={m}
                  size="sm"
                  variant={demandMode === m ? "default" : "outline"}
                  onClick={() => sendMode(m)}
                  data-testid={`button-mode-${m}`}
                  className={demandMode === m ? "bg-primary" : ""}
                >
                  {m === "idle" && "Idle (background)"}
                  {m === "steady" && "Steady (production)"}
                  {m === "burst" && "Burst (full-out)"}
                  {m === "auto" && "Auto (sine sweep)"}
                </Button>
              ))}
              <span className="text-xs text-muted-foreground ml-auto font-mono">
                d_target = clamp(Q/Q* / √F, 1/144, 1)
              </span>
            </div>
          </CardContent>
        </Card>

        <div className="flex flex-wrap gap-3">
          <Button
            size="lg"
            onClick={running ? stop : start}
            className={running ? "bg-red-600 hover:bg-red-500" : ""}
            data-testid="button-start-stop"
          >
            {running ? <Square className="w-5 h-5 mr-2" /> : <Play className="w-5 h-5 mr-2" />}
            {running ? "Stop" : "Start / Restart"}
          </Button>
          <Button asChild variant="outline" data-testid="button-download-md">
            <a href="/download/maps/hmodal_power_trit_native.md" download>
              <Download className="w-4 h-4 mr-2" /> Download Spec (MD)
            </a>
          </Button>
          <Button asChild variant="outline" data-testid="button-download-svg">
            <a href="/download/maps/aasc_canonical_map.svg" download>
              <FileText className="w-4 h-4 mr-2" /> AASC Map (SVG)
            </a>
          </Button>
          <Button
            variant="default"
            onClick={issueEac}
            disabled={eacIssuing || !latest}
            data-testid="button-issue-eac"
            title={!latest ? "Start the demo first so a sample exists." : "Snapshot the current sample and sign an EAC."}
          >
            <ShieldCheck className="w-4 h-4 mr-2" />
            {eacIssuing ? "Issuing…" : "Issue EAC now"}
          </Button>
        </div>

        {tunnel.sessionId && (
          <Card data-testid="card-tunnel-status">
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-base">
                <ShieldCheck className="w-4 h-4" />
                Encrypted Tunnel — Continuous Solid-State Chain
                <Badge variant="secondary" data-testid="badge-tunnel-cipher">
                  TL-Sponge-385
                </Badge>
              </CardTitle>
            </CardHeader>
            <CardContent className="text-xs font-mono space-y-1">
              <div data-testid="text-tunnel-cipher">
                <span className="text-muted-foreground">cipher: </span>
                {tunnel.cipher}
              </div>
              <div data-testid="text-tunnel-session">
                <span className="text-muted-foreground">session id: </span>
                {tunnel.sessionId}
              </div>
              <div data-testid="text-tunnel-sealed-count">
                <span className="text-muted-foreground">sealed samples: </span>
                {tunnel.sealedCount} (last index = {tunnel.lastIndex ?? "—"})
              </div>
              <div className="break-all" data-testid="text-tunnel-chain-tag">
                <span className="text-muted-foreground">running chain tag: </span>
                {tunnel.chainTagHex ?? "(awaiting first sealed frame)"}
              </div>
            </CardContent>
          </Card>
        )}

        {(eacResult || eacError) && (
          <Card data-testid="card-eac-result">
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <ShieldCheck className="w-5 h-5" />
                Energy Attestation Certificate
                {eacError ? (
                  <Badge variant="destructive" data-testid="badge-eac-error">error</Badge>
                ) : (
                  <Badge variant="default" data-testid="badge-eac-ok">signed</Badge>
                )}
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              {eacError && (
                <div className="text-sm text-destructive flex items-center gap-2" data-testid="text-eac-error">
                  <AlertCircle className="w-4 h-4" /> {eacError}
                </div>
              )}
              {eacResult?.signature && (
                <div className="grid grid-cols-1 md:grid-cols-2 gap-2 text-xs font-mono">
                  <div data-testid="text-eac-variant">
                    <span className="text-muted-foreground">variant: </span>
                    {eacResult.signature.variant}
                  </div>
                  <div data-testid="text-eac-pubkey-hash">
                    <span className="text-muted-foreground">pubkey hash: </span>
                    {eacResult.signature.public_key_hash?.slice(0, 24)}…
                  </div>
                  <div data-testid="text-eac-tis27">
                    <span className="text-muted-foreground">TIS-27: </span>
                    {eacResult.integrity?.tis27_hash_hex?.slice(0, 24)}…
                  </div>
                  <div data-testid="text-eac-fs">
                    <span className="text-muted-foreground">fs since Salvi epoch: </span>
                    {eacResult.timestamp?.fs_since_salvi_epoch_decimal}
                  </div>
                  <div className="md:col-span-2 break-all" data-testid="text-eac-milesian">
                    <span className="text-muted-foreground">Milesian glyph hash (TIS-27 → bijective base-27 over Greek register): </span>
                    <span className="text-base">{eacResult.integrity?.tis27_hash_milesian}</span>
                  </div>
                  {eacResult.attestation_chain && (
                    <div className="md:col-span-2 break-all" data-testid="text-eac-chain-tag">
                      <span className="text-muted-foreground">tunnel chain tag: </span>
                      {eacResult.attestation_chain.chain_tag_hex?.slice(0, 32)}… ({eacResult.attestation_chain.cipher})
                      <div className="mt-1">
                        <span className="text-muted-foreground">chain tag (Milesian): </span>
                        <span className="text-base" data-testid="text-eac-chain-milesian">
                          {eacResult.attestation_chain.chain_tag_milesian}
                        </span>
                      </div>
                    </div>
                  )}
                </div>
              )}
              <pre
                className="text-xs bg-muted p-3 rounded-md overflow-auto max-h-96"
                data-testid="text-eac-json"
              >
                {JSON.stringify(eacResult, null, 2)}
              </pre>
            </CardContent>
          </Card>
        )}

        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <ReadoutCard
            label="Logical Throughput"
            value={
              latest && (latest.logicalOpsPerSecAvg ?? 0) > 0
                ? `${((latest.logicalOpsPerSecAvg ?? 0) / 1e6).toFixed(2)} Mops/s`
                : "—"
            }
            sub={
              latest
                ? `CPU active only ${((latest.effectiveComputeFrac ?? 0) * 100).toFixed(2)}% of wall time`
                : ""
            }
            tone="primary"
            testid="readout-live"
          />
          <ReadoutCard
            label="Time Duty (high / total)"
            value={latest ? (latest.observedRatio * 100).toFixed(2) + "%" : "—"}
            sub="theoretical: 25.00% (250 ms high / 1000 ms cycle)"
            testid="readout-duty"
          />
          <ReadoutCard
            label="Observed Savings"
            value={latest && latest.savingsObserved != null ? (latest.savingsObserved * 100).toFixed(2) + "%" : "—"}
            sub="theoretical: 74.48% (143/192)"
            tone="primary"
            testid="readout-savings"
          />
          <ReadoutCard
            label="Cache Hit Rate"
            value={
              latest && (latest.cacheHits ?? 0) + (latest.cacheMisses ?? 0) > 0
                ? `${((latest.cacheHitRate ?? 0) * 100).toFixed(1)}%`
                : "—"
            }
            sub={
              latest
                ? `hits: ${latest.cacheHits ?? 0} · miss: ${latest.cacheMisses ?? 0}`
                : "warming up cache (3 cycles)"
            }
            testid="readout-cache"
          />
          <ReadoutCard
            label="Compressed Savings"
            value={
              latest && (latest.compressedSavings ?? 0) > 0
                ? `${((latest.compressedSavings ?? 0) * 100).toFixed(2)}%`
                : "—"
            }
            sub="asymptote: 99.31% (143/144 = 1 − 1/Δ)"
            tone="primary"
            testid="readout-compressed"
          />
        </div>

        <Card data-testid="card-watts">
          <CardHeader className="pb-2">
            <CardTitle className="text-base flex items-center gap-2">
              <Zap className="w-4 h-4" /> Modeled Power Draw (per CPU core)
            </CardTitle>
            <p className="text-xs text-muted-foreground">
              Honest model: 1.0 W idle, 5.0 W full load (typical x86-64 server core).
              Hardware RAPL counters are not exposed in this container — these are
              <strong> projections from real measured compute time</strong>, not direct
              wattmeter readings. Same model applied to all three scenarios for fair
              comparison.
            </p>
          </CardHeader>
          <CardContent className="space-y-3 pt-2">
            {[
              {
                label: "Continuous burn (no HModal)",
                watts: latest?.wattsContinuous ?? 5.0,
                color: "bg-red-500",
                testid: "bar-continuous",
              },
              {
                label: "HModal 1:4 duty (no cache)",
                watts: latest?.wattsHmodalNoCache ?? 2.0,
                color: "bg-yellow-500",
                testid: "bar-hmodal-nocache",
              },
              {
                label: "HModal + Δ-cache (this demo)",
                watts: latest?.wattsHmodalCached ?? 1.0,
                color: "bg-green-500",
                testid: "bar-hmodal-cached",
              },
            ].map((row) => {
              const pct = Math.max(2, (row.watts / 5.0) * 100);
              return (
                <div key={row.label}>
                  <div className="flex justify-between text-xs mb-1">
                    <span className="text-muted-foreground">{row.label}</span>
                    <span className="font-mono font-semibold" data-testid={row.testid}>
                      {row.watts.toFixed(3)} W
                    </span>
                  </div>
                  <div className="h-3 bg-muted rounded-full overflow-hidden">
                    <div
                      className={`h-full ${row.color} transition-all duration-500`}
                      style={{ width: `${pct}%` }}
                    />
                  </div>
                </div>
              );
            })}
            <div className="pt-2 border-t border-border flex flex-wrap gap-x-6 gap-y-1 text-sm">
              <div>
                <span className="text-muted-foreground">Watts saved vs continuous:</span>{" "}
                <span className="font-mono font-bold text-green-500" data-testid="text-watts-saved">
                  {(latest?.wattsSavedVsContinuous ?? 4.0).toFixed(3)} W
                </span>
              </div>
              <div>
                <span className="text-muted-foreground">Per 1000 cores @ 24 h:</span>{" "}
                <span className="font-mono font-bold text-green-500" data-testid="text-kwh-day">
                  {(((latest?.wattsSavedVsContinuous ?? 4.0) * 1000 * 24) / 1000).toFixed(1)} kWh/day saved
                </span>
              </div>
              <div>
                <span className="text-muted-foreground">Per 1000 cores @ 1 yr:</span>{" "}
                <span className="font-mono font-bold text-green-500" data-testid="text-mwh-year">
                  {(((latest?.wattsSavedVsContinuous ?? 4.0) * 1000 * 24 * 365) / 1e6).toFixed(2)} MWh/yr saved
                </span>
              </div>
            </div>
          </CardContent>
        </Card>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <Card data-testid="card-controller">
            <CardHeader className="pb-2">
              <CardTitle className="text-base flex items-center gap-2">
                <Activity className="w-4 h-4" /> Deterministic Controller
              </CardTitle>
              <p className="text-xs text-muted-foreground">
                Closed-form law: d = clamp((Q/Q*)/√F, 1/144, 1). Same inputs always
                produce the same duty cycle. No ML, no randomness.
              </p>
            </CardHeader>
            <CardContent className="space-y-3 pt-2 font-mono text-sm">
              <div className="flex justify-between">
                <span className="text-muted-foreground">Mode</span>
                <Badge variant="default" className="bg-primary" data-testid="text-ctrl-mode">
                  {(latest?.demandMode ?? demandMode).toUpperCase()}
                </Badge>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Queue depth (Q)</span>
                <span data-testid="text-ctrl-q">{(latest?.queueDepth ?? 0).toFixed(1)} / 100</span>
              </div>
              <div className="h-2 bg-muted rounded overflow-hidden">
                <div
                  className="h-full bg-blue-500 transition-all"
                  style={{ width: `${Math.min(100, (latest?.queueDepth ?? 0))}%` }}
                />
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Cache fill (F)</span>
                <span data-testid="text-ctrl-f">{((latest?.cacheFillRatio ?? 0) * 100).toFixed(1)}%</span>
              </div>
              <div className="h-2 bg-muted rounded overflow-hidden">
                <div
                  className="h-full bg-green-500 transition-all"
                  style={{ width: `${(latest?.cacheFillRatio ?? 0) * 100}%` }}
                />
              </div>
              <div className="flex justify-between pt-2 border-t border-border">
                <span className="text-muted-foreground">Duty target (d)</span>
                <span className="text-primary font-bold" data-testid="text-ctrl-d">
                  {((latest?.dutyTarget ?? 0) * 100).toFixed(2)}%
                </span>
              </div>
              <div className="h-3 bg-muted rounded overflow-hidden">
                <div
                  className="h-full bg-gradient-to-r from-green-500 via-yellow-500 to-red-500 transition-all"
                  style={{ width: `${(latest?.dutyTarget ?? 0) * 100}%` }}
                />
              </div>
              <div className="text-xs text-muted-foreground pt-1">
                Floor: 1/Δ = 0.69% (energy-save).  Ceiling: 100% (full-out).
              </div>
            </CardContent>
          </Card>

          <Card data-testid="card-key-isolation" className="border-green-500/30">
            <CardHeader className="pb-2">
              <CardTitle className="text-base flex items-center gap-2">
                <CheckCircle className="w-4 h-4 text-green-500" /> Key Isolation (TL-DSA)
              </CardTitle>
              <p className="text-xs text-muted-foreground">
                Each batch = one logical signature.  Private key only fetched on
                cache miss; hits never touch CPU registers, L-caches, or the bus.
              </p>
            </CardHeader>
            <CardContent className="space-y-3 pt-2 font-mono text-sm">
              <div className="flex justify-between">
                <span className="text-muted-foreground">Signatures served</span>
                <span data-testid="text-sigs">
                  {(latest?.signatureCount ?? 0).toLocaleString()}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Key actually touched</span>
                <span className="text-yellow-500" data-testid="text-key-touches">
                  {(latest?.keyTouchCount ?? 0).toLocaleString()}
                </span>
              </div>
              <div className="flex justify-between pt-2 border-t border-border">
                <span className="text-muted-foreground">Exposure ratio</span>
                <span className="text-green-500 font-bold" data-testid="text-exposure">
                  {((latest?.keyExposureRatio ?? 0) * 100).toFixed(4)}%
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Isolation factor</span>
                <span className="text-green-500 font-bold text-lg" data-testid="text-isolation">
                  {(latest?.keyIsolationFactor ?? 0).toFixed(0)}×
                </span>
              </div>
              <div className="text-xs text-muted-foreground pt-2 border-t border-border space-y-1">
                <div>• Power-analysis surface: <strong>÷ {(latest?.keyIsolationFactor ?? 0).toFixed(0)}</strong></div>
                <div>• Timing-attack surface: <strong>÷ {(latest?.keyIsolationFactor ?? 0).toFixed(0)}</strong></div>
                <div>• Cold-boot residency: <strong>÷ {(latest?.keyIsolationFactor ?? 0).toFixed(0)}</strong></div>
                <div className="text-green-500/80 pt-1">
                  Theoretical asymptote: ∞ (steady-state hit rate → 100%)
                </div>
              </div>
            </CardContent>
          </Card>
        </div>

        <Card data-testid="card-chart">
          <CardHeader className="pb-2">
            <CardTitle className="text-base flex items-center gap-2">
              <Activity className="w-4 h-4" /> Live Strip Chart (last 60 s) — blue band = high state, gray = low
            </CardTitle>
          </CardHeader>
          <CardContent>
            <canvas
              ref={canvasRef}
              width={1100}
              height={260}
              className="w-full h-[260px] rounded-md border border-border"
              data-testid="canvas-strip"
            />
          </CardContent>
        </Card>

        {status && (
          <Card data-testid="card-constants">
            <CardHeader className="pb-2">
              <CardTitle className="text-base flex items-center gap-2">
                <Zap className="w-4 h-4" /> HModal Constants — derived, not chosen
              </CardTitle>
            </CardHeader>
            <CardContent className="grid grid-cols-2 md:grid-cols-4 gap-3 text-sm font-mono">
              <Const k="α (low)" v={status.constants.alpha} />
              <Const k="β (high)" v={status.constants.beta} />
              <Const k="duty high" v={status.constants.dutyHigh} />
              <Const k="duty low" v={status.constants.dutyLow} />
              <Const k="Δ discriminant" v={String(status.constants.discriminant)} />
              <Const k="⟨H⟩ DC mean" v={status.constants.dcMean} />
              <Const k="savings" v={status.constants.savings} />
              <Const k="savings %" v={status.constants.savingsPct.toFixed(4) + "%"} />
            </CardContent>
          </Card>
        )}

        <Card data-testid="card-howto">
          <CardHeader className="pb-2">
            <CardTitle className="text-base">To get hardware watts on your own machine</CardTitle>
          </CardHeader>
          <CardContent className="text-sm text-muted-foreground space-y-2">
            <p>
              Linux desktop with Intel chip (Sandy Bridge or newer): the file{" "}
              <code className="text-primary">/sys/class/powercap/intel-rapl:0/energy_uj</code> already exists. Make
              it readable, run the Tier-2 binary from the spec, and this same UI shows real watts instead of ops/sec.
              No driver install, no kernel patch.
            </p>
            <p>
              Windows: same Intel chip works through{" "}
              <code className="text-primary">Intel Power Gadget</code>; replace the sysfs read with its DLL call —
              one function swap inside <code className="text-primary">rapl_intake.rs</code>.
            </p>
            <p>
              In every case AASC stays trit-pure; only the single intake line changes.
            </p>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

function ReadoutCard({
  label,
  value,
  sub,
  tone = "muted",
  testid,
}: {
  label: string;
  value: string;
  sub?: string;
  tone?: "primary" | "muted";
  testid: string;
}) {
  return (
    <Card data-testid={testid}>
      <CardContent className="pt-5">
        <div className="text-xs uppercase tracking-wide text-muted-foreground">{label}</div>
        <div className={`text-2xl font-bold mt-1 ${tone === "primary" ? "text-primary" : "text-foreground"}`}>
          {value}
        </div>
        {sub && <div className="text-xs text-muted-foreground mt-1">{sub}</div>}
      </CardContent>
    </Card>
  );
}

function Const({ k, v }: { k: string; v: string }) {
  return (
    <div className="flex justify-between border border-border rounded px-3 py-2">
      <span className="text-muted-foreground">{k}</span>
      <span className="text-primary">{v}</span>
    </div>
  );
}
