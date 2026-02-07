import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { 
  Box, 
  Code, 
  Clock, 
  Cpu, 
  Check, 
  Github, 
  ArrowRight,
  Shield,
  Zap,
  Database,
  Network,
  Building2,
  FlaskConical,
  Factory,
  Mail,
  Linkedin,
  Twitter,
  Terminal,
  Lock,
  Binary,
  Activity,
  Globe,
  Server
} from "lucide-react";
import { useState, useRef } from "react";
import { motion, useInView } from "framer-motion";
import { Link } from "wouter";
import { useQuery, useMutation } from "@tanstack/react-query";
import { apiRequest } from "@/lib/queryClient";
import { useToast } from "@/hooks/use-toast";



function AnimatedStat({ value, label, suffix, delay }: { value: string; label: string; suffix?: string; delay: number }) {
  const ref = useRef(null);
  const isInView = useInView(ref, { once: true, margin: "-100px" });
  
  return (
    <motion.div 
      ref={ref}
      initial={{ opacity: 0, y: 20 }}
      animate={isInView ? { opacity: 1, y: 0 } : { opacity: 0, y: 20 }}
      transition={{ duration: 0.5, delay }}
      className="flex flex-col"
      data-testid={`stat-${label.toLowerCase().replace(/\s+/g, '-')}`}
    >
      <span className="text-4xl md:text-5xl font-bold text-primary leading-none">
        {value}{suffix && <span className="text-2xl md:text-3xl">{suffix}</span>}
      </span>
      <span className="text-sm text-muted-foreground mt-2">{label}</span>
    </motion.div>
  );
}

function HeroSection() {
  const [email, setEmail] = useState("");
  const { toast } = useToast();

  const signupMutation = useMutation({
    mutationFn: async (data: { email: string }) => {
      const res = await apiRequest("POST", "/api/developer-signup", data);
      return res.json();
    },
    onSuccess: (data) => {
      toast({ title: "You're in!", description: data.message });
      setEmail("");
    },
    onError: () => {
      toast({ title: "Something went wrong", description: "Please try again.", variant: "destructive" });
    },
  });

  return (
    <section className="relative pt-16 pb-20 md:pt-24 md:pb-32 overflow-hidden" data-testid="section-hero">
      <div className="absolute inset-0 bg-gradient-to-br from-background via-secondary/30 to-background" />
      <div className="absolute inset-0 gradient-radial" />
      
      <div className="relative z-10 max-w-7xl mx-auto px-5">
        <div className="max-w-4xl">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5 }}
            className="flex flex-wrap gap-2 mb-6"
          >
            <Badge 
              variant="outline" 
              className="border-green-500/30 bg-green-500/10 text-green-700 px-4 py-1.5"
              data-testid="badge-status"
            >
              <Check className="w-3 h-3 mr-1" />
              Production Ready
            </Badge>
            <Badge 
              variant="outline" 
              className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5"
            >
              1,011 Tests Passing
            </Badge>
            <Badge 
              variant="outline" 
              className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5"
            >
              Post-Quantum Secure
            </Badge>
          </motion.div>
          
          <motion.h1 
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-4xl md:text-5xl lg:text-6xl font-bold leading-tight mb-6"
            data-testid="text-hero-title"
          >
            The World's First <span className="text-primary">Ternary Computing</span> Platform
          </motion.h1>
          
          <motion.p 
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.2 }}
            className="text-lg md:text-xl text-muted-foreground mb-8 max-w-3xl"
            data-testid="text-hero-description"
          >
            59% more information per digit. Femtosecond-precision timing. Post-quantum encryption. 
            A complete Rust kernel with virtual machine, network stack, and binary compatibility layer -- all shipping today.
          </motion.p>

          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.3 }}
            className="mb-10"
          >
            <form 
              onSubmit={(e) => {
                e.preventDefault();
                if (email) signupMutation.mutate({ email });
              }}
              className="flex flex-col sm:flex-row gap-3 max-w-lg"
              data-testid="form-hero-signup"
            >
              <Input
                type="email"
                placeholder="Enter your email for early access"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                className="flex-1"
                required
                data-testid="input-hero-email"
              />
              <Button 
                type="submit" 
                size="default"
                disabled={signupMutation.isPending}
                data-testid="button-hero-signup"
              >
                {signupMutation.isPending ? "Joining..." : "Get Early Access"}
                <ArrowRight className="w-4 h-4 ml-2" />
              </Button>
            </form>
            <p className="text-xs text-muted-foreground mt-2">Join developers building the next generation of computing infrastructure.</p>
          </motion.div>
          
          <motion.div 
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.35 }}
            className="flex flex-wrap gap-4 mb-12"
          >
            <Button size="lg" variant="outline" asChild className="border-primary/50 text-primary" data-testid="button-view-github">
              <a href="https://github.com/SigmaWolf-8/Ternary" target="_blank" rel="noopener noreferrer">
                <Github className="w-4 h-4 mr-2" />
                View Source
              </a>
            </Button>
            <Button size="lg" variant="outline" asChild className="border-primary/50 text-primary" data-testid="button-view-demo">
              <a href="/ternarydb">
                <Zap className="w-4 h-4 mr-2" />
                Live Demo
              </a>
            </Button>
            <Button size="lg" variant="outline" asChild className="border-primary/50 text-primary" data-testid="button-view-docs">
              <a href="/whitepaper">
                <Shield className="w-4 h-4 mr-2" />
                Whitepaper
              </a>
            </Button>
          </motion.div>
          
          <div className="flex flex-wrap gap-8 md:gap-12">
            <AnimatedStat value="+59" suffix="%" label="Information Density" delay={0.4} />
            <AnimatedStat value="1,011" label="Tests Passing" delay={0.5} />
            <AnimatedStat value="80/80" label="Roadmap Complete" delay={0.6} />
            <AnimatedStat value="35" label="VM Opcodes" delay={0.7} />
          </div>
        </div>
      </div>
    </section>
  );
}

