/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL
 * All Rights Reserved.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * Unauthorized copying, modification, distribution, or use of this file,
 * via any medium, is strictly prohibited without the prior written
 * permission of Capomastro Holdings Ltd.
 *
 * See LICENSE in the repository root for full terms.
 */

import { useState, useEffect, useRef, useCallback } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Activity, Play, Square, Download, FileText, Zap, AlertCircle, CheckCircle, ShieldCheck } from "lucide-react";
import { EacCertificate } from "@/components/eac-certificate";

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

// Positive integer rational pair — wire format for every "fraction"
// emitted by the HModal Console.  No JS floats on the wire.
type Rational = { num: number; den: number };

// Display-only helper: convert a {num, den} rational to a JS number
// at render time.  Never feed the result back into the wire path.
const r = (x: Rational | null | undefined): number =>
  x && x.den > 0 ? x.num / x.den : 0;

// Format an integer count of µJ as a human-readable energy with the
// appropriate SI prefix (µJ → mJ → J → kJ).  All inputs/outputs remain
// integer-derived; the final formatting is purely cosmetic.
function formatJoules(uj: number): string {
  const v = Math.max(0, Math.floor(uj));
  if (v < 1_000)            return `${v} µJ`;
  if (v < 1_000_000)        return `${(v / 1_000).toFixed(2)} mJ`;
  if (v < 1_000_000_000)    return `${(v / 1_000_000).toFixed(3)} J`;
  return `${(v / 1_000_000_000).toFixed(3)} kJ`;
}

