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
import { useQuery } from "@tanstack/react-query";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Clock,
  Activity,
  Zap,
  Globe,
  Copy,
  Check,
  RefreshCw,
  Radio,
  Timer,
  Shield,
  Server,
} from "lucide-react";

interface TimestampData {
  femtoseconds: string;
  humanReadable: string;
  isoDate: string;
  precision: string;
  salviEpochOffset: string;
}

interface HptpData {
  t2_server_receive_ms: number;
  t3_server_send_ms: number;
  server_processing_us: number;
  protocol: string;
  correction_model: string;
}

interface TimestampResponse {
  success: boolean;
  timestamp: TimestampData;
  epoch?: {
    salviEpoch: string;
    description: string;
  };
  hptp?: HptpData;
}

interface TimingMetrics {
  success: boolean;
  timestamp: TimestampData;
  clockSource: string;
  synchronizationStatus: string;
  estimatedAccuracy: string;
}

interface CalendarMapping {
  calendarSystem: string;
  origin: string;
  salviEpochEquivalent: string;
  yearInCalendar: number;
  description: string;
}

interface CalendarsResponse {
  success: boolean;
  salviEpoch: string;
  calendars: Record<string, any>;
  allMappings: CalendarMapping[];
}

function formatFemtoseconds(fs: string): string {
  if (!fs) return "";
  const padded = fs.padStart(15, "0");
  const len = padded.length;
  const seconds = padded.slice(0, Math.max(0, len - 15)) || "0";
  const millis = padded.slice(Math.max(0, len - 15), len - 12);
  const micros = padded.slice(len - 12, len - 9);
  const nanos = padded.slice(len - 9, len - 6);
  const picos = padded.slice(len - 6, len - 3);
  const femtos = padded.slice(len - 3);
  return `${seconds}.${millis}.${micros}.${nanos}.${picos}.${femtos}`;
}

interface LatencyCorrection {
  roundTripMs: number;
  networkDelayMs: number;
  serverProcessingUs: number;
  clockOffsetMs: number;
  correctedTimestamp: string;
  correctedFemtoseconds: string;
  protocol: string;
}

function computeHptpCorrection(
  t1: number,
  t4: number,
  hptp: HptpData,
  rawTimestamp: TimestampData
): LatencyCorrection {
  const t2 = hptp.t2_server_receive_ms;
  const t3 = hptp.t3_server_send_ms;

  const roundTripMs = (t4 - t1) - (t3 - t2);
  const networkDelayMs = roundTripMs / 2;
  const clockOffsetMs = ((t2 - t1) + (t3 - t4)) / 2;

  const rawFs = BigInt(rawTimestamp.femtoseconds);

  const elapsedSinceGenMs = (t4 - t1) / 2;
  const currentServerTimeFs = rawFs + BigInt(Math.round(elapsedSinceGenMs * 1e12));

  return {
    roundTripMs: Math.round(roundTripMs * 100) / 100,
    networkDelayMs: Math.round(networkDelayMs * 100) / 100,
    serverProcessingUs: hptp.server_processing_us,
    clockOffsetMs: Math.round(clockOffsetMs * 100) / 100,
    correctedTimestamp: formatCorrectedHumanReadable(currentServerTimeFs),
    correctedFemtoseconds: currentServerTimeFs.toString(),
    protocol: hptp.protocol,
  };
}

function formatCorrectedHumanReadable(fs: bigint): string {
  const SALVI_EPOCH = new Date("2025-04-01T00:00:00.000Z").getTime();
  const msFromFs = Number(fs / 1_000_000_000_000n);
  const date = new Date(SALVI_EPOCH + msFromFs);
  const remainingFs = fs % 1_000_000_000_000n;
  const ns = String(remainingFs / 1_000_000n).padStart(3, "0");
  const ps = String((remainingFs % 1_000_000n) / 1_000n).padStart(3, "0");
  const fsStr = String(remainingFs % 1_000n).padStart(3, "0");
  const y = date.getUTCFullYear();
  const mo = String(date.getUTCMonth() + 1).padStart(2, "0");
  const d = String(date.getUTCDate()).padStart(2, "0");
  const h = String(date.getUTCHours()).padStart(2, "0");
  const mi = String(date.getUTCMinutes()).padStart(2, "0");
  const s = String(date.getUTCSeconds()).padStart(2, "0");
  const ms = String(date.getUTCMilliseconds()).padStart(3, "0");
  return `${y}-${mo}-${d} ${h}:${mi}:${s}.${ms}.${ns}.${ps}.${fsStr} UTC`;
}