function PlatformSection() {
  const capabilities = [
    {
      icon: Cpu,
      title: "Ternary Kernel",
      description: "Complete bare-metal kernel in Rust with GF(3) arithmetic, memory management, process scheduling, and IPC.",
      stats: "26 subsystems",
    },
    {
      icon: Terminal,
      title: "35-Opcode Virtual Machine",
      description: "Register-based VM with ternary-native instructions, garbage collection, and full execution engine.",
      stats: "27 registers",
    },
    {
      icon: Clock,
      title: "Femtosecond Timing (HPTP)",
      description: "High-Precision Timing Protocol with optical clock sync and regulatory certification for FINRA 613 & MiFID II.",
      stats: "Sub-microsecond",
    },
    {
      icon: Binary,
      title: "Binary Compatibility",
      description: "Seamless Binary-Ternary Gateway enabling ternary computing on existing x86_64 hardware today.",
      stats: "Zero overhead",
    },
    {
      icon: Globe,
      title: "Torsion Network Stack",
      description: "N-dimensional torus topology with Ternary Transport Protocol, T3P application layer, and Ternary DNS.",
      stats: "Full stack",
    },
    {
      icon: Lock,
      title: "Post-Quantum Security",
      description: "Phase encryption, Lamport signatures, ternary HMAC, and sponge-based hashing resistant to quantum attacks.",
      stats: "Quantum-safe",
    },
  ];

  return (
    <section id="platform" className="py-20 md:py-28" data-testid="section-platform">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-16">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
          >
            <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
              Complete Platform
            </Badge>
          </motion.div>
          <motion.h2 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-3xl md:text-4xl font-bold mb-4"
            data-testid="text-platform-title"
          >
            Everything You Need to Build on Ternary
          </motion.h2>
          <motion.p 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.15 }}
            className="text-muted-foreground text-lg max-w-2xl mx-auto"
          >
            From kernel primitives to application-layer protocols -- a fully integrated ternary computing stack, production-tested with 1,011 passing tests.
          </motion.p>
        </div>

        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
          {capabilities.map((cap, index) => {
            const ref = useRef(null);
            const isInView = useInView(ref, { once: true, margin: "-50px" });
            return (
              <motion.div
                key={cap.title}
                ref={ref}
                initial={{ opacity: 0, y: 30 }}
                animate={isInView ? { opacity: 1, y: 0 } : { opacity: 0, y: 30 }}
                transition={{ duration: 0.5, delay: index * 0.08 }}
              >
                <Card 
                  className="p-6 md:p-8 h-full border-primary/10 bg-card/70 backdrop-blur-sm"
                  data-testid={`card-capability-${index}`}
                >
                  <div className="flex items-start justify-between mb-4 gap-3">
                    <div className="text-primary">
                      <cap.icon className="w-8 h-8" />
                    </div>
                    <Badge variant="outline" className="border-primary/30 bg-primary/5 text-primary text-xs flex-shrink-0">
                      {cap.stats}
                    </Badge>
                  </div>
                  <h3 className="text-lg font-semibold mb-2 text-foreground">{cap.title}</h3>
                  <p className="text-muted-foreground text-sm leading-relaxed">{cap.description}</p>
                </Card>
              </motion.div>
            );
          })}
        </div>
      </div>
    </section>
  );
}

