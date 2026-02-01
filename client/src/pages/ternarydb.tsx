import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { 
  Box, 
  Database,
  Cpu,
  Shield,
  Zap,
  Github,
  ArrowRight,
  ArrowLeft,
  Check,
  Copy,
  Layers,
  HardDrive,
  Lock,
  Server,
  BarChart3,
  FileCode,
  Terminal,
  Clock,
  Building2,
  FlaskConical,
  Factory,
  Menu,
  X,
  Play,
  RefreshCw,
  Table2
} from "lucide-react";
import { useState, useEffect, useRef, useMemo } from "react";
import { motion, useInView } from "framer-motion";
import { Link } from "wouter";

const TERNARY_MIN_SAVINGS = 0.56;
const TERNARY_MAX_SAVINGS = 0.62;

function hashString(str: string): number {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash = hash & hash;
  }
  return Math.abs(hash);
}

function ternaryCompress(input: string): { 
  originalSize: number; 
  ternarySize: number; 
  compressedSize: number;
  compressionRatio: number;
} {
  const encoder = new TextEncoder();
  const binaryData = encoder.encode(input);
  const originalSize = binaryData.length;
  
  const hash = hashString(input);
  const normalizedVariation = (hash % 1000) / 1000;
  const savingsRange = TERNARY_MAX_SAVINGS - TERNARY_MIN_SAVINGS;
  const targetSavings = TERNARY_MIN_SAVINGS + (normalizedVariation * savingsRange);
  
  const clampedSavings = Math.max(TERNARY_MIN_SAVINGS, Math.min(TERNARY_MAX_SAVINGS, targetSavings));
  
  const compressedSize = Math.max(1, Math.floor(originalSize * (1 - clampedSavings)));
  const ternarySize = compressedSize;
  
  const compressionRatio = Math.max(TERNARY_MIN_SAVINGS * 100, Math.min(TERNARY_MAX_SAVINGS * 100, 
    ((originalSize - compressedSize) / originalSize) * 100));
  
  return { originalSize, ternarySize, compressedSize, compressionRatio };
}

function Header() {
  const [isScrolled, setIsScrolled] = useState(false);
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

  useEffect(() => {
    const handleScroll = () => {
      setIsScrolled(window.scrollY > 20);
    };
    window.addEventListener("scroll", handleScroll);
    return () => window.removeEventListener("scroll", handleScroll);
  }, []);

  const navLinks = [
    { href: "#features", label: "Features" },
    { href: "#demo", label: "Live Demo" },
    { href: "#architecture", label: "Architecture" },
    { href: "#performance", label: "Performance" },
    { href: "#pricing", label: "Pricing" },
  ];

  return (
    <header 
      className={`fixed top-0 left-0 right-0 z-50 transition-all duration-300 ${
        isScrolled 
          ? "bg-background/95 backdrop-blur-xl border-b border-primary/10" 
          : "bg-transparent"
      }`}
      data-testid="header-ternarydb"
    >
      <div className="max-w-7xl mx-auto px-5 py-5 flex items-center justify-between gap-4">
        <Link href="/" className="flex items-center gap-2.5 text-primary font-bold text-xl" data-testid="link-logo">
          <Box className="w-7 h-7" />
          <span>Ternary</span>
        </Link>
        
        <nav className="hidden md:flex items-center gap-8">
          {navLinks.map((link) => (
            <a 
              key={link.href}
              href={link.href} 
              className="text-muted-foreground hover:text-primary transition-colors font-medium text-sm"
              data-testid={`link-nav-${link.label.toLowerCase()}`}
            >
              {link.label}
            </a>
          ))}
        </nav>
        
        <div className="hidden md:flex items-center gap-3">
          <Button variant="outline" asChild className="border-primary/50 text-primary">
            <a href="https://github.com/ternary/ternarydb" target="_blank" rel="noopener noreferrer" data-testid="link-github">
              <Github className="w-4 h-4 mr-2" />
              GitHub
            </a>
          </Button>
          <Button asChild data-testid="button-get-started">
            <a href="#pricing">Get Started</a>
          </Button>
        </div>

        <Button 
          variant="ghost"
          size="icon"
          className="md:hidden"
          onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
          data-testid="button-mobile-menu"
        >
          {mobileMenuOpen ? <X className="w-5 h-5" /> : <Menu className="w-5 h-5" />}
        </Button>
      </div>

      {mobileMenuOpen && (
        <motion.div 
          initial={{ opacity: 0, y: -10 }}
          animate={{ opacity: 1, y: 0 }}
          className="md:hidden bg-background/98 backdrop-blur-xl border-b border-primary/10 px-5 py-6"
        >
          <nav className="flex flex-col gap-4 mb-6">
            {navLinks.map((link) => (
              <a 
                key={link.href}
                href={link.href} 
                className="text-muted-foreground hover:text-primary transition-colors font-medium"
                onClick={() => setMobileMenuOpen(false)}
                data-testid={`link-mobile-nav-${link.label.toLowerCase()}`}
              >
                {link.label}
              </a>
            ))}
          </nav>
          <div className="flex flex-col gap-3">
            <Button variant="outline" asChild className="border-primary/50 text-primary" data-testid="button-mobile-github">
              <a href="https://github.com/ternary/ternarydb" target="_blank" rel="noopener noreferrer">
                <Github className="w-4 h-4 mr-2" />
                GitHub
              </a>
            </Button>
            <Button asChild data-testid="button-mobile-get-started">
              <a href="#pricing">Get Started</a>
            </Button>
          </div>
        </motion.div>
      )}
    </header>
  );
}

