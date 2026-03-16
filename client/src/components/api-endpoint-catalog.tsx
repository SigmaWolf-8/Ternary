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

import { useState, useCallback } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Clock, Calculator, Shield, Database, TrendingUp, Cpu, Key,
  Globe, Network, Radio, Star, FileText, Activity, Lock,
  Search, Play, RefreshCw, ChevronDown, ChevronRight, Copy, Check,
  Server, GitBranch, Scale, Heart, Gauge, BookOpen, Layers, Workflow,
  Stamp, Link2, Hexagon
} from "lucide-react";
import {
  SERVICE_CATALOG,
  DOMAIN_GROUPS,
  SERVICE_MAP,
  getTotalEndpoints,
  type ServiceDef,
  type EndpointDef,
  type HttpMethod,
} from "@shared/service-catalog";

const ICON_MAP: Record<string, React.ReactNode> = {
  Clock: <Clock className="w-4 h-4" />,
  Calculator: <Calculator className="w-4 h-4" />,
  Shield: <Shield className="w-4 h-4" />,
  Database: <Database className="w-4 h-4" />,
  TrendingUp: <TrendingUp className="w-4 h-4" />,
  Cpu: <Cpu className="w-4 h-4" />,
  Key: <Key className="w-4 h-4" />,
  Globe: <Globe className="w-4 h-4" />,
  Network: <Network className="w-4 h-4" />,
  Radio: <Radio className="w-4 h-4" />,
  Star: <Star className="w-4 h-4" />,
  FileText: <FileText className="w-4 h-4" />,
  Activity: <Activity className="w-4 h-4" />,
  Lock: <Lock className="w-4 h-4" />,
  Server: <Server className="w-4 h-4" />,
  GitBranch: <GitBranch className="w-4 h-4" />,
  Scale: <Scale className="w-4 h-4" />,
  Gauge: <Gauge className="w-4 h-4" />,
  BookOpen: <BookOpen className="w-4 h-4" />,
  Layers: <Layers className="w-4 h-4" />,
  Workflow: <Workflow className="w-4 h-4" />,
  Stamp: <Stamp className="w-4 h-4" />,
  Link2: <Link2 className="w-4 h-4" />,
  Hexagon: <Hexagon className="w-4 h-4" />,
};

const METHOD_COLORS: Record<HttpMethod, string> = {
  GET: "outline",
  POST: "secondary",
  PUT: "default",
  PATCH: "default",
  DELETE: "destructive",
};

function TryItPanel({ endpoint }: { endpoint: EndpointDef }) {
  const [result, setResult] = useState<any>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const execute = useCallback(async () => {
    setLoading(true);
    setError(null);
    setResult(null);
    try {
      const res = await fetch(endpoint.path);
      const text = await res.text();
      try {
        setResult(JSON.parse(text));
      } catch {
        setResult(text);
      }
    } catch (err: any) {
      setError(err.message || "Request failed");
    } finally {
      setLoading(false);
    }
  }, [endpoint.path]);

  return (
    <div className="mt-2 space-y-2">
      <Button
        size="sm"
        variant="outline"
        onClick={execute}
        disabled={loading}
        className="h-7 text-xs"
        data-testid={`tryit-${endpoint.path.replace(/\//g, "-").slice(1)}`}
      >
        {loading ? (
          <RefreshCw className="w-3 h-3 mr-1 animate-spin" />
        ) : (
          <Play className="w-3 h-3 mr-1" />
        )}
        Try it
      </Button>
      {error && (
        <div className="text-xs text-destructive bg-destructive/10 rounded px-2 py-1">
          {error}
        </div>
      )}
      {result && (
        <div className="max-h-64 overflow-auto rounded border border-border">
          <pre className="text-xs bg-muted/50 p-2 whitespace-pre-wrap break-all font-mono" data-testid={`result-${endpoint.path.replace(/\//g, "-").slice(1)}`}>
            {typeof result === "string" ? result : JSON.stringify(result, null, 2)}
          </pre>
        </div>
      )}
    </div>
  );
}