function ArchitectureSection() {
  const layers = [
    {
      label: "Applications",
      items: ["PlenumDB", "Payment Listener", "SFK Core API", "Certification Service"],
      color: "bg-primary/10 border-primary/30",
    },
    {
      label: "Protocols",
      items: ["HPTP Timing", "T3P Application", "TTP Transport", "TDNS Resolution"],
      color: "bg-primary/15 border-primary/40",
    },
    {
      label: "Virtual Machine",
      items: ["35-Opcode ISA", "27 Ternary Registers", "TAGC Garbage Collector", "GF(3) Arithmetic"],
      color: "bg-primary/20 border-primary/50",
    },
    {
      label: "Kernel Services",
      items: ["Process Scheduler", "Memory Manager", "Filesystem", "I/O Subsystem"],
      color: "bg-primary/25 border-primary/60",
    },
    {
      label: "Hardware Abstraction",
      items: ["x86_64 / AArch64 / RISC-V", "Binary-Ternary Gateway", "TPU / FPGA Drivers", "Optical Clock"],
      color: "bg-primary/30 border-primary/70",
    },
  ];

  return (
    <section id="architecture" className="py-20 md:py-28 bg-secondary/30" data-testid="section-architecture">
      <div className="max-w-7xl mx-auto px-5">
        <div className="grid lg:grid-cols-2 gap-12 items-center">
          <div>
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5 }}
            >
              <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
                Full-Stack Architecture
              </Badge>
            </motion.div>
            <motion.h2
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: 0.1 }}
              className="text-3xl md:text-4xl font-bold mb-6"
              data-testid="text-architecture-title"
            >
              Built From the Ground Up
            </motion.h2>
            <motion.p
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: 0.15 }}
              className="text-muted-foreground text-lg mb-8"
            >
              Five integrated layers spanning hardware abstraction to application services. 
              Every layer is production-tested, binary-compatible, and designed for the post-quantum era.
            </motion.p>
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: 0.2 }}
              className="space-y-3"
            >
              <div className="flex items-center gap-3 text-sm">
                <Check className="w-5 h-5 text-primary flex-shrink-0" />
                <span>Three bijective ternary representations: A {"{-1,0,+1}"}, B {"{0,1,2}"}, C {"{1,2,3}"}</span>
              </div>
              <div className="flex items-center gap-3 text-sm">
                <Check className="w-5 h-5 text-primary flex-shrink-0" />
                <span>Multi-architecture support: x86_64, AArch64, RISC-V</span>
              </div>
              <div className="flex items-center gap-3 text-sm">
                <Check className="w-5 h-5 text-primary flex-shrink-0" />
                <span>Runs on existing binary hardware via Binary-Ternary Gateway</span>
              </div>
              <div className="flex items-center gap-3 text-sm">
                <Check className="w-5 h-5 text-primary flex-shrink-0" />
                <span>FINRA 613 & MiFID II regulatory compliance built in</span>
              </div>
            </motion.div>
          </div>

          <motion.div
            initial={{ opacity: 0, x: 30 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6, delay: 0.2 }}
            className="space-y-3"
          >
            {layers.map((layer, index) => (
              <motion.div
                key={layer.label}
                initial={{ opacity: 0, x: 20 }}
                whileInView={{ opacity: 1, x: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.4, delay: 0.1 + index * 0.08 }}
              >
                <Card className={`p-4 border ${layer.color}`} data-testid={`layer-${index}`}>
                  <div className="flex items-center gap-3 mb-2">
                    <Badge variant="outline" className="border-primary/40 text-primary text-xs">
                      Layer {layers.length - index}
                    </Badge>
                    <span className="font-semibold text-sm">{layer.label}</span>
                  </div>
                  <div className="flex flex-wrap gap-2">
                    {layer.items.map((item) => (
                      <span key={item} className="text-xs text-muted-foreground bg-background/60 rounded px-2 py-1">
                        {item}
                      </span>
                    ))}
                  </div>
                </Card>
              </motion.div>
            ))}
          </motion.div>
        </div>
      </div>
    </section>
  );
}

