import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
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
  Menu,
  X,
  LogIn,
  LogOut,
  User,
  Settings
} from "lucide-react";
import { useState, useEffect, useRef } from "react";
import { motion, useInView } from "framer-motion";
import { Link } from "wouter";
import { useAuth } from "@/hooks/use-auth";
import { useQuery } from "@tanstack/react-query";

function Header() {
  const [isScrolled, setIsScrolled] = useState(false);
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const { user, isAuthenticated, isLoading } = useAuth();
  
  const { data: adminStatus } = useQuery<{ isAdmin: boolean }>({
    queryKey: ["/api/user/admin-status"],
    enabled: isAuthenticated,
  });

  useEffect(() => {
    const handleScroll = () => {
      setIsScrolled(window.scrollY > 20);
    };
    window.addEventListener("scroll", handleScroll);
    return () => window.removeEventListener("scroll", handleScroll);
  }, []);

  const navLinks = [
    { href: "#approach", label: "Approach" },
    { href: "#components", label: "Components" },
    { href: "#comparison", label: "Benefits" },
    { href: "/whitepaper", label: "Whitepaper" },
    { href: "/api-demo", label: "API Demo" },
  ];

  return (
    <header 
      className={`fixed top-0 left-0 right-0 z-50 transition-all duration-300 ${
        isScrolled 
          ? "bg-background/95 backdrop-blur-xl border-b border-primary/10" 
          : "bg-transparent"
      }`}
      data-testid="header"
    >
      <div className="max-w-7xl mx-auto px-5 py-5 flex items-center justify-between gap-4">
        <a href="#" className="flex items-center gap-2.5 text-primary font-bold text-xl" data-testid="link-logo">
          <Box className="w-7 h-7" />
          <span>PlenumNET</span>
        </a>
        
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
            <a href="https://github.com/ternary" target="_blank" rel="noopener noreferrer" data-testid="link-github">
              <Github className="w-4 h-4 mr-2" />
              GitHub
            </a>
          </Button>
          {isLoading ? (
            <div className="w-20 h-9 bg-primary/10 rounded animate-pulse" />
          ) : isAuthenticated ? (
            <div className="flex items-center gap-2">
              <span className="text-sm text-muted-foreground flex items-center gap-1.5">
                {user?.profileImageUrl ? (
                  <img src={user.profileImageUrl} alt="" className="w-6 h-6 rounded-full" />
                ) : (
                  <User className="w-4 h-4" />
                )}
                {user?.firstName || user?.email?.split('@')[0]}
              </span>
              {adminStatus?.isAdmin && (
                <>
                  <Link href="/kong-konnect">
                    <Button variant="outline" className="border-primary/50 text-primary" data-testid="button-kong-konnect">
                      <Network className="w-4 h-4 mr-2" />
                      Kong
                    </Button>
                  </Link>
                  <Link href="/github">
                    <Button variant="outline" className="border-primary/50 text-primary" data-testid="button-admin-github">
                      <Settings className="w-4 h-4 mr-2" />
                      Admin
                    </Button>
                  </Link>
                </>
              )}
              <Button variant="outline" asChild className="border-primary/50 text-primary" data-testid="button-logout">
                <a href="/api/logout">
                  <LogOut className="w-4 h-4 mr-2" />
                  Logout
                </a>
              </Button>
            </div>
          ) : (
            <Button asChild data-testid="button-login">
              <a href="/api/login">
                <LogIn className="w-4 h-4 mr-2" />
                Sign In
              </a>
            </Button>
          )}
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
              <a href="https://github.com/ternary" target="_blank" rel="noopener noreferrer">
                <Github className="w-4 h-4 mr-2" />
                GitHub
              </a>
            </Button>
            {isAuthenticated ? (
              <>
                {adminStatus?.isAdmin && (
                  <Link href="/kong-konnect">
                    <Button variant="outline" className="border-primary/50 text-primary w-full" data-testid="button-mobile-kong">
                      <Network className="w-4 h-4 mr-2" />
                      Kong Konnect
                    </Button>
                  </Link>
                )}
                <Button variant="outline" asChild className="border-primary/50 text-primary" data-testid="button-mobile-logout">
                  <a href="/api/logout">
                    <LogOut className="w-4 h-4 mr-2" />
                    Logout
                  </a>
                </Button>
              </>
            ) : (
              <Button asChild data-testid="button-mobile-login">
                <a href="/api/login">
                  <LogIn className="w-4 h-4 mr-2" />
                  Sign In with GitHub
                </a>
              </Button>
            )}
          </div>
        </motion.div>
      )}
    </header>
  );
}

