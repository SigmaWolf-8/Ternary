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
 *
 * ──────────────────────────────────────────────
 * NAVIGATION TAXONOMY (3 dropdowns, zero duplicates):
 *
 *   Platform    → "Understand the system"  (architecture + research)
 *   Developers  → "Use the tools"          (all interactive pages + docs)
 *   Company     → "Who we are"             (about + contact + legal)
 *
 * Every page appears exactly once.
 * ──────────────────────────────────────────────
 */

import { useState, useEffect, useCallback } from "react";
import { Link, useLocation } from "wouter";
import { Menu, Sun, Moon, ExternalLink, ArrowRight } from "lucide-react";
import plenumLogo from "@assets/grok-image-69a372f5-5c40-48be-b431-a4dbb4e92ff2_1771299513785.png";
import { useIsMobile } from "@/hooks/use-mobile";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  NavigationMenu,
  NavigationMenuList,
  NavigationMenuItem,
  NavigationMenuTrigger,
  NavigationMenuContent,
  NavigationMenuLink,
} from "@/components/ui/navigation-menu";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet";
import {
  Accordion,
  AccordionItem,
  AccordionTrigger,
  AccordionContent,
} from "@/components/ui/accordion";
import { cn } from "@/lib/utils";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { PLATFORM } from "@shared/constants";
import { createContext, useContext } from "react";
import { triggerInstallDialog } from "@/components/InstallExtensionCard";
import { useMutation } from "@tanstack/react-query";
import { apiRequest } from "@/lib/queryClient";
import { useToast } from "@/hooks/use-toast";

type AnchorScrollFn = (id: string) => void;
const AnchorScrollContext = createContext<AnchorScrollFn>(() => {});

type NavigateFn = (href: string) => void;
const NavigateContext = createContext<NavigateFn>(() => {});

interface NavLinkItem {
  title: string;
  subtitle?: string;
  href: string;
  external?: boolean;
}

interface NavColumn {
  heading: string;
  items: NavLinkItem[];
}

const platformColumns: NavColumn[] = [
  {
    heading: "Architecture",
    items: [
      { title: "Ternary Computing Engine", subtitle: "Native base-3 processing", href: "/#platform" },
      { title: `${PLATFORM.VM_OPCODES}-Opcode Virtual Machine`, subtitle: "Full ternary ISA", href: "/#architecture" },
      { title: "Binary-Ternary Gateway", subtitle: "Seamless interop layer", href: "/#architecture" },
      { title: "Torsion Network Stack", subtitle: "3-ary torus topology", href: "/#platform" },
      { title: "Performance Metrics", subtitle: `${PLATFORM.DENSITY_ADVANTAGE}% density advantage`, href: "/#performance" },
    ],
  },
  {
    heading: "Research & Security",
    items: [
      { title: "Whitepaper", subtitle: "Technical deep-dive", href: "/whitepaper" },
      { title: "ISA Security Primitives", subtitle: "Instruction-level hardening", href: "/isa-security" },
      { title: "CNSA 2.0 Compliance", subtitle: "Post-quantum aligned", href: "/compliance" },
      { title: "Time-Stamping Authority", subtitle: "RFC 3161 digital notary", href: "/tsa" },
      { title: "Tribonacci 28DS", subtitle: "28-dimension symmetry", href: "/tribonacci-28ds" },
      { title: "FPGA Benchmarks", subtitle: "Yosys / Vivado synthesis", href: "/fpga-benchmarks" },
    ],
  },
];

