/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
 * Patent(s) Pending.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * Unauthorized copying, modification, distribution, or use of this file,
 * via any medium, is strictly prohibited without the prior written
 * permission of Capomastro Holdings Ltd.
 *
 * See LICENSE in the repository root for full terms.
 */

import { useState, useEffect } from "react";
import { useQuery, useMutation } from "@tanstack/react-query";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Clock, Calculator, Shield, RefreshCw, Zap, Play, Copy, Check, Database, TrendingUp, Cpu } from "lucide-react";
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
  const [notValue, setNotValue] = useState<number>(1);
  const [xorA, setXorA] = useState<number>(1);
  const [xorB, setXorB] = useState<number>(-1);
  const [rotateValue, setRotateValue] = useState<number>(1);
  const [configMode, setConfigMode] = useState<string>("balanced");
  
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

  const notMutation = useMutation({
    mutationFn: async ({ value }: { value: number }) => {
      const res = await apiRequest("POST", "/api/salvi/ternary/not", { value });
      return res.json();
    },
  });

  const xorMutation = useMutation({
    mutationFn: async ({ a, b }: { a: number; b: number }) => {
      const res = await apiRequest("POST", "/api/salvi/ternary/xor", { a, b });
      return res.json();
    },
  });

  const rotateMutation = useMutation({
    mutationFn: async ({ value }: { value: number }) => {
      const res = await apiRequest("POST", "/api/salvi/ternary/rotate", { value });
      return res.json();
    },
  });

  const phaseRecombineMutation = useMutation({
    mutationFn: async (encrypted: any) => {
      const res = await apiRequest("POST", "/api/salvi/phase/recombine", encrypted);
      return res.json();
    },
  });

  const { data: selfTestData, refetch: refetchSelfTest, isFetching: isSelfTestFetching } = useQuery<any>({
    queryKey: ["/api/salvi/timing/self-test"],
    enabled: false,
  });

  const { data: errorBudgetData, refetch: refetchErrorBudget, isFetching: isErrorBudgetFetching } = useQuery<any>({
    queryKey: ["/api/salvi/timing/error-budget"],
    enabled: false,
  });

  const { data: phaseConfigData, refetch: refetchPhaseConfig, isFetching: isPhaseConfigFetching } = useQuery<any>({
    queryKey: [`/api/salvi/phase/config/${configMode}`],
    enabled: false,
  });

  const { data: phaseRecommendData, refetch: refetchPhaseRecommend, isFetching: isPhaseRecommendFetching } = useQuery<any>({
    queryKey: ["/api/salvi/phase/recommend"],
    enabled: false,
  });

  const { data: vmSpecData, refetch: refetchVmSpec, isFetching: isVmSpecFetching } = useQuery<any>({
    queryKey: ["/api/salvi/vm/spec"],
    enabled: false,
  });

  const { data: vmConformanceData, refetch: refetchVmConformance, isFetching: isVmConformanceFetching } = useQuery<any>({
    queryKey: ["/api/salvi/vm/conformance"],
    enabled: false,
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
          <TabsList className="grid w-full grid-cols-6 max-w-4xl">
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
            <TabsTrigger value="vm" data-testid="tab-vm" className="flex items-center gap-2">
              <Cpu className="w-4 h-4" />
              VM
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

            <div className="grid md:grid-cols-2 gap-6">
              <Card>
                <CardHeader>
                  <CardTitle className="text-lg">Timing Self-Test</CardTitle>
                  <CardDescription>
                    GET /api/salvi/timing/self-test - Run timing subsystem diagnostics
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <Button
                    onClick={() => refetchSelfTest()}
                    disabled={isSelfTestFetching}
                    data-testid="button-timing-self-test"
                  >
                    {isSelfTestFetching ? (
                      <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
                    ) : (
                      <Play className="w-4 h-4 mr-2" />
                    )}
                    Run Self-Test
                  </Button>

                  {selfTestData && (
                    <div className="bg-muted/50 rounded-lg p-4 font-mono text-sm space-y-2" data-testid="text-self-test-results">
                      {selfTestData.tests ? (
                        Object.entries(selfTestData.tests).map(([key, val]: [string, any]) => (
                          <div key={key} className="flex justify-between items-center">
                            <span className="text-muted-foreground capitalize">{key.replace(/_/g, ' ')}:</span>
                            <Badge variant={val === true || val === 'pass' ? 'default' : 'destructive'} data-testid={`badge-selftest-${key}`}>
                              {String(val)}
                            </Badge>
                          </div>
                        ))
                      ) : (
                        <pre className="text-xs whitespace-pre-wrap">{JSON.stringify(selfTestData, null, 2)}</pre>
                      )}
                    </div>
                  )}
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle className="text-lg">Error Budget</CardTitle>
                  <CardDescription>
                    GET /api/salvi/timing/error-budget - Precision error budget breakdown
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <Button
                    onClick={() => refetchErrorBudget()}
                    disabled={isErrorBudgetFetching}
                    data-testid="button-error-budget"
                  >
                    {isErrorBudgetFetching ? (
                      <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
                    ) : (
                      <Play className="w-4 h-4 mr-2" />
                    )}
                    Get Error Budget
                  </Button>

                  {errorBudgetData && (
                    <div className="bg-muted/50 rounded-lg p-4 font-mono text-sm space-y-2" data-testid="text-error-budget-results">
                      {errorBudgetData.budget ? (
                        Object.entries(errorBudgetData.budget).map(([key, val]: [string, any]) => (
                          <div key={key} className="flex justify-between items-center">
                            <span className="text-muted-foreground capitalize">{key.replace(/_/g, ' ')}:</span>
                            <span className="text-foreground">{typeof val === 'object' ? JSON.stringify(val) : String(val)}</span>
                          </div>
                        ))
                      ) : (
                        <pre className="text-xs whitespace-pre-wrap">{JSON.stringify(errorBudgetData, null, 2)}</pre>
                      )}
                    </div>
                  )}
                </CardContent>
              </Card>
            </div>
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

            <div className="grid md:grid-cols-3 gap-6">
              <Card>
                <CardHeader>
                  <CardTitle className="text-lg">Ternary NOT (Negation)</CardTitle>
                  <CardDescription>
                    POST /api/salvi/ternary/not
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div>
                    <Label>Trit Value</Label>
                    <Select value={notValue.toString()} onValueChange={(v) => setNotValue(parseInt(v))}>
                      <SelectTrigger data-testid="select-not-value">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="-1">-1</SelectItem>
                        <SelectItem value="0">0</SelectItem>
                        <SelectItem value="1">+1</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>

                  <Button
                    onClick={() => notMutation.mutate({ value: notValue })}
                    disabled={notMutation.isPending}
                    data-testid="button-ternary-not"
                  >
                    <Play className="w-4 h-4 mr-2" />
                    Negate
                  </Button>

                  {notMutation.data && (
                    <div className="bg-muted/50 rounded-lg p-4 font-mono text-sm">
                      <div className="flex justify-between items-center">
                        <span className="text-muted-foreground">NOT Result:</span>
                        <Badge variant="default" className="text-lg" data-testid="badge-not-result">
                          {(notMutation.data as TernaryResult).result}
                        </Badge>
                      </div>
                      <div className="mt-2 text-xs text-muted-foreground">
                        NOT({notValue}) = {(notMutation.data as TernaryResult).result}
                      </div>
                    </div>
                  )}
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle className="text-lg">Ternary XOR</CardTitle>
                  <CardDescription>
                    POST /api/salvi/ternary/xor
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <Label>Trit A</Label>
                      <Select value={xorA.toString()} onValueChange={(v) => setXorA(parseInt(v))}>
                        <SelectTrigger data-testid="select-xor-a">
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
                      <Label>Trit B</Label>
                      <Select value={xorB.toString()} onValueChange={(v) => setXorB(parseInt(v))}>
                        <SelectTrigger data-testid="select-xor-b">
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

                  <Button
                    onClick={() => xorMutation.mutate({ a: xorA, b: xorB })}
                    disabled={xorMutation.isPending}
                    data-testid="button-ternary-xor"
                  >
                    <Play className="w-4 h-4 mr-2" />
                    XOR
                  </Button>

                  {xorMutation.data && (
                    <div className="bg-muted/50 rounded-lg p-4 font-mono text-sm">
                      <div className="flex justify-between items-center">
                        <span className="text-muted-foreground">XOR Result:</span>
                        <Badge variant="default" className="text-lg" data-testid="badge-xor-result">
                          {(xorMutation.data as TernaryResult).result}
                        </Badge>
                      </div>
                      <div className="mt-2 text-xs text-muted-foreground">
                        {xorA} XOR {xorB} = {(xorMutation.data as TernaryResult).result}
                      </div>
                    </div>
                  )}
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle className="text-lg">Ternary Rotate</CardTitle>
                  <CardDescription>
                    POST /api/salvi/ternary/rotate
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div>
                    <Label>Trit Value</Label>
                    <Select value={rotateValue.toString()} onValueChange={(v) => setRotateValue(parseInt(v))}>
                      <SelectTrigger data-testid="select-rotate-value">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="-1">-1</SelectItem>
                        <SelectItem value="0">0</SelectItem>
                        <SelectItem value="1">+1</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>

                  <Button
                    onClick={() => rotateMutation.mutate({ value: rotateValue })}
                    disabled={rotateMutation.isPending}
                    data-testid="button-ternary-rotate"
                  >
                    <Play className="w-4 h-4 mr-2" />
                    Rotate
                  </Button>

                  {rotateMutation.data && (
                    <div className="bg-muted/50 rounded-lg p-4 font-mono text-sm">
                      <div className="flex justify-between items-center">
                        <span className="text-muted-foreground">Rotate Result:</span>
                        <Badge variant="default" className="text-lg" data-testid="badge-rotate-result">
                          {(rotateMutation.data as TernaryResult).result}
                        </Badge>
                      </div>
                      <div className="mt-2 text-xs text-muted-foreground">
                        ROTATE({rotateValue}) = {(rotateMutation.data as TernaryResult).result}
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

            <div className="grid md:grid-cols-2 gap-6">
              <Card>
                <CardHeader>
                  <CardTitle className="text-lg">Phase Recombine</CardTitle>
                  <CardDescription>
                    POST /api/salvi/phase/recombine - Recombine phase components
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  {phaseSplitMutation.data && (phaseSplitMutation.data as PhaseResult).encrypted ? (
                    <>
                      <Button
                        onClick={() => phaseRecombineMutation.mutate((phaseSplitMutation.data as PhaseResult).encrypted)}
                        disabled={phaseRecombineMutation.isPending}
                        data-testid="button-phase-recombine"
                      >
                        {phaseRecombineMutation.isPending ? (
                          <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
                        ) : (
                          <Shield className="w-4 h-4 mr-2" />
                        )}
                        Recombine
                      </Button>

                      {phaseRecombineMutation.data && (
                        <div className="bg-muted/50 rounded-lg p-4 font-mono text-sm space-y-2" data-testid="text-recombine-results">
                          {(phaseRecombineMutation.data as any).decrypted && (
                            <div className="flex justify-between items-center">
                              <span className="text-muted-foreground">Decrypted:</span>
                              <span className="text-foreground" data-testid="text-recombine-decrypted">{(phaseRecombineMutation.data as any).decrypted}</span>
                            </div>
                          )}
                          {(phaseRecombineMutation.data as any).phaseAlignment !== undefined && (
                            <div className="flex justify-between items-center">
                              <span className="text-muted-foreground">Phase Alignment:</span>
                              <Badge variant="default" data-testid="badge-phase-alignment">{String((phaseRecombineMutation.data as any).phaseAlignment)}</Badge>
                            </div>
                          )}
                          {(phaseRecombineMutation.data as any).timestampValid !== undefined && (
                            <div className="flex justify-between items-center">
                              <span className="text-muted-foreground">Timestamp Valid:</span>
                              <Badge variant={(phaseRecombineMutation.data as any).timestampValid ? 'default' : 'destructive'} data-testid="badge-timestamp-valid">
                                {String((phaseRecombineMutation.data as any).timestampValid)}
                              </Badge>
                            </div>
                          )}
                          {!(phaseRecombineMutation.data as any).decrypted && (
                            <pre className="text-xs whitespace-pre-wrap">{JSON.stringify(phaseRecombineMutation.data, null, 2)}</pre>
                          )}
                        </div>
                      )}
                    </>
                  ) : (
                    <div className="text-sm text-muted-foreground py-4">
                      Run a Phase Split first to enable recombination.
                    </div>
                  )}
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle className="text-lg">Phase Recommend</CardTitle>
                  <CardDescription>
                    GET /api/salvi/phase/recommend - Get recommended mode
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <Button
                    onClick={() => refetchPhaseRecommend()}
                    disabled={isPhaseRecommendFetching}
                    data-testid="button-phase-recommend"
                  >
                    {isPhaseRecommendFetching ? (
                      <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
                    ) : (
                      <Play className="w-4 h-4 mr-2" />
                    )}
                    Get Recommendation
                  </Button>

                  {phaseRecommendData && (
                    <div className="bg-muted/50 rounded-lg p-4 font-mono text-sm space-y-2" data-testid="text-phase-recommend-results">
                      <pre className="text-xs whitespace-pre-wrap">{JSON.stringify(phaseRecommendData, null, 2)}</pre>
                    </div>
                  )}
                </CardContent>
              </Card>
            </div>

            <Card>
              <CardHeader>
                <CardTitle className="text-lg">Phase Config Viewer</CardTitle>
                <CardDescription>
                  GET /api/salvi/phase/config/:mode - View encryption mode configuration
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex flex-wrap gap-4 items-end">
                  <div className="w-48">
                    <Label>Mode</Label>
                    <Select value={configMode} onValueChange={setConfigMode}>
                      <SelectTrigger data-testid="select-config-mode">
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
                  <Button
                    onClick={() => refetchPhaseConfig()}
                    disabled={isPhaseConfigFetching}
                    data-testid="button-phase-config"
                  >
                    {isPhaseConfigFetching ? (
                      <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
                    ) : (
                      <Play className="w-4 h-4 mr-2" />
                    )}
                    Fetch Config
                  </Button>
                </div>

                {phaseConfigData && (
                  <div className="bg-muted/50 rounded-lg p-4 font-mono text-sm space-y-2" data-testid="text-phase-config-results">
                    {phaseConfigData.config ? (
                      Object.entries(phaseConfigData.config).map(([key, val]: [string, any]) => (
                        <div key={key} className="flex justify-between items-center">
                          <span className="text-muted-foreground">{key}:</span>
                          <span className="text-foreground">{typeof val === 'object' ? JSON.stringify(val) : String(val)}</span>
                        </div>
                      ))
                    ) : (
                      <pre className="text-xs whitespace-pre-wrap">{JSON.stringify(phaseConfigData, null, 2)}</pre>
                    )}
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

          <TabsContent value="vm" className="space-y-6">
            <div className="grid md:grid-cols-2 gap-6">
              <Card>
                <CardHeader>
                  <CardTitle className="text-lg">VM Specification</CardTitle>
                  <CardDescription>
                    GET /api/salvi/vm/spec - TVM instruction set specification
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <Button
                    onClick={() => refetchVmSpec()}
                    disabled={isVmSpecFetching}
                    data-testid="button-vm-spec"
                  >
                    {isVmSpecFetching ? (
                      <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
                    ) : (
                      <Cpu className="w-4 h-4 mr-2" />
                    )}
                    Fetch VM Spec
                  </Button>

                  {vmSpecData && (
                    <div className="bg-muted/50 rounded-lg p-4 font-mono text-sm space-y-2" data-testid="text-vm-spec-results">
                      {vmSpecData.spec ? (
                        <>
                          {vmSpecData.spec.opcodeCount !== undefined && (
                            <div className="flex justify-between items-center">
                              <span className="text-muted-foreground">Opcode Count:</span>
                              <Badge variant="default" data-testid="badge-opcode-count">{vmSpecData.spec.opcodeCount}</Badge>
                            </div>
                          )}
                          {vmSpecData.spec.registers !== undefined && (
                            <div className="flex justify-between items-center">
                              <span className="text-muted-foreground">Registers:</span>
                              <span className="text-foreground" data-testid="text-vm-registers">{vmSpecData.spec.registers}</span>
                            </div>
                          )}
                          {vmSpecData.spec.encodingModes !== undefined && (
                            <div className="flex justify-between items-center">
                              <span className="text-muted-foreground">Encoding Modes:</span>
                              <span className="text-foreground" data-testid="text-vm-encoding">{Array.isArray(vmSpecData.spec.encodingModes) ? vmSpecData.spec.encodingModes.join(', ') : String(vmSpecData.spec.encodingModes)}</span>
                            </div>
                          )}
                          {Object.entries(vmSpecData.spec).filter(([k]) => !['opcodeCount', 'registers', 'encodingModes'].includes(k)).map(([key, val]: [string, any]) => (
                            <div key={key} className="flex justify-between items-center">
                              <span className="text-muted-foreground">{key}:</span>
                              <span className="text-foreground">{typeof val === 'object' ? JSON.stringify(val) : String(val)}</span>
                            </div>
                          ))}
                        </>
                      ) : (
                        <pre className="text-xs whitespace-pre-wrap">{JSON.stringify(vmSpecData, null, 2)}</pre>
                      )}
                    </div>
                  )}
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle className="text-lg">VM Conformance</CardTitle>
                  <CardDescription>
                    GET /api/salvi/vm/conformance - Run TVM conformance tests
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <Button
                    onClick={() => refetchVmConformance()}
                    disabled={isVmConformanceFetching}
                    data-testid="button-vm-conformance"
                  >
                    {isVmConformanceFetching ? (
                      <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
                    ) : (
                      <Cpu className="w-4 h-4 mr-2" />
                    )}
                    Run Conformance
                  </Button>

                  {vmConformanceData && (
                    <div className="bg-muted/50 rounded-lg p-4 font-mono text-sm space-y-2" data-testid="text-vm-conformance-results">
                      {vmConformanceData.results ? (
                        Object.entries(vmConformanceData.results).map(([key, val]: [string, any]) => (
                          <div key={key} className="flex justify-between items-center">
                            <span className="text-muted-foreground capitalize">{key.replace(/_/g, ' ')}:</span>
                            <Badge variant={val === true || val === 'pass' ? 'default' : 'destructive'} data-testid={`badge-conformance-${key}`}>
                              {String(val)}
                            </Badge>
                          </div>
                        ))
                      ) : (
                        <pre className="text-xs whitespace-pre-wrap">{JSON.stringify(vmConformanceData, null, 2)}</pre>
                      )}
                    </div>
                  )}
                </CardContent>
              </Card>
            </div>
          </TabsContent>
        </Tabs>

        <div className="mt-8 space-y-6">
          <h2 className="text-2xl font-bold" data-testid="text-api-reference-title">API Endpoints Reference</h2>
          <p className="text-muted-foreground text-sm">Complete reference for all public PlenumNET Framework API endpoints. Admin endpoints require authentication.</p>

          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-base flex items-center gap-2">
                  <Clock className="w-4 h-4 text-primary" />
                  Timing API
                </CardTitle>
                <CardDescription>Femtosecond-precision timestamps and Salvi Epoch</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  {[
                    { method: "GET", path: "/api/salvi/timing/timestamp", desc: "Current femtosecond timestamp" },
                    { method: "GET", path: "/api/salvi/timing/metrics", desc: "Clock source and sync status" },
                    { method: "GET", path: "/api/salvi/timing/batch/:count", desc: "Batch timestamp generation" },
                    { method: "GET", path: "/api/salvi/timing/self-test", desc: "Timing subsystem self-test" },
                    { method: "GET", path: "/api/salvi/timing/error-budget", desc: "Precision error budget" },
                  ].map((ep) => (
                    <div key={ep.path} className="flex items-start gap-2">
                      <Badge variant="outline" className="shrink-0 text-xs">{ep.method}</Badge>
                      <div className="min-w-0">
                        <code className="text-xs break-all">{ep.path}</code>
                        <div className="text-xs text-muted-foreground">{ep.desc}</div>
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-base flex items-center gap-2">
                  <Clock className="w-4 h-4 text-primary" />
                  Calendar / Epoch API
                </CardTitle>
                <CardDescription>24 ancient calendar synchronization endpoints</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  {[
                    { method: "GET", path: "/api/salvi/timing/epoch/anchors", desc: "All epoch anchor points" },
                    { method: "GET", path: "/api/salvi/timing/epoch/calendars", desc: "All 24 calendar conversions" },
                    { method: "GET", path: "/api/salvi/timing/epoch/calendars/mayan", desc: "Mayan Long Count" },
                    { method: "GET", path: "/api/salvi/timing/epoch/calendars/hebrew", desc: "Hebrew calendar" },
                    { method: "GET", path: "/api/salvi/timing/epoch/calendars/chinese", desc: "Chinese calendar" },
                    { method: "GET", path: "/api/salvi/timing/epoch/calendars/vedic", desc: "Vedic calendar" },
                    { method: "GET", path: "/api/salvi/timing/epoch/calendars/egyptian", desc: "Egyptian calendar" },
                    { method: "GET", path: "/api/salvi/timing/epoch/calendars/julian-day", desc: "Julian Day Number" },
                    { method: "GET", path: "/api/salvi/timing/epoch/calendars/islamic", desc: "Islamic Hijri" },
                    { method: "GET", path: "/api/salvi/timing/epoch/calendars/byzantine", desc: "Byzantine calendar" },
                    { method: "GET", path: "/api/salvi/timing/epoch/calendars/thirteen-moon", desc: "13-Moon Harmonic" },
                    { method: "GET", path: "/api/salvi/timing/epoch/calendars/persian", desc: "Persian/Solar Hijri" },
                    { method: "GET", path: "/api/salvi/timing/epoch/calendars/ethiopian", desc: "Ethiopian/Ge'ez" },
                    { method: "GET", path: "/api/salvi/timing/epoch/calendars/coptic", desc: "Coptic" },
                    { method: "GET", path: "/api/salvi/timing/epoch/calendars/japanese", desc: "Japanese Imperial" },
                    { method: "GET", path: "/api/salvi/timing/epoch/calendars/korean", desc: "Korean Dangun Era" },
                    { method: "GET", path: "/api/salvi/timing/epoch/calendars/thai", desc: "Thai Buddhist Era" },
                    { method: "GET", path: "/api/salvi/timing/epoch/calendars/indian-saka", desc: "Indian National/Saka" },
                    { method: "GET", path: "/api/salvi/timing/epoch/calendars/tibetan", desc: "Tibetan Rabjung" },
                    { method: "GET", path: "/api/salvi/timing/epoch/calendars/aztec", desc: "Aztec Tonalpohualli" },
                    { method: "GET", path: "/api/salvi/timing/epoch/calendars/roman", desc: "Roman Ab Urbe Condita" },
                    { method: "GET", path: "/api/salvi/timing/epoch/calendars/bengali", desc: "Bengali/Bangla" },
                    { method: "GET", path: "/api/salvi/timing/epoch/calendars/berber", desc: "Berber/Amazigh" },
                    { method: "GET", path: "/api/salvi/timing/epoch/calendars/balinese", desc: "Balinese Pawukon" },
                    { method: "GET", path: "/api/salvi/timing/epoch/calendars/zoroastrian", desc: "Zoroastrian Fasli" },
                    { method: "GET", path: "/api/salvi/timing/epoch/calendars/aboriginal", desc: "Aboriginal Australian" },
                  ].map((ep) => (
                    <div key={ep.path} className="flex items-start gap-2">
                      <Badge variant="outline" className="shrink-0 text-xs">{ep.method}</Badge>
                      <div className="min-w-0">
                        <code className="text-xs break-all">{ep.path}</code>
                        <div className="text-xs text-muted-foreground">{ep.desc}</div>
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-base flex items-center gap-2">
                  <Calculator className="w-4 h-4 text-primary" />
                  Ternary Operations API
                </CardTitle>
                <CardDescription>GF(3) arithmetic, conversion, and density analysis</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  {[
                    { method: "POST", path: "/api/salvi/ternary/convert", desc: "Convert between representations A/B/C" },
                    { method: "POST", path: "/api/salvi/ternary/add", desc: "GF(3) addition" },
                    { method: "POST", path: "/api/salvi/ternary/multiply", desc: "GF(3) multiplication" },
                    { method: "POST", path: "/api/salvi/ternary/rotate", desc: "Ternary rotation" },
                    { method: "POST", path: "/api/salvi/ternary/not", desc: "Ternary negation" },
                    { method: "POST", path: "/api/salvi/ternary/xor", desc: "Ternary XOR (field addition)" },
                    { method: "POST", path: "/api/salvi/ternary/batch", desc: "Batch ternary operations" },
                    { method: "GET", path: "/api/salvi/ternary/density/:tritCount", desc: "Information density for N trits" },
                    { method: "GET", path: "/api/salvi/ternary/density-benchmark", desc: "Density benchmark suite" },
                  ].map((ep) => (
                    <div key={ep.path} className="flex items-start gap-2">
                      <Badge variant={ep.method === "POST" ? "secondary" : "outline"} className="shrink-0 text-xs">{ep.method}</Badge>
                      <div className="min-w-0">
                        <code className="text-xs break-all">{ep.path}</code>
                        <div className="text-xs text-muted-foreground">{ep.desc}</div>
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-base flex items-center gap-2">
                  <Shield className="w-4 h-4 text-primary" />
                  Phase Encryption API
                </CardTitle>
                <CardDescription>Quantum-resistant phase-split encryption</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  {[
                    { method: "POST", path: "/api/salvi/phase/split", desc: "Split data into phase components" },
                    { method: "POST", path: "/api/salvi/phase/recombine", desc: "Recombine phase components" },
                    { method: "GET", path: "/api/salvi/phase/config/:mode", desc: "Encryption mode configuration" },
                    { method: "GET", path: "/api/salvi/phase/recommend", desc: "Recommended mode for use case" },
                  ].map((ep) => (
                    <div key={ep.path} className="flex items-start gap-2">
                      <Badge variant={ep.method === "POST" ? "secondary" : "outline"} className="shrink-0 text-xs">{ep.method}</Badge>
                      <div className="min-w-0">
                        <code className="text-xs break-all">{ep.path}</code>
                        <div className="text-xs text-muted-foreground">{ep.desc}</div>
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-base flex items-center gap-2">
                  <Database className="w-4 h-4 text-primary" />
                  Compression Demo API
                </CardTitle>
                <CardDescription>Live ternary compression demonstrations</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  {[
                    { method: "POST", path: "/api/demo/run", desc: "Run compression on dataset" },
                    { method: "POST", path: "/api/demo/upload", desc: "Upload custom data for compression" },
                    { method: "GET", path: "/api/demo/stats", desc: "Aggregated compression statistics" },
                    { method: "GET", path: "/api/demo/session/:sessionId", desc: "Session details" },
                    { method: "GET", path: "/api/demo/data/:sessionId", desc: "Session data export" },
                    { method: "GET", path: "/api/demo/history", desc: "Compression run history" },
                    { method: "GET", path: "/api/demo/files", desc: "Available demo files" },
                  ].map((ep) => (
                    <div key={ep.path} className="flex items-start gap-2">
                      <Badge variant={ep.method === "POST" ? "secondary" : "outline"} className="shrink-0 text-xs">{ep.method}</Badge>
                      <div className="min-w-0">
                        <code className="text-xs break-all">{ep.path}</code>
                        <div className="text-xs text-muted-foreground">{ep.desc}</div>
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-base flex items-center gap-2">
                  <Zap className="w-4 h-4 text-primary" />
                  VM, Docs & Whitepapers API
                </CardTitle>
                <CardDescription>Virtual machine specs, documentation, and whitepapers</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  {[
                    { method: "GET", path: "/api/salvi/vm/spec", desc: "TVM 176-opcode ISA v2.1 instruction set spec" },
                    { method: "GET", path: "/api/salvi/vm/conformance", desc: "TVM conformance test suite" },
                    { method: "GET", path: "/api/salvi/docs", desc: "Documentation index" },
                    { method: "GET", path: "/api/whitepapers", desc: "List all whitepapers" },
                    { method: "GET", path: "/api/whitepapers/active", desc: "Active whitepapers only" },
                    { method: "GET", path: "/api/whitepapers/:id", desc: "Whitepaper by ID" },
                    { method: "POST", path: "/api/whitepapers", desc: "Create whitepaper (admin)" },
                  ].map((ep) => (
                    <div key={ep.path} className="flex items-start gap-2">
                      <Badge variant={ep.method === "POST" ? "secondary" : "outline"} className="shrink-0 text-xs">{ep.method}</Badge>
                      <div className="min-w-0">
                        <code className="text-xs break-all">{ep.path}</code>
                        <div className="text-xs text-muted-foreground">{ep.desc}</div>
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-base flex items-center gap-2">
                  <Database className="w-4 h-4 text-primary" />
                  Compression DB API
                </CardTitle>
                <CardDescription>Compression with database storage</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  {[
                    { method: "POST", path: "/api/compression/file", desc: "Compress/decompress file" },
                    { method: "POST", path: "/api/compression/decompress", desc: "Decompress data" },
                    { method: "POST", path: "/api/compression/db/store", desc: "Store compressed data" },
                    { method: "GET", path: "/api/compression/db/retrieve/:id", desc: "Retrieve compressed document" },
                    { method: "GET", path: "/api/compression/db/documents", desc: "List compressed documents" },
                    { method: "GET", path: "/api/compression/db/raw/:id", desc: "Raw stored data" },
                    { method: "DELETE", path: "/api/compression/db/documents/:id", desc: "Delete document" },
                  ].map((ep) => (
                    <div key={ep.method + ep.path} className="flex items-start gap-2">
                      <Badge variant={ep.method === "POST" ? "secondary" : ep.method === "DELETE" ? "destructive" : "outline"} className="shrink-0 text-xs">{ep.method}</Badge>
                      <div className="min-w-0">
                        <code className="text-xs break-all">{ep.path}</code>
                        <div className="text-xs text-muted-foreground">{ep.desc}</div>
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-base flex items-center gap-2">
                  <Shield className="w-4 h-4 text-primary" />
                  Legal & Auth API
                </CardTitle>
                <CardDescription>Legal documents, authentication, and developer signup</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  {[
                    { method: "GET", path: "/api/legal/:type", desc: "Legal documents (terms, privacy, security, aup)" },
                    { method: "GET", path: "/api/user/admin-status", desc: "Check admin status" },
                    { method: "POST", path: "/api/developer-signup", desc: "Developer waitlist signup" },
                    { method: "GET", path: "/api/developer-signup/count", desc: "Waitlist count" },
                  ].map((ep) => (
                    <div key={ep.method + ep.path} className="flex items-start gap-2">
                      <Badge variant={ep.method === "POST" ? "secondary" : "outline"} className="shrink-0 text-xs">{ep.method}</Badge>
                      <div className="min-w-0">
                        <code className="text-xs break-all">{ep.path}</code>
                        <div className="text-xs text-muted-foreground">{ep.desc}</div>
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          </div>

          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="text-sm text-muted-foreground">Admin-Only Endpoints</CardTitle>
              <CardDescription>These endpoints require authentication and admin privileges</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
                <div>
                  <div className="font-medium text-sm mb-2">GitHub Integration</div>
                  <div className="space-y-1 text-muted-foreground">
                    {[
                      { method: "POST", path: "/api/github/token" },
                      { method: "GET", path: "/api/github/status" },
                      { method: "GET", path: "/api/github/repos/:owner/:repo/branches" },
                      { method: "GET", path: "/api/github/repos/:owner/:repo/contents" },
                      { method: "GET", path: "/api/github/file/:owner/:repo" },
                      { method: "PUT", path: "/api/github/file/:owner/:repo" },
                      { method: "DELETE", path: "/api/github/file/:owner/:repo" },
                      { method: "POST", path: "/api/github/push-workflows/:owner/:repo" },
                      { method: "POST", path: "/api/github/push-batch/:owner/:repo" },
                    ].map((ep) => (
                      <div key={ep.method + ep.path} className="flex items-start gap-2">
                        <Badge variant={ep.method === "GET" ? "outline" : ep.method === "DELETE" ? "destructive" : "secondary"} className="shrink-0 text-xs">{ep.method}</Badge>
                        <code className="text-xs break-all">{ep.path}</code>
                      </div>
                    ))}
                  </div>
                </div>
                <div>
                  <div className="font-medium text-sm mb-2">Kong Konnect Gateway</div>
                  <div className="space-y-1 text-muted-foreground">
                    {[
                      { method: "GET", path: "/api/kong/status" },
                      { method: "GET", path: "/api/kong/organization" },
                      { method: "GET", path: "/api/kong/control-planes" },
                      { method: "GET", path: "/api/kong/control-planes/:cpId/services" },
                      { method: "GET", path: "/api/kong/control-planes/:cpId/routes" },
                      { method: "GET", path: "/api/kong/control-planes/:cpId/plugins" },
                      { method: "POST", path: "/api/kong/control-planes/:cpId/services" },
                      { method: "POST", path: "/api/kong/.../routes" },
                      { method: "POST", path: "/api/kong/.../plugins" },
                    ].map((ep, i) => (
                      <div key={i} className="flex items-start gap-2">
                        <Badge variant={ep.method === "GET" ? "outline" : "secondary"} className="shrink-0 text-xs">{ep.method}</Badge>
                        <code className="text-xs break-all">{ep.path}</code>
                      </div>
                    ))}
                  </div>
                </div>
                <div>
                  <div className="font-medium text-sm mb-2">Deployment & Admin</div>
                  <div className="space-y-1 text-muted-foreground">
                    {[
                      { method: "POST", path: "/api/kong/.../sync-plenumnet" },
                      { method: "POST", path: "/api/kong/save-to-github" },
                      { method: "GET", path: "/api/kong/.../deploy-instructions" },
                      { method: "POST", path: "/api/kong/.../generate-deployment" },
                      { method: "POST", path: "/api/kong/.../deploy-to-cloud" },
                      { method: "GET", path: "/api/kong/config" },
                      { method: "GET", path: "/api/admin/developer-signups" },
                      { method: "DELETE", path: "/api/admin/developer-signups/:id" },
                    ].map((ep, i) => (
                      <div key={i} className="flex items-start gap-2">
                        <Badge variant={ep.method === "GET" ? "outline" : ep.method === "DELETE" ? "destructive" : "secondary"} className="shrink-0 text-xs">{ep.method}</Badge>
                        <code className="text-xs break-all">{ep.path}</code>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </main>

      <footer className="border-t bg-background py-6 mt-12">
        <div className="container mx-auto px-4 text-center text-sm text-muted-foreground">
          <p>PlenumNET Framework - Post-Quantum Ternary Internet</p>
          <p className="mt-1">Copyright (c) 2026 Capomastro Holdings Ltd. All Rights Reserved.</p>
        </div>
      </footer>
    </div>
  );
}
