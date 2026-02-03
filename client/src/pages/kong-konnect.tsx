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
  Cpu
} from "lucide-react";
import { useState } from "react";
import { motion } from "framer-motion";
import { Link } from "wouter";

function Header() {
  return (
    <header className="border-b border-primary/10 bg-background/95 backdrop-blur-xl sticky top-0 z-50">
      <div className="max-w-7xl mx-auto px-5 py-4 flex items-center justify-between gap-4">
        <Link href="/" className="flex items-center gap-2.5 text-primary font-bold text-xl" data-testid="link-logo">
          <Box className="w-7 h-7" />
          <span>PlenumNET</span>
        </Link>
        
        <div className="flex items-center gap-4">
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

function HeroSection() {
  return (
    <section className="py-16 md:py-24 bg-gradient-to-b from-primary/5 to-background">
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
            className="text-4xl md:text-5xl lg:text-6xl font-bold leading-tight mb-6"
            data-testid="text-hero-title"
          >
            PlenumNET + <span className="text-primary">Kong Konnect</span>
          </motion.h1>
          
          <motion.p
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.2 }}
            className="text-lg md:text-xl text-muted-foreground mb-8 max-w-2xl mx-auto"
            data-testid="text-hero-description"
          >
            Secure, manage, and optimize your ternary APIs with Kong Konnect's enterprise-grade API gateway. 
            Post-quantum security meets industry-leading API management.
          </motion.p>
          
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.3 }}
            className="flex flex-wrap gap-4 justify-center"
          >
            <Button size="lg" asChild data-testid="button-get-started">
              <a href="https://konghq.com/products/kong-konnect" target="_blank" rel="noopener noreferrer">
                Get Started with Kong Konnect
                <ExternalLink className="w-4 h-4 ml-2" />
              </a>
            </Button>
            <Button size="lg" variant="outline" asChild data-testid="button-view-docs">
              <a href="https://developer.konghq.com/gateway/" target="_blank" rel="noopener noreferrer">
                View Documentation
              </a>
            </Button>
          </motion.div>
        </div>
      </div>
    </section>
  );
}