const developersColumns: NavColumn[] = [
  {
    heading: "Interactive Labs",
    items: [
      { title: "PlenumDB Console", subtitle: "Query ternary data", href: "/ternarydb" },
      { title: "API Explorer", subtitle: "Try the REST API", href: "/api-demo" },
      { title: "HPTP Timing Lab", subtitle: "Femtosecond precision", href: "/hptp" },
      { title: "Compression Studio", subtitle: "Encode & decode", href: "/compression" },
      { title: "Ternary VM Terminal", subtitle: "Execute ternary opcodes", href: "/vm-demo" },
    ],
  },
  {
    heading: "Simulators & Calendars",
    items: [
      { title: "Quantum Simulator", subtitle: "Qutrit FT / FIPS / QVQE", href: "/quantum-sim" },
      { title: "28D Agent Array", subtitle: "Multi-agent ternary simulation", href: "/agent-array" },
      { title: "Universal Calendar", subtitle: "Cross-calendar conversion", href: "/calendar" },
      { title: "13-Moon Harmonic", subtitle: "Natural time system", href: "/13-moon" },
    ],
  },
  {
    heading: "Get Started",
    items: [
      { title: "Documentation", subtitle: "API reference & guides", href: "/docs" },
      { title: "Kong Konnect Gateway", subtitle: "API gateway management", href: "/kong-konnect" },
      { title: "Module Distribution", subtitle: "Install the framework", href: "/distribution" },
      { title: "Install TDNS Browser Extension", subtitle: "Resolve .plm addresses", href: "#install-extension-download" },
      { title: "GitHub Repository", subtitle: "Source code & issues", href: "https://github.com/SigmaWolf-8/Ternary", external: true },
    ],
  },
];

const companyItems: NavLinkItem[] = [
  { title: "About PlenumNET", href: "/about" },
  { title: "Contact", href: "/contact" },
];

const companyLegalItems: NavLinkItem[] = [
  { title: "Terms of Service", href: "/terms" },
  { title: "Privacy Policy", href: "/privacy" },
  { title: "Security Policy", href: "/security" },
  { title: "Acceptable Use Policy", href: "/aup" },
];

function NavItemLink({
  item,
  className,
  onNavigate,
}: {
  item: NavLinkItem;
  className?: string;
  onNavigate?: () => void;
}) {
  const scrollToAnchor = useContext(AnchorScrollContext);
  const isAnchor = item.href.startsWith("/#");
  const anchorId = isAnchor ? item.href.slice(2) : "";

  if (item.href === "#install-extension-download") {
    return (
      <a
        href="#"
        className={className}
        data-testid={`link-${item.title.toLowerCase().replace(/\s+/g, "-")}`}
        onClick={(e) => {
          e.preventDefault();
          onNavigate?.();
          triggerInstallDialog();
        }}
      >
        {item.title}
      </a>
    );
  }

  if (item.external) {
    return (
      <a
        href={item.href}
        target="_blank"
        rel="noopener noreferrer"
        className={cn("flex items-center gap-1", className)}
        data-testid={`link-${item.title.toLowerCase().replace(/\s+/g, "-")}`}
        onClick={onNavigate}
      >
        {item.title}
        <ExternalLink className="w-3 h-3 text-muted-foreground" />
      </a>
    );
  }

  if (isAnchor) {
    return (
      <a
        href={item.href}
        className={className}
        data-testid={`link-${item.title.toLowerCase().replace(/\s+/g, "-")}`}
        onClick={(e) => {
          e.preventDefault();
          scrollToAnchor(anchorId);
          onNavigate?.();
        }}
      >
        {item.title}
      </a>
    );
  }

  return (
    <Link
      href={item.href}
      className={className}
      data-testid={`link-${item.title.toLowerCase().replace(/\s+/g, "-")}`}
      onClick={onNavigate}
    >
      {item.title}
    </Link>
  );
}

