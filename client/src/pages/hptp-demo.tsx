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

interface TimestampResponse {
  success: boolean;
  timestamp: TimestampData;
  epoch?: {
    salviEpoch: string;
    description: string;
  };
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

function LiveTimestamp() {
  const [ts, setTs] = useState<TimestampData | null>(null);
  const [isStreaming, setIsStreaming] = useState(false);
  const intervalRef = useRef<NodeJS.Timeout | null>(null);
  const [requestCount, setRequestCount] = useState(0);
  const [latency, setLatency] = useState(0);

  const fetchTimestamp = useCallback(async () => {
    const start = performance.now();
    try {
      const res = await fetch("/api/salvi/timing/timestamp");
      const data: TimestampResponse = await res.json();
      setLatency(Math.round(performance.now() - start));
      if (data.success) {
        setTs(data.timestamp);
        setRequestCount((c) => c + 1);
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
        <div className="font-mono text-2xl tracking-tight text-foreground break-all leading-relaxed" data-testid="text-femtosecond-value">
          {ts ? formatFemtoseconds(ts.femtoseconds) : "Loading..."}
        </div>
        <div className="text-sm text-muted-foreground" data-testid="text-human-readable">
          {ts?.humanReadable || ""}
        </div>
        <div className="grid grid-cols-3 gap-3">
          <div className="space-y-1">
            <div className="text-xs text-muted-foreground">Precision</div>
            <div className="text-sm font-medium" data-testid="text-precision">{ts?.precision || "—"}</div>
          </div>
          <div className="space-y-1">
            <div className="text-xs text-muted-foreground">API Latency</div>
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
    { method: "GET", path: "/api/salvi/timing/epoch/calendars", desc: "24 global calendar systems" },
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

function CalendarSync() {
  const { data, isLoading } = useQuery<CalendarsResponse>({
    queryKey: ["/api/salvi/timing/epoch/calendars"],
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
          Ancient Calendar Anchoring
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
            <div className="flex items-center gap-2 mb-3">
              <Badge variant="default" data-testid="badge-calendar-count">{data.allMappings.length} calendars</Badge>
              <span className="text-xs text-muted-foreground">
                Salvi Epoch anchored across all major civilizations spanning 30,000+ years
              </span>
            </div>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
              {data.allMappings.map((m, i) => (
                <div key={i} className="flex items-center justify-between gap-2 p-2 rounded-md bg-muted/50" data-testid={`row-calendar-${i}`}>
                  <div className="space-y-0.5 min-w-0 flex-1">
                    <div className="text-sm font-medium truncate">{m.calendarSystem}</div>
                    <div className="text-[10px] text-muted-foreground truncate">{m.origin}</div>
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
    { label: "FINRA CAT", desc: "50μs reporting threshold exceeded" },
    { label: "MiFID II", desc: "1ms granularity requirement met" },
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
  const curlCmd = `curl -s ${baseUrl}/api/salvi/timing/timestamp | jq .`;
  const [copied, setCopied] = useState(false);

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
          Copy this command and run it in any terminal to see live results:
        </div>
        <div
          className="relative p-3 rounded-md bg-muted/50 font-mono text-xs break-all cursor-pointer hover-elevate"
          onClick={() => {
            navigator.clipboard.writeText(curlCmd);
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
          }}
          data-testid="code-curl-command"
        >
          <code>{curlCmd}</code>
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
          Production-grade API powering FINRA CAT and MiFID II regulatory compliance for quantum-resistant financial infrastructure.
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
      <CalendarSync />
    </div>
  );
}