function LiveTimestamp() {
  const [ts, setTs] = useState<TimestampData | null>(null);
  const [isStreaming, setIsStreaming] = useState(false);
  const intervalRef = useRef<NodeJS.Timeout | null>(null);
  const [requestCount, setRequestCount] = useState(0);
  const [latency, setLatency] = useState(0);
  const [correction, setCorrection] = useState<LatencyCorrection | null>(null);

  const fetchTimestamp = useCallback(async () => {
    const t1 = Date.now();
    const startPerf = performance.now();
    try {
      const res = await fetch("/api/salvi/timing/timestamp");
      const data: TimestampResponse = await res.json();
      const t4 = Date.now();
      setLatency(Math.round(performance.now() - startPerf));
      if (data.success) {
        setTs(data.timestamp);
        setRequestCount((c) => c + 1);
        if (data.hptp) {
          setCorrection(computeHptpCorrection(t1, t4, data.hptp, data.timestamp));
        }
      }
    } catch {
      /* silently retry */
    }
  }, []);

  useEffect(() => {
    fetchTimestamp();
  }, [fetchTimestamp]);

  const toggleStream = useCallback(() => {
    if (isStreaming) {
      if (intervalRef.current) clearInterval(intervalRef.current);
      intervalRef.current = null;
      setIsStreaming(false);
    } else {
      setIsStreaming(true);
      intervalRef.current = setInterval(fetchTimestamp, 200);
    }
  }, [isStreaming, fetchTimestamp]);

  useEffect(() => {
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, []);

  const [copied, setCopied] = useState(false);
  const copyToClipboard = useCallback(() => {
    if (ts) {
      navigator.clipboard.writeText(JSON.stringify(ts, null, 2));
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  }, [ts]);

  return (
    <Card data-testid="card-live-timestamp">
      <CardHeader className="flex flex-row items-center justify-between gap-2 pb-3">
        <CardTitle className="text-base flex items-center gap-2">
          <Clock className="w-4 h-4 text-primary" />
          Live Femtosecond Timestamp
        </CardTitle>
        <div className="flex items-center gap-2">
          <Badge variant={isStreaming ? "default" : "secondary"} data-testid="badge-stream-status">
            {isStreaming ? "STREAMING" : "PAUSED"}
          </Badge>
          <Button size="icon" variant="ghost" onClick={copyToClipboard} data-testid="button-copy-timestamp">
            {copied ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
          </Button>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        <div>
          <div className="text-xs text-muted-foreground mb-1">
            {correction ? "Current Server Time (HPTP-corrected)" : "Server Timestamp (uncorrected)"}
          </div>
          <div className="font-mono text-2xl tracking-tight text-foreground break-all leading-relaxed" data-testid="text-femtosecond-value">
            {correction
              ? formatFemtoseconds(correction.correctedFemtoseconds)
              : ts
                ? formatFemtoseconds(ts.femtoseconds)
                : "Loading..."}
          </div>
          <div className="text-sm text-muted-foreground" data-testid="text-human-readable">
            {correction ? correction.correctedTimestamp : ts?.humanReadable || ""}
          </div>
        </div>

        {correction && (
          <div className="rounded-md border p-3 space-y-2 bg-muted/30">
            <div className="flex items-center gap-2 text-xs font-medium text-muted-foreground">
              <Shield className="w-3 h-3" />
              HPTP Latency Correction (NTP-Symmetric Model)
            </div>
            <div className="grid grid-cols-2 gap-x-4 gap-y-1 text-xs">
              <div className="text-muted-foreground">Round-trip network delay</div>
              <div className="font-mono" data-testid="text-round-trip">{correction.roundTripMs}ms</div>
              <div className="text-muted-foreground">One-way network delay</div>
              <div className="font-mono" data-testid="text-network-delay">{correction.networkDelayMs}ms</div>
              <div className="text-muted-foreground">Server processing time</div>
              <div className="font-mono" data-testid="text-server-processing">{correction.serverProcessingUs}us</div>
              <div className="text-muted-foreground">Clock offset estimate</div>
              <div className="font-mono" data-testid="text-clock-offset">{correction.clockOffsetMs}ms</div>
              <div className="text-muted-foreground">Time advancement applied</div>
              <div className="font-mono" data-testid="text-correction-applied">+{correction.roundTripMs / 2}ms</div>
            </div>
            <div className="text-xs text-muted-foreground pt-1 border-t">
              The server timestamp is generated at T2 (request receipt). The displayed time is the estimated current server time at the moment of display, computed by adding half the round-trip time to the generation timestamp. Uses {correction.protocol} four-timestamp model (T1/T2/T3/T4) assuming symmetric network paths.
            </div>
          </div>
        )}

        <div className="grid grid-cols-3 gap-3">
          <div className="space-y-1">
            <div className="text-xs text-muted-foreground">Precision</div>
            <div className="text-sm font-medium" data-testid="text-precision">{ts?.precision || "—"}</div>
          </div>
          <div className="space-y-1">
            <div className="text-xs text-muted-foreground">Round-trip Latency</div>
            <div className="text-sm font-medium" data-testid="text-latency">{latency}ms</div>
          </div>
          <div className="space-y-1">
            <div className="text-xs text-muted-foreground">Requests</div>
            <div className="text-sm font-medium" data-testid="text-request-count">{requestCount}</div>
          </div>
        </div>
        <div className="flex gap-2">
          <Button onClick={toggleStream} variant={isStreaming ? "destructive" : "default"} data-testid="button-toggle-stream">
            {isStreaming ? (
              <>
                <Radio className="w-4 h-4 mr-2" />
                Stop Stream
              </>
            ) : (
              <>
                <Activity className="w-4 h-4 mr-2" />
                Start Live Stream
              </>
            )}
          </Button>
          <Button variant="outline" onClick={fetchTimestamp} data-testid="button-single-fetch">
            <RefreshCw className="w-4 h-4 mr-2" />
            Single Fetch
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}

function MetricsPanel() {
  const { data, isLoading } = useQuery<TimingMetrics>({
    queryKey: ["/api/salvi/timing/metrics"],
    refetchInterval: 5000,
  });

  return (
    <Card data-testid="card-metrics">
      <CardHeader className="pb-3">
        <CardTitle className="text-base flex items-center gap-2">
          <Zap className="w-4 h-4 text-primary" />
          HPTP Synchronization Status
        </CardTitle>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <div className="space-y-3">
            {[1, 2, 3].map((i) => (
              <div key={i} className="h-6 bg-muted rounded animate-pulse" />
            ))}
          </div>
        ) : data?.success ? (
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">Clock Source</span>
              <Badge variant="outline" data-testid="badge-clock-source">{data.clockSource}</Badge>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">Sync Status</span>
              <Badge variant="default" data-testid="badge-sync-status">{data.synchronizationStatus}</Badge>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">Accuracy</span>
              <Badge variant="outline" data-testid="badge-accuracy">{data.estimatedAccuracy}</Badge>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">Protocol</span>
              <Badge variant="outline" data-testid="badge-protocol">HPTP v1.0</Badge>
            </div>
          </div>
        ) : (
          <div className="text-sm text-muted-foreground">Unable to fetch metrics</div>
        )}
      </CardContent>
    </Card>
  );
}

function EndpointReference() {
  const endpoints = [
    { method: "GET", path: "/api/salvi/timing/timestamp", desc: "Current femtosecond timestamp" },
    { method: "GET", path: "/api/salvi/timing/metrics", desc: "Clock source & sync status" },
    { method: "GET", path: "/api/salvi/timing/batch/:count", desc: "Batch timestamp generation" },
    { method: "GET", path: "/api/salvi/timing/epoch/anchors", desc: "Salvi Epoch anchor points" },
    { method: "GET", path: "/api/salvi/timing/epoch/calendars", desc: "All 24 calendars (add ?date=YYYY-MM-DD for any date)" },
    { method: "GET", path: "/api/salvi/timing/epoch/calendars/:system", desc: "Individual calendar (add ?date= for any date)" },
  ];

  const [copiedIdx, setCopiedIdx] = useState<number | null>(null);
  const copyEndpoint = (path: string, idx: number) => {
    const baseUrl = window.location.origin;
    navigator.clipboard.writeText(`${baseUrl}${path}`);
    setCopiedIdx(idx);
    setTimeout(() => setCopiedIdx(null), 2000);
  };

  return (
    <Card data-testid="card-endpoints">
      <CardHeader className="pb-3">
        <CardTitle className="text-base flex items-center gap-2">
          <Server className="w-4 h-4 text-primary" />
          Live API Endpoints
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-2">
          {endpoints.map((ep, i) => (
            <div
              key={i}
              className="flex items-center gap-2 p-2 rounded-md bg-muted/50 hover-elevate cursor-pointer"
              onClick={() => copyEndpoint(ep.path, i)}
              data-testid={`row-endpoint-${i}`}
            >
              <Badge variant="outline" className="font-mono text-[10px] shrink-0">{ep.method}</Badge>
              <code className="text-xs font-mono flex-1 truncate">{ep.path}</code>
              <span className="text-xs text-muted-foreground hidden sm:inline shrink-0">{ep.desc}</span>
              {copiedIdx === i ? (
                <Check className="w-3.5 h-3.5 text-green-600 shrink-0" />
              ) : (
                <Copy className="w-3.5 h-3.5 text-muted-foreground shrink-0" />
              )}
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}

function BatchDemo() {
  const [count, setCount] = useState(5);
  const [results, setResults] = useState<any[] | null>(null);
  const [loading, setLoading] = useState(false);
  const [elapsed, setElapsed] = useState(0);

  const runBatch = async () => {
    setLoading(true);
    const start = performance.now();
    try {
      const res = await fetch(`/api/salvi/timing/batch/${count}`);
      const data = await res.json();
      setElapsed(Math.round(performance.now() - start));
      if (data.success) {
        setResults(data.timestamps);
      }
    } catch {
      /* ignore */
    }
    setLoading(false);
  };

  return (
    <Card data-testid="card-batch-demo">
      <CardHeader className="pb-3">
        <CardTitle className="text-base flex items-center gap-2">
          <Timer className="w-4 h-4 text-primary" />
          Batch Timestamp Generator
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="flex items-center gap-2">
          <div className="flex gap-1">
            {[5, 10, 25, 50].map((n) => (
              <Button
                key={n}
                size="sm"
                variant={count === n ? "default" : "outline"}
                onClick={() => setCount(n)}
                data-testid={`button-batch-${n}`}
              >
                {n}
              </Button>
            ))}
          </div>
          <Button onClick={runBatch} disabled={loading} data-testid="button-run-batch">
            {loading ? <RefreshCw className="w-4 h-4 mr-2 animate-spin" /> : <Zap className="w-4 h-4 mr-2" />}
            Generate
          </Button>
        </div>
        {results && (
          <div className="space-y-2">
            <div className="flex items-center gap-2">
              <Badge variant="outline" data-testid="badge-batch-count">{results.length} timestamps</Badge>
              <Badge variant="outline" data-testid="badge-batch-time">{elapsed}ms total</Badge>
              <Badge variant="secondary" data-testid="badge-batch-rate">
                {results.length > 0 ? `${(elapsed / results.length).toFixed(1)}ms/ts` : ""}
              </Badge>
            </div>
            <div className="max-h-48 overflow-y-auto rounded-md bg-muted/50 p-2 space-y-1" data-testid="list-batch-results">
              {results.map((ts: any, i: number) => (
                <div key={i} className="font-mono text-xs text-muted-foreground flex items-baseline gap-2">
                  <span className="text-[10px] w-6 text-right shrink-0">#{i + 1}</span>
                  <span className="truncate">{ts.humanReadable}</span>
                </div>
              ))}
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

function UniversalDateConverter() {
  const [inputDate, setInputDate] = useState("");
  const [results, setResults] = useState<CalendarsResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const convertDate = async (dateStr?: string) => {
    setLoading(true);
    setError(null);
    try {
      const url = dateStr
        ? `/api/salvi/timing/epoch/calendars?date=${encodeURIComponent(dateStr)}`
        : "/api/salvi/timing/epoch/calendars";
      const res = await fetch(url);
      const data = await res.json();
      if (data.success) {
        setResults(data);
      } else {
        setError(data.error || "Conversion failed");
      }
    } catch {
      setError("Failed to reach API");
    }
    setLoading(false);
  };

  const baseUrl = typeof window !== "undefined" ? window.location.origin : "";

  return (
    <Card data-testid="card-date-converter">
      <CardHeader className="pb-3">
        <CardTitle className="text-base flex items-center gap-2">
          <RefreshCw className="w-4 h-4 text-primary" />
          Universal Date Converter
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <p className="text-xs text-muted-foreground">
          Query any Gregorian date and get its equivalent across all 24 calendrical systems. Leave blank for today's date.
        </p>
        <div className="flex items-center gap-2 flex-wrap">
          <input
            type="date"
            value={inputDate}
            onChange={(e) => setInputDate(e.target.value)}
            className="h-9 px-3 rounded-md border bg-background text-sm font-mono"
            data-testid="input-date-converter"
          />
          <Button
            onClick={() => convertDate(inputDate || undefined)}
            disabled={loading}
            data-testid="button-convert-date"
          >
            {loading ? <RefreshCw className="w-4 h-4 mr-2 animate-spin" /> : <Globe className="w-4 h-4 mr-2" />}
            Convert
          </Button>
          <Button
            variant="outline"
            onClick={() => { setInputDate(""); convertDate(); }}
            disabled={loading}
            data-testid="button-convert-today"
          >
            Today
          </Button>
        </div>
        {error && (
          <p className="text-xs text-destructive">{error}</p>
        )}
        {results && (
          <div className="space-y-2">
            <div className="flex items-center gap-2 flex-wrap">
              <Badge variant="default" data-testid="badge-converter-count">{results.allMappings.length} systems</Badge>
              <span className="text-xs text-muted-foreground">
                Showing equivalents for: <span className="font-mono font-medium text-foreground">{inputDate || new Date().toISOString().split("T")[0]}</span>
              </span>
            </div>
            <div className="max-h-96 overflow-y-auto rounded-md space-y-1" data-testid="list-converter-results">
              {results.allMappings.map((m, i) => (
                <div key={i} className="flex items-center justify-between gap-2 p-2 rounded-md bg-muted/50" data-testid={`row-converted-${i}`}>
                  <div className="space-y-0.5 min-w-0 flex-1">
                    <div className="text-sm font-medium truncate">{m.calendarSystem}</div>
                    <div className="text-xs font-mono text-foreground truncate">{m.salviEpochEquivalent}</div>
                  </div>
                  <Badge variant="outline" className="text-[10px] shrink-0">Year {m.yearInCalendar}</Badge>
                </div>
              ))}
            </div>
          </div>
        )}
        <div className="border-t pt-3 space-y-1">
          <p className="text-xs font-medium text-muted-foreground">API Usage</p>
          <code className="block text-[10px] font-mono text-muted-foreground break-all">
            GET {baseUrl}/api/salvi/timing/epoch/calendars?date=2025-04-01
          </code>
          <code className="block text-[10px] font-mono text-muted-foreground break-all">
            GET {baseUrl}/api/salvi/timing/epoch/calendars/mayan?date=2000-01-01
          </code>
        </div>
      </CardContent>
    </Card>
  );
}

function CalendarSync() {
  const todayStr = new Date().toISOString().split("T")[0];
  const { data, isLoading } = useQuery<CalendarsResponse>({
    queryKey: ["/api/salvi/timing/epoch/calendars", todayStr],
    queryFn: async () => {
      const res = await fetch(`/api/salvi/timing/epoch/calendars?date=${todayStr}`);
      return res.json();
    },
  });

  const regionLabels: Record<string, string> = {
    "Mayan Long Count": "Mesoamerica",
    "Hebrew Calendar": "Middle East",
    "Chinese Sexagenary Cycle": "East Asia",
    "Vedic Kali Yuga": "South Asia",
    "Egyptian Civil Calendar": "North Africa",
    "Julian Day Number": "Astronomical",
    "Islamic Hijri": "Islamic World",
    "Byzantine Anno Mundi": "Eastern Europe",
    "13-Moon Natural Time": "Prehistoric",
    "Persian/Solar Hijri": "Iran/Central Asia",
    "Ethiopian/Ge'ez Calendar": "East Africa",
    "Coptic Calendar": "Egypt/Coptic",
    "Japanese Imperial (Koki)": "Japan",
    "Korean Dangun Era": "Korea",
    "Thai Buddhist Era": "Southeast Asia",
    "Indian National/Saka": "India",
    "Tibetan Rabjung Cycle": "Tibet/Mongolia",
    "Aztec Tonalpohualli": "Mesoamerica",
    "Roman Ab Urbe Condita": "Mediterranean",
    "Bengali/Bangla Calendar": "Bangladesh/Bengal",
    "Berber/Amazigh (Yennayer)": "North Africa",
    "Balinese Pawukon": "Indonesia",
    "Zoroastrian Fasli": "Persia",
    "Aboriginal Australian Seasonal": "Oceania",
  };

  return (
    <Card data-testid="card-calendar-sync">
      <CardHeader className="pb-3">
        <CardTitle className="text-base flex items-center gap-2">
          <Globe className="w-4 h-4 text-primary" />
          Calendar Epoch Origins
        </CardTitle>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <div className="space-y-2">
            {[1, 2, 3].map((i) => (
              <div key={i} className="h-8 bg-muted rounded animate-pulse" />
            ))}
          </div>
        ) : data?.allMappings ? (
          <div className="space-y-2">
            <div className="flex items-center gap-2 mb-1 flex-wrap">
              <Badge variant="default" data-testid="badge-calendar-count">{data.allMappings.length} calendars</Badge>
              <span className="text-xs text-muted-foreground">
                Synchronized via Salvi Epoch (April 1, 2025 UTC)
              </span>
            </div>
            <p className="text-xs text-muted-foreground mb-3" data-testid="text-epoch-explanation">
              Each calendar system has its own historical starting point (epoch). The dates below show when each calendar began counting, not today's date.
              Today's equivalents are shown via the Universal Date Converter above, or by querying any endpoint with <code className="font-mono bg-muted px-1 rounded">?date=YYYY-MM-DD</code>.
            </p>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
              {data.allMappings.map((m, i) => (
                <div key={i} className="flex items-center justify-between gap-2 p-2 rounded-md bg-muted/50" data-testid={`row-calendar-${i}`}>
                  <div className="space-y-0.5 min-w-0 flex-1">
                    <div className="text-sm font-medium truncate">{m.calendarSystem}</div>
                    <div className="text-[10px] text-muted-foreground truncate">
                      Epoch: {m.origin}
                    </div>
                    <div className="text-[10px] font-mono text-foreground/70 truncate">
                      Today: {m.salviEpochEquivalent}
                    </div>
                  </div>
                  <Badge variant="outline" className="text-[10px] shrink-0">
                    {regionLabels[m.calendarSystem] || `Year ${m.yearInCalendar}`}
                  </Badge>
                </div>
              ))}
            </div>
          </div>
        ) : (
          <div className="text-sm text-muted-foreground">Unable to load calendar data</div>
        )}
      </CardContent>
    </Card>
  );
}

function ComplianceBanner() {
  const standards = [
    { label: "FINRA CAT", desc: "Targeting 50μs reporting threshold" },
    { label: "MiFID II", desc: "Targeting 1ms granularity requirement" },
    { label: "CNSA 2.0", desc: "11/11 algorithms implemented" },
    { label: "FIPS 140-3", desc: "CMVP submission ready (v3.0.0)" },
  ];

  return (
    <Card data-testid="card-compliance">
      <CardHeader className="pb-3">
        <CardTitle className="text-base flex items-center gap-2">
          <Shield className="w-4 h-4 text-primary" />
          Regulatory Compliance
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="grid grid-cols-2 gap-3">
          {standards.map((s, i) => (
            <div key={i} className="p-3 rounded-md bg-muted/50 space-y-1" data-testid={`card-standard-${i}`}>
              <div className="text-sm font-medium">{s.label}</div>
              <div className="text-xs text-muted-foreground">{s.desc}</div>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}

function CurlExample() {
  const baseUrl = typeof window !== "undefined" ? window.location.origin : "";
  const [activeTab, setActiveTab] = useState<"bash" | "powershell">("bash");
  const [copied, setCopied] = useState(false);

  const commands = {
    bash: `curl -s ${baseUrl}/api/salvi/timing/timestamp | jq .`,
    powershell: `Invoke-RestMethod -Uri "${baseUrl}/api/salvi/timing/timestamp" | ConvertTo-Json -Depth 10`,
  };

  const activeCmd = commands[activeTab];

  return (
    <Card data-testid="card-curl-example">
      <CardHeader className="pb-3">
        <CardTitle className="text-base flex items-center gap-2">
          <Server className="w-4 h-4 text-primary" />
          Try It Now
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="text-sm text-muted-foreground">
          Copy this command and run it in your terminal:
        </div>
        <div className="flex gap-1">
          <Button
            size="sm"
            variant={activeTab === "bash" ? "default" : "ghost"}
            onClick={() => setActiveTab("bash")}
            data-testid="button-tab-bash"
          >
            Bash / macOS / Linux
          </Button>
          <Button
            size="sm"
            variant={activeTab === "powershell" ? "default" : "ghost"}
            onClick={() => setActiveTab("powershell")}
            data-testid="button-tab-powershell"
          >
            PowerShell
          </Button>
        </div>
        <div
          className="relative p-3 rounded-md bg-muted/50 font-mono text-xs break-all cursor-pointer hover-elevate"
          onClick={() => {
            navigator.clipboard.writeText(activeCmd);
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
          }}
          data-testid="code-curl-command"
        >
          <code>{activeCmd}</code>
          <div className="absolute top-2 right-2">
            {copied ? <Check className="w-3.5 h-3.5 text-green-600" /> : <Copy className="w-3.5 h-3.5 text-muted-foreground" />}
          </div>
        </div>
        <div className="text-xs text-muted-foreground">
          No authentication required. Public endpoint. JSON response.
        </div>
      </CardContent>
    </Card>
  );
}

export default function HPTPDemo() {
  return (
    <div className="max-w-6xl mx-auto p-4 sm:p-6 space-y-6">
      <div className="space-y-2">
        <div className="flex flex-wrap items-center gap-2">
          <h1 className="text-2xl font-bold tracking-tight" data-testid="text-page-title">
            HPTP Timing API
          </h1>
          <Badge variant="default" data-testid="badge-live">LIVE</Badge>
        </div>
        <p className="text-sm text-muted-foreground max-w-2xl" data-testid="text-page-description">
          High-Precision Timing Protocol delivering femtosecond-resolution timestamps synchronized across 24 global calendar systems spanning every inhabited continent. 
          Production-grade API designed for FINRA CAT and MiFID II timing requirements in quantum-resistant financial infrastructure.
        </p>
      </div>

      <LiveTimestamp />

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <MetricsPanel />
        <ComplianceBanner />
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <BatchDemo />
        <CurlExample />
      </div>

      <EndpointReference />
      <UniversalDateConverter />
      <CalendarSync />
    </div>
  );
}