function EndpointRow({ endpoint }: { endpoint: EndpointDef }) {
  const [copied, setCopied] = useState(false);
  const [expanded, setExpanded] = useState(false);

  const copyPath = () => {
    navigator.clipboard.writeText(endpoint.path);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  return (
    <div className="group border-b last:border-b-0 border-border/50 py-2">
      <div className="flex items-start gap-2">
        <Badge
          variant={METHOD_COLORS[endpoint.method] as any}
          className="shrink-0 text-xs font-mono w-16 justify-center"
        >
          {endpoint.method}
        </Badge>
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-1">
            <code className="text-xs break-all font-mono">{endpoint.path}</code>
            {endpoint.admin && (
              <Badge variant="outline" className="shrink-0 text-[10px] h-4 px-1 border-amber-300 text-amber-600">
                admin
              </Badge>
            )}
          </div>
          <div className="text-xs text-muted-foreground mt-0.5">{endpoint.desc}</div>
        </div>
        <div className="flex items-center gap-1 shrink-0">
          <Button
            size="icon"
            variant="ghost"
            onClick={copyPath}
            data-testid={`copy-${endpoint.path.replace(/\//g, "-").slice(1)}`}
          >
            {copied ? <Check className="w-3 h-3 text-green-500" /> : <Copy className="w-3 h-3" />}
          </Button>
          {endpoint.tryIt && endpoint.method === "GET" && !endpoint.path.includes(":") && (
            <Button
              size="icon"
              variant="ghost"
              onClick={() => setExpanded(!expanded)}
              data-testid={`expand-${endpoint.path.replace(/\//g, "-").slice(1)}`}
            >
              {expanded ? <ChevronDown className="w-3 h-3" /> : <ChevronRight className="w-3 h-3" />}
            </Button>
          )}
        </div>
      </div>
      {expanded && endpoint.tryIt && endpoint.method === "GET" && !endpoint.path.includes(":") && (
        <TryItPanel endpoint={endpoint} />
      )}
    </div>
  );
}

function CategoryCard({ service, defaultOpen }: { service: ServiceDef; defaultOpen: boolean }) {
  const [isOpen, setIsOpen] = useState(defaultOpen);
  const getCount = service.endpoints.length;
  const adminCount = service.endpoints.filter(e => e.admin).length;
  const publicCount = getCount - adminCount;

  return (
    <Card data-testid={`category-${service.id}`} className={service.deprecated ? "opacity-75" : ""}>
      <CardHeader
        className="pb-2 cursor-pointer select-none"
        onClick={() => setIsOpen(!isOpen)}
        data-testid={`toggle-category-${service.id}`}
      >
        <div className="flex items-center justify-between">
          <CardTitle className={`text-base flex items-center gap-2 ${service.color}`}>
            {ICON_MAP[service.icon] || <Activity className="w-4 h-4" />}
            {service.name}
            {service.deprecated && (
              <Badge variant="outline" className="text-[10px] h-4 px-1 border-orange-300 text-orange-500 font-normal">
                legacy
              </Badge>
            )}
          </CardTitle>
          <div className="flex items-center gap-2">
            <Badge variant="secondary" className="text-xs">
              {publicCount} public{adminCount > 0 ? ` + ${adminCount} admin` : ""}
            </Badge>
            {isOpen ? <ChevronDown className="w-4 h-4 text-muted-foreground" /> : <ChevronRight className="w-4 h-4 text-muted-foreground" />}
          </div>
        </div>
        <CardDescription>{service.description}</CardDescription>
      </CardHeader>
      {isOpen && (
        <CardContent>
          <div>
            {service.endpoints.map((ep, i) => (
              <EndpointRow key={`${ep.method}-${ep.path}-${i}`} endpoint={ep} />
            ))}
          </div>
        </CardContent>
      )}
    </Card>
  );
}

function DomainGroupSection({ group, services, activeFilter, search }: {
  group: typeof DOMAIN_GROUPS[0];
  services: ServiceDef[];
  activeFilter: string | null;
  search: string;
}) {
  const totalInGroup = services.reduce((s, c) => s + c.endpoints.length, 0);
  if (services.length === 0) return null;

  return (
    <div className="space-y-3" data-testid={`domain-${group.id}`}>
      <div className="flex items-center gap-3 pt-2">
        <h3 className="text-lg font-semibold tracking-tight" data-testid={`domain-title-${group.id}`}>
          {group.name}
        </h3>
        <Badge variant="outline" className="text-xs font-mono">
          {totalInGroup}
        </Badge>
        <div className="flex-1 h-px bg-border" />
      </div>
      <p className="text-xs text-muted-foreground -mt-1 mb-2">{group.description}</p>
      <div className="space-y-3">
        {services.map(svc => (
          <CategoryCard
            key={svc.id}
            service={svc}
            defaultOpen={activeFilter === svc.id || services.length === 1}
          />
        ))}
      </div>
    </div>
  );
}

export default function APIEndpointCatalog() {
  const [search, setSearch] = useState("");
  const [activeFilter, setActiveFilter] = useState<string | null>(null);
  const [activeDomain, setActiveDomain] = useState<string | null>(null);

  const totalEndpoints = getTotalEndpoints();

  const filterService = (svc: ServiceDef): ServiceDef | null => {
    if (activeFilter && svc.id !== activeFilter) return null;
    if (activeDomain) {
      const group = DOMAIN_GROUPS.find(g => g.id === activeDomain);
      if (group && !group.serviceIds.includes(svc.id)) return null;
    }
    if (!search) return svc;
    const lower = search.toLowerCase();
    const filteredEndpoints = svc.endpoints.filter(
      ep => ep.path.toLowerCase().includes(lower) || ep.desc.toLowerCase().includes(lower) || ep.method.toLowerCase().includes(lower)
    );
    if (filteredEndpoints.length === 0 && !svc.name.toLowerCase().includes(lower)) return null;
    return { ...svc, endpoints: filteredEndpoints.length > 0 ? filteredEndpoints : svc.endpoints };
  };

  const filteredServiceMap = new Map<string, ServiceDef>();
  SERVICE_CATALOG.forEach(svc => {
    const filtered = filterService(svc);
    if (filtered) filteredServiceMap.set(svc.id, filtered);
  });

  const filteredCount = Array.from(filteredServiceMap.values()).reduce((sum, svc) => sum + svc.endpoints.length, 0);
  const hasResults = filteredServiceMap.size > 0;

  const handleDomainClick = (domainId: string) => {
    if (activeDomain === domainId) {
      setActiveDomain(null);
    } else {
      setActiveDomain(domainId);
      setActiveFilter(null);
    }
  };

  const handleServiceClick = (svcId: string) => {
    if (activeFilter === svcId) {
      setActiveFilter(null);
    } else {
      setActiveFilter(svcId);
      setActiveDomain(null);
    }
  };

  return (
    <div className="space-y-6" data-testid="api-endpoint-catalog">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h2 className="text-2xl font-bold" data-testid="text-catalog-title">
            Complete API Reference
          </h2>
          <p className="text-muted-foreground text-sm mt-1" data-testid="text-catalog-subtitle">
            {totalEndpoints} endpoints across {SERVICE_CATALOG.length} services in {DOMAIN_GROUPS.length} domains.
            {search || activeFilter || activeDomain ? ` Showing ${filteredCount} endpoints.` : ""}
            {" "}Hover any endpoint to copy its path or try GET endpoints live.
          </p>
        </div>
        <div className="relative w-full md:w-72">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          <Input
            placeholder="Search endpoints..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="pl-9"
            data-testid="input-search-endpoints"
          />
        </div>
      </div>

      <div className="space-y-1.5">
        <div className="flex flex-wrap gap-1">
          <Button
            size="sm"
            variant={!activeFilter && !activeDomain ? "default" : "outline"}
            onClick={() => { setActiveFilter(null); setActiveDomain(null); }}
            className="h-6 text-[11px] px-2"
            data-testid="filter-all"
          >
            All ({totalEndpoints})
          </Button>
          {DOMAIN_GROUPS.map(group => {
            const groupCount = group.serviceIds.reduce((s, id) => {
              const svc = SERVICE_MAP.get(id);
              return s + (svc ? svc.endpoints.length : 0);
            }, 0);
            return (
              <Button
                key={group.id}
                size="sm"
                variant={activeDomain === group.id ? "default" : "outline"}
                onClick={() => handleDomainClick(group.id)}
                className="h-6 text-[11px] px-2"
                data-testid={`filter-domain-${group.id}`}
              >
                {group.name} ({groupCount})
              </Button>
            );
          })}
        </div>
        {(activeDomain || activeFilter) && (
          <div className="flex flex-wrap gap-1 pl-1">
            {(activeDomain ? DOMAIN_GROUPS.find(g => g.id === activeDomain)?.serviceIds || [] : SERVICE_CATALOG.map(c => c.id)).map(svcId => {
              const svc = SERVICE_MAP.get(svcId);
              if (!svc) return null;
              return (
                <Button
                  key={svc.id}
                  size="sm"
                  variant={activeFilter === svc.id ? "secondary" : "ghost"}
                  onClick={() => handleServiceClick(svc.id)}
                  className="h-5 text-[10px] px-1.5"
                  data-testid={`filter-${svc.id}`}
                >
                  {svc.name} ({svc.endpoints.length})
                </Button>
              );
            })}
          </div>
        )}
      </div>

      <div className="space-y-8">
        {activeFilter ? (
          <div className="space-y-3">
            {Array.from(filteredServiceMap.values()).map(svc => (
              <CategoryCard
                key={svc.id}
                service={svc}
                defaultOpen={true}
              />
            ))}
          </div>
        ) : (
          DOMAIN_GROUPS.map(group => {
            const groupServices = group.serviceIds
              .map(id => filteredServiceMap.get(id))
              .filter(Boolean) as ServiceDef[];
            if (groupServices.length === 0) return null;
            return (
              <DomainGroupSection
                key={group.id}
                group={group}
                services={groupServices}
                activeFilter={activeFilter}
                search={search}
              />
            );
          })
        )}
        {!hasResults && (
          <div className="text-center py-12 text-muted-foreground" data-testid="text-no-results">
            No endpoints match your search.
          </div>
        )}
      </div>
    </div>
  );
}