function ComponentCard({ 
  badge, 
  icon: Icon, 
  title, 
  description, 
  features, 
  index,
  link 
}: { 
  badge: string;
  icon: typeof Code;
  title: string;
  description: string;
  features: string[];
  index: number;
  link?: string;
}) {
  const ref = useRef(null);
  const isInView = useInView(ref, { once: true, margin: "-50px" });

  const CardContent = (
    <Card 
      className="p-6 md:p-8 h-full border-primary/10 bg-card/70 backdrop-blur-sm transition-all duration-300 group hover-elevate"
      data-testid={`card-component-${index}`}
    >
      <Badge 
        variant="outline" 
        className="mb-4 border-primary/30 bg-primary/10 text-primary text-xs"
      >
        {badge}
      </Badge>
      
      <div className="text-primary mb-5 group-hover:scale-110 transition-transform duration-300">
        <Icon className="w-10 h-10" />
      </div>
      
      <h3 className="text-xl font-semibold mb-3 text-foreground">{title}</h3>
      <p className="text-muted-foreground mb-5 text-sm leading-relaxed">{description}</p>
      
      <ul className="space-y-2">
        {features.map((feature) => (
          <li key={feature} className="flex items-start gap-2.5 text-sm text-muted-foreground">
            <Check className="w-4 h-4 text-primary flex-shrink-0 mt-0.5" />
            <span>{feature}</span>
          </li>
        ))}
      </ul>
      
      {link && (
        <div className="mt-6 pt-4 border-t border-primary/10">
          <span className="text-primary text-sm font-medium flex items-center gap-1 group-hover:gap-2 transition-all">
            Explore <ArrowRight className="w-4 h-4" />
          </span>
        </div>
      )}
    </Card>
  );

  return (
    <motion.div
      ref={ref}
      initial={{ opacity: 0, y: 30 }}
      animate={isInView ? { opacity: 1, y: 0 } : { opacity: 0, y: 30 }}
      transition={{ duration: 0.5, delay: index * 0.1 }}
    >
      {link ? (
        link.startsWith("http") ? (
          <a href={link} target="_blank" rel="noopener noreferrer" className="block h-full">
            {CardContent}
          </a>
        ) : (
          <Link href={link} className="block h-full">
            {CardContent}
          </Link>
        )
      ) : (
        CardContent
      )}
    </motion.div>
  );
}

function ComponentsSection() {
  const components = [
    {
      badge: "Core - 1,011 Tests",
      icon: Cpu,
      title: "Ternary Kernel",
      description: "Production-ready Rust kernel with GF(3) arithmetic, memory management, process scheduling, filesystem, and multi-architecture support.",
      link: "https://github.com/SigmaWolf-8/Ternary",
      features: [
        "Three bijective representations (A, B, C)",
        "Ticket spinlocks, semaphores, phase-aware mutexes",
        "Priority-based I/O scheduler & buffer cache",
        "CodeQL security scanning + GitHub Actions CI/CD",
      ],
    },
    {
      badge: "Virtual Machine",
      icon: Terminal,
      title: "Ternary VM (TVM)",
      description: "35-opcode register-based virtual machine with ternary-native arithmetic and automatic memory management.",
      link: "https://github.com/SigmaWolf-8/Ternary",
      features: [
        "GF(3) ops: TAdd, TMul, TNeg, TRot, TXor",
        "27 ternary registers with flags",
        "TAGC mark-sweep garbage collector",
        "Generational GC with young/old/permanent",
      ],
    },
    {
      badge: "Live Demo",
      icon: Database,
      title: "PlenumDB",
      description: "Ternary compression engine proving 59% information density advantage with real data. Try it live right now.",
      features: [
        "59% more information per digit",
        "3:2 binary-to-ternary compression ratio",
        "Real-time benchmarks with your own data",
        "Upload CSV, JSON, XLSX for instant results",
      ],
      link: "/ternarydb",
    },
    {
      badge: "API Gateway",
      icon: Network,
      title: "Kong Konnect + Salvi API",
      description: "Full REST API for ternary operations with enterprise-grade API gateway, rate limiting, and key management.",
      link: "/api-demo",
      features: [
        "GF(3) field operations API",
        "Phase-split encryption endpoints",
        "Femtosecond timing service",
        "Kong Konnect gateway integration",
      ],
    },
    {
      badge: "Regulatory Compliant",
      icon: Clock,
      title: "HPTP Timing Protocol",
      description: "Sub-microsecond precision timing with optical clock synchronization and built-in regulatory certification.",
      link: "https://github.com/SigmaWolf-8/Ternary",
      features: [
        "FINRA 613 compliance (50ms threshold)",
        "MiFID II compliance (100us/1ms tiers)",
        "Optical clock: Strontium, Ytterbium, Aluminum, Mercury",
        "Best master clock selection algorithm",
      ],
    },
    {
      badge: "Documentation",
      icon: Shield,
      title: "Whitepaper & Build Guides",
      description: "Comprehensive documentation covering the Unified Ternary Logic System, mathematical foundations, and build instructions.",
      link: "/whitepaper",
      features: [
        "Bijective mapping proofs",
        "Phase encryption methodology",
        "Network architecture design",
        "Step-by-step build guides + AI agent instructions",
      ],
    },
  ];

  return (
    <section id="components" className="py-20 md:py-28" data-testid="section-components">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-16">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
          >
            <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
              Ship Today
            </Badge>
          </motion.div>
          <motion.h2 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-3xl md:text-4xl font-bold mb-4"
            data-testid="text-components-title"
          >
            Deployable Components
          </motion.h2>
          <motion.p 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.15 }}
            className="text-muted-foreground text-lg max-w-2xl mx-auto"
          >
            Every component is built, tested, and ready for integration. Not a roadmap -- this is what exists right now.
          </motion.p>
        </div>
        
        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
          {components.map((component, index) => (
            <ComponentCard key={component.title} {...component} index={index} />
          ))}
        </div>
      </div>
    </section>
  );
}

