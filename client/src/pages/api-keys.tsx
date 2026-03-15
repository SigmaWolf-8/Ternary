/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 */

import { useState, useMemo } from "react";
import { useAuth } from "@/hooks/use-auth";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { apiRequest } from "@/lib/queryClient";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
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
  RefreshCw,
  Gauge,
  Shield,
  Info,
  FileText,
  Search,
  Filter,
  Save,
  Pencil,
  Tag,
  Building2,
  FolderKanban,
  X,
  Users,
  ChevronDown,
  ChevronRight,
} from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
  DialogDescription,
} from "@/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";

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
  rotationScheduledAt: string | null;
  previousKeyId: string | null;
  rateLimitTier: string;
  rateLimitRpm: number;
  entityType: string | null;
  entityName: string | null;
  project: string | null;
  department: string | null;
  tags: string[] | null;
  notes: string | null;
  createdAt: string;
}

interface KeyStats {
  total: number;
  active: number;
  revoked: number;
  expired: number;
  totalUsage: number;
}

import { SCOPE_REGISTRY } from "@shared/scopes";

const SCOPE_CATEGORIES: Record<string, string[]> = Object.fromEntries(
  SCOPE_REGISTRY.map((cat) => [cat.label, cat.scopes.map((s) => s.id)])
);

const TIER_LABELS: Record<string, string> = {
  research: "Research (100 rpm)",
  pro: "Pro (500 rpm)",
  admin: "Admin (2000 rpm)",
};

const ENTITY_TYPE_OPTIONS = [
  { value: "customer", label: "Customer" },
  { value: "vendor", label: "Vendor" },
  { value: "partner", label: "Partner" },
  { value: "internal", label: "Internal" },
  { value: "contractor", label: "Contractor" },
  { value: "government", label: "Government" },
];

const ENTITY_TYPE_ICONS: Record<string, typeof Users> = {
  customer: Users,
  vendor: Building2,
  partner: Users,
  internal: ShieldCheck,
  contractor: Users,
  government: Building2,
};

