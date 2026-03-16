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

import { useState } from "react";
import { useQuery, useMutation } from "@tanstack/react-query";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Switch } from "@/components/ui/switch";
import { FileUp, FileDown, Database, Shield, RefreshCw, Play, Download, Upload, Trash2, Eye, Archive } from "lucide-react";
import { apiRequest, queryClient } from "@/lib/queryClient";
import { useToast } from "@/hooks/use-toast";

interface TtcMetadata {
  engine: 'ttc-native' | 'ttc-ts-fallback';
  version: string;
  level: number;
  levelName: string;
  modeName: string;
  crc32: number;
  avgTau: number;
  avgDelta: number;
  predominantBase: number;
  adaptiveRepUsed: boolean;
  gf3Representation?: 'balanced' | 'unsigned' | 'native';
}

interface TtcDecompressMetadata {
  engine: 'ttc-native' | 'ttc-ts-fallback';
  version: string;
  level: number | null;
  levelName: string | null;
  crc32Verified: boolean;
  originalFileName: string | null;
  gf3Representation?: 'balanced' | 'unsigned' | 'native';
}

interface FileCompressionResult {
  fileName: string;
  originalSize: number;
  compressedSize: number;
  compressionRatio: string;
  encrypted: boolean;
  encryptionMode?: string;
  processingTimeMs: string;
  data: ArrayBuffer;
  ttcMetadata: TtcMetadata;
}

interface FileDecompressionResult {
  originalFileName: string;
  originalSize: number;
  compressedSize: number;
  compressionRatio: string;
  wasEncrypted: boolean;
  processingTimeMs: string;
  data: ArrayBuffer;
  ttcMetadata: TtcDecompressMetadata;
}

interface DbDocument {
  id: number;
  title: string;
  isCompressed: boolean;
  isEncrypted: boolean;
  encryptionMode?: string;
  originalSizeBytes: number;
  storedSizeBytes: number;
  compressionRatio: number;
  createdAt: string;
}

function parseTtcHeaders(headers: Headers): {
  originalSize: number;
  compressedSize: number;
  compressionRatio: string;
  engine: 'ttc-native' | 'legacy-zlib';
  mode: string;
  level: number;
  levelName: string;
  version: string;
  crc32: number;
  encrypted: boolean;
  processingTimeMs: string;
  predominantBase: number;
  avgTau: number;
  avgDelta: number;
  adaptiveRepUsed: boolean;
  originalFileName: string;
  wasEncrypted: boolean;
  crc32Verified: boolean;
  gf3Representation: string;
} {
  return {
    originalSize: parseInt(headers.get('X-TTC-Original-Size') || '0', 10),
    compressedSize: parseInt(headers.get('X-TTC-Compressed-Size') || '0', 10),
    compressionRatio: headers.get('X-TTC-Compression-Ratio') || '0',
    engine: (headers.get('X-TTC-Engine') || 'ttc-ts-fallback') as 'ttc-native' | 'ttc-ts-fallback',
    mode: headers.get('X-TTC-Mode') || 'BASIC',
    level: parseInt(headers.get('X-TTC-Level') || '5', 10),
    levelName: headers.get('X-TTC-Level-Name') || '',
    version: headers.get('X-TTC-Version') || '1.0',
    crc32: parseInt(headers.get('X-TTC-CRC32') || '0', 10),
    encrypted: headers.get('X-TTC-Encrypted') === 'true',
    processingTimeMs: headers.get('X-TTC-Processing-Ms') || '0',
    predominantBase: parseInt(headers.get('X-TTC-Predominant-Base') || '3', 10),
    avgTau: parseFloat(headers.get('X-TTC-Avg-Tau') || '0'),
    avgDelta: parseFloat(headers.get('X-TTC-Avg-Delta') || '0'),
    adaptiveRepUsed: headers.get('X-TTC-Adaptive-Rep') === 'true',
    originalFileName: headers.get('X-TTC-Original-Filename') || '',
    wasEncrypted: headers.get('X-TTC-Was-Encrypted') === 'true',
    crc32Verified: headers.get('X-TTC-CRC32-Verified') !== 'false',
    gf3Representation: headers.get('X-TTC-GF3-Rep') || 'balanced',
  };
}

