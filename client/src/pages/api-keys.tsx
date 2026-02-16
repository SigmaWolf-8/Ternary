/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 */

import { useState } from "react";
import { useAuth } from "@/hooks/use-auth";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { apiRequest } from "@/lib/queryClient";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useToast } from "@/hooks/use-toast";
import {
  Key,
  Plus,
  Copy,
  Trash2,
  ShieldCheck,
  Activity,
  Clock,
  AlertTriangle,
  CheckCircle2,
  XCircle,
  LogIn,
} from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
  DialogDescription,
} from "@/components/ui/dialog";

interface ApiKeyRecord {
  id: string;
  keyPrefix: string;
  name: string;
  owner: string;
  scopes: string[];
  isActive: boolean;
  expiresAt: string | null;
  revokedAt: string | null;
  lastUsedAt: string | null;
  usageCount: number;
  createdAt: string;
}

interface KeyStats {
  total: number;
  active: number;
  revoked: number;
  expired: number;
  totalUsage: number;
}

const SCOPE_CATEGORIES: Record<string, string[]> = {
  Ternary: ["read:ternary", "write:ternary"],
  "Phase Encryption": ["read:phase", "write:phase"],
  HPTP: ["read:hptp", "write:hptp"],
  Compression: ["read:compression", "write:compression"],
  Calendar: ["read:calendar"],
  "Agent Array": ["read:agent-array", "write:agent-array"],
  Whitepaper: ["read:whitepaper"],
  Admin: ["admin:keys"],
};

