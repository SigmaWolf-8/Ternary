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

import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { 
  Box, 
  Shield,
  Zap,
  Network,
  Clock,
  Check,
  ExternalLink,
  Server,
  Lock,
  BarChart3,
  Layers,
  Globe,
  Cpu,
  RefreshCw,
  CheckCircle,
  XCircle,
  User,
  Route,
  Upload,
  GitBranch,
  Copy,
  Key,
  ChevronDown,
  ChevronRight,
  AlertCircle,
  Database,
  FileText,
  Settings,
  Activity
} from "lucide-react";
import { SiGithub } from "react-icons/si";
import { useState } from "react";
import { motion } from "framer-motion";
import { Link } from "wouter";
import { useQuery, useMutation } from "@tanstack/react-query";
import { apiRequest, queryClient } from "@/lib/queryClient";
import { useToast } from "@/hooks/use-toast";
import {
  getTotalEndpoints,
  getTotalServices,
  getKongServiceCatalog,
} from "@shared/service-catalog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

interface DataPlaneGroup {
  id: string;
  region: string;
  state: string;
  hostnames: string[];
  controlPlaneId: string;
  kind: string;
}

interface KongControlPlaneStatus {
  id: string;
  name: string;
  description?: string;
  clusterType?: string;
  controlPlaneEndpoint?: string;
  proxyUrls: string[];
  cloudGateway: boolean;
  dataPlaneState: string;
  services: number;
  routes: number;
  configSynced: boolean;
}

interface KongStatus {
  connected: boolean;
  gatewayReady?: boolean;
  configSynced?: boolean;
  activeProxyUrls?: string[];
  dataPlaneGroups?: DataPlaneGroup[];
  error?: string;
  user?: {
    id: string;
    email: string;
    fullName: string;
    preferredName: string;
    active: boolean;
  };
  controlPlanes?: KongControlPlaneStatus[];
}

interface ControlPlane {
  id: string;
  name: string;
  description?: string;
  cluster_type?: string;
  control_plane_endpoint?: string;
  created_at?: string;
  updated_at?: string;
}

interface ControlPlanesResponse {
  data: ControlPlane[];
}

interface Service {
  id: string;
  name: string;
  host: string;
  port: number;
  protocol: string;
  path?: string;
  enabled: boolean;
  tags?: string[];
}

interface RouteData {
  id: string;
  name: string;
  paths?: string[];
  methods?: string[];
  protocols?: string[];
}

const kongCategoryIcons: Record<string, typeof Shield> = {
  core: Cpu,
  tools: Database,
  reference: FileText,
  platform: Globe,
  admin: Settings
};

const kongCategoryLabels: Record<string, string> = {
  core: "Core Computing",
  tools: "Tools & Storage",
  reference: "Reference & Docs",
  platform: "Platform Services",
  admin: "Administration"
};