function ScopeSelector({
  selected,
  onChange,
}: {
  selected: string[];
  onChange: (scopes: string[]) => void;
}) {
  const [expandedCategories, setExpandedCategories] = useState<Set<string>>(new Set());

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

  const toggleCategory = (catLabel: string) => {
    setExpandedCategories((prev) => {
      const next = new Set(prev);
      if (next.has(catLabel)) {
        next.delete(catLabel);
      } else {
        next.add(catLabel);
      }
      return next;
    });
  };

  const selectCategoryAll = (catScopes: string[]) => {
    const allSelected = catScopes.every((s) => selected.includes(s));
    if (allSelected) {
      onChange(selected.filter((s) => !catScopes.includes(s)));
    } else {
      const merged = new Set([...selected, ...catScopes]);
      onChange(Array.from(merged));
    }
  };

  const totalScopes = Object.values(SCOPE_CATEGORIES).flat().length;

  return (
    <div className="space-y-3">
      <div className="flex items-center gap-2 flex-wrap">
        <Button variant="outline" size="sm" onClick={selectAll} data-testid="button-select-all-scopes">
          Select All ({totalScopes})
        </Button>
        <Button variant="outline" size="sm" onClick={clearAll} data-testid="button-clear-all-scopes">
          Clear All
        </Button>
        <span className="text-xs text-muted-foreground ml-auto">
          {selected.length}/{totalScopes} selected
        </span>
      </div>
      <div className="space-y-1 max-h-[320px] overflow-y-auto pr-1">
        {SCOPE_REGISTRY.map((category) => {
          const catScopes = category.scopes.map((s) => s.id);
          const selectedCount = catScopes.filter((s) => selected.includes(s)).length;
          const isExpanded = expandedCategories.has(category.label);
          const allSelected = selectedCount === catScopes.length;

          return (
            <div key={category.id} className="border rounded-md" data-testid={`scope-category-${category.id}`}>
              <div
                className="flex items-center justify-between px-3 py-2 cursor-pointer hover:bg-muted/50 transition-colors"
                onClick={() => toggleCategory(category.label)}
                data-testid={`toggle-category-${category.id}`}
              >
                <div className="flex items-center gap-2">
                  {isExpanded ? (
                    <ChevronDown className="w-3.5 h-3.5 text-muted-foreground" />
                  ) : (
                    <ChevronRight className="w-3.5 h-3.5 text-muted-foreground" />
                  )}
                  <span className="text-sm font-medium">{category.label}</span>
                  {selectedCount > 0 && (
                    <Badge variant="secondary" className="text-[10px] h-5 px-1.5">
                      {selectedCount}/{catScopes.length}
                    </Badge>
                  )}
                </div>
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-6 text-[11px] px-2"
                  onClick={(e) => {
                    e.stopPropagation();
                    selectCategoryAll(catScopes);
                  }}
                  data-testid={`button-toggle-all-${category.id}`}
                >
                  {allSelected ? "Deselect" : "Select All"}
                </Button>
              </div>
              {isExpanded && (
                <div className="px-3 pb-2 flex gap-1 flex-wrap">
                  {category.scopes.map((scope) => (
                    <Badge
                      key={scope.id}
                      variant={selected.includes(scope.id) ? "default" : "outline"}
                      className="cursor-pointer select-none toggle-elevate text-[11px]"
                      onClick={() => toggle(scope.id)}
                      data-testid={`badge-scope-${scope.id}`}
                    >
                      {scope.id}
                    </Badge>
                  ))}
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}

function TagInput({
  tags,
  onChange,
}: {
  tags: string[];
  onChange: (tags: string[]) => void;
}) {
  const [input, setInput] = useState("");

  const addTag = () => {
    const trimmed = input.trim().toLowerCase();
    if (trimmed && !tags.includes(trimmed) && tags.length < 20) {
      onChange([...tags, trimmed]);
      setInput("");
    }
  };

  const removeTag = (tag: string) => {
    onChange(tags.filter((t) => t !== tag));
  };

  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2">
        <Input
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              e.preventDefault();
              addTag();
            }
          }}
          placeholder="Type tag and press Enter"
          className="flex-1"
          data-testid="input-tag"
        />
        <Button variant="outline" size="sm" onClick={addTag} data-testid="button-add-tag">
          <Plus className="w-3 h-3 mr-1" />
          Add
        </Button>
      </div>
      {tags.length > 0 && (
        <div className="flex gap-1 flex-wrap">
          {tags.map((tag) => (
            <Badge key={tag} variant="secondary" className="gap-1">
              <Tag className="w-2.5 h-2.5" />
              {tag}
              <button
                onClick={() => removeTag(tag)}
                className="ml-0.5 hover:text-destructive"
                data-testid={`button-remove-tag-${tag}`}
              >
                <X className="w-2.5 h-2.5" />
              </button>
            </Badge>
          ))}
        </div>
      )}
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

function EntityTypeBadge({ type }: { type: string }) {
  const Icon = ENTITY_TYPE_ICONS[type] || Users;
  return (
    <Badge variant="outline" className="text-[10px] gap-1">
      <Icon className="w-2.5 h-2.5" />
      {type}
    </Badge>
  );
}

function TierBadge({ tier }: { tier: string }) {
  const variant = tier === "admin" ? "default" : tier === "pro" ? "secondary" : "outline";
  return (
    <Badge variant={variant} className="text-[10px]" data-testid={`badge-tier-${tier}`}>
      <Gauge className="w-2.5 h-2.5 mr-1" />
      {tier}
    </Badge>
  );
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

function daysUntil(dateStr: string): number {
  return Math.ceil((new Date(dateStr).getTime() - Date.now()) / 86400000);
}

function getKeyStatus(key: ApiKeyRecord): string {
  if (key.revokedAt) return "revoked";
  if (key.expiresAt && new Date(key.expiresAt) < new Date()) return "expired";
  if (key.isActive) return "active";
  return "inactive";
}

export default function ApiKeysPage() {
  const { user, isAuthenticated, isLoading: authLoading } = useAuth();
  const { toast } = useToast();
  const queryClient = useQueryClient();

  const [showCreateDialog, setShowCreateDialog] = useState(false);
  const [newKeyName, setNewKeyName] = useState("");
  const [newKeyScopes, setNewKeyScopes] = useState<string[]>([]);
  const [newKeyExpiry, setNewKeyExpiry] = useState(90);
  const [newKeyTier, setNewKeyTier] = useState("research");
  const [newKeyRotation, setNewKeyRotation] = useState(true);
  const [newKeyEntityType, setNewKeyEntityType] = useState("");
  const [newKeyEntityName, setNewKeyEntityName] = useState("");
  const [newKeyProject, setNewKeyProject] = useState("");
  const [newKeyDepartment, setNewKeyDepartment] = useState("");
  const [newKeyTags, setNewKeyTags] = useState<string[]>([]);
  const [newKeyNotes, setNewKeyNotes] = useState("");
  const [generatedKey, setGeneratedKey] = useState<string | null>(null);

  const [revokeTarget, setRevokeTarget] = useState<ApiKeyRecord | null>(null);
  const [rotateTarget, setRotateTarget] = useState<ApiKeyRecord | null>(null);
  const [rotatedKey, setRotatedKey] = useState<string | null>(null);

  const [editTarget, setEditTarget] = useState<ApiKeyRecord | null>(null);
  const [editName, setEditName] = useState("");
  const [editEntityType, setEditEntityType] = useState("");
  const [editEntityName, setEditEntityName] = useState("");
  const [editProject, setEditProject] = useState("");
  const [editDepartment, setEditDepartment] = useState("");
  const [editTags, setEditTags] = useState<string[]>([]);
  const [editNotes, setEditNotes] = useState("");

  const [searchQuery, setSearchQuery] = useState("");
  const [filterStatus, setFilterStatus] = useState("all");
  const [filterEntityType, setFilterEntityType] = useState("all");
  const [filterTier, setFilterTier] = useState("all");
  const [filterTag, setFilterTag] = useState("all");
  const [showFilters, setShowFilters] = useState(false);

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

  const { data: expiringData } = useQuery<{
    success: boolean;
    keys: Array<{
      id: string;
      keyPrefix: string;
      name: string;
      owner: string;
      expiresAt: string | null;
      rotationScheduledAt: string | null;
      rateLimitTier: string;
      rateLimitRpm: number;
    }>;
    withinDays: number;
  }>({
    queryKey: ["/api/keys/expiring"],
    enabled: isAuthenticated,
  });

  const { data: anomaliesData } = useQuery<{
    success: boolean;
    anomalies: Array<{
      keyId: string;
      keyName: string;
      keyPrefix: string;
      type: string;
      severity: string;
      description: string;
      date: string;
      value: number;
    }>;
    withinDays: number;
  }>({
    queryKey: ["/api/keys/anomalies"],
    enabled: isAuthenticated,
  });

  const { data: auditData } = useQuery<{
    success: boolean;
    events: Array<{
      id: number;
      keyId: string;
      eventType: string;
      actorId: string;
      actorEmail: string | null;
      details: Record<string, unknown> | null;
      ipAddress: string | null;
      createdAt: string;
    }>;
  }>({
    queryKey: ["/api/keys/audit"],
    enabled: isAuthenticated,
  });

  const generateMutation = useMutation({
    mutationFn: async (body: {
      name: string;
      scopes: string[];
      expiresDays: number;
      rateLimitTier: string;
      enableRotation: boolean;
      entityType?: string;
      entityName?: string;
      project?: string;
      department?: string;
      tags?: string[];
      notes?: string;
    }) => {
      const res = await apiRequest("POST", "/api/keys/generate", body);
      return res.json();
    },
    onSuccess: (data) => {
      setGeneratedKey(data.key);
      setNewKeyName("");
      setNewKeyScopes([]);
      setNewKeyExpiry(90);
      setNewKeyTier("research");
      setNewKeyRotation(true);
      setNewKeyEntityType("");
      setNewKeyEntityName("");
      setNewKeyProject("");
      setNewKeyDepartment("");
      setNewKeyTags([]);
      setNewKeyNotes("");
      queryClient.invalidateQueries({ queryKey: ["/api/keys"] });
      queryClient.invalidateQueries({ queryKey: ["/api/keys/stats"] });
      queryClient.invalidateQueries({ queryKey: ["/api/keys/audit"] });
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
      queryClient.invalidateQueries({ queryKey: ["/api/keys/audit"] });
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

  const rotateMutation = useMutation({
    mutationFn: async (id: string) => {
      const res = await apiRequest("POST", `/api/keys/rotate/${id}`);
      return res.json();
    },
    onSuccess: (data) => {
      setRotatedKey(data.newKey);
      queryClient.invalidateQueries({ queryKey: ["/api/keys"] });
      queryClient.invalidateQueries({ queryKey: ["/api/keys/stats"] });
      queryClient.invalidateQueries({ queryKey: ["/api/keys/audit"] });
      toast({ title: "Key Rotated", description: "New key generated. Old key valid for 7 days." });
    },
    onError: () => {
      toast({
        title: "Error",
        description: "Failed to rotate key.",
        variant: "destructive",
      });
    },
  });

  const updateTierMutation = useMutation({
    mutationFn: async ({ id, tier }: { id: string; tier: string }) => {
      const res = await apiRequest("PATCH", `/api/keys/${id}/rate-limit`, { tier });
      return res.json();
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["/api/keys"] });
      queryClient.invalidateQueries({ queryKey: ["/api/keys/audit"] });
      toast({ title: "Rate Limit Updated", description: "Tier has been changed." });
    },
    onError: () => {
      toast({
        title: "Error",
        description: "Failed to update rate limit.",
        variant: "destructive",
      });
    },
  });

  const updateMetadataMutation = useMutation({
    mutationFn: async ({ id, data }: { id: string; data: Record<string, unknown> }) => {
      const res = await apiRequest("PATCH", `/api/keys/${id}/metadata`, data);
      return res.json();
    },
    onSuccess: () => {
      setEditTarget(null);
      queryClient.invalidateQueries({ queryKey: ["/api/keys"] });
      queryClient.invalidateQueries({ queryKey: ["/api/keys/audit"] });
      toast({ title: "Key Saved", description: "Key details have been updated." });
    },
    onError: () => {
      toast({
        title: "Error",
        description: "Failed to save key details.",
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
      rateLimitTier: newKeyTier,
      enableRotation: newKeyRotation,
      entityType: (newKeyEntityType && newKeyEntityType !== "none") ? newKeyEntityType : undefined,
      entityName: newKeyEntityName || undefined,
      project: newKeyProject || undefined,
      department: newKeyDepartment || undefined,
      tags: newKeyTags.length > 0 ? newKeyTags : undefined,
      notes: newKeyNotes || undefined,
    });
  };

  const openEditDialog = (keyRecord: ApiKeyRecord) => {
    setEditTarget(keyRecord);
    setEditName(keyRecord.name);
    setEditEntityType(keyRecord.entityType || "");
    setEditEntityName(keyRecord.entityName || "");
    setEditProject(keyRecord.project || "");
    setEditDepartment(keyRecord.department || "");
    setEditTags(keyRecord.tags || []);
    setEditNotes(keyRecord.notes || "");
  };

  const handleSaveMetadata = () => {
    if (!editTarget || !editName.trim()) {
      toast({
        title: "Validation Error",
        description: "Name is required.",
        variant: "destructive",
      });
      return;
    }
    updateMetadataMutation.mutate({
      id: editTarget.id,
      data: {
        name: editName,
        entityType: editEntityType || null,
        entityName: editEntityName || null,
        project: editProject || null,
        department: editDepartment || null,
        tags: editTags,
        notes: editNotes || null,
      },
    });
  };

  const keys = keysData?.keys || [];
  const stats = statsData?.stats;

  const allTags = useMemo(() => {
    const tagSet = new Set<string>();
    keys.forEach((k) => (k.tags || []).forEach((t) => tagSet.add(t)));
    return Array.from(tagSet).sort();
  }, [keys]);

  const allEntityTypes = useMemo(() => {
    const set = new Set<string>();
    keys.forEach((k) => {
      if (k.entityType) set.add(k.entityType);
    });
    return Array.from(set).sort();
  }, [keys]);

  const filteredKeys = useMemo(() => {
    let result = keys;

    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      result = result.filter(
        (k) =>
          k.name.toLowerCase().includes(q) ||
          k.keyPrefix.toLowerCase().includes(q) ||
          k.owner.toLowerCase().includes(q) ||
          (k.entityName && k.entityName.toLowerCase().includes(q)) ||
          (k.project && k.project.toLowerCase().includes(q)) ||
          (k.department && k.department.toLowerCase().includes(q)) ||
          (k.tags || []).some((t) => t.toLowerCase().includes(q))
      );
    }

    if (filterStatus !== "all") {
      result = result.filter((k) => getKeyStatus(k) === filterStatus);
    }

    if (filterEntityType !== "all") {
      result = result.filter((k) => k.entityType === filterEntityType);
    }

    if (filterTier !== "all") {
      result = result.filter((k) => k.rateLimitTier === filterTier);
    }

    if (filterTag !== "all") {
      result = result.filter((k) => (k.tags || []).includes(filterTag));
    }

    return result;
  }, [keys, searchQuery, filterStatus, filterEntityType, filterTier, filterTag]);

  const activeFilterCount = [filterStatus, filterEntityType, filterTier, filterTag].filter(
    (f) => f !== "all"
  ).length;

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

  return (
    <div className="p-4 md:p-6 space-y-6 max-w-6xl mx-auto">
      <div className="flex items-center justify-between gap-4 flex-wrap">
        <div>
          <h1 className="text-xl font-semibold flex items-center gap-2" data-testid="text-page-title">
            <Key className="w-5 h-5" />
            API Key Management
          </h1>
          <p className="text-sm text-muted-foreground mt-1">
            Generate, manage, rotate, and rate-limit API keys for external integrations.
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

      {expiringData && expiringData.keys.length > 0 && (
        <Card className="overflow-hidden border-amber-300 dark:border-amber-700">
          <div className="p-3 border-b bg-amber-50 dark:bg-amber-950/30 flex items-center gap-2">
            <AlertTriangle className="w-4 h-4 text-amber-600 dark:text-amber-400" />
            <h2 className="text-sm font-medium text-amber-700 dark:text-amber-300">
              Keys Expiring Within {expiringData.withinDays} Days ({expiringData.keys.length})
            </h2>
          </div>
          <div className="divide-y">
            {expiringData.keys.map((ek) => (
              <div
                key={ek.id}
                className="p-3 flex items-center justify-between gap-3 flex-wrap"
                data-testid={`row-expiring-key-${ek.id}`}
              >
                <div className="flex items-center gap-2 flex-wrap min-w-0">
                  <span className="font-medium text-sm">{ek.name}</span>
                  <TierBadge tier={ek.rateLimitTier} />
                  <code className="text-xs text-muted-foreground bg-muted px-1 rounded">
                    {ek.keyPrefix}...
                  </code>
                  {ek.expiresAt && (
                    <span className="text-xs text-amber-600 dark:text-amber-400 font-medium">
                      Expires in {daysUntil(ek.expiresAt)}d ({formatDate(ek.expiresAt)})
                    </span>
                  )}
                  {ek.rotationScheduledAt && (
                    <span className="text-xs text-blue-600 dark:text-blue-400">
                      Rotation scheduled
                    </span>
                  )}
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => {
                    const full = keys.find((k) => k.id === ek.id);
                    if (full) setRotateTarget(full);
                  }}
                  data-testid={`button-rotate-expiring-${ek.id}`}
                >
                  <RefreshCw className="w-3 h-3 mr-1" />
                  Rotate Now
                </Button>
              </div>
            ))}
          </div>
        </Card>
      )}

      <Card className="overflow-hidden">
        <div className="p-3 border-b space-y-3">
          <div className="flex items-center justify-between gap-3 flex-wrap">
            <h2 className="text-sm font-medium">API Keys</h2>
            <div className="flex items-center gap-2 flex-wrap">
              <div className="relative">
                <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground" />
                <Input
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  placeholder="Search keys, entities, tags..."
                  className="pl-8 w-[240px]"
                  data-testid="input-search-keys"
                />
              </div>
              <Button
                variant={showFilters ? "secondary" : "outline"}
                size="sm"
                onClick={() => setShowFilters(!showFilters)}
                data-testid="button-toggle-filters"
              >
                <Filter className="w-3.5 h-3.5 mr-1" />
                Filters
                {activeFilterCount > 0 && (
                  <Badge variant="default" className="ml-1 text-[10px] px-1.5 py-0">
                    {activeFilterCount}
                  </Badge>
                )}
              </Button>
            </div>
          </div>

          {showFilters && (
            <div className="flex items-end gap-3 flex-wrap pt-1">
              <div className="space-y-1">
                <Label className="text-xs text-muted-foreground">Status</Label>
                <Select value={filterStatus} onValueChange={setFilterStatus}>
                  <SelectTrigger className="w-[120px] h-8 text-xs" data-testid="select-filter-status">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">All</SelectItem>
                    <SelectItem value="active">Active</SelectItem>
                    <SelectItem value="revoked">Revoked</SelectItem>
                    <SelectItem value="expired">Expired</SelectItem>
                    <SelectItem value="inactive">Inactive</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-1">
                <Label className="text-xs text-muted-foreground">Entity Type</Label>
                <Select value={filterEntityType} onValueChange={setFilterEntityType}>
                  <SelectTrigger className="w-[130px] h-8 text-xs" data-testid="select-filter-entity-type">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">All</SelectItem>
                    {allEntityTypes.map((et) => (
                      <SelectItem key={et} value={et}>{et}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-1">
                <Label className="text-xs text-muted-foreground">Tier</Label>
                <Select value={filterTier} onValueChange={setFilterTier}>
                  <SelectTrigger className="w-[120px] h-8 text-xs" data-testid="select-filter-tier">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">All</SelectItem>
                    <SelectItem value="research">Research</SelectItem>
                    <SelectItem value="pro">Pro</SelectItem>
                    <SelectItem value="admin">Admin</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-1">
                <Label className="text-xs text-muted-foreground">Tag</Label>
                <Select value={filterTag} onValueChange={setFilterTag}>
                  <SelectTrigger className="w-[130px] h-8 text-xs" data-testid="select-filter-tag">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">All</SelectItem>
                    {allTags.map((tag) => (
                      <SelectItem key={tag} value={tag}>{tag}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              {activeFilterCount > 0 && (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => {
                    setFilterStatus("all");
                    setFilterEntityType("all");
                    setFilterTier("all");
                    setFilterTag("all");
                  }}
                  data-testid="button-clear-filters"
                >
                  <X className="w-3 h-3 mr-1" />
                  Clear
                </Button>
              )}
            </div>
          )}
        </div>
        {keysLoading ? (
          <div className="flex items-center justify-center p-8">
            <div className="inline-flex items-center justify-center w-6 h-6 rounded-full border-2 border-primary/20 border-t-primary animate-spin" />
          </div>
        ) : filteredKeys.length === 0 ? (
          <div className="flex flex-col items-center justify-center p-8 text-center">
            <Key className="w-8 h-8 text-muted-foreground mb-2" />
            <p className="text-sm text-muted-foreground">
              {keys.length === 0
                ? "No API keys created yet."
                : "No keys match the current filters."}
            </p>
            {keys.length === 0 && (
              <p className="text-xs text-muted-foreground mt-1">
                Generate your first key to start integrating with external services.
              </p>
            )}
          </div>
        ) : (
          <div className="divide-y">
            {filteredKeys.map((keyRecord) => (
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
                    <TierBadge tier={keyRecord.rateLimitTier} />
                    {keyRecord.entityType && (
                      <EntityTypeBadge type={keyRecord.entityType} />
                    )}
                    <code className="text-xs text-muted-foreground bg-muted px-1 rounded" data-testid={`text-key-prefix-${keyRecord.id}`}>
                      {keyRecord.keyPrefix}...
                    </code>
                  </div>
                  <div className="flex items-center gap-3 flex-wrap text-xs text-muted-foreground">
                    <span>Owner: {keyRecord.owner}</span>
                    {keyRecord.entityName && (
                      <span className="flex items-center gap-1">
                        <Building2 className="w-3 h-3" />
                        {keyRecord.entityName}
                      </span>
                    )}
                    {keyRecord.project && (
                      <span className="flex items-center gap-1">
                        <FolderKanban className="w-3 h-3" />
                        {keyRecord.project}
                      </span>
                    )}
                    {keyRecord.department && (
                      <span>{keyRecord.department}</span>
                    )}
                    <span>Created: {formatDate(keyRecord.createdAt)}</span>
                    {keyRecord.expiresAt && (
                      <span className={
                        daysUntil(keyRecord.expiresAt) <= 14
                          ? "text-amber-600 dark:text-amber-400 font-medium"
                          : ""
                      }>
                        Expires: {formatDate(keyRecord.expiresAt)}
                        {daysUntil(keyRecord.expiresAt) > 0 && daysUntil(keyRecord.expiresAt) <= 14 && (
                          ` (${daysUntil(keyRecord.expiresAt)}d)`
                        )}
                      </span>
                    )}
                    <span>{keyRecord.rateLimitRpm} rpm</span>
                    <span>Used: {keyRecord.usageCount.toLocaleString()} times</span>
                    {keyRecord.lastUsedAt && (
                      <span>Last: {formatDate(keyRecord.lastUsedAt)}</span>
                    )}
                    {keyRecord.rotationScheduledAt && (
                      <span className="text-blue-600 dark:text-blue-400">
                        Rotation: {formatDate(keyRecord.rotationScheduledAt)}
                      </span>
                    )}
                  </div>
                  <div className="flex gap-1 flex-wrap">
                    {(keyRecord.scopes as string[]).map((scope) => (
                      <Badge key={scope} variant="outline" className="text-[10px]">
                        {scope}
                      </Badge>
                    ))}
                  </div>
                  {(keyRecord.tags || []).length > 0 && (
                    <div className="flex gap-1 flex-wrap">
                      {(keyRecord.tags || []).map((tag) => (
                        <Badge key={tag} variant="secondary" className="text-[10px] gap-0.5">
                          <Tag className="w-2 h-2" />
                          {tag}
                        </Badge>
                      ))}
                    </div>
                  )}
                </div>
                <div className="flex items-center gap-1">
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={() => openEditDialog(keyRecord)}
                    data-testid={`button-edit-${keyRecord.id}`}
                  >
                    <Pencil className="w-4 h-4" />
                  </Button>
                  {keyRecord.isActive && !keyRecord.revokedAt && (
                    <>
                      <Select
                        value={keyRecord.rateLimitTier}
                        onValueChange={(tier) =>
                          updateTierMutation.mutate({ id: keyRecord.id, tier })
                        }
                      >
                        <SelectTrigger className="w-[110px] h-9 text-xs" data-testid={`select-tier-${keyRecord.id}`}>
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="research">Research</SelectItem>
                          <SelectItem value="pro">Pro</SelectItem>
                          <SelectItem value="admin">Admin</SelectItem>
                        </SelectContent>
                      </Select>
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => setRotateTarget(keyRecord)}
                        data-testid={`button-rotate-${keyRecord.id}`}
                      >
                        <RefreshCw className="w-4 h-4" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => setRevokeTarget(keyRecord)}
                        data-testid={`button-revoke-${keyRecord.id}`}
                      >
                        <Trash2 className="w-4 h-4 text-destructive" />
                      </Button>
                    </>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </Card>

      <Card className="overflow-hidden" data-testid="card-anomalies">
        <div className="p-3 border-b flex items-center gap-2">
          <Shield className="w-4 h-4" />
          <h2 className="text-sm font-medium">Security Alerts</h2>
          <Tooltip>
            <TooltipTrigger asChild>
              <Info className="w-3 h-3 text-muted-foreground cursor-help" />
            </TooltipTrigger>
            <TooltipContent side="right" className="max-w-xs text-xs">
              <p>Monitors for usage spikes ({">"}300% day-over-day), high failure rates ({">"}50 in 7 days), unusual IP dispersion ({">"}10 IPs in 24h), and tier escalations. Checked across the last 7 days.</p>
            </TooltipContent>
          </Tooltip>
        </div>
        {anomaliesData && anomaliesData.anomalies.length > 0 ? (
          <div className="divide-y">
            {anomaliesData.anomalies.map((a, idx) => (
              <div key={idx} className="p-3 flex items-center gap-3 flex-wrap" data-testid={`row-anomaly-${idx}`}>
                <Badge
                  variant={a.severity === "high" ? "destructive" : a.severity === "medium" ? "secondary" : "outline"}
                  className="text-[10px] shrink-0"
                >
                  {a.severity}
                </Badge>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 flex-wrap">
                    <span className="text-sm font-medium">{a.keyName}</span>
                    <code className="text-[10px] text-muted-foreground bg-muted px-1 rounded">{a.keyPrefix}...</code>
                    <Badge variant="outline" className="text-[10px]">{a.type.replace(/_/g, " ")}</Badge>
                  </div>
                  <p className="text-xs text-muted-foreground mt-0.5">{a.description}</p>
                </div>
                <span className="text-[10px] text-muted-foreground shrink-0">{formatDate(a.date)}</span>
              </div>
            ))}
          </div>
        ) : (
          <div className="p-6 text-center">
            <CheckCircle2 className="w-6 h-6 text-green-500 mx-auto mb-2" />
            <p className="text-sm text-muted-foreground">No anomalies detected in the last 7 days.</p>
          </div>
        )}
      </Card>

      <Card className="overflow-hidden" data-testid="card-audit-trail">
        <div className="p-3 border-b flex items-center gap-2">
          <FileText className="w-4 h-4" />
          <h2 className="text-sm font-medium">Audit Trail</h2>
          <Tooltip>
            <TooltipTrigger asChild>
              <Info className="w-3 h-3 text-muted-foreground cursor-help" />
            </TooltipTrigger>
            <TooltipContent side="right" className="max-w-xs text-xs">
              <p>Records every key lifecycle action: generation, revocation, rotation, tier changes, and metadata updates. Includes who performed the action and from which IP address.</p>
            </TooltipContent>
          </Tooltip>
        </div>
        {auditData && auditData.events.length > 0 ? (
          <div className="divide-y max-h-80 overflow-y-auto">
            {auditData.events.slice(0, 25).map((evt) => (
              <div key={evt.id} className="p-2 px-3 flex items-center gap-3 flex-wrap text-xs" data-testid={`row-audit-${evt.id}`}>
                <Badge variant="outline" className="text-[10px] shrink-0">
                  {evt.eventType.replace(/_/g, " ")}
                </Badge>
                <div className="flex-1 min-w-0 flex items-center gap-2 flex-wrap">
                  {evt.details && (evt.details as any).keyPrefix && (
                    <code className="text-[10px] text-muted-foreground bg-muted px-1 rounded">
                      {(evt.details as any).keyPrefix}...
                    </code>
                  )}
                  {evt.details && (evt.details as any).keyName && (
                    <span className="text-muted-foreground">{(evt.details as any).keyName}</span>
                  )}
                  {evt.eventType === "tier_change" && evt.details && (
                    <span className="text-muted-foreground">
                      {(evt.details as any).fromTier} {"->"} {(evt.details as any).toTier}
                    </span>
                  )}
                  <span className="text-muted-foreground">by {evt.actorEmail || evt.actorId}</span>
                </div>
                <span className="text-[10px] text-muted-foreground shrink-0">{formatDate(evt.createdAt)}</span>
              </div>
            ))}
          </div>
        ) : (
          <div className="p-6 text-center">
            <FileText className="w-6 h-6 text-muted-foreground mx-auto mb-2" />
            <p className="text-sm text-muted-foreground">No audit events recorded yet.</p>
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
          <div className="mt-2">
            <p className="font-medium text-foreground mb-1 flex items-center gap-1">
              Rate Limit Tiers
              <Tooltip>
                <TooltipTrigger asChild>
                  <Info className="w-3 h-3 text-muted-foreground cursor-help" />
                </TooltipTrigger>
                <TooltipContent side="top" className="max-w-xs text-xs">
                  <p><strong>Research</strong>: 100 requests/minute. For development and testing.</p>
                  <p><strong>Pro</strong>: 500 requests/minute. For production integrations.</p>
                  <p><strong>Admin</strong>: 2000 requests/minute. For internal and high-volume services.</p>
                  <p>Tier changes are audit-logged and require admin approval.</p>
                </TooltipContent>
              </Tooltip>
            </p>
            <div className="flex gap-3 flex-wrap">
              {Object.entries(TIER_LABELS).map(([tier, label]) => (
                <span key={tier} className="bg-muted px-2 py-0.5 rounded text-[11px]">{label}</span>
              ))}
            </div>
          </div>
          <div className="mt-2">
            <p className="font-medium text-foreground mb-1 flex items-center gap-1">
              Rotation & Grace Period
              <Tooltip>
                <TooltipTrigger asChild>
                  <Info className="w-3 h-3 text-muted-foreground cursor-help" />
                </TooltipTrigger>
                <TooltipContent side="top" className="max-w-xs text-xs">
                  <p>When a key is rotated, a new key is generated with the same scopes and tier. The old key remains valid for a <strong>7-day grace period</strong>, allowing you to update integrations without downtime. After 7 days, the old key is automatically revoked.</p>
                  <p>Auto-rotation runs every 6 hours and triggers when a key reaches its scheduled rotation date (typically at expiry).</p>
                </TooltipContent>
              </Tooltip>
            </p>
            <p>Keys can be rotated manually or automatically at expiry. Both old and new keys work during the 7-day overlap.</p>
          </div>
          <div className="mt-2">
            <p className="font-medium text-foreground mb-1 flex items-center gap-1">
              HPTP Binding
              <Tooltip>
                <TooltipTrigger asChild>
                  <Info className="w-3 h-3 text-muted-foreground cursor-help" />
                </TooltipTrigger>
                <TooltipContent side="top" className="max-w-xs text-xs">
                  <p>High-Precision Timing Protocol (HPTP) binding adds a femtosecond-precision timestamp to API key validation. This creates a time-bound authentication window, ensuring keys can only be used within specific timing constraints, providing an additional layer of quantum-resistant security.</p>
                </TooltipContent>
              </Tooltip>
            </p>
            <p>Optional timing-bound validation using the Salvi Framework's femtosecond-precision HPTP for quantum-resistant authentication.</p>
          </div>
          <p className="mt-2">
            Validate connectivity: <code className="bg-muted px-1 rounded">GET /api/keys/validate-external</code> with your key.
            Rate limit headers: <code className="bg-muted px-1 rounded">X-RateLimit-Limit</code>, <code className="bg-muted px-1 rounded">X-RateLimit-Remaining</code>, <code className="bg-muted px-1 rounded">X-RateLimit-Reset</code>.
          </p>
        </div>
      </Card>

      <Dialog open={showCreateDialog} onOpenChange={(open) => {
        if (!open) {
          setShowCreateDialog(false);
          setGeneratedKey(null);
        }
      }}>
        <DialogContent className="max-w-lg max-h-[85vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>
              {generatedKey ? "Key Generated" : "Generate New API Key"}
            </DialogTitle>
            <DialogDescription>
              {generatedKey
                ? "Copy and store this key securely. It will not be shown again."
                : "Configure the name, scopes, WBS tags, and settings for the new API key."}
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

              <div className="border rounded-md p-3 space-y-3">
                <p className="text-xs font-medium flex items-center gap-1.5">
                  <FolderKanban className="w-3.5 h-3.5" />
                  WBS Tagging
                </p>
                <div className="grid grid-cols-2 gap-3">
                  <div className="space-y-1">
                    <Label className="text-xs">Entity Type</Label>
                    <Select value={newKeyEntityType} onValueChange={setNewKeyEntityType}>
                      <SelectTrigger data-testid="select-new-entity-type">
                        <SelectValue placeholder="Select type..." />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="none">None</SelectItem>
                        {ENTITY_TYPE_OPTIONS.map((opt) => (
                          <SelectItem key={opt.value} value={opt.value}>{opt.label}</SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="space-y-1">
                    <Label className="text-xs">Entity Name</Label>
                    <Input
                      value={newKeyEntityName}
                      onChange={(e) => setNewKeyEntityName(e.target.value)}
                      placeholder="e.g., Acme Corp"
                      data-testid="input-new-entity-name"
                    />
                  </div>
                </div>
                <div className="grid grid-cols-2 gap-3">
                  <div className="space-y-1">
                    <Label className="text-xs">Project</Label>
                    <Input
                      value={newKeyProject}
                      onChange={(e) => setNewKeyProject(e.target.value)}
                      placeholder="e.g., Phase 2 Integration"
                      data-testid="input-new-project"
                    />
                  </div>
                  <div className="space-y-1">
                    <Label className="text-xs">Department</Label>
                    <Input
                      value={newKeyDepartment}
                      onChange={(e) => setNewKeyDepartment(e.target.value)}
                      placeholder="e.g., Engineering"
                      data-testid="input-new-department"
                    />
                  </div>
                </div>
                <div className="space-y-1">
                  <Label className="text-xs">Tags</Label>
                  <TagInput tags={newKeyTags} onChange={setNewKeyTags} />
                </div>
                <div className="space-y-1">
                  <Label className="text-xs">Notes</Label>
                  <Textarea
                    value={newKeyNotes}
                    onChange={(e) => setNewKeyNotes(e.target.value)}
                    placeholder="Optional notes about this key..."
                    className="text-xs resize-none"
                    rows={2}
                    data-testid="textarea-new-notes"
                  />
                </div>
              </div>

              <div className="grid grid-cols-2 gap-3">
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
                  <Label>Rate Limit Tier</Label>
                  <Select value={newKeyTier} onValueChange={setNewKeyTier}>
                    <SelectTrigger data-testid="select-key-tier">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="research">Research (100 rpm)</SelectItem>
                      <SelectItem value="pro">Pro (500 rpm)</SelectItem>
                      <SelectItem value="admin">Admin (2000 rpm)</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>
              <div className="flex items-center gap-2">
                <input
                  type="checkbox"
                  id="enable-rotation"
                  checked={newKeyRotation}
                  onChange={(e) => setNewKeyRotation(e.target.checked)}
                  className="rounded"
                  data-testid="checkbox-rotation"
                />
                <Label htmlFor="enable-rotation" className="text-sm cursor-pointer">
                  Enable auto-rotation (rotates at expiry, 7-day grace period)
                </Label>
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

      <Dialog open={!!editTarget} onOpenChange={(open) => !open && setEditTarget(null)}>
        <DialogContent className="max-w-lg max-h-[85vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <Pencil className="w-4 h-4" />
              Edit Key Details
            </DialogTitle>
            <DialogDescription>
              Update the name, WBS tags, and notes for "{editTarget?.name}". Changes are saved immediately.
            </DialogDescription>
          </DialogHeader>
          {editTarget && (
            <div className="space-y-4">
              <div className="flex items-center gap-2 text-xs text-muted-foreground">
                <code className="bg-muted px-1 rounded">{editTarget.keyPrefix}...</code>
                <KeyStatusBadge keyRecord={editTarget} />
                <TierBadge tier={editTarget.rateLimitTier} />
              </div>

              <div className="space-y-2">
                <Label htmlFor="edit-name">Key Name</Label>
                <Input
                  id="edit-name"
                  value={editName}
                  onChange={(e) => setEditName(e.target.value)}
                  data-testid="input-edit-name"
                />
              </div>

              <div className="border rounded-md p-3 space-y-3">
                <p className="text-xs font-medium flex items-center gap-1.5">
                  <FolderKanban className="w-3.5 h-3.5" />
                  WBS Tagging
                </p>
                <div className="grid grid-cols-2 gap-3">
                  <div className="space-y-1">
                    <Label className="text-xs">Entity Type</Label>
                    <Select value={editEntityType || "none"} onValueChange={(v) => setEditEntityType(v === "none" ? "" : v)}>
                      <SelectTrigger data-testid="select-edit-entity-type">
                        <SelectValue placeholder="Select type..." />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="none">None</SelectItem>
                        {ENTITY_TYPE_OPTIONS.map((opt) => (
                          <SelectItem key={opt.value} value={opt.value}>{opt.label}</SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="space-y-1">
                    <Label className="text-xs">Entity Name</Label>
                    <Input
                      value={editEntityName}
                      onChange={(e) => setEditEntityName(e.target.value)}
                      placeholder="e.g., Acme Corp"
                      data-testid="input-edit-entity-name"
                    />
                  </div>
                </div>
                <div className="grid grid-cols-2 gap-3">
                  <div className="space-y-1">
                    <Label className="text-xs">Project</Label>
                    <Input
                      value={editProject}
                      onChange={(e) => setEditProject(e.target.value)}
                      placeholder="e.g., Phase 2 Integration"
                      data-testid="input-edit-project"
                    />
                  </div>
                  <div className="space-y-1">
                    <Label className="text-xs">Department</Label>
                    <Input
                      value={editDepartment}
                      onChange={(e) => setEditDepartment(e.target.value)}
                      placeholder="e.g., Engineering"
                      data-testid="input-edit-department"
                    />
                  </div>
                </div>
                <div className="space-y-1">
                  <Label className="text-xs">Tags</Label>
                  <TagInput tags={editTags} onChange={setEditTags} />
                </div>
                <div className="space-y-1">
                  <Label className="text-xs">Notes</Label>
                  <Textarea
                    value={editNotes}
                    onChange={(e) => setEditNotes(e.target.value)}
                    placeholder="Optional notes..."
                    className="text-xs resize-none"
                    rows={2}
                    data-testid="textarea-edit-notes"
                  />
                </div>
              </div>

              <DialogFooter>
                <Button
                  variant="outline"
                  onClick={() => setEditTarget(null)}
                  data-testid="button-cancel-edit"
                >
                  Cancel
                </Button>
                <Button
                  onClick={handleSaveMetadata}
                  disabled={updateMetadataMutation.isPending}
                  data-testid="button-save-key"
                >
                  <Save className="w-4 h-4 mr-2" />
                  {updateMetadataMutation.isPending ? "Saving..." : "Save Key"}
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

      <Dialog open={!!rotateTarget || !!rotatedKey} onOpenChange={(open) => {
        if (!open) {
          setRotateTarget(null);
          setRotatedKey(null);
        }
      }}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{rotatedKey ? "Key Rotated" : "Rotate API Key"}</DialogTitle>
            <DialogDescription>
              {rotatedKey
                ? "A new key has been generated. The old key remains valid for 7 days."
                : `Rotate "${rotateTarget?.name}"? A new key will be generated with the same scopes. The old key stays valid for a 7-day grace period before automatic revocation.`}
            </DialogDescription>
          </DialogHeader>
          {rotatedKey ? (
            <div className="space-y-4">
              <div className="flex items-center gap-2 bg-muted rounded-md p-3">
                <AlertTriangle className="w-4 h-4 text-amber-500 shrink-0" />
                <p className="text-xs text-amber-600 dark:text-amber-400">
                  Store this new key securely. It will not be shown again.
                </p>
              </div>
              <div className="flex items-center gap-2">
                <Input
                  value={rotatedKey}
                  readOnly
                  className="font-mono text-xs"
                  data-testid="input-rotated-key"
                />
                <Button
                  size="icon"
                  variant="outline"
                  onClick={() => copyToClipboard(rotatedKey)}
                  data-testid="button-copy-rotated-key"
                >
                  <Copy className="w-4 h-4" />
                </Button>
              </div>
              <DialogFooter>
                <Button
                  onClick={() => {
                    setRotateTarget(null);
                    setRotatedKey(null);
                  }}
                  data-testid="button-done-rotate"
                >
                  Done
                </Button>
              </DialogFooter>
            </div>
          ) : (
            <DialogFooter>
              <Button
                variant="outline"
                onClick={() => setRotateTarget(null)}
                data-testid="button-cancel-rotate"
              >
                Cancel
              </Button>
              <Button
                onClick={() => rotateTarget && rotateMutation.mutate(rotateTarget.id)}
                disabled={rotateMutation.isPending}
                data-testid="button-confirm-rotate"
              >
                <RefreshCw className="w-4 h-4 mr-2" />
                {rotateMutation.isPending ? "Rotating..." : "Rotate Key"}
              </Button>
            </DialogFooter>
          )}
        </DialogContent>
      </Dialog>
    </div>
  );
}