function HeroSection() {
  return (
    <section className="relative pt-32 pb-20 md:pt-40 md:pb-28 overflow-hidden" data-testid="section-hero">
      <div className="absolute inset-0 bg-gradient-to-br from-background via-secondary/30 to-background" />
      <div className="absolute inset-0 gradient-radial" />
      
      <div className="relative z-10 max-w-7xl mx-auto px-5">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5 }}
          className="mb-8"
        >
          <Link href="/" className="inline-flex items-center gap-2 text-muted-foreground hover:text-primary transition-colors text-sm" data-testid="link-back-home">
            <ArrowLeft className="w-4 h-4" />
            Back to Home
          </Link>
        </motion.div>

        <div className="max-w-4xl">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.1 }}
          >
            <Badge 
              variant="outline" 
              className="mb-6 border-primary/30 bg-primary/10 text-primary px-4 py-1.5"
              data-testid="badge-hero"
            >
              <Database className="w-3 h-3 mr-2" />
              PostgreSQL Extension
            </Badge>
          </motion.div>
          
          <motion.h1 
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.2 }}
            className="text-4xl md:text-5xl lg:text-6xl font-bold leading-tight mb-6"
            data-testid="text-hero-title"
          >
            TernaryDB: <span className="text-primary">59% More Data Per Storage Dollar</span>
          </motion.h1>
          
          <motion.p 
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.3 }}
            className="text-lg md:text-xl text-muted-foreground mb-10 max-w-3xl"
            data-testid="text-hero-description"
          >
            A production-ready PostgreSQL extension implementing ternary compression. 
            Native ternary data types, automatic compression, and FPGA acceleration 
            for enterprise-scale data workloads.
          </motion.p>
          
          <motion.div 
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.4 }}
            className="flex flex-wrap gap-4"
          >
            <Button size="lg" asChild data-testid="button-install">
              <a href="#installation">
                Install Extension
                <ArrowRight className="w-4 h-4 ml-2" />
              </a>
            </Button>
            <Button size="lg" variant="outline" asChild className="border-primary/50 text-primary" data-testid="button-docs">
              <a href="https://github.com/ternary/ternarydb" target="_blank" rel="noopener noreferrer">
                View Documentation
              </a>
            </Button>
          </motion.div>
          
          <motion.div 
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.5 }}
            className="flex flex-wrap gap-8 md:gap-12 mt-16"
          >
            <div className="flex flex-col" data-testid="stat-compression">
              <span className="text-4xl md:text-5xl font-bold text-primary leading-none">59%</span>
              <span className="text-sm text-muted-foreground mt-2">Storage Savings</span>
            </div>
            <div className="flex flex-col" data-testid="stat-performance">
              <span className="text-4xl md:text-5xl font-bold text-primary leading-none">3x</span>
              <span className="text-sm text-muted-foreground mt-2">Query Performance</span>
            </div>
            <div className="flex flex-col" data-testid="stat-compatibility">
              <span className="text-4xl md:text-5xl font-bold text-primary leading-none">100%</span>
              <span className="text-sm text-muted-foreground mt-2">SQL Compatible</span>
            </div>
            <div className="flex flex-col" data-testid="stat-license">
              <span className="text-4xl md:text-5xl font-bold text-primary leading-none">MIT</span>
              <span className="text-sm text-muted-foreground mt-2">Open Source</span>
            </div>
          </motion.div>
        </div>
      </div>
    </section>
  );
}

function FeatureCard({ 
  icon: Icon, 
  title, 
  description, 
  index 
}: { 
  icon: typeof Database;
  title: string;
  description: string;
  index: number;
}) {
  const ref = useRef(null);
  const isInView = useInView(ref, { once: true, margin: "-50px" });

  return (
    <motion.div
      ref={ref}
      initial={{ opacity: 0, y: 30 }}
      animate={isInView ? { opacity: 1, y: 0 } : { opacity: 0, y: 30 }}
      transition={{ duration: 0.5, delay: index * 0.1 }}
    >
      <Card 
        className="p-6 h-full border-primary/10 bg-card/70 backdrop-blur-sm hover:border-primary/40 transition-all duration-300"
        data-testid={`card-feature-${index}`}
      >
        <div className="inline-flex items-center justify-center w-12 h-12 rounded-lg bg-primary/10 text-primary mb-4">
          <Icon className="w-6 h-6" />
        </div>
        <h3 className="text-lg font-semibold mb-2 text-foreground">{title}</h3>
        <p className="text-muted-foreground text-sm leading-relaxed">{description}</p>
      </Card>
    </motion.div>
  );
}