function PerformanceSection() {
  const comparisonItems = [
    { label: "Information per Digit", current: "1.0 bit", ternary: "1.585 bits (+59%)", highlight: true },
    { label: "Storage Efficiency", current: "Baseline", ternary: "3:2 compression ratio", highlight: true },
    { label: "Quantum Resistance", current: "Vulnerable", ternary: "Post-quantum encryption", highlight: true },
    { label: "Logic States", current: "2 states (0,1)", ternary: "3 states per trit", highlight: true },
    { label: "Timing Precision", current: "Milliseconds", ternary: "Femtosecond (10^-15s)", highlight: true },
    { label: "Representation Types", current: "Single (0,1)", ternary: "Three bijective (A, B, C)", highlight: true },
    { label: "Arithmetic Base", current: "Modulo 2", ternary: "GF(3) Galois field", highlight: true },
    { label: "Regulatory Compliance", current: "Custom build", ternary: "FINRA 613 & MiFID II built-in", highlight: true },
  ];

  return (
    <section id="performance" className="py-20 md:py-28 bg-secondary/30" data-testid="section-performance">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-16">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
          >
            <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
              Proven Results
            </Badge>
          </motion.div>
          <motion.h2 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-3xl md:text-4xl font-bold mb-4"
            data-testid="text-performance-title"
          >
            Why Ternary Wins
          </motion.h2>
          <motion.p 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.15 }}
            className="text-muted-foreground text-lg max-w-2xl mx-auto"
          >
            Not theoretical advantages -- measured, tested, and verifiable performance improvements you can see in our live demo.
          </motion.p>
        </div>

        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5, delay: 0.2 }}
        >
          <Card className="max-w-4xl mx-auto p-6 md:p-10 border-primary/10 bg-card/80 backdrop-blur-sm">
            <div className="flex flex-col md:flex-row justify-between gap-6 mb-8 pb-6 border-b border-foreground/10">
              <div className="flex-1 text-center">
                <h3 className="text-xl font-semibold mb-2">Binary Systems</h3>
                <p className="text-sm text-muted-foreground">Current infrastructure</p>
              </div>
              <div className="flex-1 text-center">
                <h3 className="text-xl font-semibold text-primary mb-2">PlenumNET Ternary</h3>
                <p className="text-sm text-muted-foreground">Production-ready platform</p>
              </div>
            </div>

            <div className="space-y-0">
              {comparisonItems.map((item, index) => (
                <motion.div 
                  key={item.label}
                  initial={{ opacity: 0, x: -20 }}
                  whileInView={{ opacity: 1, x: 0 }}
                  viewport={{ once: true }}
                  transition={{ duration: 0.3, delay: index * 0.05 }}
                  className="flex flex-col md:flex-row items-start md:items-center justify-between py-4 border-b border-foreground/5 last:border-b-0 gap-2"
                  data-testid={`comparison-item-${index}`}
                >
                  <div className="flex-1 md:flex-[2] font-medium text-sm md:text-base">{item.label}</div>
                  <div className="flex-1 text-muted-foreground text-sm md:text-center">{item.current}</div>
                  <div className="flex-1 text-sm md:text-center font-semibold text-primary">
                    {item.ternary}
                  </div>
                </motion.div>
              ))}
            </div>

            <div className="mt-8 pt-6 border-t border-foreground/10 text-center">
              <Button asChild data-testid="button-try-demo">
                <a href="/ternarydb">
                  <Zap className="w-4 h-4 mr-2" />
                  Verify It Yourself -- Live Demo
                  <ArrowRight className="w-4 h-4 ml-2" />
                </a>
              </Button>
            </div>
          </Card>
        </motion.div>
      </div>
    </section>
  );
}