function AnimatedStat({ value, label, delay }: { value: string; label: string; delay: number }) {
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
      <span className="text-4xl md:text-5xl font-bold text-primary leading-none">{value}</span>
      <span className="text-sm text-muted-foreground mt-2">{label}</span>
    </motion.div>
  );
}

function HeroSection() {
  return (
    <section className="relative pt-32 pb-20 md:pt-40 md:pb-28 overflow-hidden" data-testid="section-hero">
      <div className="absolute inset-0 bg-gradient-to-br from-background via-secondary/30 to-background" />
      <div className="absolute inset-0 gradient-radial" />
      
      <div className="relative z-10 max-w-7xl mx-auto px-5">
        <div className="max-w-3xl">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5 }}
          >
            <Badge 
              variant="outline" 
              className="mb-6 border-primary/30 bg-primary/10 text-primary px-4 py-1.5"
              data-testid="badge-hero"
            >
              Deployable Today
            </Badge>
          </motion.div>
          
          <motion.h1 
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-4xl md:text-5xl lg:text-6xl font-bold leading-tight mb-6"
            data-testid="text-hero-title"
          >
            Post-Quantum Internet, <span className="text-primary">Deployed Pragmatically</span>
          </motion.h1>
          
          <motion.p 
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.2 }}
            className="text-lg md:text-xl text-muted-foreground mb-10 max-w-2xl"
            data-testid="text-hero-description"
          >
            We're building the quantum-resistant internet through market-tested components, 
            starting with what can be deployed today within existing infrastructure.
          </motion.p>
          
          <motion.div 
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.3 }}
            className="flex flex-wrap gap-4"
          >
            <Button size="lg" asChild data-testid="button-get-started">
              <a href="#components">
                Get Started
                <ArrowRight className="w-4 h-4 ml-2" />
              </a>
            </Button>
            <Button size="lg" variant="outline" asChild className="border-primary/50 text-primary" data-testid="button-view-docs">
              <a href="https://github.com/ternary" target="_blank" rel="noopener noreferrer">
                View Documentation
              </a>
            </Button>
          </motion.div>
          
          <div className="flex flex-wrap gap-8 md:gap-12 mt-16">
            <AnimatedStat value="+59%" label="Information Density" delay={0.4} />
            <AnimatedStat value="12.7%" label="Efficiency Gain" delay={0.5} />
            <AnimatedStat value="$0.01" label="Per Million Timestamps" delay={0.6} />
            <AnimatedStat value="Phase 1" label="Live Today" delay={0.7} />
          </div>
        </div>
      </div>
    </section>
  );
}