function MegaDropdownItem({ item }: { item: NavLinkItem }) {
  const scrollToAnchor = useContext(AnchorScrollContext);
  const isAnchor = item.href.startsWith("/#");
  const isDialogTrigger = item.href === "#install-extension-download";
  const anchorId = isAnchor ? item.href.slice(2) : "";
  const baseClass =
    "block select-none rounded-md p-3 leading-none no-underline outline-none transition-colors hover-elevate";

  const content = (
    <>
      <div className="text-sm font-medium leading-none flex items-center gap-1">
        {item.title}
        {item.external && <ExternalLink className="w-3 h-3 text-muted-foreground" />}
      </div>
      {item.subtitle && (
        <p className="line-clamp-2 text-xs leading-snug text-muted-foreground mt-1">
          {item.subtitle}
        </p>
      )}
    </>
  );

  if (isDialogTrigger) {
    return (
      <li>
        <NavigationMenuLink asChild>
          <a
            href="#"
            className={baseClass}
            data-testid={`link-${item.title.toLowerCase().replace(/\s+/g, "-")}`}
            onClick={(e) => {
              e.preventDefault();
              triggerInstallDialog();
            }}
          >
            {content}
          </a>
        </NavigationMenuLink>
      </li>
    );
  }

  if (isAnchor) {
    return (
      <li>
        <NavigationMenuLink asChild>
          <a
            href={item.href}
            className={baseClass}
            data-testid={`link-${item.title.toLowerCase().replace(/\s+/g, "-")}`}
            onClick={(e) => {
              e.preventDefault();
              scrollToAnchor(anchorId);
            }}
          >
            {content}
          </a>
        </NavigationMenuLink>
      </li>
    );
  }

  if (item.external) {
    return (
      <li>
        <NavigationMenuLink asChild>
          <a
            href={item.href}
            target="_blank"
            rel="noopener noreferrer"
            className={baseClass}
            data-testid={`link-${item.title.toLowerCase().replace(/\s+/g, "-")}`}
          >
            {content}
          </a>
        </NavigationMenuLink>
      </li>
    );
  }

  const navigate = useContext(NavigateContext);
  return (
    <li>
      <NavigationMenuLink asChild>
        <a
          href={item.href}
          className={baseClass}
          data-testid={`link-${item.title.toLowerCase().replace(/\s+/g, "-")}`}
          onClick={(e) => {
            e.preventDefault();
            navigate(item.href);
          }}
        >
          {content}
        </a>
      </NavigationMenuLink>
    </li>
  );
}

function MegaDropdown({ columns }: { columns: NavColumn[] }) {
  const colCount = columns.length;
  const gridClass = colCount === 2
    ? "grid gap-3 p-4 md:w-[460px] lg:w-[520px] md:grid-cols-2"
    : "grid gap-3 p-4 md:w-[600px] lg:w-[700px] md:grid-cols-3";

  return (
    <div className={gridClass}>
      {columns.map((col) => (
        <div key={col.heading}>
          <h4 className="mb-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground px-3">
            {col.heading}
          </h4>
          <ul className="space-y-0">
            {col.items.map((item) => (
              <MegaDropdownItem key={item.title} item={item} />
            ))}
          </ul>
        </div>
      ))}
    </div>
  );
}

function StandardDropdownItem({ item }: { item: NavLinkItem }) {
  const scrollToAnchor = useContext(AnchorScrollContext);
  const isAnchor = item.href.startsWith("/#");
  const anchorId = isAnchor ? item.href.slice(2) : "";
  const baseClass =
    "block select-none rounded-md px-3 py-2 text-sm no-underline outline-none transition-colors hover-elevate";

  if (item.external) {
    return (
      <li>
        <NavigationMenuLink asChild>
          <a
            href={item.href}
            target="_blank"
            rel="noopener noreferrer"
            className={cn(baseClass, "flex items-center gap-1")}
            data-testid={`link-${item.title.toLowerCase().replace(/\s+/g, "-")}`}
          >
            {item.title}
            <ExternalLink className="w-3 h-3 text-muted-foreground" />
          </a>
        </NavigationMenuLink>
      </li>
    );
  }

  if (isAnchor) {
    return (
      <li>
        <NavigationMenuLink asChild>
          <a
            href={item.href}
            className={baseClass}
            data-testid={`link-${item.title.toLowerCase().replace(/\s+/g, "-")}`}
            onClick={(e) => {
              e.preventDefault();
              scrollToAnchor(anchorId);
            }}
          >
            {item.title}
          </a>
        </NavigationMenuLink>
      </li>
    );
  }

  const navigate = useContext(NavigateContext);
  return (
    <li>
      <NavigationMenuLink asChild>
        <a
          href={item.href}
          className={baseClass}
          data-testid={`link-${item.title.toLowerCase().replace(/\s+/g, "-")}`}
          onClick={(e) => {
            e.preventDefault();
            navigate(item.href);
          }}
        >
          {item.title}
        </a>
      </NavigationMenuLink>
    </li>
  );
}

