import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { 
  Box, 
  ArrowLeft,
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
  Building,
  Route,
  Plug,
  Upload,
  FileCode,
  GitBranch,
  Terminal,
  Copy,
  Play,
  Download
} from "lucide-react";
import { SiGithub } from "react-icons/si";
import { useState } from "react";
import { motion } from "framer-motion";
import { Link } from "wouter";
import { useQuery, useMutation } from "@tanstack/react-query";
import { apiRequest, queryClient } from "@/lib/queryClient";
import { useToast } from "@/hooks/use-toast";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

interface KongStatus {
  connected: boolean;
  error?: string;
  user?: {
    id: string;
    email: string;
    fullName: string;
    preferredName: string;
    active: boolean;
  };
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
}

interface RouteData {
  id: string;
  name: string;
  paths?: string[];
  methods?: string[];
  protocols?: string[];
}

function Header() {
  return (
    <header className="border-b border-primary/10 bg-background/95 backdrop-blur-xl sticky top-0 z-50">
      <div className="max-w-7xl mx-auto px-5 py-4 flex items-center justify-between gap-4">
        <Link href="/" className="flex items-center gap-2.5 text-primary font-bold text-xl" data-testid="link-logo">
          <Box className="w-7 h-7" />
          <span>PlenumNET</span>
        </Link>
        
        <div className="flex items-center gap-4">
          <Link href="/api-demo">
            <Button variant="ghost" size="sm" data-testid="button-api-demo">
              API Demo
            </Button>
          </Link>
          <Link href="/">
            <Button variant="ghost" size="sm" data-testid="button-back-home">
              <ArrowLeft className="w-4 h-4 mr-2" />
              Back to Home
            </Button>
          </Link>
        </div>
      </div>
    </header>
  );
}