function CalendarPreviewSection() {
  const problems = [
    {
      problem: "Calendar Fragmentation",
      description: "12+ epoch dates with incompatible rules. Hebrew lunisolar, Islamic lunar, and Mayan vigesimal need 144 conversion functions.",
      solution: "Single JDN intermediary: O(n) instead of O(n\u00B2). 24 functions cover all 12 calendars.",
      icon: Globe,
    },
    {
      problem: "Y2038 Overflow",
      description: "32-bit timestamps overflow January 19, 2038. Billions of systems will fail.",
      solution: "128-bit femtosecond timestamps. No rollover until year ~3.9 x 10\u00B2\u2079.",
      icon: Clock,
    },
    {
      problem: "Precision Drift",
      description: "IEEE 754 floating-point errors accumulate. 1ms/day becomes 365ms/year -- fails regulatory compliance.",
      solution: "Integer-only calculations. Zero accumulation error. FINRA 613 & MiFID II ready.",
      icon: Shield,
    },
  ];

  return (
    <section className="py-20 md:py-28" data-testid="section-calendar-preview">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-16">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
          >
            <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
              Calendar Synchronization API
            </Badge>
          </motion.div>
          <motion.h2 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-3xl md:text-4xl font-bold mb-4"
            data-testid="text-calendar-preview-title"
          >
            One Timestamp. Every Calendar. 30,000 Years.
          </motion.h2>
          <motion.p 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.15 }}
            className="text-muted-foreground text-lg max-w-3xl mx-auto"
          >
            Convert any date across 12 major calendar systems -- from Mayan Long Count to Islamic Hijri -- with femtosecond precision.
            The 13-Moon Harmonic Calendar places the Day Out of Time at the golden ratio point (364/\u03C6 = Day 225, November 11),
            creating an 8/5 Fibonacci moon split that embeds organic growth mathematics into temporal architecture.
          </motion.p>
        </div>

        <div className="grid md:grid-cols-3 gap-6 mb-12">
          {problems.map((item, index) => (
            <motion.div
              key={item.problem}
              initial={{ opacity: 0, y: 30 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: index * 0.1 }}
            >
              <Card 
                className="p-6 md:p-8 h-full border-primary/10 bg-card/70 backdrop-blur-sm"
                data-testid={`card-calendar-problem-${index}`}
              >
                <div className="inline-flex items-center justify-center w-12 h-12 rounded-full bg-primary/10 text-primary mb-4">
                  <item.icon className="w-6 h-6" />
                </div>
                <h3 className="text-lg font-semibold mb-2">{item.problem}</h3>
                <p className="text-muted-foreground text-sm leading-relaxed mb-4">{item.description}</p>
                <div className="pt-4 border-t border-primary/10">
                  <div className="flex items-start gap-2">
                    <Check className="w-4 h-4 text-primary flex-shrink-0 mt-0.5" />
                    <p className="text-sm font-medium">{item.solution}</p>
                  </div>
                </div>
              </Card>
            </motion.div>
          ))}
        </div>

        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5, delay: 0.3 }}
        >
          <Card className="max-w-4xl mx-auto p-6 md:p-8 border-primary/10 bg-card/80 backdrop-blur-sm">
            <div className="grid sm:grid-cols-3 gap-6 text-center mb-8">
              <div>
                <div className="text-3xl font-bold text-primary">12</div>
                <div className="text-sm text-muted-foreground mt-1">Calendar Systems</div>
              </div>
              <div>
                <div className="text-3xl font-bold text-primary">30,000+</div>
                <div className="text-sm text-muted-foreground mt-1">Years of Coverage</div>
              </div>
              <div>
                <div className="text-3xl font-bold text-primary">10\u207B\u00B9\u2075s</div>
                <div className="text-sm text-muted-foreground mt-1">Timing Precision</div>
              </div>
            </div>

            <div className="flex flex-col sm:flex-row items-center justify-center gap-4 pt-6 border-t border-foreground/10">
              <Button asChild data-testid="button-explore-calendar">
                <Link href="/calendar">
                  <Globe className="w-4 h-4 mr-2" />
                  Explore Calendar API
                  <ArrowRight className="w-4 h-4 ml-2" />
                </Link>
              </Button>
              <Button variant="outline" asChild data-testid="button-calendar-docs">
                <Link href="/docs">View Documentation</Link>
              </Button>
            </div>
          </Card>
        </motion.div>
      </div>
    </section>
  );
}