function ScopeSelector({
  selected,
  onChange,
}: {
  selected: string[];
  onChange: (scopes: string[]) => void;
}) {
  const toggle = (scope: string) => {
    if (selected.includes(scope)) {
      onChange(selected.filter((s) => s !== scope));
    } else {
      onChange([...selected, scope]);
    }
  };

  const selectAll = () => {
    const all = Object.values(SCOPE_CATEGORIES).flat();
    onChange(all);
  };

  const clearAll = () => onChange([]);

  return (
    <div className="space-y-3">
      <div className="flex items-center gap-2 flex-wrap">
        <Button variant="outline" size="sm" onClick={selectAll} data-testid="button-select-all-scopes">
          Select All
        </Button>
        <Button variant="outline" size="sm" onClick={clearAll} data-testid="button-clear-all-scopes">
          Clear All
        </Button>
      </div>
      <div className="space-y-2">
        {Object.entries(SCOPE_CATEGORIES).map(([category, scopes]) => (
          <div key={category}>
            <p className="text-xs font-medium text-muted-foreground mb-1">{category}</p>
            <div className="flex gap-1 flex-wrap">
              {scopes.map((scope) => (
                <Badge
                  key={scope}
                  variant={selected.includes(scope) ? "default" : "outline"}
                  className="cursor-pointer select-none toggle-elevate"
                  onClick={() => toggle(scope)}
                  data-testid={`badge-scope-${scope}`}
                >
                  {scope}
                </Badge>
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

function KeyStatusBadge({ keyRecord }: { keyRecord: ApiKeyRecord }) {
  if (keyRecord.revokedAt) {
    return <Badge variant="destructive">Revoked</Badge>;
  }
  if (keyRecord.expiresAt && new Date(keyRecord.expiresAt) < new Date()) {
    return <Badge variant="secondary">Expired</Badge>;
  }
  if (keyRecord.isActive) {
    return <Badge variant="default">Active</Badge>;
  }
  return <Badge variant="secondary">Inactive</Badge>;
}

function formatDate(dateStr: string | null): string {
  if (!dateStr) return "Never";
  return new Date(dateStr).toLocaleDateString("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export default function ApiKeysPage() {
  const { user, isAuthenticated, isLoading: authLoading } = useAuth();
  const { toast } = useToast();
  const queryClient = useQueryClient();
  const [showCreateDialog, setShowCreateDialog] = useState(false);
  const [newKeyName, setNewKeyName] = useState("");
  const [newKeyScopes, setNewKeyScopes] = useState<string[]>([]);
  const [newKeyExpiry, setNewKeyExpiry] = useState(90);
  const [generatedKey, setGeneratedKey] = useState<string | null>(null);
  const [revokeTarget, setRevokeTarget] = useState<ApiKeyRecord | null>(null);

  const { data: keysData, isLoading: keysLoading } = useQuery<{
    success: boolean;
    keys: ApiKeyRecord[];
  }>({
    queryKey: ["/api/keys"],
    enabled: isAuthenticated,
  });

  const { data: statsData } = useQuery<{
    success: boolean;
    stats: KeyStats;
  }>({
    queryKey: ["/api/keys/stats"],
    enabled: isAuthenticated,
  });

  const generateMutation = useMutation({
    mutationFn: async (body: { name: string; scopes: string[]; expiresDays: number }) => {
      const res = await apiRequest("POST", "/api/keys/generate", body);
      return res.json();
    },
    onSuccess: (data) => {
      setGeneratedKey(data.key);
      setNewKeyName("");
      setNewKeyScopes([]);
      setNewKeyExpiry(90);
      queryClient.invalidateQueries({ queryKey: ["/api/keys"] });
      queryClient.invalidateQueries({ queryKey: ["/api/keys/stats"] });
      toast({ title: "Key Generated", description: "Your new API key has been created." });
    },
    onError: () => {
      toast({
        title: "Error",
        description: "Failed to generate API key.",
        variant: "destructive",
      });
    },
  });

  const revokeMutation = useMutation({
    mutationFn: async (id: string) => {
      await apiRequest("POST", `/api/keys/revoke/${id}`);
    },
    onSuccess: () => {
      setRevokeTarget(null);
      queryClient.invalidateQueries({ queryKey: ["/api/keys"] });
      queryClient.invalidateQueries({ queryKey: ["/api/keys/stats"] });
      toast({ title: "Key Revoked", description: "The API key has been permanently revoked." });
    },
    onError: () => {
      toast({
        title: "Error",
        description: "Failed to revoke key.",
        variant: "destructive",
      });
    },
  });

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text).then(() => {
      toast({ title: "Copied", description: "API key copied to clipboard." });
    });
  };

  const handleGenerate = () => {
    if (!newKeyName.trim() || newKeyScopes.length === 0) {
      toast({
        title: "Validation Error",
        description: "Name and at least one scope are required.",
        variant: "destructive",
      });
      return;
    }
    generateMutation.mutate({
      name: newKeyName,
      scopes: newKeyScopes,
      expiresDays: newKeyExpiry,
    });
  };

  if (authLoading) {
    return (
      <div className="flex items-center justify-center min-h-[60vh]">
        <div className="inline-flex items-center justify-center w-8 h-8 rounded-full border-2 border-primary/20 border-t-primary animate-spin" />
      </div>
    );
  }

  if (!isAuthenticated) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[60vh] gap-4 p-6">
        <ShieldCheck className="w-12 h-12 text-muted-foreground" />
        <h2 className="text-xl font-semibold">Admin Access Required</h2>
        <p className="text-sm text-muted-foreground text-center max-w-md">
          Sign in with an admin account to manage API keys.
        </p>
        <Button asChild data-testid="button-login-redirect">
          <a href="/api/login">
            <LogIn className="w-4 h-4 mr-2" />
            Sign In
          </a>
        </Button>
      </div>
    );
  }

  const keys = keysData?.keys || [];
  const stats = statsData?.stats;

  return (
    <div className="p-4 md:p-6 space-y-6 max-w-6xl mx-auto">
      <div className="flex items-center justify-between gap-4 flex-wrap">
        <div>
          <h1 className="text-xl font-semibold flex items-center gap-2" data-testid="text-page-title">
            <Key className="w-5 h-5" />
            API Key Management
          </h1>
          <p className="text-sm text-muted-foreground mt-1">
            Generate, manage, and revoke API keys for external integrations.
          </p>
        </div>
        <Button onClick={() => setShowCreateDialog(true)} data-testid="button-create-key">
          <Plus className="w-4 h-4 mr-2" />
          Generate Key
        </Button>
      </div>

      {stats && (
        <div className="grid grid-cols-2 md:grid-cols-5 gap-3">
          <Card className="p-3">
            <div className="text-xs text-muted-foreground">Total Keys</div>
            <div className="text-lg font-semibold" data-testid="text-stat-total">{stats.total}</div>
          </Card>
          <Card className="p-3">
            <div className="flex items-center gap-1 text-xs text-muted-foreground">
              <CheckCircle2 className="w-3 h-3" /> Active
            </div>
            <div className="text-lg font-semibold text-green-600 dark:text-green-400" data-testid="text-stat-active">{stats.active}</div>
          </Card>
          <Card className="p-3">
            <div className="flex items-center gap-1 text-xs text-muted-foreground">
              <XCircle className="w-3 h-3" /> Revoked
            </div>
            <div className="text-lg font-semibold" data-testid="text-stat-revoked">{stats.revoked}</div>
          </Card>
          <Card className="p-3">
            <div className="flex items-center gap-1 text-xs text-muted-foreground">
              <Clock className="w-3 h-3" /> Expired
            </div>
            <div className="text-lg font-semibold" data-testid="text-stat-expired">{stats.expired}</div>
          </Card>
          <Card className="p-3">
            <div className="flex items-center gap-1 text-xs text-muted-foreground">
              <Activity className="w-3 h-3" /> Total Requests
            </div>
            <div className="text-lg font-semibold" data-testid="text-stat-usage">{stats.totalUsage.toLocaleString()}</div>
          </Card>
        </div>
      )}

      <Card className="overflow-hidden">
        <div className="p-3 border-b">
          <h2 className="text-sm font-medium">API Keys</h2>
        </div>
        {keysLoading ? (
          <div className="flex items-center justify-center p-8">
            <div className="inline-flex items-center justify-center w-6 h-6 rounded-full border-2 border-primary/20 border-t-primary animate-spin" />
          </div>
        ) : keys.length === 0 ? (
          <div className="flex flex-col items-center justify-center p-8 text-center">
            <Key className="w-8 h-8 text-muted-foreground mb-2" />
            <p className="text-sm text-muted-foreground">No API keys created yet.</p>
            <p className="text-xs text-muted-foreground mt-1">
              Generate your first key to start integrating with external services.
            </p>
          </div>
        ) : (
          <div className="divide-y">
            {keys.map((keyRecord) => (
              <div
                key={keyRecord.id}
                className="p-3 flex items-start gap-3 flex-wrap"
                data-testid={`row-api-key-${keyRecord.id}`}
              >
                <div className="flex-1 min-w-0 space-y-1">
                  <div className="flex items-center gap-2 flex-wrap">
                    <span className="font-medium text-sm" data-testid={`text-key-name-${keyRecord.id}`}>
                      {keyRecord.name}
                    </span>
                    <KeyStatusBadge keyRecord={keyRecord} />
                    <code className="text-xs text-muted-foreground bg-muted px-1 rounded" data-testid={`text-key-prefix-${keyRecord.id}`}>
                      {keyRecord.keyPrefix}...
                    </code>
                  </div>
                  <div className="flex items-center gap-3 flex-wrap text-xs text-muted-foreground">
                    <span>Owner: {keyRecord.owner}</span>
                    <span>Created: {formatDate(keyRecord.createdAt)}</span>
                    {keyRecord.expiresAt && (
                      <span>Expires: {formatDate(keyRecord.expiresAt)}</span>
                    )}
                    <span>Used: {keyRecord.usageCount.toLocaleString()} times</span>
                    {keyRecord.lastUsedAt && (
                      <span>Last: {formatDate(keyRecord.lastUsedAt)}</span>
                    )}
                  </div>
                  <div className="flex gap-1 flex-wrap">
                    {(keyRecord.scopes as string[]).map((scope) => (
                      <Badge key={scope} variant="outline" className="text-[10px]">
                        {scope}
                      </Badge>
                    ))}
                  </div>
                </div>
                {keyRecord.isActive && !keyRecord.revokedAt && (
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={() => setRevokeTarget(keyRecord)}
                    data-testid={`button-revoke-${keyRecord.id}`}
                  >
                    <Trash2 className="w-4 h-4 text-destructive" />
                  </Button>
                )}
              </div>
            ))}
          </div>
        )}
      </Card>

      <Card className="p-4 space-y-3">
        <h2 className="text-sm font-medium flex items-center gap-2">
          <ShieldCheck className="w-4 h-4" />
          Integration Guide
        </h2>
        <div className="text-xs text-muted-foreground space-y-2">
          <p>Use any of the following methods to authenticate API requests:</p>
          <div className="bg-muted rounded-md p-3 space-y-1 font-mono text-[11px]">
            <p className="text-foreground">X-API-Key: plm_your_key_here</p>
            <p className="text-foreground">Authorization: Bearer plm_your_key_here</p>
            <p className="text-foreground">?api_key=plm_your_key_here</p>
          </div>
          <p>
            Validate connectivity: <code className="bg-muted px-1 rounded">GET /api/keys/validate-external</code> with your key.
          </p>
        </div>
      </Card>

      <Dialog open={showCreateDialog} onOpenChange={(open) => {
        if (!open) {
          setShowCreateDialog(false);
          setGeneratedKey(null);
        }
      }}>
        <DialogContent className="max-w-lg">
          <DialogHeader>
            <DialogTitle>
              {generatedKey ? "Key Generated" : "Generate New API Key"}
            </DialogTitle>
            <DialogDescription>
              {generatedKey
                ? "Copy and store this key securely. It will not be shown again."
                : "Configure the name, scopes, and expiry for the new API key."}
            </DialogDescription>
          </DialogHeader>

          {generatedKey ? (
            <div className="space-y-4">
              <div className="flex items-center gap-2 bg-muted rounded-md p-3">
                <AlertTriangle className="w-4 h-4 text-amber-500 shrink-0" />
                <p className="text-xs text-amber-600 dark:text-amber-400">
                  Store this key securely. It will not be shown again.
                </p>
              </div>
              <div className="flex items-center gap-2">
                <Input
                  value={generatedKey}
                  readOnly
                  className="font-mono text-xs"
                  data-testid="input-generated-key"
                />
                <Button
                  size="icon"
                  variant="outline"
                  onClick={() => copyToClipboard(generatedKey)}
                  data-testid="button-copy-key"
                >
                  <Copy className="w-4 h-4" />
                </Button>
              </div>
              <DialogFooter>
                <Button
                  onClick={() => {
                    setShowCreateDialog(false);
                    setGeneratedKey(null);
                  }}
                  data-testid="button-done"
                >
                  Done
                </Button>
              </DialogFooter>
            </div>
          ) : (
            <div className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="key-name">Key Name</Label>
                <Input
                  id="key-name"
                  value={newKeyName}
                  onChange={(e) => setNewKeyName(e.target.value)}
                  placeholder="e.g., Production Integration"
                  data-testid="input-key-name"
                />
              </div>
              <div className="space-y-2">
                <Label>Expiry (days, 0 = never)</Label>
                <Input
                  type="number"
                  value={newKeyExpiry}
                  onChange={(e) => setNewKeyExpiry(parseInt(e.target.value) || 0)}
                  min={0}
                  max={3650}
                  data-testid="input-key-expiry"
                />
              </div>
              <div className="space-y-2">
                <Label>Scopes</Label>
                <ScopeSelector selected={newKeyScopes} onChange={setNewKeyScopes} />
              </div>
              <DialogFooter>
                <Button
                  variant="outline"
                  onClick={() => setShowCreateDialog(false)}
                  data-testid="button-cancel-create"
                >
                  Cancel
                </Button>
                <Button
                  onClick={handleGenerate}
                  disabled={generateMutation.isPending}
                  data-testid="button-submit-generate"
                >
                  {generateMutation.isPending ? "Generating..." : "Generate Key"}
                </Button>
              </DialogFooter>
            </div>
          )}
        </DialogContent>
      </Dialog>

      <Dialog open={!!revokeTarget} onOpenChange={(open) => !open && setRevokeTarget(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Revoke API Key</DialogTitle>
            <DialogDescription>
              Are you sure you want to revoke "{revokeTarget?.name}"? This action is permanent and cannot be undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setRevokeTarget(null)}
              data-testid="button-cancel-revoke"
            >
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={() => revokeTarget && revokeMutation.mutate(revokeTarget.id)}
              disabled={revokeMutation.isPending}
              data-testid="button-confirm-revoke"
            >
              {revokeMutation.isPending ? "Revoking..." : "Revoke Key"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