function TimelineItem({ 
  date, 
  title, 
  description, 
  tags, 
  isLast,
  index 
}: { 
  date: string; 
  title: string; 
  description: string; 
  tags: string[];
  isLast: boolean;
  index: number;
}) {
  const ref = useRef(null);
  const isInView = useInView(ref, { once: true, margin: "-50px" });

  return (
    <motion.div 
      ref={ref}
      initial={{ opacity: 0, x: -20 }}
      animate={isInView ? { opacity: 1, x: 0 } : { opacity: 0, x: -20 }}
      transition={{ duration: 0.5, delay: index * 0.1 }}
      className="flex relative mb-12 last:mb-0"
      data-testid={`timeline-item-${index}`}
    >
      <div className="hidden md:block w-24 text-right pr-8 text-primary font-semibold text-sm pt-1 flex-shrink-0">
        {date}
      </div>
      
      <div className="hidden md:flex flex-col items-center flex-shrink-0">
        <div className="w-3 h-3 rounded-full bg-primary border-2 border-background z-10" />
        {!isLast && <div className="w-0.5 flex-1 bg-primary/30 mt-2" />}
      </div>
      
      <div className="flex-1 md:pl-8">
        <span className="md:hidden text-primary font-semibold text-sm mb-2 block">{date}</span>
        <h3 className="text-xl font-semibold mb-3 text-foreground">{title}</h3>
        <p className="text-muted-foreground mb-4">{description}</p>
        <div className="flex flex-wrap gap-2">
          {tags.map((tag) => (
            <Badge 
              key={tag} 
              variant="outline" 
              className="border-primary/30 bg-primary/10 text-primary text-xs"
            >
              {tag}
            </Badge>
          ))}
        </div>
      </div>
    </motion.div>
  );
}

function ApproachSection() {
  const timelineItems = [
    {
      date: "NOW",
      title: "Phase 1: Reality-Based Foundation",
      description: "Deploying market-ready components that work within existing infrastructure.",
      tags: ["Ternary Repository", "Timing API", "Open Source"],
    },
    {
      date: "Q3 2024",
      title: "Phase 2: Trojan Horse Deployments",
      description: "Strategic partnerships in finance, research, and IoT with ternary-overlay solutions.",
      tags: ["Financial Pilot", "Research Networks", "IIoT Edge"],
    },
    {
      date: "2025",
      title: "Phase 3: Component Commercialization",
      description: "FPGA acceleration cards, timing certification, and ternary database engines.",
      tags: ["FPGA Cards", "Database Engine", "Enterprise"],
    },
    {
      date: "2026+",
      title: "Phase 4: Standards & Ecosystem",
      description: "Reference implementations, IETF standards, and hardware partnerships.",
      tags: ["Standards", "RISC-V Extensions", "Full Architecture"],
    },
  ];

  return (
    <section id="approach" className="py-20 md:py-28" data-testid="section-approach">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-16">
          <motion.h2 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
            className="text-3xl md:text-4xl font-bold mb-4"
            data-testid="text-approach-title"
          >
            Our Pragmatic, Phased Approach
          </motion.h2>
          <motion.p 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-muted-foreground text-lg max-w-2xl mx-auto"
          >
            We're implementing the visionary architecture through digestible, revenue-generating components
          </motion.p>
        </div>
        
        <div id="timeline" className="max-w-3xl mx-auto">
          {timelineItems.map((item, index) => (
            <TimelineItem 
              key={item.date} 
              {...item} 
              isLast={index === timelineItems.length - 1}
              index={index}
            />
          ))}
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
      className="p-6 md:p-8 h-full border-primary/10 bg-card/70 backdrop-blur-sm hover:border-primary/40 transition-all duration-300 group"
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
            Learn more <ArrowRight className="w-4 h-4" />
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
        <Link href={link} className="block h-full">
          {CardContent}
        </Link>
      ) : (
        CardContent
      )}
    </motion.div>
  );
}