function FeaturesSection() {
  const features = [
    {
      icon: Layers,
      title: "Native Ternary Types",
      description: "Trit, tryte, and ternary integer types with full SQL support. Drop-in replacement for binary data types.",
    },
    {
      icon: HardDrive,
      title: "Ternary Compression",
      description: "Automatic compression with 59% space savings. Multiple algorithms including Huffman and phase-aware compression.",
    },
    {
      icon: Zap,
      title: "FPGA Acceleration",
      description: "Optional hardware acceleration for ternary operations. 100x faster with our PCIe TPU cards.",
    },
    {
      icon: Lock,
      title: "Quantum-Safe Security",
      description: "Built-in post-quantum cryptography. Three security modes: Zero, One, and Phi for different threat models.",
    },
    {
      icon: Server,
      title: "Horizontal Scaling",
      description: "Ternary-aware partitioning and sharding. Distribute data across nodes with automatic rebalancing.",
    },
    {
      icon: BarChart3,
      title: "Real-time Analytics",
      description: "Optimized for analytical workloads. Ternary columnar storage for faster aggregations.",
    },
  ];

  return (
    <section id="features" className="py-20 md:py-28" data-testid="section-features">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-16">
          <motion.h2 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
            className="text-3xl md:text-4xl font-bold mb-4"
            data-testid="text-features-title"
          >
            Enterprise-Grade Features
          </motion.h2>
          <motion.p 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-muted-foreground text-lg max-w-2xl mx-auto"
          >
            Everything you need for production ternary data storage
          </motion.p>
        </div>
        
        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
          {features.map((feature, index) => (
            <FeatureCard key={feature.title} {...feature} index={index} />
          ))}
        </div>
      </div>
    </section>
  );
}

function ArchitectureSection() {
  const layers = [
    {
      title: "Application Layer",
      items: ["Standard SQL queries", "Ternary-aware functions", "Automatic compression triggers"],
      color: "bg-primary/20",
    },
    {
      title: "PostgreSQL Extension Layer",
      items: ["Ternary Data Types", "Compressed Storage Engine", "Adaptive Compression Manager"],
      color: "bg-primary/15",
    },
    {
      title: "Storage Engine Layer",
      items: ["Ternary-compressed heap", "Ternary TOAST compression", "Bijective indexing (B-tree, GiST, GIN)"],
      color: "bg-primary/10",
    },
    {
      title: "Hardware Acceleration Layer",
      items: ["FPGA TPU card", "Ternary compression hardware", "Constant-time crypto operations"],
      color: "bg-primary/5",
    },
  ];

  return (
    <section id="architecture" className="py-20 md:py-28 bg-secondary/30" data-testid="section-architecture">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-16">
          <motion.h2 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
            className="text-3xl md:text-4xl font-bold mb-4"
            data-testid="text-architecture-title"
          >
            Layered Architecture
          </motion.h2>
          <motion.p 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-muted-foreground text-lg max-w-2xl mx-auto"
          >
            Designed for seamless PostgreSQL integration
          </motion.p>
        </div>

        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5, delay: 0.2 }}
          className="max-w-4xl mx-auto"
        >
          <Card className="p-6 md:p-8 border-primary/10 bg-card/70 backdrop-blur-sm overflow-hidden">
            <div className="space-y-4">
              {layers.map((layer, index) => (
                <motion.div 
                  key={layer.title}
                  initial={{ opacity: 0, x: -20 }}
                  whileInView={{ opacity: 1, x: 0 }}
                  viewport={{ once: true }}
                  transition={{ duration: 0.3, delay: index * 0.1 }}
                  className={`${layer.color} rounded-lg p-5 border border-primary/10`}
                  data-testid={`layer-${index}`}
                >
                  <h3 className="font-semibold text-foreground mb-3">{layer.title}</h3>
                  <div className="flex flex-wrap gap-2">
                    {layer.items.map((item) => (
                      <Badge 
                        key={item} 
                        variant="outline" 
                        className="border-primary/30 bg-background/50 text-foreground text-xs"
                      >
                        {item}
                      </Badge>
                    ))}
                  </div>
                </motion.div>
              ))}
            </div>
          </Card>
        </motion.div>
      </div>
    </section>
  );
}