function DesktopNav({ onOpenChange }: { onOpenChange?: (open: boolean) => void }) {
  return (
    <NavigationMenu
      className="hidden md:flex [&_button]:bg-transparent [&_button:hover]:bg-transparent justify-start"
      delayDuration={100}
      skipDelayDuration={300}
      onValueChange={(val: string) => onOpenChange?.(val !== "")}
    >
      <NavigationMenuList>
        <NavigationMenuItem>
          <NavigationMenuTrigger
            data-testid="nav-trigger-platform"
            style={{ color: "#E4DFD5", fontSize: "20px", fontWeight: 800, letterSpacing: "0.12em", fontFamily: "'Orbitron', sans-serif", textTransform: "uppercase" as const, textShadow: "0 1px 0 rgba(0,0,0,0.1), 0 2px 0 rgba(0,0,0,0.07), 0 3px 8px rgba(0,0,0,0.08), 0 0 20px rgba(56,189,248,0.15)" }}
            className="hover:!text-[#38BDF8] data-[state=open]:!text-[#38BDF8]"
          >
            Platform
          </NavigationMenuTrigger>
          <NavigationMenuContent className="left-0">
            <MegaDropdown columns={platformColumns} />
          </NavigationMenuContent>
        </NavigationMenuItem>

        <NavigationMenuItem>
          <NavigationMenuTrigger
            data-testid="nav-trigger-developers"
            style={{ color: "#E4DFD5", fontSize: "20px", fontWeight: 800, letterSpacing: "0.12em", fontFamily: "'Orbitron', sans-serif", textTransform: "uppercase" as const, textShadow: "0 1px 0 rgba(0,0,0,0.1), 0 2px 0 rgba(0,0,0,0.07), 0 3px 8px rgba(0,0,0,0.08), 0 0 20px rgba(56,189,248,0.15)" }}
            className="hover:!text-[#38BDF8] data-[state=open]:!text-[#38BDF8]"
          >
            Developers
          </NavigationMenuTrigger>
          <NavigationMenuContent className="left-0">
            <MegaDropdown columns={developersColumns} />
          </NavigationMenuContent>
        </NavigationMenuItem>

        <NavigationMenuItem>
          <NavigationMenuTrigger
            data-testid="nav-trigger-company"
            style={{ color: "#E4DFD5", fontSize: "20px", fontWeight: 800, letterSpacing: "0.12em", fontFamily: "'Orbitron', sans-serif", textTransform: "uppercase" as const, textShadow: "0 1px 0 rgba(0,0,0,0.1), 0 2px 0 rgba(0,0,0,0.07), 0 3px 8px rgba(0,0,0,0.08), 0 0 20px rgba(56,189,248,0.15)" }}
            className="hover:!text-[#38BDF8] data-[state=open]:!text-[#38BDF8]"
          >
            Company
          </NavigationMenuTrigger>
          <NavigationMenuContent className="right-0 left-auto">
            <ul className="w-[220px] p-2">
              {companyItems.map((item) => (
                <StandardDropdownItem key={item.title} item={item} />
              ))}
              <li>
                <div className="my-1 mx-3 h-px bg-border" />
              </li>
              {companyLegalItems.map((item) => (
                <StandardDropdownItem key={item.title} item={item} />
              ))}
            </ul>
          </NavigationMenuContent>
        </NavigationMenuItem>
      </NavigationMenuList>
    </NavigationMenu>
  );
}