function ConnectionStatus() {
  const { data: status, isLoading, refetch } = useQuery<KongStatus>({
    queryKey: ['/api/kong/status'],
    refetchInterval: 30000
  });

  return (
    <Card className="border-primary/20">
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="text-lg flex items-center gap-2">
            <Network className="w-5 h-5 text-primary" />
            Connection Status
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
          <div className="space-y-3">
            <div className="flex items-center gap-2 text-green-600">
              <CheckCircle className="w-5 h-5" />
              <span className="font-medium">Connected to Kong Konnect</span>
            </div>
            {status.user && (
              <div className="grid grid-cols-2 gap-3 text-sm mt-4">
                <div className="flex items-center gap-2">
                  <User className="w-4 h-4 text-muted-foreground" />
                  <span className="text-muted-foreground">User:</span>
                  <span className="font-medium">{status.user.fullName || status.user.preferredName || 'N/A'}</span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="text-muted-foreground">Email:</span>
                  <span className="font-medium">{status.user.email}</span>
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

function ControlPlanesList({ selectedCP, setSelectedCP }: { selectedCP: string | null; setSelectedCP: (cp: string | null) => void }) {
  
  const { data: cpData, isLoading: cpLoading } = useQuery<ControlPlanesResponse>({
    queryKey: ['/api/kong/control-planes']
  });

  const { data: servicesData, isLoading: servicesLoading } = useQuery<{ data: Service[] }>({
    queryKey: ['/api/kong/control-planes', selectedCP, 'services'],
    enabled: !!selectedCP
  });

  const { data: routesData, isLoading: routesLoading } = useQuery<{ data: RouteData[] }>({
    queryKey: ['/api/kong/control-planes', selectedCP, 'routes'],
    enabled: !!selectedCP
  });

  const controlPlanes = cpData?.data || [];

  return (
    <div className="space-y-6">
      <Card className="border-primary/20">
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <Server className="w-5 h-5 text-primary" />
            Control Planes
          </CardTitle>
          <CardDescription>
            Your Kong Konnect control planes
          </CardDescription>
        </CardHeader>
        <CardContent>
          {cpLoading ? (
            <div className="flex items-center gap-2 text-muted-foreground">
              <RefreshCw className="w-4 h-4 animate-spin" />
              Loading control planes...
            </div>
          ) : controlPlanes.length === 0 ? (
            <p className="text-muted-foreground text-sm">No control planes found. Create one in Kong Konnect to get started.</p>
          ) : (
            <div className="space-y-3">
              {controlPlanes.map((cp) => (
                <div 
                  key={cp.id}
                  className={`p-4 border rounded-lg cursor-pointer transition-colors ${
                    selectedCP === cp.id 
                      ? 'border-primary bg-primary/5' 
                      : 'border-border hover:border-primary/50'
                  }`}
                  onClick={() => setSelectedCP(cp.id)}
                  data-testid={`control-plane-${cp.id}`}
                >
                  <div className="flex items-center justify-between">
                    <div>
                      <h4 className="font-medium">{cp.name}</h4>
                      {cp.description && (
                        <p className="text-sm text-muted-foreground">{cp.description}</p>
                      )}
                    </div>
                    <Badge variant="outline" className="text-xs">
                      {cp.cluster_type || 'Standard'}
                    </Badge>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {selectedCP && (
        <div className="grid md:grid-cols-2 gap-6">
          <Card className="border-primary/20">
            <CardHeader>
              <CardTitle className="text-lg flex items-center gap-2">
                <Layers className="w-5 h-5 text-primary" />
                Services
              </CardTitle>
            </CardHeader>
            <CardContent>
              {servicesLoading ? (
                <div className="flex items-center gap-2 text-muted-foreground">
                  <RefreshCw className="w-4 h-4 animate-spin" />
                  Loading services...
                </div>
              ) : !servicesData?.data?.length ? (
                <p className="text-muted-foreground text-sm">No services configured</p>
              ) : (
                <div className="space-y-2">
                  {servicesData.data.map((service) => (
                    <div key={service.id} className="p-3 bg-secondary/50 rounded-lg">
                      <div className="flex items-center justify-between">
                        <span className="font-medium text-sm">{service.name}</span>
                        <Badge variant={service.enabled ? "default" : "secondary"} className="text-xs">
                          {service.enabled ? 'Enabled' : 'Disabled'}
                        </Badge>
                      </div>
                      <p className="text-xs text-muted-foreground mt-1">
                        {service.protocol}://{service.host}:{service.port}{service.path || ''}
                      </p>
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>

          <Card className="border-primary/20">
            <CardHeader>
              <CardTitle className="text-lg flex items-center gap-2">
                <Route className="w-5 h-5 text-primary" />
                Routes
              </CardTitle>
            </CardHeader>
            <CardContent>
              {routesLoading ? (
                <div className="flex items-center gap-2 text-muted-foreground">
                  <RefreshCw className="w-4 h-4 animate-spin" />
                  Loading routes...
                </div>
              ) : !routesData?.data?.length ? (
                <p className="text-muted-foreground text-sm">No routes configured</p>
              ) : (
                <div className="space-y-2">
                  {routesData.data.map((route) => (
                    <div key={route.id} className="p-3 bg-secondary/50 rounded-lg">
                      <span className="font-medium text-sm">{route.name || route.id}</span>
                      {route.paths && route.paths.length > 0 && (
                        <div className="flex flex-wrap gap-1 mt-1">
                          {route.paths.map((path, idx) => (
                            <Badge key={idx} variant="outline" className="text-xs">
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
      )}
    </div>
  );
}

function SyncSection({ selectedCP }: { selectedCP: string | null }) {
  const { toast } = useToast();
  const [githubOwner, setGithubOwner] = useState("SigmaWolf-8");
  const [githubRepo, setGithubRepo] = useState("Ternary");
  const [githubPath, setGithubPath] = useState("kong/kong.yaml");

  const syncMutation = useMutation({
    mutationFn: async (cpId: string) => {
      const response = await apiRequest("POST", `/api/kong/control-planes/${cpId}/sync-plenumnet`, {});
      return response.json();
    },
    onSuccess: (data) => {
      toast({
        title: "Sync Complete",
        description: `Services: ${data.services || 0}, Routes: ${data.routes || 0}, Plugins: ${data.plugins || 0}, Errors: ${data.errors || 0}`
      });
      queryClient.invalidateQueries({ queryKey: ['/api/kong/control-planes', selectedCP, 'services'] });
      queryClient.invalidateQueries({ queryKey: ['/api/kong/control-planes', selectedCP, 'routes'] });
    },
    onError: (error) => {
      toast({
        title: "Sync Failed",
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
    <div className="grid md:grid-cols-2 gap-6">
      <Card className="border-primary/20">
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <Upload className="w-5 h-5 text-primary" />
            Sync to Kong Konnect
          </CardTitle>
          <CardDescription>
            Deploy PlenumNET API services to your Kong control plane
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <p className="text-sm text-muted-foreground">
            This will create the following services in Kong:
          </p>
          <ul className="text-sm space-y-1">
            <li className="flex items-center gap-2">
              <Check className="w-4 h-4 text-green-500" />
              <code className="text-xs bg-secondary px-1 rounded">plenumnet-timing</code> - Femtosecond Timing API
            </li>
            <li className="flex items-center gap-2">
              <Check className="w-4 h-4 text-green-500" />
              <code className="text-xs bg-secondary px-1 rounded">plenumnet-ternary</code> - Ternary Operations API
            </li>
            <li className="flex items-center gap-2">
              <Check className="w-4 h-4 text-green-500" />
              <code className="text-xs bg-secondary px-1 rounded">plenumnet-phase</code> - Phase Encryption API
            </li>
            <li className="flex items-center gap-2">
              <Check className="w-4 h-4 text-green-500" />
              <code className="text-xs bg-secondary px-1 rounded">plenumnet-demo</code> - Demo Compression API
            </li>
            <li className="flex items-center gap-2">
              <Check className="w-4 h-4 text-green-500" />
              <code className="text-xs bg-secondary px-1 rounded">plenumnet-whitepapers</code> - Whitepaper API
            </li>
            <li className="flex items-center gap-2">
              <Check className="w-4 h-4 text-green-500" />
              <code className="text-xs bg-secondary px-1 rounded">plenumnet-docs</code> - API Documentation
            </li>
          </ul>
          <Button 
            onClick={() => selectedCP && syncMutation.mutate(selectedCP)}
            disabled={!selectedCP || syncMutation.isPending}
            className="w-full"
            data-testid="button-sync-kong"
          >
            {syncMutation.isPending ? (
              <>
                <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
                Syncing...
              </>
            ) : (
              <>
                <Upload className="w-4 h-4 mr-2" />
                Sync PlenumNET to Kong
              </>
            )}
          </Button>
          {!selectedCP && (
            <p className="text-xs text-muted-foreground text-center">
              Select a control plane above to sync
            </p>
          )}
        </CardContent>
      </Card>

      <Card className="border-primary/20">
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <SiGithub className="w-5 h-5" />
            Save to GitHub
          </CardTitle>
          <CardDescription>
            Store Kong configuration in your GitHub repository for version control and AI agents
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-3">
            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-1">
                <Label htmlFor="github-owner" className="text-xs">Owner</Label>
                <Input
                  id="github-owner"
                  value={githubOwner}
                  onChange={(e) => setGithubOwner(e.target.value)}
                  placeholder="username"
                  className="h-8 text-sm"
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
                  className="h-8 text-sm"
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
                className="h-8 text-sm"
                data-testid="input-github-path"
              />
            </div>
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

interface DeploymentPackage {
  success: boolean;
  message: string;
  certificateUploaded: boolean;
  certificateId?: string;
  controlPlane: {
    id: string;
    name: string;
    endpoint: string;
  };
  files: {
    "tls.crt": string;
    "tls.key": string;
    "docker-compose.yml": string;
    "deploy.sh": string;
  };
  instructions: string[];
}

interface CloudDeployResult {
  success: boolean;
  message: string;
  platform: string;
  githubRepo: string;
  deployUrls: {
    render: string;
    railway: string;
  };
  controlPlane: {
    id: string;
    name: string;
    endpoint: string;
  };
  instructions: string[];
}

function DeploySection({ selectedCP }: { selectedCP: string | null }) {
  const { toast } = useToast();
  const [copied, setCopied] = useState(false);
  const [deploymentPackage, setDeploymentPackage] = useState<DeploymentPackage | null>(null);
  const [cloudDeployResult, setCloudDeployResult] = useState<CloudDeployResult | null>(null);
  const [githubOwner, setGithubOwner] = useState("SigmaWolf-8");
  const [githubRepo, setGithubRepo] = useState("Ternary");

  const { data: deployData, isLoading, error } = useQuery<{
    success: boolean;
    controlPlane: {
      id: string;
      name: string;
      clusterType: string;
      controlPlaneEndpoint: string;
      telemetryEndpoint: string;
      proxyUrls: Array<{ host: string; port: number; protocol: string }>;
    };
    hasProxyUrl: boolean;
    deploymentInstructions: {
      docker: {
        title: string;
        description: string;
        prerequisites: string[];
        steps: string[];
        command: string;
      };
    };
  }>({
    queryKey: ['/api/kong/control-planes', selectedCP, 'deploy-instructions'],
    enabled: !!selectedCP,
  });

  const generateMutation = useMutation({
    mutationFn: async (cpId: string) => {
      const response = await apiRequest("POST", `/api/kong/control-planes/${cpId}/generate-deployment`, {});
      return response.json();
    },
    onSuccess: (data: DeploymentPackage) => {
      setDeploymentPackage(data);
      toast({
        title: "Deployment Package Generated!",
        description: data.certificateUploaded 
          ? "Certificates uploaded to Kong Konnect" 
          : "Package ready for download"
      });
    },
    onError: (error) => {
      toast({
        title: "Generation Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "destructive"
      });
    }
  });

  const cloudDeployMutation = useMutation({
    mutationFn: async ({ cpId, platform }: { cpId: string; platform: string }) => {
      const response = await apiRequest("POST", `/api/kong/control-planes/${cpId}/deploy-to-cloud`, {
        platform,
        owner: githubOwner,
        repo: githubRepo
      });
      return response.json();
    },
    onSuccess: (data: CloudDeployResult) => {
      setCloudDeployResult(data);
      toast({
        title: "Deployment Ready!",
        description: "Click the deploy link to launch your Kong Gateway"
      });
    },
    onError: (error) => {
      toast({
        title: "Cloud Deploy Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "destructive"
      });
    }
  });

  const downloadFile = (filename: string, content: string) => {
    const blob = new Blob([content], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  const downloadAllFiles = () => {
    if (!deploymentPackage) return;
    Object.entries(deploymentPackage.files).forEach(([filename, content]) => {
      setTimeout(() => downloadFile(filename, content), 100);
    });
  };

  const copyCommand = async () => {
    if (deployData?.deploymentInstructions?.docker?.command) {
      await navigator.clipboard.writeText(deployData.deploymentInstructions.docker.command);
      setCopied(true);
      toast({ title: "Copied!", description: "Docker command copied to clipboard" });
      setTimeout(() => setCopied(false), 2000);
    }
  };

  if (!selectedCP) {
    return null;
  }

  return (
    <Card className="border-primary/20">
      <CardHeader>
        <CardTitle className="text-lg flex items-center gap-2">
          <Terminal className="w-5 h-5 text-primary" />
          Deploy Data Plane
        </CardTitle>
        <CardDescription>
          Deploy a Kong Gateway data plane to access your APIs through Kong proxy
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {isLoading ? (
          <div className="flex items-center justify-center py-4">
            <RefreshCw className="w-5 h-5 animate-spin text-muted-foreground" />
          </div>
        ) : error ? (
          <p className="text-sm text-red-500">Failed to load deployment instructions</p>
        ) : deployData?.hasProxyUrl ? (
          <div className="space-y-3">
            <div className="flex items-center gap-2 text-green-600">
              <CheckCircle className="w-5 h-5" />
              <span className="font-medium">Data Plane Active</span>
            </div>
            <p className="text-sm text-muted-foreground">
              Your Kong Gateway is ready. Access APIs through the proxy URL.
            </p>
            {deployData.controlPlane.proxyUrls.map((url, i) => (
              <div key={i} className="flex items-center gap-2">
                <code className="text-xs bg-secondary px-2 py-1 rounded flex-1">
                  {url.protocol}://{url.host}:{url.port}
                </code>
                <Button size="sm" variant="outline" asChild>
                  <a href={`${url.protocol}://${url.host}/api/timing/timestamp`} target="_blank" rel="noopener noreferrer">
                    <Play className="w-3 h-3 mr-1" />
                    Test
                  </a>
                </Button>
              </div>
            ))}
          </div>
        ) : deploymentPackage ? (
          <div className="space-y-4">
            <div className="flex items-center gap-2 text-green-600">
              <CheckCircle className="w-5 h-5" />
              <span className="font-medium">Deployment Package Ready!</span>
            </div>
            
            {deploymentPackage.certificateUploaded && (
              <div className="flex items-center gap-2 text-sm text-muted-foreground">
                <Lock className="w-4 h-4" />
                Certificate uploaded to Kong Konnect
              </div>
            )}
            
            <div className="grid grid-cols-2 gap-2">
              {Object.keys(deploymentPackage.files).map((filename) => (
                <Button
                  key={filename}
                  variant="outline"
                  size="sm"
                  onClick={() => downloadFile(filename, deploymentPackage.files[filename as keyof typeof deploymentPackage.files])}
                  className="justify-start"
                  data-testid={`button-download-${filename.replace('.', '-')}`}
                >
                  <FileCode className="w-4 h-4 mr-2" />
                  {filename}
                </Button>
              ))}
            </div>

            <Button onClick={downloadAllFiles} className="w-full" data-testid="button-download-all">
              <Server className="w-4 h-4 mr-2" />
              Download All Files
            </Button>

            <div className="bg-secondary/50 rounded-md p-3">
              <p className="text-xs font-medium mb-2">Quick Deploy:</p>
              <ol className="text-xs space-y-1 text-muted-foreground list-decimal list-inside">
                {deploymentPackage.instructions.map((step, i) => (
                  <li key={i}>{step}</li>
                ))}
              </ol>
            </div>

            <Button 
              variant="ghost" 
              size="sm"
              onClick={() => setDeploymentPackage(null)}
              className="w-full"
            >
              Generate New Package
            </Button>
          </div>
        ) : cloudDeployResult ? (
          <div className="space-y-4">
            <div className="flex items-center gap-2 text-green-600">
              <CheckCircle className="w-5 h-5" />
              <span className="font-medium">Cloud Deployment Ready!</span>
            </div>
            
            <div className="text-sm text-muted-foreground">
              Files pushed to <a href={cloudDeployResult.githubRepo} target="_blank" rel="noopener noreferrer" className="text-primary hover:underline">{cloudDeployResult.githubRepo}</a>
            </div>

            <div className="grid gap-2">
              <Button className="w-full" asChild data-testid="button-deploy-render">
                <a href={cloudDeployResult.deployUrls.render} target="_blank" rel="noopener noreferrer">
                  <Globe className="w-4 h-4 mr-2" />
                  Deploy to Render (Free)
                </a>
              </Button>
              <Button variant="outline" className="w-full" asChild data-testid="button-deploy-railway">
                <a href={cloudDeployResult.deployUrls.railway} target="_blank" rel="noopener noreferrer">
                  <Zap className="w-4 h-4 mr-2" />
                  Deploy to Railway ($5 credit)
                </a>
              </Button>
            </div>

            <div className="bg-secondary/50 rounded-md p-3">
              <p className="text-xs font-medium mb-2">Next Steps:</p>
              <ol className="text-xs space-y-1 text-muted-foreground list-decimal list-inside">
                {cloudDeployResult.instructions.map((step, i) => (
                  <li key={i}>{step}</li>
                ))}
              </ol>
            </div>

            <Button 
              variant="ghost" 
              size="sm"
              onClick={() => setCloudDeployResult(null)}
              className="w-full"
            >
              Deploy Again
            </Button>
          </div>
        ) : (
          <div className="space-y-4">
            <div className="flex items-center gap-2 text-amber-600">
              <XCircle className="w-5 h-5" />
              <span className="font-medium">No Data Plane Connected</span>
            </div>
            <p className="text-sm text-muted-foreground">
              Deploy Kong Gateway to a free cloud platform with one click.
            </p>

            <div className="grid grid-cols-2 gap-2">
              <div className="space-y-1">
                <Label htmlFor="deploy-owner" className="text-xs">GitHub Owner</Label>
                <Input
                  id="deploy-owner"
                  value={githubOwner}
                  onChange={(e) => setGithubOwner(e.target.value)}
                  placeholder="username"
                  className="h-8 text-sm"
                  data-testid="input-deploy-owner"
                />
              </div>
              <div className="space-y-1">
                <Label htmlFor="deploy-repo" className="text-xs">Repository</Label>
                <Input
                  id="deploy-repo"
                  value={githubRepo}
                  onChange={(e) => setGithubRepo(e.target.value)}
                  placeholder="repo-name"
                  className="h-8 text-sm"
                  data-testid="input-deploy-repo"
                />
              </div>
            </div>
            
            <Button 
              onClick={() => selectedCP && cloudDeployMutation.mutate({ cpId: selectedCP, platform: "render" })}
              disabled={cloudDeployMutation.isPending || !githubOwner || !githubRepo}
              className="w-full"
              data-testid="button-cloud-deploy"
            >
              {cloudDeployMutation.isPending ? (
                <>
                  <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
                  Preparing Deployment...
                </>
              ) : (
                <>
                  <Globe className="w-4 h-4 mr-2" />
                  Deploy to Cloud (Free)
                </>
              )}
            </Button>

            <div className="text-xs text-muted-foreground space-y-1">
              <p className="font-medium">This will automatically:</p>
              <ul className="list-disc list-inside space-y-0.5 ml-2">
                <li>Generate TLS certificates</li>
                <li>Upload certificate to Kong Konnect</li>
                <li>Push Dockerfile to your GitHub repo</li>
                <li>Create one-click deploy link for Render/Railway</li>
              </ul>
            </div>
            
            <div className="flex gap-2">
              <Button variant="outline" className="flex-1" asChild>
                <a href="https://render.com" target="_blank" rel="noopener noreferrer">
                  <Globe className="w-4 h-4 mr-2" />
                  Render (Free)
                </a>
              </Button>
              <Button variant="outline" className="flex-1" asChild>
                <a href="https://railway.app" target="_blank" rel="noopener noreferrer">
                  <Zap className="w-4 h-4 mr-2" />
                  Railway ($5)
                </a>
              </Button>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

function HeroSection() {
  return (
    <section className="py-12 md:py-16 bg-gradient-to-b from-primary/5 to-background">
      <div className="max-w-7xl mx-auto px-5">
        <div className="max-w-4xl mx-auto text-center">
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
            className="text-lg text-muted-foreground mb-8 max-w-2xl mx-auto"
            data-testid="text-hero-description"
          >
            Manage your Kong Konnect API gateway directly from PlenumNET. 
            View control planes, services, and routes in real-time.
          </motion.p>
        </div>
      </div>
    </section>
  );
}

function DashboardSection() {
  const [selectedCP, setSelectedCP] = useState<string | null>(null);
  
  return (
    <section className="py-8 md:py-12" data-testid="section-dashboard">
      <div className="max-w-7xl mx-auto px-5 space-y-6">
        <ConnectionStatus />
        <ControlPlanesList selectedCP={selectedCP} setSelectedCP={setSelectedCP} />
        <SyncSection selectedCP={selectedCP} />
        <DeploySection selectedCP={selectedCP} />
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
      description: "Route timing-critical APIs through Kong with FINRA Rule 613 CAT compliant precision."
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
            Leverage Kong Konnect's powerful API management capabilities with PlenumNET's ternary computing infrastructure.
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
              <Card className="h-full border-primary/10 hover:border-primary/30 transition-colors">
                <CardHeader>
                  <div className="w-10 h-10 rounded-lg bg-primary/10 flex items-center justify-center mb-3">
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
        <Card className="bg-primary text-primary-foreground overflow-hidden">
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
                <Button size="lg" variant="outline" className="border-primary-foreground/30 text-primary-foreground hover:bg-primary-foreground/10" data-testid="button-cta-api-demo">
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
    <footer className="bg-background border-t border-primary/10 py-8">
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
      <Header />
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