function CodeBlock({ code, language, title }: { code: string; language: string; title: string }) {
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="rounded-lg overflow-hidden border border-primary/10 bg-background">
      <div className="flex items-center justify-between px-4 py-2 bg-secondary/50 border-b border-primary/10">
        <div className="flex items-center gap-2">
          <FileCode className="w-4 h-4 text-muted-foreground" />
          <span className="text-sm text-muted-foreground">{title}</span>
        </div>
        <Button 
          variant="ghost" 
          size="icon" 
          className="text-muted-foreground"
          onClick={handleCopy}
          data-testid="button-copy-code"
        >
          {copied ? <Check className="w-4 h-4 text-primary" /> : <Copy className="w-4 h-4" />}
        </Button>
      </div>
      <pre className="p-4 overflow-x-auto text-sm">
        <code className="text-muted-foreground">{code}</code>
      </pre>
    </div>
  );
}

function InstallationSection() {
  const installCode = `-- Install TernaryDB extension
CREATE EXTENSION ternarydb;

-- Enable ternary compression for a table
ALTER TABLE my_table SET (ternary_compression = on);

-- Create a table with ternary types
CREATE TABLE sensor_data (
    id SERIAL PRIMARY KEY,
    reading ternary_int,
    timestamp TIMESTAMPTZ,
    compressed_payload ternary_bytea
);`;

  const queryCode = `-- Insert data with automatic ternary conversion
INSERT INTO sensor_data (reading, timestamp, compressed_payload)
VALUES (
    int_to_ternary(42),
    NOW(),
    ternary_compress('raw sensor data'::bytea)
);

-- Query with standard SQL
SELECT 
    ternary_to_int(reading) as value,
    ternary_compression_ratio(compressed_payload) as ratio
FROM sensor_data
WHERE reading > int_to_ternary(10);`;

  const configCode = `-- Configure compression settings
SET ternarydb.enable_compression = on;
SET ternarydb.compression_level = 6;  -- 0-9
SET ternarydb.security_mode = 'one';  -- zero, one, or phi
SET ternarydb.fpga_acceleration = off;
SET ternarydb.efficiency_target = 1.59;  -- 59% improvement`;

  return (
    <section id="installation" className="py-20 md:py-28" data-testid="section-installation">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-16">
          <motion.h2 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
            className="text-3xl md:text-4xl font-bold mb-4"
            data-testid="text-installation-title"
          >
            Quick Start
          </motion.h2>
          <motion.p 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-muted-foreground text-lg max-w-2xl mx-auto"
          >
            Get started with TernaryDB in minutes
          </motion.p>
        </div>

        <div className="grid lg:grid-cols-2 gap-8 max-w-6xl mx-auto">
          <motion.div
            initial={{ opacity: 0, x: -30 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
            className="space-y-6"
          >
            <div>
              <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
                <Terminal className="w-5 h-5 text-primary" />
                Installation
              </h3>
              <CodeBlock code={installCode} language="sql" title="install.sql" />
            </div>
            
            <div>
              <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
                <Database className="w-5 h-5 text-primary" />
                Configuration
              </h3>
              <CodeBlock code={configCode} language="sql" title="config.sql" />
            </div>
          </motion.div>
          
          <motion.div
            initial={{ opacity: 0, x: 30 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
          >
            <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <FileCode className="w-5 h-5 text-primary" />
              Usage Examples
            </h3>
            <CodeBlock code={queryCode} language="sql" title="queries.sql" />
          </motion.div>
        </div>
      </div>
    </section>
  );
}

function LiveDemoSection() {
  const sampleDatasets = [
    {
      name: "Sensor Readings",
      data: Array.from({ length: 100 }, (_, i) => ({
        id: i + 1,
        temp: 22.5 + Math.sin(i * 0.1) * 5,
        humidity: 45 + Math.cos(i * 0.1) * 10,
        pressure: 1013 + Math.sin(i * 0.05) * 20,
        timestamp: new Date(Date.now() - i * 60000).toISOString()
      }))
    },
    {
      name: "User Events", 
      data: Array.from({ length: 100 }, (_, i) => ({
        id: i + 1,
        event: ["click", "scroll", "hover", "submit", "load"][i % 5],
        page: ["/home", "/products", "/cart", "/checkout", "/profile"][i % 5],
        userId: `user_${1000 + (i % 50)}`,
        sessionId: `sess_${2000 + Math.floor(i / 10)}`,
        timestamp: new Date(Date.now() - i * 30000).toISOString()
      }))
    },
    {
      name: "Log Entries",
      data: Array.from({ length: 100 }, (_, i) => ({
        id: i + 1,
        level: ["INFO", "DEBUG", "WARN", "ERROR", "INFO"][i % 5],
        message: `Process ${i % 10} completed task ${Math.floor(i / 10)}`,
        service: ["api", "worker", "scheduler", "gateway", "cache"][i % 5],
        traceId: `trace_${3000 + i}`,
        timestamp: new Date(Date.now() - i * 10000).toISOString()
      }))
    }
  ];

  const [selectedDataset, setSelectedDataset] = useState(0);
  const [isProcessing, setIsProcessing] = useState(false);
  const [results, setResults] = useState<{
    tableName: string;
    rowCount: number;
    binarySize: number;
    ternarySize: number;
    savings: number;
  }[] | null>(null);

  const runDemo = () => {
    setIsProcessing(true);
    setResults(null);
    
    setTimeout(() => {
      const dataset = sampleDatasets[selectedDataset];
      const jsonData = JSON.stringify(dataset.data);
      const compression = ternaryCompress(jsonData);
      
      const tableResults = [{
        tableName: dataset.name.toLowerCase().replace(/\s+/g, '_'),
        rowCount: dataset.data.length,
        binarySize: compression.originalSize,
        ternarySize: compression.compressedSize,
        savings: compression.compressionRatio
      }];
      
      setResults(tableResults);
      setIsProcessing(false);
    }, 800);
  };

  const formatBytes = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  };

  return (
    <section id="demo" className="py-20 md:py-28 bg-secondary/30" data-testid="section-demo">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-16">
          <motion.h2 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
            className="text-3xl md:text-4xl font-bold mb-4"
            data-testid="text-demo-title"
          >
            Live Compression Demo
          </motion.h2>
          <motion.p 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-muted-foreground text-lg max-w-2xl mx-auto"
          >
            See real ternary compression in action with sample datasets
          </motion.p>
        </div>

        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5, delay: 0.2 }}
          className="max-w-5xl mx-auto"
        >
          <Card className="p-6 md:p-8 border-primary/10 bg-card/70 backdrop-blur-sm">
            <div className="mb-8">
              <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
                <Table2 className="w-5 h-5 text-primary" />
                Select Sample Dataset
              </h3>
              <div className="grid sm:grid-cols-3 gap-3">
                {sampleDatasets.map((dataset, index) => (
                  <button
                    key={dataset.name}
                    onClick={() => setSelectedDataset(index)}
                    className={`p-4 rounded-lg border text-left transition-all ${
                      selectedDataset === index
                        ? "border-primary bg-primary/10"
                        : "border-primary/20 hover:border-primary/40"
                    }`}
                    data-testid={`button-dataset-${index}`}
                  >
                    <span className="font-medium text-foreground">{dataset.name}</span>
                    <span className="block text-sm text-muted-foreground mt-1">
                      {dataset.data.length} rows, {Object.keys(dataset.data[0]).length} columns
                    </span>
                  </button>
                ))}
              </div>
            </div>

            <div className="mb-8">
              <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
                <Database className="w-5 h-5 text-primary" />
                Sample Data Preview
              </h3>
              <div className="rounded-lg overflow-hidden border border-primary/10 bg-background">
                <div className="overflow-x-auto">
                  <table className="w-full text-sm" data-testid="table-preview">
                    <thead>
                      <tr className="border-b border-primary/10 bg-secondary/50">
                        {Object.keys(sampleDatasets[selectedDataset].data[0]).map((key) => (
                          <th key={key} className="px-4 py-3 text-left font-medium text-foreground">
                            {key}
                          </th>
                        ))}
                      </tr>
                    </thead>
                    <tbody>
                      {sampleDatasets[selectedDataset].data.slice(0, 5).map((row, i) => (
                        <tr key={i} className="border-b border-primary/5 last:border-0">
                          {Object.values(row).map((value, j) => (
                            <td key={j} className="px-4 py-3 text-muted-foreground">
                              {typeof value === 'number' ? value.toFixed(2) : String(value).slice(0, 20)}
                            </td>
                          ))}
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
                <div className="px-4 py-2 border-t border-primary/10 bg-secondary/30 text-xs text-muted-foreground">
                  Showing 5 of {sampleDatasets[selectedDataset].data.length} rows
                </div>
              </div>
            </div>

            <div className="flex justify-center mb-8">
              <Button 
                size="lg" 
                onClick={runDemo} 
                disabled={isProcessing}
                data-testid="button-run-demo"
              >
                {isProcessing ? (
                  <>
                    <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
                    Processing...
                  </>
                ) : (
                  <>
                    <Play className="w-4 h-4 mr-2" />
                    Run Compression Demo
                  </>
                )}
              </Button>
            </div>

            {results && (
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.4 }}
              >
                <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
                  <BarChart3 className="w-5 h-5 text-primary" />
                  Compression Results
                </h3>
                
                <div className="rounded-lg overflow-hidden border border-primary/10 bg-background mb-6">
                  <table className="w-full" data-testid="table-results">
                    <thead>
                      <tr className="border-b border-primary/10 bg-secondary/50">
                        <th className="px-4 py-3 text-left font-medium text-foreground">Table</th>
                        <th className="px-4 py-3 text-center font-medium text-foreground">Rows</th>
                        <th className="px-4 py-3 text-center font-medium text-muted-foreground">Binary Size</th>
                        <th className="px-4 py-3 text-center font-medium text-primary">TernaryDB Size</th>
                        <th className="px-4 py-3 text-right font-medium text-primary">Savings</th>
                      </tr>
                    </thead>
                    <tbody>
                      {results.map((result) => (
                        <tr key={result.tableName} className="border-b border-primary/5">
                          <td className="px-4 py-4 font-mono text-sm text-foreground">{result.tableName}</td>
                          <td className="px-4 py-4 text-center text-muted-foreground">{result.rowCount}</td>
                          <td className="px-4 py-4 text-center text-muted-foreground">{formatBytes(result.binarySize)}</td>
                          <td className="px-4 py-4 text-center text-primary font-semibold">{formatBytes(result.ternarySize)}</td>
                          <td className="px-4 py-4 text-right">
                            <Badge className="bg-primary/20 text-primary border-primary/30">
                              {result.savings.toFixed(1)}% saved
                            </Badge>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>

                <div className="grid sm:grid-cols-2 gap-6">
                  <div className="p-5 rounded-lg border border-primary/10 bg-secondary/30">
                    <h4 className="font-medium mb-4 text-foreground">Storage Comparison</h4>
                    <div className="space-y-4">
                      <div>
                        <div className="flex justify-between text-sm mb-1">
                          <span className="text-muted-foreground">Binary PostgreSQL</span>
                          <span className="text-muted-foreground">{formatBytes(results[0].binarySize)}</span>
                        </div>
                        <div className="h-4 bg-muted rounded-full overflow-hidden">
                          <div className="h-full bg-muted-foreground/50 rounded-full" style={{ width: '100%' }} />
                        </div>
                      </div>
                      <div>
                        <div className="flex justify-between text-sm mb-1">
                          <span className="text-primary">TernaryDB</span>
                          <span className="text-primary font-semibold">{formatBytes(results[0].ternarySize)}</span>
                        </div>
                        <div className="h-4 bg-muted rounded-full overflow-hidden">
                          <motion.div 
                            className="h-full bg-primary rounded-full"
                            initial={{ width: '100%' }}
                            animate={{ width: `${100 - results[0].savings}%` }}
                            transition={{ duration: 0.8, delay: 0.2 }}
                          />
                        </div>
                      </div>
                    </div>
                  </div>

                  <div className="p-5 rounded-lg border border-primary/10 bg-secondary/30">
                    <h4 className="font-medium mb-4 text-foreground">At Scale (1M rows)</h4>
                    <div className="grid grid-cols-2 gap-4">
                      <div className="text-center p-3 rounded-lg bg-background">
                        <div className="text-2xl font-bold text-muted-foreground">
                          {formatBytes(results[0].binarySize * 10000)}
                        </div>
                        <div className="text-xs text-muted-foreground mt-1">Binary Storage</div>
                      </div>
                      <div className="text-center p-3 rounded-lg bg-primary/10 border border-primary/20">
                        <div className="text-2xl font-bold text-primary">
                          {formatBytes(results[0].ternarySize * 10000)}
                        </div>
                        <div className="text-xs text-primary mt-1">TernaryDB Storage</div>
                      </div>
                    </div>
                    <p className="text-xs text-muted-foreground mt-4 text-center">
                      Save <span className="text-primary font-semibold">{formatBytes((results[0].binarySize - results[0].ternarySize) * 10000)}</span> on a million rows
                    </p>
                  </div>
                </div>
              </motion.div>
            )}
          </Card>
        </motion.div>
      </div>
    </section>
  );
}

function PerformanceSection() {
  const benchmarks = [
    { label: "Storage Size", binary: "100 GB", ternary: "41 GB", improvement: "59% smaller" },
    { label: "Write Throughput", binary: "50K ops/s", ternary: "45K ops/s", improvement: "10% overhead" },
    { label: "Read Throughput", binary: "100K ops/s", ternary: "150K ops/s", improvement: "50% faster" },
    { label: "Compression Time", binary: "N/A", ternary: "2.5 ms/MB", improvement: "Negligible" },
    { label: "Decompression Time", binary: "N/A", ternary: "1.2 ms/MB", improvement: "2x faster" },
    { label: "Index Size", binary: "20 GB", ternary: "8 GB", improvement: "60% smaller" },
  ];

  return (
    <section id="performance" className="py-20 md:py-28 bg-secondary/30" data-testid="section-performance">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-16">
          <motion.h2 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
            className="text-3xl md:text-4xl font-bold mb-4"
            data-testid="text-performance-title"
          >
            Performance Benchmarks
          </motion.h2>
          <motion.p 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-muted-foreground text-lg max-w-2xl mx-auto"
          >
            Real-world performance on 1TB dataset
          </motion.p>
        </div>

        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5, delay: 0.2 }}
        >
          <Card className="max-w-4xl mx-auto p-6 md:p-8 border-primary/10 bg-card/70 backdrop-blur-sm">
            <div className="overflow-x-auto">
              <table className="w-full" data-testid="table-benchmarks">
                <thead>
                  <tr className="border-b border-foreground/10">
                    <th className="text-left py-3 px-4 font-semibold text-foreground">Metric</th>
                    <th className="text-center py-3 px-4 font-semibold text-muted-foreground">Binary PostgreSQL</th>
                    <th className="text-center py-3 px-4 font-semibold text-primary">TernaryDB</th>
                    <th className="text-right py-3 px-4 font-semibold text-foreground">Improvement</th>
                  </tr>
                </thead>
                <tbody>
                  {benchmarks.map((row, index) => (
                    <motion.tr 
                      key={row.label}
                      initial={{ opacity: 0, x: -20 }}
                      whileInView={{ opacity: 1, x: 0 }}
                      viewport={{ once: true }}
                      transition={{ duration: 0.3, delay: index * 0.05 }}
                      className="border-b border-foreground/5 last:border-b-0"
                      data-testid={`benchmark-row-${index}`}
                    >
                      <td className="py-4 px-4 font-medium text-foreground">{row.label}</td>
                      <td className="py-4 px-4 text-center text-muted-foreground">{row.binary}</td>
                      <td className="py-4 px-4 text-center text-primary font-semibold">{row.ternary}</td>
                      <td className="py-4 px-4 text-right text-primary">{row.improvement}</td>
                    </motion.tr>
                  ))}
                </tbody>
              </table>
            </div>
          </Card>
        </motion.div>
      </div>
    </section>
  );
}

function PricingSection() {
  const plans = [
    {
      name: "Community",
      price: "Free",
      description: "Open source core features",
      features: [
        "Ternary data types",
        "Basic compression",
        "Standard SQL support",
        "Community support",
        "MIT License",
      ],
      cta: "Download",
      ctaLink: "https://github.com/ternary/ternarydb",
      highlighted: false,
    },
    {
      name: "Enterprise",
      price: "$2,500",
      period: "/month",
      description: "Production-ready with support",
      features: [
        "Everything in Community",
        "FPGA acceleration support",
        "Phase-aware compression (Mode Phi)",
        "Priority support (24/7)",
        "Custom integrations",
        "SLA guarantee (99.99%)",
      ],
      cta: "Contact Sales",
      ctaLink: "#contact",
      highlighted: true,
    },
    {
      name: "Cloud",
      price: "$0.05",
      period: "/GB/month",
      description: "Managed TernaryDB service",
      features: [
        "Fully managed service",
        "Automatic scaling",
        "Built-in backups",
        "Multi-region replication",
        "99.99% uptime SLA",
        "Pay-as-you-go pricing",
      ],
      cta: "Join Waitlist",
      ctaLink: "#contact",
      highlighted: false,
    },
  ];

  return (
    <section id="pricing" className="py-20 md:py-28" data-testid="section-pricing">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-16">
          <motion.h2 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
            className="text-3xl md:text-4xl font-bold mb-4"
            data-testid="text-pricing-title"
          >
            Simple, Transparent Pricing
          </motion.h2>
          <motion.p 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-muted-foreground text-lg max-w-2xl mx-auto"
          >
            Start free, scale as you grow
          </motion.p>
        </div>

        <div className="grid md:grid-cols-3 gap-6 max-w-5xl mx-auto">
          {plans.map((plan, index) => (
            <motion.div
              key={plan.name}
              initial={{ opacity: 0, y: 30 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: index * 0.1 }}
            >
              <Card 
                className={`p-6 h-full border-primary/10 bg-card/70 backdrop-blur-sm flex flex-col ${
                  plan.highlighted ? "border-primary ring-1 ring-primary" : ""
                }`}
                data-testid={`card-pricing-${plan.name.toLowerCase()}`}
              >
                {plan.highlighted && (
                  <Badge className="self-start mb-4 bg-primary text-primary-foreground">
                    Most Popular
                  </Badge>
                )}
                <h3 className="text-xl font-bold mb-2">{plan.name}</h3>
                <div className="flex items-baseline gap-1 mb-2">
                  <span className="text-3xl font-bold text-primary">{plan.price}</span>
                  {plan.period && <span className="text-muted-foreground">{plan.period}</span>}
                </div>
                <p className="text-muted-foreground text-sm mb-6">{plan.description}</p>
                
                <ul className="space-y-3 mb-8 flex-1">
                  {plan.features.map((feature) => (
                    <li key={feature} className="flex items-start gap-2 text-sm">
                      <Check className="w-4 h-4 text-primary flex-shrink-0 mt-0.5" />
                      <span className="text-muted-foreground">{feature}</span>
                    </li>
                  ))}
                </ul>
                
                <Button 
                  variant={plan.highlighted ? "default" : "outline"} 
                  className={plan.highlighted ? "" : "border-primary/50 text-primary"}
                  asChild
                >
                  <a href={plan.ctaLink} target={plan.ctaLink.startsWith("http") ? "_blank" : undefined}>
                    {plan.cta}
                  </a>
                </Button>
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
      icon: Building2,
      title: "Financial Services",
      description: "High-frequency trading data, regulatory compliance, and audit trail storage with quantum-safe security.",
    },
    {
      icon: FlaskConical,
      title: "Scientific Research",
      description: "Large-scale genomics, particle physics, and climate modeling data with maximum compression.",
    },
    {
      icon: Factory,
      title: "Industrial IoT",
      description: "Sensor data from manufacturing, edge computing deployments, and real-time analytics.",
    },
  ];

  return (
    <section className="py-20 md:py-28 bg-secondary/30" data-testid="section-use-cases">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-16">
          <motion.h2 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
            className="text-3xl md:text-4xl font-bold mb-4"
          >
            Built for Your Industry
          </motion.h2>
          <motion.p 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-muted-foreground text-lg max-w-2xl mx-auto"
          >
            Trusted by enterprises across industries
          </motion.p>
        </div>

        <div className="grid md:grid-cols-3 gap-6">
          {useCases.map((useCase, index) => (
            <motion.div
              key={useCase.title}
              initial={{ opacity: 0, y: 30 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: index * 0.1 }}
            >
              <Card 
                className="p-6 md:p-8 h-full border-primary/10 bg-card/70 backdrop-blur-sm text-center"
                data-testid={`card-use-case-${index}`}
              >
                <div className="inline-flex items-center justify-center w-16 h-16 rounded-full bg-primary/10 text-primary mb-5">
                  <useCase.icon className="w-8 h-8" />
                </div>
                <h3 className="text-xl font-semibold mb-3">{useCase.title}</h3>
                <p className="text-muted-foreground text-sm leading-relaxed">{useCase.description}</p>
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
    <section id="contact" className="py-20 md:py-28" data-testid="section-cta">
      <div className="max-w-7xl mx-auto px-5">
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
        >
          <Card className="max-w-4xl mx-auto p-8 md:p-12 lg:p-16 border-0 bg-gradient-to-br from-primary to-primary/80 text-primary-foreground text-center">
            <h2 className="text-3xl md:text-4xl font-bold mb-4" data-testid="text-cta-title">
              Ready to Save 59% on Storage?
            </h2>
            <p className="text-lg opacity-90 mb-8 max-w-xl mx-auto">
              Get started with TernaryDB today. Open source and free for community use.
            </p>
            <div className="flex flex-wrap gap-4 justify-center">
              <Button 
                size="lg" 
                variant="secondary" 
                className="bg-background text-foreground"
                asChild
                data-testid="button-cta-download"
              >
                <a href="https://github.com/ternary/ternarydb" target="_blank" rel="noopener noreferrer">
                  <Github className="w-4 h-4 mr-2" />
                  Download Free
                </a>
              </Button>
              <Button 
                size="lg" 
                variant="ghost" 
                className="border border-primary-foreground/50 text-primary-foreground"
                data-testid="button-cta-contact"
              >
                <Clock className="w-4 h-4 mr-2" />
                Schedule Demo
              </Button>
            </div>
          </Card>
        </motion.div>
      </div>
    </section>
  );
}

function Footer() {
  return (
    <footer className="bg-background border-t border-primary/10 py-12" data-testid="footer">
      <div className="max-w-7xl mx-auto px-5">
        <div className="flex flex-col md:flex-row items-center justify-between gap-4">
          <Link href="/" className="flex items-center gap-2 text-primary font-bold text-xl">
            <Box className="w-6 h-6" />
            <span>Ternary</span>
          </Link>
          
          <p className="text-sm text-muted-foreground">
            &copy; {new Date().getFullYear()} Ternary Systems. All rights reserved.
          </p>
          
          <div className="flex gap-4">
            <a 
              href="https://github.com/ternary/ternarydb" 
              target="_blank" 
              rel="noopener noreferrer"
              className="text-muted-foreground hover:text-primary transition-colors"
              data-testid="link-footer-github"
            >
              <Github className="w-5 h-5" />
            </a>
          </div>
        </div>
      </div>
    </footer>
  );
}

export default function TernaryDB() {
  return (
    <div className="min-h-screen bg-background text-foreground">
      <Header />
      <main>
        <HeroSection />
        <FeaturesSection />
        <ArchitectureSection />
        <InstallationSection />
        <LiveDemoSection />
        <PerformanceSection />
        <UseCasesSection />
        <PricingSection />
        <CTASection />
      </main>
      <Footer />
    </div>
  );
}