function MobileAccordionSection({
  title,
  items,
  onNavigate,
}: {
  title: string;
  items: NavLinkItem[];
  onNavigate: () => void;
}) {
  return (
    <AccordionItem value={title}>
      <AccordionTrigger
        className="text-sm"
        data-testid={`mobile-accordion-${title.toLowerCase().replace(/\s+/g, "-")}`}
      >
        {title}
      </AccordionTrigger>
      <AccordionContent>
        <div className="flex flex-col gap-1 pl-2">
          {items.map((item) => (
            <NavItemLink
              key={item.title}
              item={item}
              className="block py-2 px-2 text-sm text-foreground/80 rounded-md hover-elevate"
              onNavigate={onNavigate}
            />
          ))}
        </div>
      </AccordionContent>
    </AccordionItem>
  );
}

function MobileNav({ onClose, navEmail, setNavEmail, navSignupMutation }: {
  onClose: () => void;
  navEmail: string;
  setNavEmail: (v: string) => void;
  navSignupMutation: ReturnType<typeof useMutation<{ message: string }, Error, { email: string }>>;
}) {
  const allPlatformItems = platformColumns.flatMap((c) => c.items);
  const allDevelopersItems = developersColumns.flatMap((c) => c.items);
  const allCompanyItems = [...companyItems, ...companyLegalItems];

  return (
    <div className="flex flex-col gap-4 mt-4">
      <Accordion type="multiple" className="w-full">
        <MobileAccordionSection title="Platform" items={allPlatformItems} onNavigate={onClose} />
        <MobileAccordionSection title="Developers" items={allDevelopersItems} onNavigate={onClose} />
        <MobileAccordionSection title="Company" items={allCompanyItems} onNavigate={onClose} />
      </Accordion>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          if (navEmail) navSignupMutation.mutate({ email: navEmail });
        }}
        className="flex flex-col gap-2"
        data-testid="form-mobile-signup"
      >
        <Input
          type="email"
          placeholder="Enter your email for early access"
          value={navEmail}
          onChange={(e) => setNavEmail(e.target.value)}
          required
          data-testid="input-mobile-email"
          aria-label="Email address for early access"
        />
        <Button
          type="submit"
          variant="outline"
          className="w-full border-border text-foreground hover:bg-muted/50"
          disabled={navSignupMutation.isPending}
          data-testid="button-mobile-signup"
        >
          {navSignupMutation.isPending ? "Joining..." : "Join the Waitlist"}
          <ArrowRight className="w-4 h-4 ml-2" />
        </Button>
      </form>
    </div>
  );
}

