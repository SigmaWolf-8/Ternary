/**
 * Copyright (c) 2025–2026 Capomastro Holdings Ltd. (Canada)
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

interface FileCompressionResult {
  success: boolean;
  fileName: string;
  originalSize: number;
  compressedSize: number;
  compressionRatio: string;
  encrypted: boolean;
  encryptionMode?: string;
  processingTimeMs: string;
  data: string;
  header: {
    magic: string;
    version: number;
    originalFileName: string;
    originalSize: number;
    compressedSize: number;
    compressionRatio: number;
    encrypted: boolean;
    encryptionMode?: string;
    checksum: number;
    timestamp: string;
  };
}

interface FileDecompressionResult {
  success: boolean;
  originalFileName: string;
  originalSize: number;
  compressedSize: number;
  wasEncrypted: boolean;
  encryptionMode?: string;
  processingTimeMs: string;
  data: string;
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

function FileCompressionTab() {
  const [fileName, setFileName] = useState("");
  const [fileContent, setFileContent] = useState("");
  const [encrypt, setEncrypt] = useState(false);
  const [encryptionMode, setEncryptionMode] = useState("balanced");
  const [compressResult, setCompressResult] = useState<FileCompressionResult | null>(null);
  const [decompressResult, setDecompressResult] = useState<FileDecompressionResult | null>(null);
  const [ternFileContent, setTernFileContent] = useState("");
  const { toast } = useToast();

  const compressMutation = useMutation({
    mutationFn: async () => {
      const bytes = fileBytes || new TextEncoder().encode(fileContent);
      let binary = '';
      bytes.forEach(b => binary += String.fromCharCode(b));
      const base64 = btoa(binary);
      const res = await apiRequest("POST", "/api/compression/file", {
        fileName: fileName || "untitled.txt",
        content: base64,
        encrypt,
        encryptionMode: encrypt ? encryptionMode : undefined,
      });
      return res.json();
    },
    onSuccess: (data: FileCompressionResult) => {
      setCompressResult(data);
      toast({ title: "File compressed", description: `${data.fileName} created (${data.compressionRatio}% ratio)` });
    },
    onError: () => {
      toast({ title: "Compression failed", variant: "destructive" });
    },
  });

  const decompressMutation = useMutation({
    mutationFn: async () => {
      const res = await apiRequest("POST", "/api/compression/decompress", {
        content: ternFileContent,
      });
      return res.json();
    },
    onSuccess: (data: FileDecompressionResult) => {
      setDecompressResult(data);
      toast({ title: "File decompressed", description: `Restored ${data.originalFileName}` });
    },
    onError: () => {
      toast({ title: "Decompression failed", variant: "destructive" });
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
      const bytes = new Uint8Array(buffer);
      let binary = '';
      bytes.forEach(b => binary += String.fromCharCode(b));
      setTernFileContent(btoa(binary));
    };
    reader.readAsArrayBuffer(file);
  };

  const downloadCompressed = () => {
    if (!compressResult) return;
    const binary = atob(compressResult.data);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
    const blob = new Blob([bytes], { type: 'application/octet-stream' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = compressResult.fileName;
    a.click();
    URL.revokeObjectURL(url);
  };

  const base64ToBytes = (b64: string): Uint8Array => {
    const binary = atob(b64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
    return bytes;
  };

  const downloadDecompressed = () => {
    if (!decompressResult) return;
    const bytes = base64ToBytes(decompressResult.data);
    const blob = new Blob([bytes], { type: 'application/octet-stream' });
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
                  onClick={() => setTernFileContent(compressResult.data)}
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
              disabled={!ternFileContent || decompressMutation.isPending}
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
              <div className="font-bold mb-1">1. Ternary Encode</div>
              <div className="text-muted-foreground">Binary bytes are converted to base-3 trit sequences (5 trits per byte)</div>
            </div>
            <div className="bg-muted/50 rounded-md p-4">
              <div className="font-bold mb-1">2. RLE Compress</div>
              <div className="text-muted-foreground">Run-length encoding compresses repeated trit patterns</div>
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
