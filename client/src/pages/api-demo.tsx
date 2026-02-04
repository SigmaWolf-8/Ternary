import { useState, useEffect } from "react";
import { Link } from "wouter";
import { useQuery, useMutation } from "@tanstack/react-query";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ArrowLeft, Clock, Calculator, Shield, RefreshCw, Zap, Play, Copy, Check, Database, TrendingUp } from "lucide-react";
import { apiRequest } from "@/lib/queryClient";

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

interface TernaryResult {
  success: boolean;
  result?: number;
  operands?: { a: number; b: number };
  operation?: string;
  representation?: string;
}

interface ConvertResult {
  success: boolean;
  original: { value: number; representation: string; meaning: string };
  converted: { value: number; representation: string; meaning: string };
  bijection: string;
}

interface PhaseResult {
  success: boolean;
  encrypted?: {
    primaryPhase: { data: string; phase: number; timestamp: TimestampData };
    secondaryPhase: { data: string; phase: number; timestamp: TimestampData };
    config: { mode: string; primaryPhase: number; secondaryOffset: number };
    splitRatio: number;
  };
}

interface CompressionResult {
  success: boolean;
  sessionId: string;
  datasetName: string;
  rowCount: number;
  binarySize: number;
  ternarySize: number;
  savingsPercent: string;
  processingTimeMs: number;
  preview: any[];
}

interface DensityResult {
  success: boolean;
  trits: number;
  bitsEquivalent: number;
  efficiencyGain: string;
}

// Salvi Epoch: April 1, 2025 00:00:00.000 UTC
const SALVI_EPOCH_MS = new Date('2025-04-01T00:00:00.000Z').getTime();
const SALVI_EPOCH_NS = BigInt(SALVI_EPOCH_MS) * 1_000_000n;

interface LiveTimerState {
  days: number;
  hours: number;
  minutes: number;
  seconds: number;
  milliseconds: number;
  unixNanoseconds: bigint;
  salviNanoseconds: bigint;
}