export function MarketingTopNav() {
  const isMobile = useIsMobile();
  const [mobileOpen, setMobileOpen] = useState(false);
  const [location, setLocation] = useLocation();
  const [navEmail, setNavEmail] = useState("");
  const { toast } = useToast();

  const navSignupMutation = useMutation({
    mutationFn: async (data: { email: string }) => {
      const res = await apiRequest("POST", "/api/developer-signup", data);
      return res.json();
    },
    onSuccess: (data: { message: string }) => {
      toast({ title: "You're in!", description: data.message });
      setNavEmail("");
    },
    onError: () => {
      toast({ title: "Something went wrong", description: "Please try again.", variant: "destructive" });
    },
  });

  const handleAnchorScroll = useCallback((id: string) => {
    if (location !== "/") {
      setLocation("/");
      setTimeout(() => {
        const el = document.getElementById(id);
        if (el) el.scrollIntoView({ behavior: "smooth" });
      }, 250);
    } else {
      setTimeout(() => {
        const el = document.getElementById(id);
        if (el) el.scrollIntoView({ behavior: "smooth" });
      }, 80);
    }
  }, [location, setLocation]);

  const [theme, setTheme] = useState<"light" | "dark">(() => {
    if (typeof window !== "undefined") {
      return document.documentElement.classList.contains("dark") ? "dark" : "light";
    }
    return "light";
  });

  useEffect(() => {
    const root = document.documentElement;
    if (theme === "dark") {
      root.classList.add("dark");
    } else {
      root.classList.remove("dark");
    }
    localStorage.setItem("theme", theme);
  }, [theme]);

  useEffect(() => {
    const saved = localStorage.getItem("theme");
    if (saved === "dark" || saved === "light") {
      setTheme(saved);
    } else if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
      setTheme("dark");
    }
  }, []);

  const [menuOpen, setMenuOpen] = useState(false);
  const [waitlistOpen, setWaitlistOpen] = useState(false);

  const handleNavigate = useCallback((href: string) => {
    setLocation(href);
  }, [setLocation]);

  return (
    <NavigateContext.Provider value={handleNavigate}>
      <AnchorScrollContext.Provider value={handleAnchorScroll}>
        <a
          href="#main-content"
          className="sr-only focus:not-sr-only focus:absolute focus:z-[100] focus:top-2 focus:left-2 focus:bg-background focus:text-foreground focus:px-4 focus:py-2 focus:rounded-md focus:border"
          data-testid="link-skip-navigation"
        >
          Skip to main content
        </a>
        <header
          className="fixed top-0 z-[9999] w-full flex flex-col"
          style={{
            height: "125px",
            background: "linear-gradient(180deg, hsl(20,14%,8%) 0%, hsl(20,12%,6%) 100%)",
            boxShadow: [
              "inset 0 1px 0 rgba(255,255,255,0.07)",
              "inset 0 14px 48px rgba(0,0,0,0.6)",
              "inset 0 -14px 48px rgba(0,0,0,0.45)",
              "0 4px 28px rgba(0,0,0,0.65)",
              "0 1px 0 rgba(255,255,255,0.04)",
            ].join(", "),
          }}
          data-testid="marketing-top-nav"
        >
          <div className="flex-1 mx-auto flex max-w-7xl items-center gap-4 px-4 lg:px-6">
            {!isMobile && <DesktopNav onOpenChange={setMenuOpen} />}

            <div className="flex-1" />
            <div className="flex items-center gap-2">
              <Popover open={waitlistOpen} onOpenChange={setWaitlistOpen}>
                <PopoverTrigger asChild>
                  <Button
                    size="sm"
                    className="h-6 text-[9px] px-3"
                    style={{
                      background: "linear-gradient(180deg, hsl(20,10%,18%) 0%, hsl(20,12%,10%) 100%)",
                      border: "1px solid hsl(20,10%,22%)",
                      color: "#8A8578",
                      fontFamily: "'Segoe UI', -apple-system, sans-serif",
                      fontWeight: 600,
                      letterSpacing: "0.04em",
                      boxShadow: "inset 0 1px 0 rgba(255,255,255,0.08), 0 2px 4px rgba(0,0,0,0.4), 0 1px 0 rgba(255,255,255,0.03)",
                    }}
                    data-testid="button-nav-waitlist-trigger"
                  >
                    Join Waitlist
                  </Button>
                </PopoverTrigger>
                <PopoverContent
                  className="w-80 p-4"
                  style={{
                    background: "hsl(20,12%,7%)",
                    border: "1px solid hsl(20,10%,15%)",
                  }}
                  align="end"
                >
                  <form
                    onSubmit={(e) => {
                      e.preventDefault();
                      if (navEmail) {
                        navSignupMutation.mutate({ email: navEmail }, {
                          onSuccess: () => setWaitlistOpen(false),
                        });
                      }
                    }}
                    className="flex flex-col gap-3"
                    data-testid="form-nav-signup"
                  >
                    <p style={{ color: "#E4DFD5", fontSize: "13px", fontFamily: "'Segoe UI', -apple-system, sans-serif" }}>
                      Get early access to PlenumNET
                    </p>
                    <Input
                      type="email"
                      placeholder="Enter your email"
                      value={navEmail}
                      onChange={(e) => setNavEmail(e.target.value)}
                      className="h-9 text-sm"
                      style={{
                        background: "hsl(20,12%,9%)",
                        border: "1px solid hsl(20,10%,18%)",
                        color: "#E4DFD5",
                        fontFamily: "'Consolas', 'Menlo', monospace",
                      }}
                      required
                      data-testid="input-nav-email"
                      aria-label="Email address for early access"
                    />
                    <Button
                      type="submit"
                      className="h-9 w-full text-xs"
                      style={{
                        background: "#38BDF8",
                        color: "#090807",
                        fontFamily: "'Orbitron', sans-serif",
                        fontWeight: 700,
                        fontSize: "11px",
                        letterSpacing: "0.06em",
                      }}
                      disabled={navSignupMutation.isPending}
                      data-testid="button-nav-signup"
                    >
                      {navSignupMutation.isPending ? "Joining..." : "Submit"}
                    </Button>
                  </form>
                </PopoverContent>
              </Popover>

              <Button
                variant="ghost"
                size="icon"
                onClick={() => setTheme(theme === "light" ? "dark" : "light")}
                style={{ color: "#5A5548" }}
                className="hover:!text-[#E4DFD5]"
                aria-label={`Switch to ${theme === "light" ? "dark" : "light"} mode`}
                data-testid="button-theme-toggle"
              >
                {theme === "light" ? (
                  <Moon className="w-4 h-4" />
                ) : (
                  <Sun className="w-4 h-4" />
                )}
              </Button>

              {isMobile && (
                <Sheet open={mobileOpen} onOpenChange={setMobileOpen}>
                  <SheetTrigger asChild>
                    <Button
                      variant="ghost"
                      size="icon"
                      style={{ color: "#5A5548" }}
                      className="hover:!text-[#E4DFD5]"
                      aria-label="Open menu"
                      data-testid="button-mobile-menu"
                    >
                      <Menu className="w-5 h-5" />
                    </Button>
                  </SheetTrigger>
                  <SheetContent
                    side="right"
                    className="w-[300px] sm:w-[360px]"
                    style={{
                      background: "hsl(20,12%,6%)",
                      borderLeft: "1px solid hsl(20,10%,15%)",
                      color: "#E4DFD5",
                    }}
                  >
                    <SheetHeader>
                      <SheetTitle
                        className="flex items-center gap-2 uppercase"
                        style={{
                          fontFamily: "'Orbitron', sans-serif",
                          fontWeight: 800,
                          letterSpacing: "0.12em",
                          color: "#E4DFD5",
                        }}
                      >
                        <img src={plenumLogo} alt="PlenumNET" className="w-5 h-5" />
                        PlenumNET
                      </SheetTitle>
                    </SheetHeader>
                    <MobileNav onClose={() => setMobileOpen(false)} navEmail={navEmail} setNavEmail={setNavEmail} navSignupMutation={navSignupMutation} />
                  </SheetContent>
                </Sheet>
              )}
            </div>
          </div>
          <div className="pb-8">
          <svg
            className="w-full"
            height="16"
            viewBox="0 0 1000 16"
            preserveAspectRatio="none"
            fill="none"
            style={{ color: "#38BDF8" }}
          >
            <line x1="0" y1="8" x2="1000" y2="8"
              stroke="currentColor" strokeWidth="1" opacity="0.3" />
            <line x1="80" y1="1" x2="80" y2="15"
              stroke="currentColor" strokeWidth="1.5" opacity="0.6" />
            <line x1="92" y1="1" x2="92" y2="15"
              stroke="currentColor" strokeWidth="1.5" opacity="0.6" />
            <line x1="420" y1="3" x2="420" y2="13"
              stroke="currentColor" strokeWidth="1" opacity="0.35" />
            <line x1="780" y1="1" x2="780" y2="15"
              stroke="currentColor" strokeWidth="1.5" opacity="0.5" />
          </svg>
          </div>
        </header>
      </AnchorScrollContext.Provider>
    </NavigateContext.Provider>
  );
}