function ComponentsSection() {
  const components = [
    {
      badge: "Available Now",
      icon: Code,
      title: "GitHub Ternary Repository",
      description: "Open-source ternary computing library with 59% more data per transfer.",
      link: "/github",
      features: [
        "59% more data per transfer",
        "Drop-in replacement for binary protocols",
        "C, Python, Rust bindings available",
        "Zero runtime dependencies",
      ],
    },
    {
      badge: "Available Now",
      icon: Clock,
      title: "Timing API",
      description: "Nanosecond-precision timestamps with cryptographic guarantees. FINRA 613 compliant.",
      features: [
        "$0.01 per million timestamps",
        "Cryptographic proof of time",
        "REST API with SDK",
        "99.99% SLA guaranteed",
      ],
    },
    {
      badge: "Q2 2024",
      icon: Cpu,
      title: "FPGA Acceleration",
      description: "Hardware acceleration for ternary operations. PCIe cards for data centers.",
      features: [
        "100x faster ternary operations",
        "Standard PCIe form factor",
        "Linux driver included",
        "Development kit available",
      ],
    },
    {
      badge: "Q3 2024",
      icon: Database,
      title: "PlenumDB",
      description: "Purpose-built database engine optimized for ternary data structures and queries.",
      features: [
        "Native ternary storage format",
        "3x storage efficiency",
        "SQL-compatible interface",
        "Horizontal scaling built-in",
      ],
      link: "/ternarydb",
    },
    {
      badge: "2025",
      icon: Shield,
      title: "Quantum-Safe PKI",
      description: "Post-quantum cryptography infrastructure for certificate management and key exchange.",
      features: [
        "NIST PQC algorithms",
        "Backward compatible",
        "Hardware security module support",
        "Enterprise key management",
      ],
    },
    {
      badge: "2025",
      icon: Network,
      title: "PlenumMesh",
      description: "Distributed network protocol for ternary data transmission and routing.",
      features: [
        "Self-healing topology",
        "Integrated load balancing",
        "End-to-end encryption",
        "Cross-datacenter support",
      ],
    },
  ];

  return (
    <section id="components" className="py-20 md:py-28 bg-secondary/30" data-testid="section-components">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-16">
          <motion.h2 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
            className="text-3xl md:text-4xl font-bold mb-4"
            data-testid="text-components-title"
          >
            Market-Ready Components
          </motion.h2>
          <motion.p 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-muted-foreground text-lg max-w-2xl mx-auto"
          >
            Deployable solutions generating value today
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

function ComparisonSection() {
  const comparisonItems = [
    { label: "Information Density", current: "1.0x", ternary: "1.59x", ternaryGood: true },
    { label: "Quantum Resistance", current: "Vulnerable", ternary: "Post-Quantum Safe", ternaryGood: true },
    { label: "Timing Precision", current: "Milliseconds", ternary: "Nanoseconds", ternaryGood: true },
    { label: "Protocol Efficiency", current: "Baseline", ternary: "+12.7%", ternaryGood: true },
    { label: "Deployment Model", current: "Replace All", ternary: "Incremental", ternaryGood: true },
    { label: "Open Source", current: "Varies", ternary: "Open Source", ternaryGood: true },
  ];

  return (
    <section id="comparison" className="py-20 md:py-28" data-testid="section-comparison">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-16">
          <motion.h2 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
            className="text-3xl md:text-4xl font-bold mb-4"
            data-testid="text-comparison-title"
          >
            Why PlenumNET?
          </motion.h2>
          <motion.p 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-muted-foreground text-lg max-w-2xl mx-auto"
          >
            Measurable improvements over current infrastructure
          </motion.p>
        </div>

        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5, delay: 0.2 }}
        >
          <Card className="max-w-4xl mx-auto p-6 md:p-10 border-primary/10 bg-secondary/50 backdrop-blur-sm">
            <div className="flex flex-col md:flex-row justify-between gap-6 mb-8 pb-6 border-b border-foreground/10">
              <div className="flex-1 text-center">
                <h3 className="text-xl font-semibold mb-2">Current Internet</h3>
                <p className="text-sm text-muted-foreground">Legacy binary infrastructure</p>
              </div>
              <div className="flex-1 text-center">
                <h3 className="text-xl font-semibold text-primary mb-2">PlenumNET Architecture</h3>
                <p className="text-sm text-muted-foreground">Next-generation foundation</p>
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
                  <div className={`flex-1 text-sm md:text-center font-semibold ${item.ternaryGood ? "text-primary" : "text-foreground"}`}>
                    {item.ternary}
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

function TargetMarketsSection() {
  const markets = [
    {
      icon: Building2,
      title: "Financial Services",
      description: "FINRA 613 compliant timing for HFT, regulatory reporting, and audit trails.",
    },
    {
      icon: FlaskConical,
      title: "Research Networks",
      description: "High-performance computing and scientific data transfer optimization.",
    },
    {
      icon: Factory,
      title: "Industrial IoT",
      description: "Edge computing and bandwidth optimization for manufacturing systems.",
    },
  ];

  return (
    <section className="py-20 md:py-28 bg-secondary/30" data-testid="section-markets">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-16">
          <motion.h2 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
            className="text-3xl md:text-4xl font-bold mb-4"
          >
            Target Markets
          </motion.h2>
          <motion.p 
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-muted-foreground text-lg max-w-2xl mx-auto"
          >
            Focused deployments with measurable ROI
          </motion.p>
        </div>

        <div className="grid md:grid-cols-3 gap-6">
          {markets.map((market, index) => (
            <motion.div
              key={market.title}
              initial={{ opacity: 0, y: 30 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: index * 0.1 }}
            >
              <Card 
                className="p-6 md:p-8 h-full border-primary/10 bg-card/70 backdrop-blur-sm text-center hover:border-primary/40 transition-all duration-300"
                data-testid={`card-market-${index}`}
              >
                <div className="inline-flex items-center justify-center w-16 h-16 rounded-full bg-primary/10 text-primary mb-5">
                  <market.icon className="w-8 h-8" />
                </div>
                <h3 className="text-xl font-semibold mb-3">{market.title}</h3>
                <p className="text-muted-foreground text-sm leading-relaxed">{market.description}</p>
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
              Ready to Build the Future?
            </h2>
            <p className="text-lg opacity-90 mb-8 max-w-xl mx-auto">
              Start with our open-source library today, or schedule a demo to see how PlenumNET can transform your infrastructure.
            </p>
            <div className="flex flex-wrap gap-4 justify-center">
              <Button 
                size="lg" 
                variant="secondary" 
                className="bg-background text-foreground"
                asChild
                data-testid="button-cta-github"
              >
                <a href="https://github.com/SigmaWolf-8/Ternary" target="_blank" rel="noopener noreferrer">
                  <Github className="w-4 h-4 mr-2" />
                  Get Ternary Repository
                </a>
              </Button>
              <Button 
                size="lg" 
                variant="ghost" 
                className="border border-primary-foreground/50 text-primary-foreground"
                data-testid="button-cta-demo"
              >
                <Zap className="w-4 h-4 mr-2" />
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
  const footerLinks = {
    Product: [
      { label: "GitHub Ternary Repository", href: "https://github.com/SigmaWolf-8/Ternary" },
      { label: "Timing API", href: "/api-demo" },
      { label: "Kong Konnect Integration", href: "/kong-konnect" },
      { label: "PlenumDB", href: "/ternarydb" },
    ],
    Developers: [
      { label: "Documentation", href: "#" },
      { label: "Whitepaper", href: "/whitepaper" },
      { label: "API Demo", href: "/api-demo" },
      { label: "GitHub", href: "https://github.com/SigmaWolf-8/Ternary" },
    ],
    Company: [
      { label: "About", href: "#" },
      { label: "Blog", href: "#" },
      { label: "Careers", href: "#" },
      { label: "Contact", href: "#contact" },
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
              Building the quantum-resistant internet, one component at a time.
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
                href="#contact" 
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
      <Header />
      <main>
        <HeroSection />
        <ApproachSection />
        <ComponentsSection />
        <ComparisonSection />
        <TargetMarketsSection />
        <CTASection />
      </main>
      <Footer />
    </div>
  );
}
