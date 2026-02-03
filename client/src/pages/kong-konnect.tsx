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
  Plug
} from "lucide-react";
import { useState } from "react";
import { motion } from "framer-motion";
import { Link } from "wouter";
import { useQuery } from "@tanstack/react-query";

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

function ControlPlanesList() {
  const [selectedCP, setSelectedCP] = useState<string | null>(null);
  
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
  return (
    <section className="py-8 md:py-12" data-testid="section-dashboard">
      <div className="max-w-7xl mx-auto px-5 space-y-6">
        <ConnectionStatus />
        <ControlPlanesList />
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