function FileCompressionTab() {
  const [fileName, setFileName] = useState("");
  const [fileContent, setFileContent] = useState("");
  const [encrypt, setEncrypt] = useState(false);
  const [encryptionMode, setEncryptionMode] = useState("balanced");
  const [compressResult, setCompressResult] = useState<FileCompressionResult | null>(null);
  const [decompressResult, setDecompressResult] = useState<FileDecompressionResult | null>(null);
  const [ternFileBuffer, setTernFileBuffer] = useState<ArrayBuffer | null>(null);
  const { toast } = useToast();

  const compressMutation = useMutation({
    mutationFn: async () => {
      const bytes = fileBytes || new TextEncoder().encode(fileContent);
      const headers: Record<string, string> = {
        'Content-Type': 'application/octet-stream',
        'X-TTC-Filename': fileName || 'untitled.txt',
      };
      if (encrypt) {
        headers['X-TTC-Encrypt'] = 'true';
        headers['X-TTC-Encryption-Mode'] = encryptionMode;
      }
      const res = await fetch('/api/compression/file', {
        method: 'POST',
        headers,
        body: bytes,
        credentials: 'include',
      });
      if (!res.ok) {
        const err = await res.json().catch(() => ({ error: 'Compression failed' }));
        throw new Error(err.error || 'Compression failed');
      }
      const h = parseTtcHeaders(res.headers);
      const contentDisp = res.headers.get('Content-Disposition') || '';
      const fnMatch = contentDisp.match(/filename="([^"]+)"/);
      const outputName = fnMatch ? fnMatch[1] : (fileName || 'output').replace(/\.[^.]+$/, '') + '.tern';
      const buffer = await res.arrayBuffer();
      return {
        fileName: outputName,
        originalSize: h.originalSize,
        compressedSize: h.compressedSize,
        compressionRatio: h.compressionRatio,
        encrypted: h.encrypted,
        encryptionMode: encrypt ? encryptionMode : undefined,
        processingTimeMs: h.processingTimeMs,
        data: buffer,
        ttcMetadata: {
          engine: h.engine,
          version: h.version,
          level: h.level,
          levelName: h.levelName,
          modeName: h.mode,
          crc32: h.crc32,
          avgTau: h.avgTau,
          avgDelta: h.avgDelta,
          predominantBase: h.predominantBase,
          adaptiveRepUsed: h.adaptiveRepUsed,
          gf3Representation: h.gf3Representation as 'balanced' | 'unsigned' | 'native',
        },
      } as FileCompressionResult;
    },
    onSuccess: (data: FileCompressionResult) => {
      setCompressResult(data);
      toast({ title: "File compressed", description: `${data.fileName} created (${data.compressionRatio}% ratio)` });
    },
    onError: (err: Error) => {
      toast({ title: "Compression failed", description: err.message, variant: "destructive" });
    },
  });

  const decompressMutation = useMutation({
    mutationFn: async () => {
      if (!ternFileBuffer) throw new Error('No .tern file loaded');
      const res = await fetch('/api/compression/decompress', {
        method: 'POST',
        headers: { 'Content-Type': 'application/octet-stream' },
        body: ternFileBuffer,
        credentials: 'include',
      });
      if (!res.ok) {
        const err = await res.json().catch(() => ({ error: 'Decompression failed' }));
        throw new Error(err.error || 'Decompression failed');
      }
      const h = parseTtcHeaders(res.headers);
      const buffer = await res.arrayBuffer();
      return {
        originalFileName: h.originalFileName || 'decompressed.bin',
        originalSize: h.originalSize,
        compressedSize: h.compressedSize,
        compressionRatio: h.compressionRatio,
        wasEncrypted: h.wasEncrypted,
        processingTimeMs: h.processingTimeMs,
        data: buffer,
        ttcMetadata: {
          engine: h.engine,
          version: h.version,
          level: h.level,
          levelName: h.levelName,
          crc32Verified: h.crc32Verified,
          originalFileName: h.originalFileName,
        },
      } as FileDecompressionResult;
    },
    onSuccess: (data: FileDecompressionResult) => {
      setDecompressResult(data);
      toast({ title: "File decompressed", description: `Restored ${data.originalFileName}` });
    },
    onError: (err: Error) => {
      toast({ title: "Decompression failed", description: err.message, variant: "destructive" });
    },
  });

  const [fileBytes, setFileBytes] = useState<Uint8Array | null>(null);

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    setFileName(file.name);
    const reader = new FileReader();
    reader.onload = (event) => {
      const buffer = event.target?.result as ArrayBuffer;
      const bytes = new Uint8Array(buffer);
      setFileBytes(bytes);
      const decoder = new TextDecoder('utf-8', { fatal: false });
      setFileContent(decoder.decode(bytes));
    };
    reader.readAsArrayBuffer(file);
  };

  const handleTernFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (event) => {
      const buffer = event.target?.result as ArrayBuffer;
      setTernFileBuffer(buffer);
    };
    reader.readAsArrayBuffer(file);
  };

  const downloadCompressed = () => {
    if (!compressResult) return;
    const blob = new Blob([compressResult.data], { type: 'application/octet-stream' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = compressResult.fileName;
    a.click();
    URL.revokeObjectURL(url);
  };

  const downloadDecompressed = () => {
    if (!decompressResult) return;
    const blob = new Blob([decompressResult.data], { type: 'application/octet-stream' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = decompressResult.originalFileName;
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="space-y-6">
      <div className="grid md:grid-cols-2 gap-6">
        <Card>
          <CardHeader>
            <CardTitle className="text-lg flex items-center gap-2">
              <FileUp className="w-5 h-5" />
              Compress File
            </CardTitle>
            <CardDescription>
              Upload any file to ternary-compress it into a .tern file
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label>Select File</Label>
              <Input
                type="file"
                onChange={handleFileSelect}
                data-testid="input-file-compress"
              />
              {fileName && (
                <div className="text-sm text-muted-foreground">
                  Selected: {fileName} ({fileContent.length.toLocaleString()} bytes)
                </div>
              )}
            </div>

            <div className="space-y-2">
              <Label>Or paste content directly</Label>
              <Textarea
                value={fileContent}
                onChange={(e) => { setFileContent(e.target.value); if (!fileName) setFileName("pasted-content.txt"); }}
                placeholder="Paste text content here..."
                className="min-h-[80px] text-xs font-mono"
                data-testid="textarea-file-content"
              />
            </div>

            <div className="flex items-center justify-between gap-4">
              <div className="flex items-center gap-2">
                <Switch
                  checked={encrypt}
                  onCheckedChange={setEncrypt}
                  data-testid="switch-encrypt-file"
                />
                <Label className="text-sm">Phase Encrypt</Label>
              </div>
              {encrypt && (
                <Select value={encryptionMode} onValueChange={setEncryptionMode}>
                  <SelectTrigger className="w-40" data-testid="select-encrypt-mode-file">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="high_security">High Security</SelectItem>
                    <SelectItem value="balanced">Balanced</SelectItem>
                    <SelectItem value="performance">Performance</SelectItem>
                    <SelectItem value="adaptive">Adaptive</SelectItem>
                  </SelectContent>
                </Select>
              )}
            </div>

            <Button
              onClick={() => compressMutation.mutate()}
              disabled={!fileContent || compressMutation.isPending}
              className="w-full"
              data-testid="button-compress-file"
            >
              {compressMutation.isPending ? (
                <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
              ) : (
                <Archive className="w-4 h-4 mr-2" />
              )}
              Compress to .tern
            </Button>

            {compressResult && (
              <div className="space-y-3 pt-2">
                <div className="grid grid-cols-2 gap-3">
                  <div className="bg-muted/50 rounded-md p-3 text-center">
                    <div className="text-lg font-bold">{compressResult.originalSize.toLocaleString()}</div>
                    <div className="text-xs text-muted-foreground">Original (bytes)</div>
                  </div>
                  <div className="bg-muted/50 rounded-md p-3 text-center">
                    <div className="text-lg font-bold">{compressResult.compressedSize.toLocaleString()}</div>
                    <div className="text-xs text-muted-foreground">Compressed (bytes)</div>
                  </div>
                </div>
                <div className="flex items-center justify-between gap-2 text-sm">
                  <div>
                    <span className="text-muted-foreground">Processed in </span>
                    <span className="font-mono">{compressResult.processingTimeMs}ms</span>
                  </div>
                  {compressResult.encrypted && (
                    <Badge variant="outline">
                      <Shield className="w-3 h-3 mr-1" />
                      {compressResult.encryptionMode}
                    </Badge>
                  )}
                </div>
                <div className="bg-muted/30 border rounded-md p-3 space-y-2" data-testid="ttc-metadata-compress">
                  <div className="flex items-center gap-2 flex-wrap mb-1">
                    <Badge variant="secondary" className="text-xs" data-testid="badge-engine">
                      {compressResult.ttcMetadata.engine === 'ttc-native' ? 'TTC v' + compressResult.ttcMetadata.version + ' Native' : 'Legacy'}
                    </Badge>
                    <Badge variant="outline" className="text-xs" data-testid="badge-mode">
                      {compressResult.ttcMetadata.modeName}
                    </Badge>
                    <Badge variant="outline" className="text-xs" data-testid="badge-level">
                      L{compressResult.ttcMetadata.level} {compressResult.ttcMetadata.levelName}
                    </Badge>
                  </div>
                  <div className="grid grid-cols-2 sm:grid-cols-3 gap-2 text-xs">
                    <div>
                      <span className="text-muted-foreground">CRC32: </span>
                      <span className="font-mono" data-testid="text-crc32">{compressResult.ttcMetadata.crc32.toString(16).toUpperCase()}</span>
                    </div>
                    <div>
                      <span className="text-muted-foreground">Base: </span>
                      <span className="font-mono" data-testid="text-predominant-base">{compressResult.ttcMetadata.predominantBase}</span>
                    </div>
                    <div>
                      <span className="text-muted-foreground">Adaptive: </span>
                      <span data-testid="text-adaptive-rep">{compressResult.ttcMetadata.adaptiveRepUsed ? 'Yes' : 'No'}</span>
                    </div>
                    <div>
                      <span className="text-muted-foreground">Avg Tau: </span>
                      <span className="font-mono" data-testid="text-avg-tau">{compressResult.ttcMetadata.avgTau.toFixed(4)}</span>
                    </div>
                    <div>
                      <span className="text-muted-foreground">Avg Delta: </span>
                      <span className="font-mono" data-testid="text-avg-delta">{compressResult.ttcMetadata.avgDelta.toFixed(4)}</span>
                    </div>
                    <div>
                      <span className="text-muted-foreground">GF(3): </span>
                      <span className="font-mono" data-testid="text-gf3-rep">{compressResult.ttcMetadata.gf3Representation || 'balanced'}</span>
                    </div>
                  </div>
                </div>
                <Button onClick={downloadCompressed} variant="outline" className="w-full" data-testid="button-download-tern">
                  <Download className="w-4 h-4 mr-2" />
                  Download {compressResult.fileName}
                </Button>
              </div>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-lg flex items-center gap-2">
              <FileDown className="w-5 h-5" />
              Decompress .tern File
            </CardTitle>
            <CardDescription>
              Upload a .tern file to restore the original
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label>Select .tern File</Label>
              <Input
                type="file"
                accept=".tern"
                onChange={handleTernFileSelect}
                data-testid="input-file-decompress"
              />
            </div>

            {compressResult && (
              <div className="space-y-2">
                <Label>Or use last compressed output</Label>
                <Button
                  variant="outline"
                  onClick={() => setTernFileBuffer(compressResult.data)}
                  className="w-full"
                  data-testid="button-use-last-compressed"
                >
                  <Upload className="w-4 h-4 mr-2" />
                  Use "{compressResult.fileName}"
                </Button>
              </div>
            )}

            <Button
              onClick={() => decompressMutation.mutate()}
              disabled={!ternFileBuffer || decompressMutation.isPending}
              className="w-full"
              data-testid="button-decompress-file"
            >
              {decompressMutation.isPending ? (
                <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
              ) : (
                <Play className="w-4 h-4 mr-2" />
              )}
              Decompress
            </Button>

            {decompressResult && (
              <div className="space-y-3 pt-2">
                <div className="grid grid-cols-2 gap-3">
                  <div className="bg-muted/50 rounded-md p-3 text-center">
                    <div className="text-lg font-bold">{decompressResult.compressedSize.toLocaleString()}</div>
                    <div className="text-xs text-muted-foreground">Compressed (bytes)</div>
                  </div>
                  <div className="bg-muted/50 rounded-md p-3 text-center">
                    <div className="text-lg font-bold">{decompressResult.originalSize.toLocaleString()}</div>
                    <div className="text-xs text-muted-foreground">Restored (bytes)</div>
                  </div>
                </div>
                <div className="flex items-center justify-between gap-2 text-sm">
                  <div>
                    <span className="text-muted-foreground">Restored in </span>
                    <span className="font-mono">{decompressResult.processingTimeMs}ms</span>
                  </div>
                  {decompressResult.wasEncrypted && (
                    <Badge variant="outline">
                      <Shield className="w-3 h-3 mr-1" />
                      Decrypted
                    </Badge>
                  )}
                </div>
                <div className="bg-muted/30 border rounded-md p-3 space-y-1" data-testid="ttc-metadata-decompress">
                  <div className="flex items-center gap-2">
                    <Badge variant="secondary" className="text-xs">
                      {decompressResult.ttcMetadata.engine === 'ttc-native' ? 'TTC v' + decompressResult.ttcMetadata.version + ' Native' : 'Legacy'}
                    </Badge>
                    <Badge variant={decompressResult.ttcMetadata.crc32Verified ? 'default' : 'destructive'} className="text-xs" data-testid="badge-crc32-verified">
                      CRC32 {decompressResult.ttcMetadata.crc32Verified ? 'Verified' : 'MISMATCH'}
                    </Badge>
                  </div>
                </div>
                <Button onClick={downloadDecompressed} variant="outline" className="w-full" data-testid="button-download-decompressed">
                  <Download className="w-4 h-4 mr-2" />
                  Download {decompressResult.originalFileName}
                </Button>
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">How .tern File Compression Works</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid md:grid-cols-4 gap-4 text-center text-sm">
            <div className="bg-muted/50 rounded-md p-4">
              <div className="font-bold mb-1">1. Domain Analysis</div>
              <div className="text-muted-foreground">TTC v4.2 analyzes input domain (temporal, image, audio, source) for optimal encoding</div>
            </div>
            <div className="bg-muted/50 rounded-md p-4">
              <div className="font-bold mb-1">2. Ternary rANS</div>
              <div className="text-muted-foreground">Asymmetric numeral systems with pure ternary 3^k window sizes and GURFT fast-path</div>
            </div>
            <div className="bg-muted/50 rounded-md p-4">
              <div className="font-bold mb-1">3. Phase Encrypt</div>
              <div className="text-muted-foreground">Optional dual-phase split with guardian tamper detection</div>
            </div>
            <div className="bg-muted/50 rounded-md p-4">
              <div className="font-bold mb-1">4. .tern Package</div>
              <div className="text-muted-foreground">Header + compressed data bundled with checksum verification</div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

function DatabaseCompressionTab() {
  const [title, setTitle] = useState("");
  const [content, setContent] = useState("");
  const [compress, setCompress] = useState(true);
  const [encrypt, setEncrypt] = useState(false);
  const [encryptionMode, setEncryptionMode] = useState("balanced");
  const [viewingDoc, setViewingDoc] = useState<{ content: string; title: string; id: number } | null>(null);
  const [viewingRaw, setViewingRaw] = useState<{ storedContent: string; title: string; id: number } | null>(null);
  const { toast } = useToast();

  const { data: docsData, isLoading: docsLoading } = useQuery<{ success: boolean; documents: DbDocument[] }>({
    queryKey: ["/api/compression/db/documents"],
  });

  const storeMutation = useMutation({
    mutationFn: async () => {
      const res = await apiRequest("POST", "/api/compression/db/store", {
        title,
        content,
        compress,
        encrypt: compress ? encrypt : false,
        encryptionMode: encrypt ? encryptionMode : undefined,
      });
      return res.json();
    },
    onSuccess: () => {
      toast({ title: "Document stored", description: compress ? "Content compressed and stored in PostgreSQL" : "Content stored uncompressed" });
      setTitle("");
      setContent("");
      queryClient.invalidateQueries({ queryKey: ["/api/compression/db/documents"] });
    },
    onError: () => {
      toast({ title: "Store failed", variant: "destructive" });
    },
  });

  const retrieveMutation = useMutation({
    mutationFn: async (id: number) => {
      const res = await fetch(`/api/compression/db/retrieve/${id}`);
      return res.json();
    },
    onSuccess: (data: any) => {
      setViewingDoc({ content: data.document.content, title: data.document.title, id: data.document.id });
    },
  });

  const rawMutation = useMutation({
    mutationFn: async (id: number) => {
      const res = await fetch(`/api/compression/db/raw/${id}`);
      return res.json();
    },
    onSuccess: (data: any) => {
      setViewingRaw({ storedContent: data.raw.storedContent, title: data.raw.title, id: data.raw.id });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: async (id: number) => {
      await apiRequest("DELETE", `/api/compression/db/documents/${id}`);
    },
    onSuccess: () => {
      toast({ title: "Document deleted" });
      queryClient.invalidateQueries({ queryKey: ["/api/compression/db/documents"] });
    },
  });

  const documents = docsData?.documents || [];

  return (
    <div className="space-y-6">
      <div className="grid md:grid-cols-2 gap-6">
        <Card>
          <CardHeader>
            <CardTitle className="text-lg flex items-center gap-2">
              <Database className="w-5 h-5" />
              Store Document
            </CardTitle>
            <CardDescription>
              Write data to PostgreSQL with transparent ternary compression
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label>Document Title</Label>
              <Input
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                placeholder="My Document"
                data-testid="input-db-title"
              />
            </div>

            <div className="space-y-2">
              <Label>Content</Label>
              <Textarea
                value={content}
                onChange={(e) => setContent(e.target.value)}
                placeholder="Enter document content to store..."
                className="min-h-[120px] text-sm"
                data-testid="textarea-db-content"
              />
              {content && (
                <div className="text-xs text-muted-foreground">
                  {content.length} characters
                </div>
              )}
            </div>

            <div className="space-y-3">
              <div className="flex items-center justify-between gap-4">
                <div className="flex items-center gap-2">
                  <Switch
                    checked={compress}
                    onCheckedChange={setCompress}
                    data-testid="switch-db-compress"
                  />
                  <Label className="text-sm">Ternary Compress</Label>
                </div>
              </div>

              {compress && (
                <div className="flex items-center justify-between gap-4">
                  <div className="flex items-center gap-2">
                    <Switch
                      checked={encrypt}
                      onCheckedChange={setEncrypt}
                      data-testid="switch-db-encrypt"
                    />
                    <Label className="text-sm">Phase Encrypt</Label>
                  </div>
                  {encrypt && (
                    <Select value={encryptionMode} onValueChange={setEncryptionMode}>
                      <SelectTrigger className="w-40" data-testid="select-db-encrypt-mode">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="high_security">High Security</SelectItem>
                        <SelectItem value="balanced">Balanced</SelectItem>
                        <SelectItem value="performance">Performance</SelectItem>
                        <SelectItem value="adaptive">Adaptive</SelectItem>
                      </SelectContent>
                    </Select>
                  )}
                </div>
              )}
            </div>

            <Button
              onClick={() => storeMutation.mutate()}
              disabled={!title || !content || storeMutation.isPending}
              className="w-full"
              data-testid="button-db-store"
            >
              {storeMutation.isPending ? (
                <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
              ) : (
                <Database className="w-4 h-4 mr-2" />
              )}
              Store in PostgreSQL
            </Button>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-lg flex items-center gap-2">
              <Eye className="w-5 h-5" />
              Retrieved Content
            </CardTitle>
            <CardDescription>
              Content is automatically decompressed (and decrypted) when retrieved
            </CardDescription>
          </CardHeader>
          <CardContent>
            {viewingDoc ? (
              <div className="space-y-3">
                <div className="flex items-center justify-between gap-2">
                  <h3 className="font-medium">{viewingDoc.title}</h3>
                  <Badge variant="outline">ID: {viewingDoc.id}</Badge>
                </div>
                <div className="bg-muted/50 rounded-md p-3 text-sm font-mono whitespace-pre-wrap max-h-[250px] overflow-y-auto" data-testid="text-retrieved-content">
                  {viewingDoc.content}
                </div>
                <Button variant="outline" onClick={() => setViewingDoc(null)} className="w-full">
                  Clear
                </Button>
              </div>
            ) : viewingRaw ? (
              <div className="space-y-3">
                <div className="flex items-center justify-between gap-2">
                  <h3 className="font-medium">Raw: {viewingRaw.title}</h3>
                  <Badge variant="destructive">RAW DB VALUE</Badge>
                </div>
                <div className="bg-muted/50 rounded-md p-3 text-xs font-mono whitespace-pre-wrap max-h-[250px] overflow-y-auto break-all" data-testid="text-raw-content">
                  {viewingRaw.storedContent}
                </div>
                <Button variant="outline" onClick={() => setViewingRaw(null)} className="w-full">
                  Clear
                </Button>
              </div>
            ) : (
              <div className="text-center text-muted-foreground py-12">
                <Eye className="w-8 h-8 mx-auto mb-3 opacity-30" />
                <div>Select a document to view its retrieved (decompressed) content or raw database value</div>
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Stored Documents</CardTitle>
          <CardDescription>
            Documents in the compressed_documents PostgreSQL table
          </CardDescription>
        </CardHeader>
        <CardContent>
          {docsLoading ? (
            <div className="text-center text-muted-foreground py-6">Loading...</div>
          ) : documents.length === 0 ? (
            <div className="text-center text-muted-foreground py-6">
              No documents stored yet. Create one above.
            </div>
          ) : (
            <div className="space-y-2">
              {documents.map((doc) => (
                <div
                  key={doc.id}
                  className="flex items-center justify-between gap-4 p-3 bg-muted/30 rounded-md"
                  data-testid={`row-doc-${doc.id}`}
                >
                  <div className="flex items-center gap-3 min-w-0 flex-1">
                    <div className="min-w-0">
                      <div className="font-medium text-sm truncate">{doc.title}</div>
                      <div className="text-xs text-muted-foreground flex items-center gap-2 flex-wrap">
                        <span>{doc.originalSizeBytes?.toLocaleString() || 0} bytes</span>
                        {doc.isCompressed && (
                          <Badge variant="secondary" className="text-xs">Compressed</Badge>
                        )}
                        {doc.isEncrypted && (
                          <Badge variant="outline" className="text-xs">
                            <Shield className="w-3 h-3 mr-1" />
                            {doc.encryptionMode}
                          </Badge>
                        )}
                        {doc.compressionRatio != null && (
                          <span className="font-mono">{doc.compressionRatio.toFixed(1)}%</span>
                        )}
                      </div>
                    </div>
                  </div>
                  <div className="flex items-center gap-1 shrink-0">
                    <Button
                      size="icon"
                      variant="ghost"
                      onClick={() => retrieveMutation.mutate(doc.id)}
                      disabled={retrieveMutation.isPending}
                      data-testid={`button-retrieve-${doc.id}`}
                    >
                      <Eye className="w-4 h-4" />
                    </Button>
                    <Button
                      size="icon"
                      variant="ghost"
                      onClick={() => rawMutation.mutate(doc.id)}
                      disabled={rawMutation.isPending}
                      data-testid={`button-raw-${doc.id}`}
                    >
                      <Database className="w-4 h-4" />
                    </Button>
                    <Button
                      size="icon"
                      variant="ghost"
                      onClick={() => deleteMutation.mutate(doc.id)}
                      disabled={deleteMutation.isPending}
                      data-testid={`button-delete-${doc.id}`}
                    >
                      <Trash2 className="w-4 h-4" />
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">How Transparent DB Compression Works</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid md:grid-cols-3 gap-4 text-center text-sm">
            <div className="bg-muted/50 rounded-md p-4">
              <div className="font-bold mb-2">Write Path</div>
              <div className="text-muted-foreground">
                Application writes normal text. The compression layer intercepts, ternary-encodes, RLE-compresses, and optionally phase-encrypts before storing in PostgreSQL.
              </div>
            </div>
            <div className="bg-muted/50 rounded-md p-4">
              <div className="font-bold mb-2">Storage</div>
              <div className="text-muted-foreground">
                PostgreSQL stores a JSON envelope containing the compressed/encrypted data, size metadata, and compression flags. The raw column value is not human-readable.
              </div>
            </div>
            <div className="bg-muted/50 rounded-md p-4">
              <div className="font-bold mb-2">Read Path</div>
              <div className="text-muted-foreground">
                When queried, the layer detects the compression envelope, phase-decrypts if needed, RLE-decompresses, and ternary-decodes back to the original content.
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

export default function CompressionPage() {
  return (
    <div className="min-h-screen bg-background">
      <main className="container mx-auto px-4 py-8">
        <div className="mb-8">
          <h1 className="text-3xl font-bold mb-2" data-testid="text-compression-title">Ternary Compression</h1>
          <p className="text-muted-foreground">
            Two distinct compression systems: standalone file compression with .tern format, and transparent database compression for PostgreSQL storage.
          </p>
        </div>

        <Tabs defaultValue="file" className="space-y-6">
          <TabsList className="grid w-full grid-cols-2 max-w-md">
            <TabsTrigger value="file" data-testid="tab-file-compression" className="flex items-center gap-2">
              <FileUp className="w-4 h-4" />
              File Compression
            </TabsTrigger>
            <TabsTrigger value="database" data-testid="tab-db-compression" className="flex items-center gap-2">
              <Database className="w-4 h-4" />
              Database Compression
            </TabsTrigger>
          </TabsList>

          <TabsContent value="file">
            <FileCompressionTab />
          </TabsContent>

          <TabsContent value="database">
            <DatabaseCompressionTab />
          </TabsContent>
        </Tabs>
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