function ConnectionStatus() {
  const { data: status, isLoading, refetch } = useQuery<KongStatus>({
    queryKey: ['/api/kong/status'],
    refetchInterval: 30000
  });

  const syncedCount = status?.controlPlanes?.filter(cp => cp.configSynced).length || 0;
  const totalCPs = status?.controlPlanes?.length || 0;

  return (
    <Card>
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between gap-2">
          <CardTitle className="text-lg flex items-center gap-2">
            <Network className="w-5 h-5 text-primary" />
            Kong Konnect Connection
          </CardTitle>
          <Button 
            variant="ghost" 
            size="icon" 
            onClick={() => refetch()}
            disabled={isLoading}
            data-testid="button-refresh-status"
          >
            <RefreshCw className={`w-4 h-4 ${isLoading ? 'animate-spin' : ''}`} />
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <div className="flex items-center gap-2 text-muted-foreground">
            <RefreshCw className="w-4 h-4 animate-spin" />
            Checking connection...
          </div>
        ) : status?.connected ? (
          <div className="space-y-4">
            <div className="flex items-center gap-2 text-green-600 dark:text-green-400">
              <CheckCircle className="w-5 h-5" />
              <span className="font-medium">Connected to Kong Konnect</span>
            </div>
            {status.user && (
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 text-sm">
                <div className="flex items-center gap-2">
                  <User className="w-4 h-4 text-muted-foreground" />
                  <span className="text-muted-foreground">Account:</span>
                  <span className="font-medium">{status.user.fullName || status.user.preferredName || 'N/A'}</span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="text-muted-foreground">Email:</span>
                  <span className="font-medium truncate">{status.user.email}</span>
                </div>
              </div>
            )}
            <div className="border-t pt-3 grid grid-cols-1 sm:grid-cols-3 gap-3">
              <div className="flex items-center gap-2 text-sm">
                <Server className="w-4 h-4 text-muted-foreground" />
                <span className="text-muted-foreground">Control Planes:</span>
                <span className="font-medium">{totalCPs}</span>
              </div>
              <div className="flex items-center gap-2 text-sm">
                {syncedCount === totalCPs && totalCPs > 0 ? (
                  <CheckCircle className="w-4 h-4 text-green-600 dark:text-green-400" />
                ) : syncedCount > 0 ? (
                  <AlertCircle className="w-4 h-4 text-amber-500" />
                ) : (
                  <XCircle className="w-4 h-4 text-red-500" />
                )}
                <span className="text-muted-foreground">Synced:</span>
                <span className="font-medium">{syncedCount}/{totalCPs}</span>
              </div>
              <div className="flex items-center gap-2 text-sm">
                {status.gatewayReady ? (
                  <CheckCircle className="w-4 h-4 text-green-600 dark:text-green-400" />
                ) : (
                  <XCircle className="w-4 h-4 text-amber-500" />
                )}
                <span className="text-muted-foreground">Data Plane:</span>
                <span className="font-medium">{status.gatewayReady ? 'Ready' : 'Not connected'}</span>
              </div>
            </div>
            {status.activeProxyUrls && status.activeProxyUrls.length > 0 && (
              <div className="border-t pt-3 space-y-1">
                <span className="text-xs text-muted-foreground font-medium">Gateway Proxy Hostnames:</span>
                <div className="flex flex-wrap gap-1">
                  {status.activeProxyUrls.map((url, idx) => (
                    <Badge key={idx} variant="outline" className="text-xs font-mono">
                      {url}
                    </Badge>
                  ))}
                </div>
              </div>
            )}
          </div>
        ) : (
          <div className="flex items-center gap-2 text-red-600">
            <XCircle className="w-5 h-5" />
            <span>Not connected: {status?.error || 'Unknown error'}</span>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

function ControlPlanesOverview({ status }: { status: KongStatus | undefined }) {
  if (!status?.controlPlanes?.length) return null;

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-lg flex items-center gap-2">
          <Server className="w-5 h-5 text-primary" />
          Control Planes — Deployment Status
        </CardTitle>
        <CardDescription>
          Service deployment status across all {status.controlPlanes.length} control planes
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div className="space-y-3">
          {status.controlPlanes.map((cp) => (
            <div 
              key={cp.id}
              className="p-4 border rounded-md"
              data-testid={`cp-status-${cp.id}`}
            >
              <div className="flex items-center justify-between gap-2 flex-wrap">
                <div className="flex items-center gap-3">
                  <div>
                    <h4 className="font-medium">{cp.name}</h4>
                    {cp.description && (
                      <p className="text-sm text-muted-foreground">{cp.description}</p>
                    )}
                  </div>
                </div>
                <div className="flex items-center gap-2 flex-wrap">
                  <Badge variant={cp.configSynced ? "default" : "secondary"} className="text-xs">
                    {cp.services} services
                  </Badge>
                  <Badge variant={cp.configSynced ? "default" : "secondary"} className="text-xs">
                    {cp.routes} routes
                  </Badge>
                  <Badge variant="outline" className="text-xs">
                    {cp.clusterType?.replace("CLUSTER_TYPE_", "") || 'Standard'}
                  </Badge>
                  {cp.configSynced ? (
                    <Badge variant="default" className="text-xs">
                      <CheckCircle className="w-3 h-3 mr-1" />
                      Synced
                    </Badge>
                  ) : (
                    <Badge variant="secondary" className="text-xs">
                      <AlertCircle className="w-3 h-3 mr-1" />
                      Needs Sync
                    </Badge>
                  )}
                </div>
              </div>
              {cp.proxyUrls.length > 0 && (
                <div className="mt-2 flex flex-wrap gap-1">
                  {cp.proxyUrls.map((url, idx) => (
                    <Badge key={idx} variant="outline" className="text-xs font-mono">{url}</Badge>
                  ))}
                </div>
              )}
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}

function ServiceCatalogSection() {
  const catalog = getKongServiceCatalog('https://plenumnet.replit.app');

  const [expandedServices, setExpandedServices] = useState<Set<string>>(new Set());

  const toggleService = (name: string) => {
    setExpandedServices(prev => {
      const next = new Set(prev);
      if (next.has(name)) next.delete(name);
      else next.add(name);
      return next;
    });
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between gap-2 flex-wrap">
          <div>
            <CardTitle className="text-lg flex items-center gap-2">
              <Layers className="w-5 h-5 text-primary" />
              PlenumNET Service Catalog
            </CardTitle>
            <CardDescription>
              Complete API service inventory for Kong Gateway deployment
            </CardDescription>
          </div>
          <div className="flex gap-2">
            <Badge variant="default" data-testid="badge-total-services">
              {catalog.totalServices} Services
            </Badge>
            <Badge variant="outline" data-testid="badge-total-endpoints">
              {catalog.totalEndpoints} Endpoints
            </Badge>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <div className="space-y-6">
          {Object.entries(catalog.categories).map(([catKey, services]) => {
            if (!services.length) return null;
            const CatIcon = kongCategoryIcons[catKey] || Layers;
            const catLabel = kongCategoryLabels[catKey] || catKey;
            const catEndpoints = services.reduce((s: number, svc: any) => s + svc.endpointCount, 0);
            return (
              <div key={catKey} data-testid={`catalog-category-${catKey}`}>
                <div className="flex items-center gap-2 mb-3">
                  <CatIcon className="w-4 h-4 text-primary" />
                  <h3 className="font-medium text-sm">{catLabel}</h3>
                  <Badge variant="secondary" className="text-xs">{services.length} services</Badge>
                  <Badge variant="outline" className="text-xs">{catEndpoints} endpoints</Badge>
                </div>
                <div className="space-y-2">
                  {services.map((svc: any) => (
                    <div 
                      key={svc.name} 
                      className="border rounded-md overflow-hidden"
                      data-testid={`catalog-service-${svc.name}`}
                    >
                      <button
                        className="w-full flex items-center justify-between p-3 text-left hover-elevate"
                        onClick={() => toggleService(svc.name)}
                        data-testid={`button-expand-${svc.name}`}
                      >
                        <div className="flex items-center gap-2 min-w-0">
                          {expandedServices.has(svc.name) ? (
                            <ChevronDown className="w-4 h-4 text-muted-foreground flex-shrink-0" />
                          ) : (
                            <ChevronRight className="w-4 h-4 text-muted-foreground flex-shrink-0" />
                          )}
                          <code className="text-xs bg-secondary px-1.5 py-0.5 rounded font-mono flex-shrink-0">{svc.name}</code>
                          <span className="text-sm text-muted-foreground truncate">{svc.label}</span>
                        </div>
                        <div className="flex items-center gap-2 flex-shrink-0">
                          <Badge variant="outline" className="text-xs font-mono">{svc.routePath}</Badge>
                          <Badge variant="secondary" className="text-xs">{svc.endpointCount}</Badge>
                        </div>
                      </button>
                      {expandedServices.has(svc.name) && (
                        <div className="border-t px-3 py-2 bg-secondary/30">
                          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-1">
                            {svc.endpoints.map((ep: string, idx: number) => {
                              const [method, ...pathParts] = ep.split(" ");
                              const epPath = pathParts.join(" ");
                              return (
                                <div key={idx} className="flex items-center gap-1.5 text-xs py-0.5">
                                  <Badge 
                                    variant={method === "GET" ? "outline" : method === "POST" ? "default" : "secondary"} 
                                    className="text-[10px] px-1 py-0 min-w-[36px] text-center"
                                  >
                                    {method}
                                  </Badge>
                                  <code className="text-muted-foreground font-mono">{epPath}</code>
                                </div>
                              );
                            })}
                          </div>
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              </div>
            );
          })}
        </div>
      </CardContent>
    </Card>
  );
}

function SyncSection({ selectedCP, setSelectedCP }: { selectedCP: string | null; setSelectedCP: (cp: string | null) => void }) {
  const { toast } = useToast();
  const [githubOwner, setGithubOwner] = useState("SigmaWolf-8");
  const [githubRepo, setGithubRepo] = useState("Ternary");
  const [githubPath, setGithubPath] = useState("kong/kong.yaml");

  const { data: cpData } = useQuery<ControlPlanesResponse>({
    queryKey: ['/api/kong/control-planes']
  });
  const controlPlanes = cpData?.data || [];

  const syncMutation = useMutation({
    mutationFn: async (cpId: string) => {
      const response = await apiRequest("POST", `/api/kong/control-planes/${cpId}/sync-plenumnet`, {});
      return response.json();
    },
    onSuccess: (data) => {
      toast({
        title: "Sync Complete",
        description: `${data.totalServices || data.services || 0} services deployed (${data.totalEndpoints || '90+'} endpoints), ${data.routes || 0} routes, ${data.plugins || 0} plugins. Errors: ${data.errors || 0}`
      });
      queryClient.invalidateQueries({ queryKey: ['/api/kong/status'] });
      queryClient.invalidateQueries({ queryKey: ['/api/kong/control-planes'] });
    },
    onError: (error) => {
      toast({
        title: "Sync Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "destructive"
      });
    }
  });

  const syncAllMutation = useMutation({
    mutationFn: async () => {
      const response = await apiRequest("POST", "/api/kong/sync-all-control-planes", {});
      return response.json();
    },
    onSuccess: (data) => {
      const successCount = data.results?.filter((r: any) => r.success).length || 0;
      toast({
        title: "All Control Planes Synced",
        description: `${successCount}/${data.controlPlanesProcessed} control planes synced successfully`
      });
      queryClient.invalidateQueries({ queryKey: ['/api/kong/status'] });
      queryClient.invalidateQueries({ queryKey: ['/api/kong/control-planes'] });
    },
    onError: (error) => {
      toast({
        title: "Sync All Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "destructive"
      });
    }
  });

  const githubMutation = useMutation({
    mutationFn: async () => {
      const response = await apiRequest("POST", "/api/kong/save-to-github", {
        owner: githubOwner,
        repo: githubRepo,
        path: githubPath,
        message: `Update Kong Konnect configuration - ${new Date().toISOString()}`
      });
      return response.json();
    },
    onSuccess: (data) => {
      toast({
        title: "Saved to GitHub",
        description: data.message
      });
    },
    onError: (error) => {
      toast({
        title: "GitHub Save Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "destructive"
      });
    }
  });

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <Upload className="w-5 h-5 text-primary" />
            Deploy Services to Kong
          </CardTitle>
          <CardDescription>
            Sync all {getTotalServices()} PlenumNET services ({getTotalEndpoints()} endpoints) to your Kong control planes
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="p-4 border rounded-md bg-secondary/30">
            <h4 className="text-sm font-medium mb-3 flex items-center gap-2">
              <Globe className="w-4 h-4 text-primary" />
              Sync All Control Planes
            </h4>
            <p className="text-sm text-muted-foreground mb-3">
              Deploy all {getTotalServices()} services with routes and rate-limiting plugins to every control plane simultaneously.
            </p>
            <Button 
              onClick={() => syncAllMutation.mutate()}
              disabled={syncAllMutation.isPending || controlPlanes.length === 0}
              className="w-full"
              data-testid="button-sync-all"
            >
              {syncAllMutation.isPending ? (
                <>
                  <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
                  Syncing All {controlPlanes.length} Control Planes...
                </>
              ) : (
                <>
                  <Upload className="w-4 h-4 mr-2" />
                  Sync All {controlPlanes.length} Control Planes
                </>
              )}
            </Button>
          </div>

          <div className="border-t pt-4">
            <h4 className="text-sm font-medium mb-3 flex items-center gap-2">
              <Server className="w-4 h-4 text-muted-foreground" />
              Or sync individual control plane:
            </h4>
            <div className="grid grid-cols-1 gap-2 mb-3">
              {controlPlanes.map((cp) => (
                <button
                  key={cp.id}
                  className={`p-3 border rounded-md text-left transition-colors ${
                    selectedCP === cp.id 
                      ? 'border-primary bg-primary/5' 
                      : 'hover-elevate'
                  }`}
                  onClick={() => setSelectedCP(cp.id)}
                  data-testid={`select-cp-${cp.id}`}
                >
                  <div className="flex items-center justify-between gap-2">
                    <span className="font-medium text-sm">{cp.name}</span>
                    <Badge variant="outline" className="text-xs">
                      {cp.cluster_type?.replace("CLUSTER_TYPE_", "") || 'Standard'}
                    </Badge>
                  </div>
                </button>
              ))}
            </div>
            <Button 
              onClick={() => selectedCP && syncMutation.mutate(selectedCP)}
              disabled={!selectedCP || syncMutation.isPending}
              variant="outline"
              className="w-full"
              data-testid="button-sync-single"
            >
              {syncMutation.isPending ? (
                <>
                  <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
                  Syncing...
                </>
              ) : (
                <>
                  <Upload className="w-4 h-4 mr-2" />
                  {selectedCP 
                    ? `Sync to ${controlPlanes.find(cp => cp.id === selectedCP)?.name || 'Selected'}`
                    : 'Select a control plane first'
                  }
                </>
              )}
            </Button>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <SiGithub className="w-5 h-5" />
            Save to GitHub
          </CardTitle>
          <CardDescription>
            Store Kong configuration in your GitHub repository for version control
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-3">
            <div className="space-y-1">
              <Label htmlFor="github-owner" className="text-xs">Owner</Label>
              <Input
                id="github-owner"
                value={githubOwner}
                onChange={(e) => setGithubOwner(e.target.value)}
                placeholder="username"
                data-testid="input-github-owner"
              />
            </div>
            <div className="space-y-1">
              <Label htmlFor="github-repo" className="text-xs">Repository</Label>
              <Input
                id="github-repo"
                value={githubRepo}
                onChange={(e) => setGithubRepo(e.target.value)}
                placeholder="repo-name"
                data-testid="input-github-repo"
              />
            </div>
          </div>
          <div className="space-y-1">
            <Label htmlFor="github-path" className="text-xs">File Path</Label>
            <Input
              id="github-path"
              value={githubPath}
              onChange={(e) => setGithubPath(e.target.value)}
              placeholder="kong/kong.yaml"
              data-testid="input-github-path"
            />
          </div>
          <Button 
            onClick={() => githubMutation.mutate()}
            disabled={githubMutation.isPending || !githubOwner || !githubRepo}
            variant="outline"
            className="w-full"
            data-testid="button-save-github"
          >
            {githubMutation.isPending ? (
              <>
                <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
                Saving...
              </>
            ) : (
              <>
                <GitBranch className="w-4 h-4 mr-2" />
                Save Configuration to GitHub
              </>
            )}
          </Button>
          <p className="text-xs text-muted-foreground text-center">
            Requires GitHub token configured in GitHub Manager
          </p>
        </CardContent>
      </Card>
    </div>
  );
}

function ControlPlaneDetails({ selectedCP }: { selectedCP: string | null }) {
  const { data: servicesData, isLoading: servicesLoading } = useQuery<{ data: Service[] }>({
    queryKey: ['/api/kong/control-planes', selectedCP, 'services'],
    enabled: !!selectedCP
  });

  const { data: routesData, isLoading: routesLoading } = useQuery<{ data: RouteData[] }>({
    queryKey: ['/api/kong/control-planes', selectedCP, 'routes'],
    enabled: !!selectedCP
  });

  if (!selectedCP) return null;

  return (
    <div className="grid md:grid-cols-2 gap-6">
      <Card>
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <Layers className="w-5 h-5 text-primary" />
            Deployed Services
          </CardTitle>
          <CardDescription>
            Services registered on the selected control plane
          </CardDescription>
        </CardHeader>
        <CardContent>
          {servicesLoading ? (
            <div className="flex items-center gap-2 text-muted-foreground">
              <RefreshCw className="w-4 h-4 animate-spin" />
              Loading...
            </div>
          ) : !servicesData?.data?.length ? (
            <p className="text-muted-foreground text-sm">No services deployed yet. Click Sync above to deploy.</p>
          ) : (
            <div className="space-y-2">
              {servicesData.data.map((service) => (
                <div key={service.id} className="p-3 bg-secondary/50 rounded-md" data-testid={`deployed-service-${service.name}`}>
                  <div className="flex items-center justify-between gap-2">
                    <span className="font-medium text-sm">{service.name}</span>
                    <Badge variant={service.enabled ? "default" : "secondary"} className="text-xs">
                      {service.enabled ? 'Active' : 'Disabled'}
                    </Badge>
                  </div>
                  <p className="text-xs text-muted-foreground mt-1 font-mono">
                    {service.protocol}://{service.host}:{service.port}{service.path || ''}
                  </p>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <Route className="w-5 h-5 text-primary" />
            Deployed Routes
          </CardTitle>
          <CardDescription>
            Route rules on the selected control plane
          </CardDescription>
        </CardHeader>
        <CardContent>
          {routesLoading ? (
            <div className="flex items-center gap-2 text-muted-foreground">
              <RefreshCw className="w-4 h-4 animate-spin" />
              Loading...
            </div>
          ) : !routesData?.data?.length ? (
            <p className="text-muted-foreground text-sm">No routes deployed yet.</p>
          ) : (
            <div className="space-y-2">
              {routesData.data.map((route) => (
                <div key={route.id} className="p-3 bg-secondary/50 rounded-md" data-testid={`deployed-route-${route.name || route.id}`}>
                  <span className="font-medium text-sm">{route.name || route.id}</span>
                  {route.paths && route.paths.length > 0 && (
                    <div className="flex flex-wrap gap-1 mt-1">
                      {route.paths.map((path, idx) => (
                        <Badge key={idx} variant="outline" className="text-xs font-mono">
                          {path}
                        </Badge>
                      ))}
                    </div>
                  )}
                  {route.methods && route.methods.length > 0 && (
                    <div className="flex flex-wrap gap-1 mt-1">
                      {route.methods.map((method, idx) => (
                        <Badge key={idx} variant="secondary" className="text-xs">
                          {method}
                        </Badge>
                      ))}
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

function HeroSection() {
  return (
    <section className="py-12 md:py-16 border-b" data-testid="section-hero">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5 }}
          >
            <Badge variant="outline" className="mb-6 border-primary/30 bg-primary/10 text-primary">
              API Gateway Integration
            </Badge>
          </motion.div>
          
          <motion.h1
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-4xl md:text-5xl font-bold leading-tight mb-6"
            data-testid="text-hero-title"
          >
            PlenumNET + <span className="text-primary">Kong Konnect</span>
          </motion.h1>
          
          <motion.p
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.2 }}
            className="text-lg text-muted-foreground mb-4 max-w-2xl mx-auto"
            data-testid="text-hero-description"
          >
            Manage your Kong Konnect API gateway directly from PlenumNET. 
            Deploy {getTotalServices()} services covering {getTotalEndpoints()} endpoints across all control planes.
          </motion.p>

          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.3 }}
            className="flex flex-wrap gap-3 justify-center"
          >
            <Badge variant="secondary" className="text-xs">
              <Activity className="w-3 h-3 mr-1" />
              {getTotalServices()} API Services
            </Badge>
            <Badge variant="secondary" className="text-xs">
              <Globe className="w-3 h-3 mr-1" />
              {getTotalEndpoints()} Endpoints
            </Badge>
            <Badge variant="secondary" className="text-xs">
              <Shield className="w-3 h-3 mr-1" />
              Rate Limited
            </Badge>
            <Badge variant="secondary" className="text-xs">
              <Lock className="w-3 h-3 mr-1" />
              CNSA 2.0
            </Badge>
          </motion.div>
        </div>
      </div>
    </section>
  );
}

function DashboardSection() {
  const [selectedCP, setSelectedCP] = useState<string | null>(null);
  
  const { data: status } = useQuery<KongStatus>({
    queryKey: ['/api/kong/status'],
    refetchInterval: 30000
  });

  return (
    <section className="py-8 md:py-12" data-testid="section-dashboard">
      <div className="max-w-7xl mx-auto px-5 space-y-6">
        <ConnectionStatus />
        <ControlPlanesOverview status={status} />
        <ServiceCatalogSection />
        <SyncSection selectedCP={selectedCP} setSelectedCP={setSelectedCP} />
        <ControlPlaneDetails selectedCP={selectedCP} />
      </div>
    </section>
  );
}

function FeaturesSection() {
  const features = [
    {
      icon: Shield,
      title: "Post-Quantum API Security",
      description: "Combine PlenumNET's quantum-resistant protocols with Kong's advanced authentication."
    },
    {
      icon: Clock,
      title: "Femtosecond Timing",
      description: "Route timing-critical APIs through Kong with precision targeting FINRA Rule 613 CAT timing requirements."
    },
    {
      icon: Network,
      title: "Multi-Cloud Deployment",
      description: "Deploy ternary APIs across AWS, Azure, and GCP with Kong's unified control plane."
    },
    {
      icon: Zap,
      title: "High-Performance Routing",
      description: "Sub-millisecond latency for ternary data transfers with intelligent load balancing."
    },
    {
      icon: BarChart3,
      title: "Real-Time Analytics",
      description: "Monitor API usage, performance metrics, and ternary compression ratios."
    },
    {
      icon: Lock,
      title: "Enterprise Authentication",
      description: "OAuth 2.0, JWT, mTLS, and API key authentication for secure ternary data access."
    }
  ];

  return (
    <section className="py-12 md:py-16 bg-secondary/30" data-testid="section-features">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-10">
          <h2 className="text-2xl md:text-3xl font-bold mb-4">Integration Features</h2>
          <p className="text-muted-foreground max-w-2xl mx-auto">
            Leverage Kong Konnect's API management capabilities with PlenumNET's ternary computing infrastructure.
          </p>
        </div>
        
        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
          {features.map((feature, index) => (
            <motion.div
              key={feature.title}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: index * 0.1 }}
            >
              <Card className="h-full">
                <CardHeader>
                  <div className="w-10 h-10 rounded-md bg-primary/10 flex items-center justify-center mb-3">
                    <feature.icon className="w-5 h-5 text-primary" />
                  </div>
                  <CardTitle className="text-base">{feature.title}</CardTitle>
                </CardHeader>
                <CardContent>
                  <CardDescription className="text-sm">{feature.description}</CardDescription>
                </CardContent>
              </Card>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}

function CTASection() {
  return (
    <section className="py-12 md:py-16" data-testid="section-cta">
      <div className="max-w-7xl mx-auto px-5">
        <Card className="bg-primary text-primary-foreground overflow-visible">
          <CardContent className="p-8 md:p-10 text-center">
            <h2 className="text-2xl md:text-3xl font-bold mb-4">
              Need Help With Kong Konnect?
            </h2>
            <p className="text-lg opacity-90 mb-6 max-w-xl mx-auto">
              Visit the Kong Konnect documentation or explore PlenumNET APIs.
            </p>
            <div className="flex flex-wrap gap-4 justify-center">
              <Button size="lg" variant="secondary" asChild data-testid="button-cta-kong">
                <a href="https://docs.konghq.com/konnect/" target="_blank" rel="noopener noreferrer">
                  Kong Konnect Docs
                  <ExternalLink className="w-4 h-4 ml-2" />
                </a>
              </Button>
              <Link href="/api-demo">
                <Button size="lg" variant="outline" className="border-primary-foreground/30 text-primary-foreground" data-testid="button-cta-api-demo">
                  Explore PlenumNET APIs
                </Button>
              </Link>
            </div>
          </CardContent>
        </Card>
      </div>
    </section>
  );
}

function Footer() {
  return (
    <footer className="bg-background border-t py-8">
      <div className="max-w-7xl mx-auto px-5">
        <div className="flex flex-col md:flex-row items-center justify-between gap-4">
          <Link href="/" className="flex items-center gap-2 text-primary font-bold text-xl">
            <Box className="w-6 h-6" />
            <span>PlenumNET</span>
          </Link>
          
          <p className="text-sm text-muted-foreground">
            All Rights Reserved and Preserved | &copy; Capomastro Holdings Ltd 2026
          </p>
        </div>
      </div>
    </footer>
  );
}

export default function KongKonnect() {
  return (
    <div className="min-h-screen bg-background text-foreground">
      <main>
        <HeroSection />
        <DashboardSection />
        <FeaturesSection />
        <CTASection />
      </main>
      <Footer />
    </div>
  );
}