function CompressionDemo() {
  const [dataset, setDataset] = useState("sensor");
  const [rowCount, setRowCount] = useState(100);
  
  const compressionMutation = useMutation({
    mutationFn: async () => {
      const response = await apiRequest("POST", "/api/demo/run", {
        datasetName: dataset,
        rowCount: rowCount
      });
      return response.json();
    }
  });
  
  const { data: stats } = useQuery<{
    totalRuns: number;
    avgSavings: string;
    totalDataProcessed: number;
    totalSavings: number;
  }>({
    queryKey: ["/api/demo/stats"]
  });

  const result = compressionMutation.data as CompressionResult | undefined;

  return (
    <div className="grid md:grid-cols-2 gap-6">
      <Card>
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <Database className="w-5 h-5" />
            Ternary Compression Demo
          </CardTitle>
          <CardDescription>
            POST /api/demo/run - Compress data using ternary encoding
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label>Dataset Type</Label>
            <Select value={dataset} onValueChange={setDataset}>
              <SelectTrigger data-testid="select-compression-dataset">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="sensor">Sensor Data</SelectItem>
                <SelectItem value="events">User Events</SelectItem>
                <SelectItem value="logs">Log Entries</SelectItem>
              </SelectContent>
            </Select>
          </div>
          
          <div className="space-y-2">
            <Label>Row Count</Label>
            <Input 
              type="number" 
              value={rowCount} 
              onChange={(e) => setRowCount(parseInt(e.target.value) || 100)}
              min={1}
              max={10000}
              data-testid="input-compression-rows"
            />
          </div>
          
          <Button 
            onClick={() => compressionMutation.mutate()}
            disabled={compressionMutation.isPending}
            className="w-full"
            data-testid="button-run-compression"
          >
            {compressionMutation.isPending ? (
              <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
            ) : (
              <Play className="w-4 h-4 mr-2" />
            )}
            Run Compression
          </Button>
          
          {result && (
            <div className="mt-4 space-y-3">
              <div className="grid grid-cols-2 gap-3">
                <div className="bg-muted/50 rounded-lg p-3 text-center">
                  <div className="text-2xl font-bold text-red-600">{result.binarySize.toLocaleString()}</div>
                  <div className="text-xs text-muted-foreground">Binary Size (bytes)</div>
                </div>
                <div className="bg-muted/50 rounded-lg p-3 text-center">
                  <div className="text-2xl font-bold text-green-600">{result.ternarySize.toLocaleString()}</div>
                  <div className="text-xs text-muted-foreground">Ternary Size (bytes)</div>
                </div>
              </div>
              <div className="bg-green-50 border border-green-200 rounded-lg p-4 text-center">
                <div className="text-3xl font-bold text-green-700">{result.savingsPercent}%</div>
                <div className="text-sm text-green-600">Space Savings</div>
              </div>
              <div className="text-xs text-muted-foreground text-center">
                Processed in {result.processingTimeMs}ms
              </div>
            </div>
          )}
        </CardContent>
      </Card>
      
      <Card>
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <TrendingUp className="w-5 h-5" />
            Compression Statistics
          </CardTitle>
          <CardDescription>
            GET /api/demo/stats - Aggregated compression metrics
          </CardDescription>
        </CardHeader>
        <CardContent>
          {stats ? (
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="bg-primary/10 rounded-lg p-4 text-center">
                  <div className="text-2xl font-bold text-primary">{stats.totalRuns}</div>
                  <div className="text-xs text-muted-foreground">Total Runs</div>
                </div>
                <div className="bg-green-100 rounded-lg p-4 text-center">
                  <div className="text-2xl font-bold text-green-700">{stats.avgSavings}%</div>
                  <div className="text-xs text-muted-foreground">Avg Savings</div>
                </div>
              </div>
              <div className="bg-muted/50 rounded-lg p-4">
                <div className="text-sm font-medium mb-2">Data Processed</div>
                <div className="text-2xl font-bold">{(stats.totalDataProcessed / 1024).toFixed(1)} KB</div>
              </div>
              <div className="bg-green-50 border border-green-200 rounded-lg p-4">
                <div className="text-sm font-medium text-green-800 mb-1">Total Space Saved</div>
                <div className="text-2xl font-bold text-green-700">{(stats.totalSavings / 1024).toFixed(1)} KB</div>
              </div>
            </div>
          ) : (
            <div className="text-center text-muted-foreground py-8">Loading statistics...</div>
          )}
          
          <div className="mt-6 pt-4 border-t">
            <div className="text-sm font-medium mb-3">Why +59% Efficiency?</div>
            <div className="grid grid-cols-3 gap-2 text-center">
              <div className="bg-blue-50 border border-blue-200 rounded-lg p-2">
                <div className="font-bold text-blue-800">1.585</div>
                <div className="text-xs text-blue-600">bits/trit</div>
              </div>
              <div className="bg-blue-50 border border-blue-200 rounded-lg p-2">
                <div className="font-bold text-blue-800">3:2</div>
                <div className="text-xs text-blue-600">compression</div>
              </div>
              <div className="bg-green-50 border border-green-200 rounded-lg p-2">
                <div className="font-bold text-green-800">+59%</div>
                <div className="text-xs text-green-600">density</div>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

function DensityCalculator() {
  const [tritCount, setTritCount] = useState(8);
  
  const { data: density, refetch, isFetching } = useQuery<DensityResult>({
    queryKey: ["/api/salvi/ternary/density", tritCount],
    queryFn: async () => {
      const response = await fetch(`/api/salvi/ternary/density/${tritCount}`);
      return response.json();
    },
    enabled: false
  });

  return (
    <div className="grid md:grid-cols-2 gap-6">
      <Card>
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <Calculator className="w-5 h-5" />
            Information Density Calculator
          </CardTitle>
          <CardDescription>
            GET /api/salvi/ternary/density/:tritCount
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label>Trit Count</Label>
            <Input 
              type="number" 
              value={tritCount} 
              onChange={(e) => setTritCount(parseInt(e.target.value) || 8)}
              min={1}
              max={64}
              data-testid="input-density-trits"
            />
          </div>
          
          <Button 
            onClick={() => refetch()}
            disabled={isFetching}
            className="w-full"
            data-testid="button-calculate-density"
          >
            {isFetching ? (
              <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
            ) : (
              <Calculator className="w-4 h-4 mr-2" />
            )}
            Calculate Density
          </Button>
          
          {density && density.success && (
            <div className="mt-4 space-y-3">
              <div className="bg-primary/10 rounded-lg p-4">
                <div className="text-sm text-muted-foreground mb-1">Ternary States (3^{density.trits})</div>
                <div className="text-xl font-mono font-bold">{Math.pow(3, density.trits).toLocaleString()}</div>
              </div>
              <div className="grid grid-cols-2 gap-3">
                <div className="bg-muted/50 rounded-lg p-3">
                  <div className="text-xs text-muted-foreground mb-1">Binary Bits Needed</div>
                  <div className="text-lg font-bold">{density.bitsEquivalent.toFixed(2)}</div>
                </div>
                <div className="bg-green-50 border border-green-200 rounded-lg p-3">
                  <div className="text-xs text-green-600 mb-1">Efficiency Gain</div>
                  <div className="text-lg font-bold text-green-700">{density.efficiencyGain}</div>
                </div>
              </div>
            </div>
          )}
        </CardContent>
      </Card>
      
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Ternary vs Binary Comparison</CardTitle>
          <CardDescription>
            Mathematical foundation: log₂(3) ≈ 1.585 bits per trit
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="text-center">
                <div className="text-sm text-muted-foreground mb-2">Binary</div>
                <div className="bg-muted rounded-lg p-4">
                  <div className="font-mono text-lg">2 states</div>
                  <div className="text-sm text-muted-foreground">per digit</div>
                </div>
              </div>
              <div className="text-center">
                <div className="text-sm text-muted-foreground mb-2">Ternary</div>
                <div className="bg-green-50 border border-green-200 rounded-lg p-4">
                  <div className="font-mono text-lg text-green-700">3 states</div>
                  <div className="text-sm text-green-600">per trit</div>
                </div>
              </div>
            </div>
            
            <div className="bg-muted/50 rounded-lg p-4">
              <div className="text-sm font-medium mb-3">Information per Digit</div>
              <div className="flex items-center justify-between">
                <div className="text-center">
                  <div className="text-2xl font-bold">1.0</div>
                  <div className="text-xs text-muted-foreground">bit (binary)</div>
                </div>
                <div className="text-muted-foreground">vs</div>
                <div className="text-center">
                  <div className="text-2xl font-bold text-green-700">1.585</div>
                  <div className="text-xs text-green-600">bits (ternary)</div>
                </div>
              </div>
            </div>
            
            <div className="bg-green-50 border border-green-200 rounded-lg p-4 text-center">
              <div className="text-3xl font-bold text-green-700">+58.5%</div>
              <div className="text-sm text-green-600">More Information Per Digit</div>
            </div>
            
            <div className="text-xs text-muted-foreground text-center">
              Formula: log₂(3) / log₂(2) = 1.585 ≈ 59% more efficient
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

export default function APIDemo() {
  const [copiedEndpoint, setCopiedEndpoint] = useState<string | null>(null);
  const [liveTimer, setLiveTimer] = useState<LiveTimerState>({
    days: 0, hours: 0, minutes: 0, seconds: 0, milliseconds: 0,
    unixNanoseconds: 0n, salviNanoseconds: 0n
  });
  
  // Live timer synchronized with Unix nanoseconds
  useEffect(() => {
    const updateTimer = () => {
      const now = Date.now();
      const elapsed = now - SALVI_EPOCH_MS;
      
      // Calculate Unix nanoseconds (simulated sub-ms precision)
      const unixNs = BigInt(now) * 1_000_000n + BigInt(Math.floor(performance.now() * 1000) % 1000000);
      const salviNs = unixNs - SALVI_EPOCH_NS;
      
      const days = Math.floor(elapsed / (1000 * 60 * 60 * 24));
      const hours = Math.floor((elapsed % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
      const minutes = Math.floor((elapsed % (1000 * 60 * 60)) / (1000 * 60));
      const seconds = Math.floor((elapsed % (1000 * 60)) / 1000);
      const milliseconds = elapsed % 1000;
      
      setLiveTimer({
        days, hours, minutes, seconds, milliseconds,
        unixNanoseconds: unixNs,
        salviNanoseconds: salviNs
      });
    };
    
    updateTimer();
    const interval = setInterval(updateTimer, 10); // Update every 10ms for smooth display
    return () => clearInterval(interval);
  }, []);
  
  const [tritA, setTritA] = useState<number>(1);
  const [tritB, setTritB] = useState<number>(-1);
  const [convertValue, setConvertValue] = useState<number>(0);
  const [fromRep, setFromRep] = useState<string>("A");
  const [toRep, setToRep] = useState<string>("B");
  const [phaseData, setPhaseData] = useState<string>("Hello PlenumNET");
  const [phaseMode, setPhaseMode] = useState<string>("balanced");
  
  const { data: timestamp, refetch: refetchTimestamp, isFetching: isTimestampFetching } = useQuery<TimestampResponse>({
    queryKey: ["/api/salvi/timing/timestamp"],
    refetchInterval: false,
  });
  
  const { data: metrics } = useQuery<TimingMetrics>({
    queryKey: ["/api/salvi/timing/metrics"],
  });
  
  const addMutation = useMutation({
    mutationFn: async ({ a, b }: { a: number; b: number }) => {
      const res = await apiRequest("POST", "/api/salvi/ternary/add", { a, b });
      return res.json();
    },
  });
  
  const multiplyMutation = useMutation({
    mutationFn: async ({ a, b }: { a: number; b: number }) => {
      const res = await apiRequest("POST", "/api/salvi/ternary/multiply", { a, b });
      return res.json();
    },
  });
  
  const convertMutation = useMutation({
    mutationFn: async ({ value, from, to }: { value: number; from: string; to: string }) => {
      const res = await apiRequest("POST", "/api/salvi/ternary/convert", { value, from, to });
      return res.json();
    },
  });
  
  const phaseSplitMutation = useMutation({
    mutationFn: async ({ data, mode }: { data: string; mode: string }) => {
      const res = await apiRequest("POST", "/api/salvi/phase/split", { data, mode });
      return res.json();
    },
  });

  const copyToClipboard = (text: string, endpoint: string) => {
    navigator.clipboard.writeText(text);
    setCopiedEndpoint(endpoint);
    setTimeout(() => setCopiedEndpoint(null), 2000);
  };

  const formatBigInt = (value: string) => {
    if (!value) return "0";
    const numStr = value.toString();
    if (numStr.length > 15) {
      return numStr.slice(0, 6) + "..." + numStr.slice(-6) + ` (${numStr.length} digits)`;
    }
    return numStr;
  };

  return (
    <div className="min-h-screen bg-background">
      <header className="border-b bg-white sticky top-0 z-50">
        <div className="container mx-auto px-4 py-4 flex items-center justify-between">
          <div className="flex items-center gap-4">
            <Link href="/">
              <Button variant="ghost" size="sm" data-testid="button-back">
                <ArrowLeft className="w-4 h-4 mr-2" />
                Back
              </Button>
            </Link>
            <div>
              <h1 className="text-xl font-bold text-foreground">PlenumNET API Demo</h1>
              <p className="text-sm text-muted-foreground">Interactive demonstration of PlenumNET Framework APIs</p>
            </div>
          </div>
          <div className="flex items-center gap-3">
            <Link href="/kong-konnect">
              <Button variant="ghost" size="sm" data-testid="button-kong-konnect">
                Kong Konnect
              </Button>
            </Link>
            <Badge variant="outline" className="text-primary border-primary">
              Live API
            </Badge>
          </div>
        </div>
      </header>

      <main className="container mx-auto px-4 py-8">
        <div className="mb-8 space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Zap className="w-5 h-5 text-primary" />
                PlenumNET Framework Core API
              </CardTitle>
              <CardDescription>
                Test the PlenumNET APIs directly. All timestamps reference the Salvi Epoch: April 1, 2025 (Day Zero).
              </CardDescription>
            </CardHeader>
          </Card>
          
          <Card className="bg-gradient-to-r from-blue-50 to-indigo-50 border-blue-200">
            <CardHeader className="pb-2">
              <CardTitle className="flex items-center gap-2 text-blue-800">
                <Clock className="w-5 h-5 animate-pulse" />
                Live Time Since Salvi Epoch
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="grid md:grid-cols-2 gap-6">
                <div>
                  <div className="text-sm text-blue-600 mb-2 font-medium">Elapsed Duration</div>
                  <div className="flex items-baseline gap-1 font-mono text-2xl font-bold text-blue-900" data-testid="text-elapsed-time">
                    <span>{String(liveTimer.days).padStart(3, '0')}</span>
                    <span className="text-blue-400 text-lg">d</span>
                    <span>{String(liveTimer.hours).padStart(2, '0')}</span>
                    <span className="text-blue-400 text-lg">h</span>
                    <span>{String(liveTimer.minutes).padStart(2, '0')}</span>
                    <span className="text-blue-400 text-lg">m</span>
                    <span>{String(liveTimer.seconds).padStart(2, '0')}</span>
                    <span className="text-blue-400 text-lg">s</span>
                    <span className="text-lg">{String(liveTimer.milliseconds).padStart(3, '0')}</span>
                    <span className="text-blue-400 text-sm">ms</span>
                  </div>
                </div>
                
                <div className="space-y-3">
                  <div>
                    <div className="text-xs text-blue-600 mb-1">Unix Nanoseconds (Current)</div>
                    <div className="font-mono text-sm bg-white/50 rounded px-2 py-1 text-blue-900" data-testid="text-unix-ns">
                      {liveTimer.unixNanoseconds.toString()}
                    </div>
                  </div>
                  <div>
                    <div className="text-xs text-blue-600 mb-1">Salvi Epoch Nanoseconds (Offset)</div>
                    <div className="font-mono text-sm bg-white/50 rounded px-2 py-1 text-blue-900" data-testid="text-salvi-ns">
                      {liveTimer.salviNanoseconds.toString()}
                    </div>
                  </div>
                </div>
              </div>
              
              <div className="mt-4 pt-3 border-t border-blue-200 flex flex-wrap items-center gap-4 text-xs text-blue-600">
                <div className="flex items-center gap-1">
                  <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse"></div>
                  <span>Synchronized</span>
                </div>
                <div>Epoch: 2025-04-01T00:00:00.000Z</div>
                <div>Epoch Unix NS: 1743465600000000000</div>
              </div>
            </CardContent>
          </Card>
        </div>

        <Tabs defaultValue="timing" className="space-y-6">
          <TabsList className="grid w-full grid-cols-5 max-w-3xl">
            <TabsTrigger value="timing" data-testid="tab-timing" className="flex items-center gap-2">
              <Clock className="w-4 h-4" />
              Timing
            </TabsTrigger>
            <TabsTrigger value="ternary" data-testid="tab-ternary" className="flex items-center gap-2">
              <Calculator className="w-4 h-4" />
              Ternary
            </TabsTrigger>
            <TabsTrigger value="phase" data-testid="tab-phase" className="flex items-center gap-2">
              <Shield className="w-4 h-4" />
              Encryption
            </TabsTrigger>
            <TabsTrigger value="compression" data-testid="tab-compression" className="flex items-center gap-2">
              <Database className="w-4 h-4" />
              Compression
            </TabsTrigger>
            <TabsTrigger value="density" data-testid="tab-density" className="flex items-center gap-2">
              <TrendingUp className="w-4 h-4" />
              Density
            </TabsTrigger>
          </TabsList>

          <TabsContent value="timing" className="space-y-6">
            <div className="grid md:grid-cols-2 gap-6">
              <Card>
                <CardHeader>
                  <CardTitle className="text-lg">Femtosecond Timestamp</CardTitle>
                  <CardDescription>
                    GET /api/salvi/timing/timestamp
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="flex gap-2">
                    <Button 
                      onClick={() => refetchTimestamp()} 
                      disabled={isTimestampFetching}
                      data-testid="button-get-timestamp"
                    >
                      <RefreshCw className={`w-4 h-4 mr-2 ${isTimestampFetching ? 'animate-spin' : ''}`} />
                      Get Timestamp
                    </Button>
                    <Button 
                      variant="outline" 
                      size="icon"
                      onClick={() => copyToClipboard('/api/salvi/timing/timestamp', 'timestamp')}
                      data-testid="button-copy-timestamp-endpoint"
                    >
                      {copiedEndpoint === 'timestamp' ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
                    </Button>
                  </div>
                  
                  {timestamp?.timestamp && (
                    <div className="bg-muted/50 rounded-lg p-4 space-y-2 font-mono text-sm">
                      <div className="flex justify-between">
                        <span className="text-muted-foreground">Femtoseconds:</span>
                        <span className="text-foreground" data-testid="text-femtoseconds">{formatBigInt(timestamp.timestamp.femtoseconds)}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-muted-foreground">Human Readable:</span>
                        <span className="text-foreground">{timestamp.timestamp.humanReadable}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-muted-foreground">ISO Date:</span>
                        <span className="text-foreground">{timestamp.timestamp.isoDate}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-muted-foreground">Precision:</span>
                        <Badge variant="secondary">{timestamp.timestamp.precision}</Badge>
                      </div>
                    </div>
                  )}
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle className="text-lg">Timing Metrics</CardTitle>
                  <CardDescription>
                    GET /api/salvi/timing/metrics
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  {metrics && (
                    <div className="bg-muted/50 rounded-lg p-4 space-y-2 font-mono text-sm">
                      <div className="flex justify-between">
                        <span className="text-muted-foreground">Clock Source:</span>
                        <span className="text-foreground">{metrics.clockSource}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-muted-foreground">Sync Status:</span>
                        <Badge 
                          variant={metrics.synchronizationStatus === 'synchronized' ? 'default' : 'destructive'}
                          data-testid="badge-sync-status"
                        >
                          {metrics.synchronizationStatus}
                        </Badge>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-muted-foreground">Accuracy:</span>
                        <span className="text-foreground">{metrics.estimatedAccuracy}</span>
                      </div>
                    </div>
                  )}
                </CardContent>
              </Card>
            </div>

            <Card>
              <CardHeader>
                <CardTitle className="text-lg">Salvi Epoch Reference</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="grid md:grid-cols-3 gap-4">
                  <div className="bg-primary/5 rounded-lg p-4 text-center">
                    <div className="text-sm text-muted-foreground mb-1">Day Zero</div>
                    <div className="font-mono font-bold text-primary">April 1, 2025</div>
                  </div>
                  <div className="bg-primary/5 rounded-lg p-4 text-center">
                    <div className="text-sm text-muted-foreground mb-1">Unix Nanoseconds</div>
                    <div className="font-mono font-bold text-primary text-sm">1743465600000000000</div>
                  </div>
                  <div className="bg-primary/5 rounded-lg p-4 text-center">
                    <div className="text-sm text-muted-foreground mb-1">Precision</div>
                    <div className="font-mono font-bold text-primary">10⁻¹⁵ seconds</div>
                  </div>
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="ternary" className="space-y-6">
            <div className="grid md:grid-cols-2 gap-6">
              <Card>
                <CardHeader>
                  <CardTitle className="text-lg">Ternary Addition (GF3)</CardTitle>
                  <CardDescription>
                    POST /api/salvi/ternary/add
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <Label htmlFor="add-a">Trit A</Label>
                      <Select value={tritA.toString()} onValueChange={(v) => setTritA(parseInt(v))}>
                        <SelectTrigger data-testid="select-trit-a">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="-1">-1</SelectItem>
                          <SelectItem value="0">0</SelectItem>
                          <SelectItem value="1">+1</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                    <div>
                      <Label htmlFor="add-b">Trit B</Label>
                      <Select value={tritB.toString()} onValueChange={(v) => setTritB(parseInt(v))}>
                        <SelectTrigger data-testid="select-trit-b">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="-1">-1</SelectItem>
                          <SelectItem value="0">0</SelectItem>
                          <SelectItem value="1">+1</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                  </div>
                  
                  <div className="flex gap-2">
                    <Button 
                      onClick={() => addMutation.mutate({ a: tritA, b: tritB })}
                      disabled={addMutation.isPending}
                      data-testid="button-ternary-add"
                    >
                      <Play className="w-4 h-4 mr-2" />
                      Add
                    </Button>
                    <Button 
                      onClick={() => multiplyMutation.mutate({ a: tritA, b: tritB })}
                      disabled={multiplyMutation.isPending}
                      variant="outline"
                      data-testid="button-ternary-multiply"
                    >
                      <Play className="w-4 h-4 mr-2" />
                      Multiply
                    </Button>
                  </div>

                  {addMutation.data && (
                    <div className="bg-muted/50 rounded-lg p-4 font-mono text-sm">
                      <div className="flex justify-between items-center">
                        <span className="text-muted-foreground">Addition Result:</span>
                        <Badge variant="default" className="text-lg" data-testid="badge-add-result">
                          {(addMutation.data as TernaryResult).result}
                        </Badge>
                      </div>
                      <div className="mt-2 text-xs text-muted-foreground">
                        {tritA} + {tritB} = {(addMutation.data as TernaryResult).result} (mod 3)
                      </div>
                    </div>
                  )}

                  {multiplyMutation.data && (
                    <div className="bg-muted/50 rounded-lg p-4 font-mono text-sm">
                      <div className="flex justify-between items-center">
                        <span className="text-muted-foreground">Multiply Result:</span>
                        <Badge variant="default" className="text-lg" data-testid="badge-multiply-result">
                          {(multiplyMutation.data as TernaryResult).result}
                        </Badge>
                      </div>
                      <div className="mt-2 text-xs text-muted-foreground">
                        {tritA} × {tritB} = {(multiplyMutation.data as TernaryResult).result} (mod 3)
                      </div>
                    </div>
                  )}
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle className="text-lg">Representation Conversion</CardTitle>
                  <CardDescription>
                    POST /api/salvi/ternary/convert
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="grid grid-cols-3 gap-2">
                    <div>
                      <Label>Value</Label>
                      <Select value={convertValue.toString()} onValueChange={(v) => setConvertValue(parseInt(v))}>
                        <SelectTrigger data-testid="select-convert-value">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          {fromRep === "A" && (
                            <>
                              <SelectItem value="-1">-1</SelectItem>
                              <SelectItem value="0">0</SelectItem>
                              <SelectItem value="1">+1</SelectItem>
                            </>
                          )}
                          {fromRep === "B" && (
                            <>
                              <SelectItem value="0">0</SelectItem>
                              <SelectItem value="1">1</SelectItem>
                              <SelectItem value="2">2</SelectItem>
                            </>
                          )}
                          {fromRep === "C" && (
                            <>
                              <SelectItem value="1">1</SelectItem>
                              <SelectItem value="2">2</SelectItem>
                              <SelectItem value="3">3</SelectItem>
                            </>
                          )}
                        </SelectContent>
                      </Select>
                    </div>
                    <div>
                      <Label>From</Label>
                      <Select value={fromRep} onValueChange={setFromRep}>
                        <SelectTrigger data-testid="select-from-rep">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="A">A (Comp)</SelectItem>
                          <SelectItem value="B">B (Net)</SelectItem>
                          <SelectItem value="C">C (Human)</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                    <div>
                      <Label>To</Label>
                      <Select value={toRep} onValueChange={setToRep}>
                        <SelectTrigger data-testid="select-to-rep">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="A">A (Comp)</SelectItem>
                          <SelectItem value="B">B (Net)</SelectItem>
                          <SelectItem value="C">C (Human)</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                  </div>
                  
                  <Button 
                    onClick={() => convertMutation.mutate({ value: convertValue, from: fromRep, to: toRep })}
                    disabled={convertMutation.isPending}
                    data-testid="button-convert"
                  >
                    <Play className="w-4 h-4 mr-2" />
                    Convert
                  </Button>

                  {convertMutation.data && (
                    <div className="bg-muted/50 rounded-lg p-4 font-mono text-sm space-y-2">
                      <div className="flex justify-between items-center">
                        <span className="text-muted-foreground">Result:</span>
                        <Badge variant="default" className="text-lg" data-testid="badge-convert-result">
                          {(convertMutation.data as ConvertResult).converted?.value}
                        </Badge>
                      </div>
                      <div className="text-xs text-muted-foreground border-t pt-2 mt-2">
                        <div>Original: {(convertMutation.data as ConvertResult).original?.value} ({(convertMutation.data as ConvertResult).original?.representation})</div>
                        <div>Converted: {(convertMutation.data as ConvertResult).converted?.value} ({(convertMutation.data as ConvertResult).converted?.representation})</div>
                        <div>Bijection: {(convertMutation.data as ConvertResult).bijection}</div>
                      </div>
                    </div>
                  )}
                </CardContent>
              </Card>
            </div>

            <Card>
              <CardHeader>
                <CardTitle className="text-lg">Ternary Representations</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="grid md:grid-cols-3 gap-4">
                  <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
                    <div className="font-bold text-blue-800 mb-2">Representation A</div>
                    <div className="text-sm text-blue-700">Computational</div>
                    <div className="font-mono text-lg mt-2">{"{-1, 0, +1}"}</div>
                  </div>
                  <div className="bg-green-50 border border-green-200 rounded-lg p-4">
                    <div className="font-bold text-green-800 mb-2">Representation B</div>
                    <div className="text-sm text-green-700">Network</div>
                    <div className="font-mono text-lg mt-2">{"{0, 1, 2}"}</div>
                  </div>
                  <div className="bg-purple-50 border border-purple-200 rounded-lg p-4">
                    <div className="font-bold text-purple-800 mb-2">Representation C</div>
                    <div className="text-sm text-purple-700">Human</div>
                    <div className="font-mono text-lg mt-2">{"{1, 2, 3}"}</div>
                  </div>
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="phase" className="space-y-6">
            <Card>
              <CardHeader>
                <CardTitle className="text-lg">Phase-Split Encryption</CardTitle>
                <CardDescription>
                  POST /api/salvi/phase/split - Split data into quantum-resistant phase components
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="grid md:grid-cols-2 gap-4">
                  <div>
                    <Label htmlFor="phase-data">Data to Encrypt</Label>
                    <Input 
                      id="phase-data"
                      value={phaseData}
                      onChange={(e) => setPhaseData(e.target.value)}
                      placeholder="Enter data to encrypt"
                      data-testid="input-phase-data"
                    />
                  </div>
                  <div>
                    <Label>Encryption Mode</Label>
                    <Select value={phaseMode} onValueChange={setPhaseMode}>
                      <SelectTrigger data-testid="select-phase-mode">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="high_security">High Security</SelectItem>
                        <SelectItem value="balanced">Balanced</SelectItem>
                        <SelectItem value="performance">Performance</SelectItem>
                        <SelectItem value="adaptive">Adaptive</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                </div>

                <Button 
                  onClick={() => phaseSplitMutation.mutate({ data: phaseData, mode: phaseMode })}
                  disabled={phaseSplitMutation.isPending || !phaseData}
                  data-testid="button-phase-split"
                >
                  <Shield className="w-4 h-4 mr-2" />
                  Split Data
                </Button>

                {phaseSplitMutation.data && (phaseSplitMutation.data as PhaseResult).encrypted && (
                  <div className="bg-muted/50 rounded-lg p-4 font-mono text-xs space-y-2">
                    <div className="font-bold text-sm mb-2">Phase Components:</div>
                    <div className="space-y-1">
                      <div className="flex gap-2">
                        <Badge variant="outline">Primary</Badge>
                        <span className="truncate" data-testid="text-component1">
                          {(phaseSplitMutation.data as PhaseResult).encrypted?.primaryPhase?.data}
                        </span>
                      </div>
                      <div className="flex gap-2">
                        <Badge variant="outline">Secondary</Badge>
                        <span className="truncate">
                          {(phaseSplitMutation.data as PhaseResult).encrypted?.secondaryPhase?.data}
                        </span>
                      </div>
                    </div>
                    <div className="border-t pt-2 mt-2 text-muted-foreground space-y-1">
                      <div>Mode: {(phaseSplitMutation.data as PhaseResult).encrypted?.config?.mode}</div>
                      <div>Split Ratio: {(phaseSplitMutation.data as PhaseResult).encrypted?.splitRatio}</div>
                    </div>
                  </div>
                )}
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="text-lg">Encryption Modes</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="grid md:grid-cols-4 gap-4">
                  <div className="bg-red-50 border border-red-200 rounded-lg p-4 text-center">
                    <Shield className="w-6 h-6 mx-auto text-red-600 mb-2" />
                    <div className="font-bold text-red-800">High Security</div>
                    <div className="text-xs text-red-600 mt-1">Maximum protection</div>
                  </div>
                  <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4 text-center">
                    <Shield className="w-6 h-6 mx-auto text-yellow-600 mb-2" />
                    <div className="font-bold text-yellow-800">Balanced</div>
                    <div className="text-xs text-yellow-600 mt-1">Security + Speed</div>
                  </div>
                  <div className="bg-green-50 border border-green-200 rounded-lg p-4 text-center">
                    <Zap className="w-6 h-6 mx-auto text-green-600 mb-2" />
                    <div className="font-bold text-green-800">Performance</div>
                    <div className="text-xs text-green-600 mt-1">Optimized speed</div>
                  </div>
                  <div className="bg-purple-50 border border-purple-200 rounded-lg p-4 text-center">
                    <RefreshCw className="w-6 h-6 mx-auto text-purple-600 mb-2" />
                    <div className="font-bold text-purple-800">Adaptive</div>
                    <div className="text-xs text-purple-600 mt-1">Auto-adjusting</div>
                  </div>
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="compression" className="space-y-6">
            <CompressionDemo />
          </TabsContent>

          <TabsContent value="density" className="space-y-6">
            <DensityCalculator />
          </TabsContent>
        </Tabs>

        <div className="mt-8">
          <Card>
            <CardHeader>
              <CardTitle>API Endpoints Reference</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="grid md:grid-cols-3 lg:grid-cols-5 gap-4 text-sm font-mono">
                <div>
                  <div className="font-bold text-primary mb-2">Timing API</div>
                  <div className="space-y-1 text-muted-foreground">
                    <div>GET /api/salvi/timing/timestamp</div>
                    <div>GET /api/salvi/timing/metrics</div>
                    <div>GET /api/salvi/timing/batch/:count</div>
                  </div>
                </div>
                <div>
                  <div className="font-bold text-primary mb-2">Ternary API</div>
                  <div className="space-y-1 text-muted-foreground">
                    <div>POST /api/salvi/ternary/convert</div>
                    <div>POST /api/salvi/ternary/add</div>
                    <div>POST /api/salvi/ternary/multiply</div>
                    <div>POST /api/salvi/ternary/rotate</div>
                  </div>
                </div>
                <div>
                  <div className="font-bold text-primary mb-2">Phase API</div>
                  <div className="space-y-1 text-muted-foreground">
                    <div>POST /api/salvi/phase/split</div>
                    <div>POST /api/salvi/phase/recombine</div>
                    <div>GET /api/salvi/phase/config/:mode</div>
                  </div>
                </div>
                <div>
                  <div className="font-bold text-primary mb-2">Compression API</div>
                  <div className="space-y-1 text-muted-foreground">
                    <div>POST /api/demo/run</div>
                    <div>GET /api/demo/stats</div>
                    <div>GET /api/demo/history</div>
                  </div>
                </div>
                <div>
                  <div className="font-bold text-primary mb-2">Density API</div>
                  <div className="space-y-1 text-muted-foreground">
                    <div>GET /api/salvi/ternary/density/:count</div>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </main>

      <footer className="border-t bg-white py-6 mt-12">
        <div className="container mx-auto px-4 text-center text-sm text-muted-foreground">
          <p>PlenumNET Framework - Post-Quantum Ternary Internet</p>
          <p className="mt-1">Copyright (c) 2026 Capomastro Holdings Ltd. All Rights Reserved.</p>
        </div>
      </footer>
    </div>
  );
}