function FeaturesSection() {
  const features = [
    {
      icon: Shield,
      title: "Post-Quantum API Security",
      description: "Combine PlenumNET's quantum-resistant protocols with Kong's advanced authentication and authorization."
    },
    {
      icon: Clock,
      title: "Femtosecond Timing",
      description: "Route timing-critical APIs through Kong with FINRA Rule 613 CAT compliant precision timestamps."
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
      description: "Monitor API usage, performance metrics, and ternary compression ratios in real-time."
    },
    {
      icon: Lock,
      title: "Enterprise Authentication",
      description: "OAuth 2.0, JWT, mTLS, and API key authentication for secure ternary data access."
    }
  ];

  return (
    <section className="py-16 md:py-24" data-testid="section-features">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-12">
          <h2 className="text-3xl md:text-4xl font-bold mb-4">Integration Features</h2>
          <p className="text-muted-foreground text-lg max-w-2xl mx-auto">
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
                  <div className="w-12 h-12 rounded-lg bg-primary/10 flex items-center justify-center mb-4">
                    <feature.icon className="w-6 h-6 text-primary" />
                  </div>
                  <CardTitle className="text-lg">{feature.title}</CardTitle>
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

function ArchitectureSection() {
  const layers = [
    {
      name: "Client Layer",
      components: ["Web Applications", "Mobile Apps", "IoT Devices", "AI Agents"],
      color: "bg-blue-500/10 border-blue-500/30"
    },
    {
      name: "Kong Konnect Gateway",
      components: ["Authentication", "Rate Limiting", "Load Balancing", "Analytics"],
      color: "bg-primary/10 border-primary/30"
    },
    {
      name: "PlenumNET APIs",
      components: ["Ternary Operations", "Femtosecond Timing", "Phase Encryption", "PlenumDB"],
      color: "bg-green-500/10 border-green-500/30"
    },
    {
      name: "Data Layer",
      components: ["Ternary Storage", "Binary Bridge", "Quantum-Safe Vault", "Metrics Store"],
      color: "bg-purple-500/10 border-purple-500/30"
    }
  ];

  return (
    <section className="py-16 md:py-24 bg-secondary/30" data-testid="section-architecture">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-12">
          <h2 className="text-3xl md:text-4xl font-bold mb-4">Integration Architecture</h2>
          <p className="text-muted-foreground text-lg max-w-2xl mx-auto">
            Kong Konnect sits between your clients and PlenumNET services, providing enterprise-grade API management.
          </p>
        </div>
        
        <div className="max-w-4xl mx-auto space-y-4">
          {layers.map((layer, index) => (
            <motion.div
              key={layer.name}
              initial={{ opacity: 0, x: -20 }}
              whileInView={{ opacity: 1, x: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: index * 0.1 }}
            >
              <Card className={`${layer.color} border`}>
                <CardContent className="p-6">
                  <div className="flex flex-col md:flex-row md:items-center gap-4">
                    <div className="md:w-48 flex-shrink-0">
                      <h3 className="font-semibold text-foreground">{layer.name}</h3>
                    </div>
                    <div className="flex flex-wrap gap-2">
                      {layer.components.map((component) => (
                        <Badge key={component} variant="secondary" className="text-xs">
                          {component}
                        </Badge>
                      ))}
                    </div>
                  </div>
                </CardContent>
              </Card>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}

function UseCasesSection() {
  const useCases = [
    {
      icon: Globe,
      title: "Global API Distribution",
      description: "Deploy PlenumNET APIs across multiple regions with Kong's multi-cloud support for low-latency access worldwide.",
      benefits: ["Multi-region routing", "Edge caching", "Geo-based load balancing"]
    },
    {
      icon: Cpu,
      title: "AI/LLM Integration",
      description: "Route AI agent requests through Kong's AI Gateway to PlenumNET's ternary computing infrastructure.",
      benefits: ["Model Context Protocol support", "Semantic routing", "Usage metering"]
    },
    {
      icon: Server,
      title: "Event-Driven Architecture",
      description: "Combine Kong Event Gateway with PlenumNET for real-time ternary data streaming via Apache Kafka.",
      benefits: ["Kafka integration", "Event streaming", "Real-time analytics"]
    },
    {
      icon: Layers,
      title: "API Monetization",
      description: "Use Kong's metering and billing features to monetize access to PlenumNET's compression APIs.",
      benefits: ["Usage-based pricing", "Subscription management", "Invoice generation"]
    }
  ];

  return (
    <section className="py-16 md:py-24" data-testid="section-use-cases">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-12">
          <h2 className="text-3xl md:text-4xl font-bold mb-4">Use Cases</h2>
          <p className="text-muted-foreground text-lg max-w-2xl mx-auto">
            How enterprises leverage PlenumNET + Kong Konnect together.
          </p>
        </div>
        
        <div className="grid md:grid-cols-2 gap-8">
          {useCases.map((useCase, index) => (
            <motion.div
              key={useCase.title}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: index * 0.1 }}
            >
              <Card className="h-full border-primary/10">
                <CardHeader>
                  <div className="flex items-start gap-4">
                    <div className="w-12 h-12 rounded-lg bg-primary/10 flex items-center justify-center flex-shrink-0">
                      <useCase.icon className="w-6 h-6 text-primary" />
                    </div>
                    <div>
                      <CardTitle className="text-lg mb-2">{useCase.title}</CardTitle>
                      <CardDescription>{useCase.description}</CardDescription>
                    </div>
                  </div>
                </CardHeader>
                <CardContent>
                  <ul className="space-y-2">
                    {useCase.benefits.map((benefit) => (
                      <li key={benefit} className="flex items-center gap-2 text-sm text-muted-foreground">
                        <Check className="w-4 h-4 text-primary flex-shrink-0" />
                        {benefit}
                      </li>
                    ))}
                  </ul>
                </CardContent>
              </Card>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}

function QuickStartSection() {
  const [copiedStep, setCopiedStep] = useState<number | null>(null);

  const steps = [
    {
      title: "Create Kong Konnect Account",
      description: "Sign up for Kong Konnect and create a new control plane for your PlenumNET APIs.",
      code: "# Visit https://konghq.com/products/kong-konnect\n# Create a new control plane"
    },
    {
      title: "Configure Gateway Service",
      description: "Add PlenumNET API endpoints as upstream services in Kong.",
      code: `curl -X POST https://admin.konghq.com/services \\
  -H "Authorization: Bearer $KONG_TOKEN" \\
  -d "name=plenumnet-timing" \\
  -d "url=https://your-app.replit.app/api/salvi/timing"`
    },
    {
      title: "Add Routes",
      description: "Create routes to expose PlenumNET APIs through Kong Gateway.",
      code: `curl -X POST https://admin.konghq.com/services/plenumnet-timing/routes \\
  -H "Authorization: Bearer $KONG_TOKEN" \\
  -d "paths[]=/api/timing" \\
  -d "methods[]=GET" \\
  -d "methods[]=POST"`
    },
    {
      title: "Enable Plugins",
      description: "Add authentication, rate limiting, and analytics plugins.",
      code: `# Enable JWT authentication
curl -X POST https://admin.konghq.com/services/plenumnet-timing/plugins \\
  -H "Authorization: Bearer $KONG_TOKEN" \\
  -d "name=jwt"

# Enable rate limiting
curl -X POST https://admin.konghq.com/services/plenumnet-timing/plugins \\
  -d "name=rate-limiting" \\
  -d "config.minute=100"`
    }
  ];

  const copyCode = (index: number, code: string) => {
    navigator.clipboard.writeText(code);
    setCopiedStep(index);
    setTimeout(() => setCopiedStep(null), 2000);
  };

  return (
    <section className="py-16 md:py-24 bg-secondary/30" data-testid="section-quickstart">
      <div className="max-w-4xl mx-auto px-5">
        <div className="text-center mb-12">
          <h2 className="text-3xl md:text-4xl font-bold mb-4">Quick Start Guide</h2>
          <p className="text-muted-foreground text-lg">
            Get PlenumNET APIs running behind Kong Konnect in minutes.
          </p>
        </div>
        
        <div className="space-y-6">
          {steps.map((step, index) => (
            <motion.div
              key={step.title}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: index * 0.1 }}
            >
              <Card className="border-primary/10">
                <CardHeader>
                  <div className="flex items-start gap-4">
                    <div className="w-8 h-8 rounded-full bg-primary text-primary-foreground flex items-center justify-center font-bold text-sm flex-shrink-0">
                      {index + 1}
                    </div>
                    <div>
                      <CardTitle className="text-lg">{step.title}</CardTitle>
                      <CardDescription className="mt-1">{step.description}</CardDescription>
                    </div>
                  </div>
                </CardHeader>
                <CardContent>
                  <div className="relative">
                    <pre className="bg-secondary/70 border border-primary/10 rounded-lg p-4 overflow-x-auto text-sm font-mono text-muted-foreground">
                      <code>{step.code}</code>
                    </pre>
                    <Button
                      size="sm"
                      variant="ghost"
                      className="absolute top-2 right-2"
                      onClick={() => copyCode(index, step.code)}
                      data-testid={`button-copy-step-${index}`}
                    >
                      {copiedStep === index ? (
                        <Check className="w-4 h-4 text-green-500" />
                      ) : (
                        "Copy"
                      )}
                    </Button>
                  </div>
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
    <section className="py-16 md:py-24" data-testid="section-cta">
      <div className="max-w-7xl mx-auto px-5">
        <Card className="bg-primary text-primary-foreground overflow-hidden">
          <CardContent className="p-8 md:p-12 text-center">
            <h2 className="text-3xl md:text-4xl font-bold mb-4">
              Ready to Secure Your APIs?
            </h2>
            <p className="text-lg opacity-90 mb-8 max-w-xl mx-auto">
              Combine the power of Kong Konnect's API gateway with PlenumNET's post-quantum infrastructure.
            </p>
            <div className="flex flex-wrap gap-4 justify-center">
              <Button size="lg" variant="secondary" asChild data-testid="button-cta-kong">
                <a href="https://konghq.com/products/kong-konnect" target="_blank" rel="noopener noreferrer">
                  Start with Kong Konnect
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
        <FeaturesSection />
        <ArchitectureSection />
        <UseCasesSection />
        <QuickStartSection />
        <CTASection />
      </main>
      <Footer />
    </div>
  );
}