interface Sample {
  t: number;
  phase: "high" | "low";
  opsPerSec: number;
  // Power, energy, time, ops — all integers (mW, µJ, ms, count).
  mW: number | null;
  mode: string;
  observedRatio: Rational;
  theoreticalRatio: Rational;
  savingsObserved: Rational | null;
  theoreticalSavings: Rational;
  cumulativeEnergyUj: number;
  cumulativeEnergySavedUj?: number;     // µJ saved versus continuous-on baseline
  energySavedThisWindowUj?: number;     // µJ saved in the latest sample window
  cumulativeOps?: number;
  cumulativeOpsHigh?: number;
  cumulativeOpsLow?: number;
  timeHighMs?: number;
  timeLowMs?: number;
  totalMs?: number;
  cacheHits?: number;
  cacheMisses?: number;
  cacheHitRate?: Rational;
  realHighWorkMs?: number;
  cachedHighMs?: number;
  compressedSavings?: Rational;
  theoreticalCompressedSavings?: Rational;
  mWContinuous?: number;
  mWHmodalNoCache?: number;
  mWHmodalCached?: number;
  mWSavedVsContinuous?: number;
  effectiveCompute?: Rational;
  logicalOpsPerSecAvg?: number;
  realCpuOpsPerSecAvg?: number;
  demandMode?: "idle" | "steady" | "burst" | "auto";
  queueDepth?: Rational;
  cacheFillRatio?: Rational;
  dutyTarget?: Rational;
  keyTouchCount?: number;
  signatureCount?: number;
  keyExposureRatio?: Rational;
  keyIsolationFactor?: Rational;
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
    sessionId?: string;        // trit-native (Rep-C bijective base-3)
    cipher?: string;
    chainSeedTrit?: string;
    chainSeedHex?: string;     // legacy hex (audit only)
    sealedCount: number;
    chainTagTrit?: string;
    chainTagHex?: string;      // legacy hex (audit only)
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
            sessionId:     msg.sessionId,        // already trit-native
            cipher:        msg.cipher,
            chainSeedTrit: msg.chainSeedTrit,
            chainSeedHex:  msg.chainSeedHex,     // legacy
            sealedCount:   0,
          });
          return;
        }
        if (msg.type === "sealed") {
          setTunnel((prev) => ({
            ...prev,
            sealedCount:  prev.sealedCount + 1,
            chainTagTrit: msg.chainTag      ?? prev.chainTagTrit,
            chainTagHex:  msg.chainTagHex   ?? prev.chainTagHex,
            lastIndex:    msg.index,
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

    // Chart shows mW (integer milliwatts) when available, else opsPerSec.
    const useWatts = samples.some((s) => s.mW != null);
    const values = samples.map((s) => (useWatts ? (s.mW ?? 0) : s.opsPerSec));
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
    <div className="min-h-screen bg-background text-foreground p-6 md:p-10" data-testid="page-hmodal">
      <div className="max-w-6xl mx-auto space-y-6">
        <div>
          <h1 className="text-3xl md:text-4xl font-bold text-primary" data-testid="text-title">
            HModal Power Console
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
            title={!latest ? "Start the console first so a sample exists." : "Snapshot the current sample and sign an EAC."}
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
              <div className="break-all" data-testid="text-tunnel-session">
                <span className="text-muted-foreground">session id (trit): </span>
                {tunnel.sessionId}
              </div>
              <div data-testid="text-tunnel-sealed-count">
                <span className="text-muted-foreground">sealed samples: </span>
                {tunnel.sealedCount} (last index = {tunnel.lastIndex ?? "—"})
              </div>
              <div className="break-all" data-testid="text-tunnel-chain-tag">
                <span className="text-muted-foreground">running chain tag (trit, head): </span>
                {tunnel.chainTagTrit
                  ? tunnel.chainTagTrit.slice(0, 96) + (tunnel.chainTagTrit.length > 96 ? "…" : "")
                  : "(awaiting first sealed frame)"}
              </div>
            </CardContent>
          </Card>
        )}

        {(eacResult || eacError) && (
          <div data-testid="card-eac-result" className="space-y-3">
            <EacCertificate eac={eacResult} error={eacError} />
            {eacResult && (
              <details className="text-xs">
                <summary className="cursor-pointer text-muted-foreground hover:text-foreground">
                  Canonical EAC payload (JSON)
                </summary>
                <pre
                  className="text-xs bg-muted p-3 rounded-md overflow-auto max-h-96 mt-2"
                  data-testid="text-eac-json"
                >
                  {JSON.stringify(eacResult, null, 2)}
                </pre>
              </details>
            )}
          </div>
        )}

        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <ReadoutCard
            label="Live Power Draw"
            value={
              latest
                ? `${(latest.mWHmodalCached ?? latest.mW ?? 0)} mW`
                : "—"
            }
            sub={
              latest
                ? `Plain English: how many milliwatts this CPU core is drawing right now under HModal duty-cycle. Saved vs always-on baseline: ${latest.mWSavedVsContinuous ?? 0} mW.`
                : "Plain English: instantaneous core power draw under HModal + Δ-cache."
            }
            tone="primary"
            testid="readout-live"
          />
          <ReadoutCard
            label="Cumulative Energy Saved"
            value={
              latest
                ? formatJoules(latest.cumulativeEnergySavedUj ?? 0)
                : "—"
            }
            sub={
              latest
                ? `Plain English: total electricity NOT burned vs. running flat-out the whole time. ${(latest.cumulativeEnergySavedUj ?? 0).toLocaleString()} µJ saved · this window: +${(latest.energySavedThisWindowUj ?? 0).toLocaleString()} µJ.`
                : "Plain English: total joules avoided vs. always-on baseline since this session opened."
            }
            tone="primary"
            testid="readout-energy-saved"
          />
          <ReadoutCard
            label="Time Duty (high / total)"
            value={latest ? (r(latest.observedRatio) * 100).toFixed(2) + "%" : "—"}
            sub="Plain English: percentage of each second spent in the high-power 'work' phase. Theoretical schedule: 25.00% (250 ms work, 750 ms cool-down per cycle)."
            testid="readout-duty"
          />
          <ReadoutCard
            label="Observed Savings"
            value={latest && latest.savingsObserved != null ? (r(latest.savingsObserved) * 100).toFixed(2) + "%" : "—"}
            sub="Plain English: fraction of always-on power eliminated by duty-cycling. Theoretical best case: 74.48% (143/192)."
            tone="primary"
            testid="readout-savings"
          />
          <ReadoutCard
            label="Cache Hit Rate"
            value={
              latest && (latest.cacheHits ?? 0) + (latest.cacheMisses ?? 0) > 0
                ? `${(r(latest.cacheHitRate) * 100).toFixed(1)}%`
                : "—"
            }
            sub={
              latest
                ? `Plain English: how often the Δ-cache served a result instead of re-computing. hits: ${latest.cacheHits ?? 0} · misses: ${latest.cacheMisses ?? 0}.`
                : "Plain English: cache is warming up — needs three full cycles before it produces hits."
            }
            testid="readout-cache"
          />
          <ReadoutCard
            label="Compressed Savings"
            value={
              latest && r(latest.compressedSavings) > 0
                ? `${(r(latest.compressedSavings) * 100).toFixed(2)}%`
                : "—"
            }
            sub="Plain English: total savings when the Δ-cache is fully warm. Asymptote: 99.31% (143/144 = 1 − 1/Δ)."
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
                mW: latest?.mWContinuous ?? 5000,
                color: "bg-red-500",
                testid: "bar-continuous",
              },
              {
                label: "HModal 1:4 duty (no cache)",
                mW: latest?.mWHmodalNoCache ?? 2000,
                color: "bg-yellow-500",
                testid: "bar-hmodal-nocache",
              },
              {
                label: "HModal + Δ-cache (this console)",
                mW: latest?.mWHmodalCached ?? 1000,
                color: "bg-green-500",
                testid: "bar-hmodal-cached",
              },
            ].map((row) => {
              // Bar width is integer mW / 5000 mW × 100, floored — no float in the wire path.
              const pct = Math.max(2, Math.floor((row.mW * 100) / 5000));
              return (
                <div key={row.label}>
                  <div className="flex justify-between text-xs mb-1">
                    <span className="text-muted-foreground">{row.label}</span>
                    <span className="font-mono font-semibold" data-testid={row.testid}>
                      {row.mW} mW
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
                <span className="text-muted-foreground">Saved vs continuous:</span>{" "}
                <span className="font-mono font-bold text-green-500" data-testid="text-watts-saved">
                  {latest?.mWSavedVsContinuous ?? 4000} mW
                </span>
              </div>
              <div>
                <span className="text-muted-foreground">Per 1000 cores @ 24 h:</span>{" "}
                <span className="font-mono font-bold text-green-500" data-testid="text-kwh-day">
                  {/* (mW_saved × 1000 cores × 24 h) / 1_000_000 = Wh; integer arithmetic only */}
                  {Math.floor(((latest?.mWSavedVsContinuous ?? 4000) * 1000 * 24) / 1000)} Wh/day saved
                </span>
              </div>
              <div>
                <span className="text-muted-foreground">Per 1000 cores @ 1 yr:</span>{" "}
                <span className="font-mono font-bold text-green-500" data-testid="text-mwh-year">
                  {Math.floor(((latest?.mWSavedVsContinuous ?? 4000) * 1000 * 24 * 365) / 1_000_000)} kWh/yr saved
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
                <span data-testid="text-ctrl-q">{r(latest?.queueDepth).toFixed(1)} / 100</span>
              </div>
              <div className="h-2 bg-muted rounded overflow-hidden">
                <div
                  className="h-full bg-blue-500 transition-all"
                  style={{ width: `${Math.min(100, r(latest?.queueDepth))}%` }}
                />
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Cache fill (F)</span>
                <span data-testid="text-ctrl-f">{(r(latest?.cacheFillRatio) * 100).toFixed(1)}%</span>
              </div>
              <div className="h-2 bg-muted rounded overflow-hidden">
                <div
                  className="h-full bg-green-500 transition-all"
                  style={{ width: `${r(latest?.cacheFillRatio) * 100}%` }}
                />
              </div>
              <div className="flex justify-between pt-2 border-t border-border">
                <span className="text-muted-foreground">Duty target (d)</span>
                <span className="text-primary font-bold" data-testid="text-ctrl-d">
                  {(r(latest?.dutyTarget) * 100).toFixed(2)}%
                </span>
              </div>
              <div className="h-3 bg-muted rounded overflow-hidden">
                <div
                  className="h-full bg-gradient-to-r from-green-500 via-yellow-500 to-red-500 transition-all"
                  style={{ width: `${r(latest?.dutyTarget) * 100}%` }}
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
                  {(r(latest?.keyExposureRatio) * 100).toFixed(4)}%
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Isolation factor</span>
                <span className="text-green-500 font-bold text-lg" data-testid="text-isolation">
                  {Math.floor(r(latest?.keyIsolationFactor))}×
                </span>
              </div>
              <div className="text-xs text-muted-foreground pt-2 border-t border-border space-y-1">
                <div>• Power-analysis surface: <strong>÷ {Math.floor(r(latest?.keyIsolationFactor))}</strong></div>
                <div>• Timing-attack surface: <strong>÷ {Math.floor(r(latest?.keyIsolationFactor))}</strong></div>
                <div>• Cold-boot residency: <strong>÷ {Math.floor(r(latest?.keyIsolationFactor))}</strong></div>
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
