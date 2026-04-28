import { useState, useEffect, useRef, useCallback } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Activity, Play, Square, Download, FileText, Zap, AlertCircle, CheckCircle } from "lucide-react";

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
}

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
  const wsRef = useRef<WebSocket | null>(null);
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

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
        </div>

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