function TargetMarketsSection() {
  const markets = [
    {
      icon: Building2,
      title: "Financial Services",
      description: "FINRA 613 & MiFID II compliant timing for high-frequency trading, regulatory reporting, and immutable audit trails with femtosecond precision.",
      stats: "Compliance built-in",
    },
    {
      icon: FlaskConical,
      title: "Research & HPC",
      description: "59% data density improvement for scientific computing, genomics, and large-scale simulations. Less bandwidth, more throughput.",
      stats: "59% density gain",
    },
    {
      icon: Factory,
      title: "Industrial IoT & Edge",
      description: "Bandwidth-optimized edge computing for manufacturing, autonomous systems, and real-time sensor networks.",
      stats: "3:2 compression",
    },
    {
      icon: Server,
      title: "Blockchain & DeFi",
      description: "Post-quantum secure witnessing, payment settlement via XRPL and Algorand, with Hedera HCS consensus integration.",
      stats: "Quantum-resistant",
    },
    {
      icon: Shield,
      title: "Defense & Intelligence",
      description: "Phase encryption with timing-window enforcement and Lamport one-time signatures for maximum security posture.",
      stats: "Post-quantum",
    },
    {
      icon: Activity,
      title: "Telecommunications",
      description: "Torsion Network topology with geodesic routing optimized for next-generation network infrastructure.",
      stats: "N-dimensional",
    },
  ];

  return (
    <section className="py-20 md:py-28" data-testid="section-markets">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-16">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
          >
            <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
              Market Opportunity
            </Badge>
          </motion.div>
          <motion.h2 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-3xl md:text-4xl font-bold mb-4"
          >
            Built for Industries That Demand More
          </motion.h2>
          <motion.p 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.15 }}
            className="text-muted-foreground text-lg max-w-2xl mx-auto"
          >
            Targeted deployments with measurable ROI across sectors where efficiency, security, and compliance are non-negotiable.
          </motion.p>
        </div>

        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
          {markets.map((market, index) => (
            <motion.div
              key={market.title}
              initial={{ opacity: 0, y: 30 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: index * 0.08 }}
            >
              <Card 
                className="p-6 md:p-8 h-full border-primary/10 bg-card/70 backdrop-blur-sm"
                data-testid={`card-market-${index}`}
              >
                <div className="flex items-start justify-between mb-4 gap-3">
                  <div className="inline-flex items-center justify-center w-12 h-12 rounded-full bg-primary/10 text-primary flex-shrink-0">
                    <market.icon className="w-6 h-6" />
                  </div>
                  <Badge variant="outline" className="border-primary/30 bg-primary/5 text-primary text-xs flex-shrink-0">
                    {market.stats}
                  </Badge>
                </div>
                <h3 className="text-lg font-semibold mb-2">{market.title}</h3>
                <p className="text-muted-foreground text-sm leading-relaxed">{market.description}</p>
              </Card>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}

function DeveloperCTASection() {
  const [email, setEmail] = useState("");
  const [name, setName] = useState("");
  const { toast } = useToast();

  const { data: countData } = useQuery<{ count: number }>({
    queryKey: ["/api/developer-signup/count"],
  });

  const signupMutation = useMutation({
    mutationFn: async (data: { email: string; name?: string }) => {
      const res = await apiRequest("POST", "/api/developer-signup", data);
      return res.json();
    },
    onSuccess: (data) => {
      toast({ title: "You're in!", description: data.message });
      setEmail("");
      setName("");
    },
    onError: () => {
      toast({ title: "Something went wrong", description: "Please try again.", variant: "destructive" });
    },
  });

  return (
    <section id="early-access" className="py-20 md:py-28 bg-secondary/30" data-testid="section-developer-cta">
      <div className="max-w-7xl mx-auto px-5">
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
        >
          <Card className="max-w-4xl mx-auto p-8 md:p-12 lg:p-16 border-0 bg-gradient-to-br from-primary to-primary/80 text-primary-foreground">
            <div className="text-center mb-8">
              <h2 className="text-3xl md:text-4xl font-bold mb-4" data-testid="text-cta-title">
                Build the Future of Computing
              </h2>
              <p className="text-lg opacity-90 max-w-2xl mx-auto mb-2">
                Get early access to the PlenumNET SDK, developer documentation, and direct support from the core team. 
                Be among the first to build applications on ternary infrastructure.
              </p>
              {countData && countData.count > 0 && (
                <p className="text-sm opacity-70" data-testid="text-signup-count">
                  {countData.count} developer{countData.count !== 1 ? "s" : ""} already signed up
                </p>
              )}
            </div>

            <form
              onSubmit={(e) => {
                e.preventDefault();
                if (email) signupMutation.mutate({ email, name: name || undefined });
              }}
              className="max-w-lg mx-auto space-y-3"
              data-testid="form-developer-signup"
            >
              <Input
                type="text"
                placeholder="Your name (optional)"
                value={name}
                onChange={(e) => setName(e.target.value)}
                className="bg-primary-foreground/10 border-primary-foreground/20 text-primary-foreground placeholder:text-primary-foreground/50"
                data-testid="input-signup-name"
              />
              <Input
                type="email"
                placeholder="developer@company.com"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                required
                className="bg-primary-foreground/10 border-primary-foreground/20 text-primary-foreground placeholder:text-primary-foreground/50"
                data-testid="input-signup-email"
              />
              <Button 
                type="submit" 
                size="lg"
                variant="secondary"
                className="w-full bg-background text-foreground"
                disabled={signupMutation.isPending}
                data-testid="button-developer-signup"
              >
                {signupMutation.isPending ? "Signing Up..." : "Get Early Access"}
                <ArrowRight className="w-4 h-4 ml-2" />
              </Button>
            </form>

            <div className="flex flex-wrap gap-6 justify-center mt-8 text-sm opacity-80">
              <div className="flex items-center gap-2">
                <Check className="w-4 h-4" />
                <span>SDK Access</span>
              </div>
              <div className="flex items-center gap-2">
                <Check className="w-4 h-4" />
                <span>Developer Docs</span>
              </div>
              <div className="flex items-center gap-2">
                <Check className="w-4 h-4" />
                <span>Core Team Support</span>
              </div>
              <div className="flex items-center gap-2">
                <Check className="w-4 h-4" />
                <span>Priority Updates</span>
              </div>
            </div>
          </Card>
        </motion.div>
      </div>
    </section>
  );
}

function Footer() {
  const footerLinks = {
    Platform: [
      { label: "Ternary Kernel", href: "https://github.com/SigmaWolf-8/Ternary" },
      { label: "PlenumDB Demo", href: "/ternarydb" },
      { label: "Salvi API", href: "/api-demo" },
      { label: "Kong Gateway", href: "/kong-konnect" },
    ],
    Developers: [
      { label: "Whitepaper", href: "/whitepaper" },
      { label: "API Demo", href: "/api-demo" },
      { label: "Build Guide", href: "https://github.com/SigmaWolf-8/Ternary/blob/main/KERNEL-BUILD-GUIDE.md" },
      { label: "GitHub", href: "https://github.com/SigmaWolf-8/Ternary" },
    ],
    Company: [
      { label: "About", href: "#" },
      { label: "Blog", href: "#" },
      { label: "Careers", href: "#" },
      { label: "Contact", href: "#early-access" },
    ],
    Legal: [
      { label: "Privacy", href: "#" },
      { label: "Terms", href: "#" },
      { label: "Security", href: "#" },
    ],
  };

  return (
    <footer className="bg-background border-t border-primary/10 py-16" data-testid="footer">
      <div className="max-w-7xl mx-auto px-5">
        <div className="grid grid-cols-2 md:grid-cols-5 gap-8 mb-12">
          <div className="col-span-2 md:col-span-1">
            <a href="#" className="flex items-center gap-2 text-primary font-bold text-xl mb-4" data-testid="link-footer-logo">
              <Box className="w-6 h-6" />
              <span>PlenumNET</span>
            </a>
            <p className="text-sm text-muted-foreground mb-4">
              The world's first ternary computing platform. Post-quantum security, 59% density advantage, shipping today.
            </p>
            <div className="flex gap-3">
              <a 
                href="#" 
                className="text-muted-foreground hover:text-primary transition-colors"
                data-testid="link-social-twitter"
              >
                <Twitter className="w-5 h-5" />
              </a>
              <a 
                href="#" 
                className="text-muted-foreground hover:text-primary transition-colors"
                data-testid="link-social-linkedin"
              >
                <Linkedin className="w-5 h-5" />
              </a>
              <a 
                href="https://github.com/SigmaWolf-8/Ternary" 
                target="_blank" 
                rel="noopener noreferrer"
                className="text-muted-foreground hover:text-primary transition-colors"
                data-testid="link-social-github"
              >
                <Github className="w-5 h-5" />
              </a>
              <a 
                href="#early-access" 
                className="text-muted-foreground hover:text-primary transition-colors"
                data-testid="link-social-email"
              >
                <Mail className="w-5 h-5" />
              </a>
            </div>
          </div>

          {Object.entries(footerLinks).map(([category, links]) => (
            <div key={category}>
              <h4 className="font-semibold mb-4 text-foreground">{category}</h4>
              <ul className="space-y-2">
                {links.map((link) => (
                  <li key={link.label}>
                    {link.href.startsWith("/") && !link.href.startsWith("/#") ? (
                      <Link 
                        href={link.href}
                        className="text-sm text-muted-foreground hover:text-primary transition-colors"
                      >
                        {link.label}
                      </Link>
                    ) : (
                      <a 
                        href={link.href} 
                        className="text-sm text-muted-foreground hover:text-primary transition-colors"
                        target={link.href.startsWith("http") ? "_blank" : undefined}
                        rel={link.href.startsWith("http") ? "noopener noreferrer" : undefined}
                      >
                        {link.label}
                      </a>
                    )}
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>

        <div className="pt-8 border-t border-primary/10 text-center text-sm text-muted-foreground">
          <p>All Rights Reserved and Preserved | &copy; Capomastro Holdings Ltd 2026</p>
        </div>
      </div>
    </footer>
  );
}

export default function Landing() {
  return (
    <div className="min-h-screen bg-background text-foreground">
      <main>
        <HeroSection />
        <PlatformSection />
        <ArchitectureSection />
        <ComponentsSection />
        <PerformanceSection />
        <CalendarPreviewSection />
        <TargetMarketsSection />
        <DeveloperCTASection />
      </main>
      <Footer />
    </div>
  );
}
