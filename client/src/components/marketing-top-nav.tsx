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

import { useState, useEffect, useRef, useCallback } from "react";
import { Link, useLocation } from "wouter";
import { Menu, Sun, Moon, ExternalLink } from "lucide-react";
import plenumLogo from "@assets/grok-image-69a372f5-5c40-48be-b431-a4dbb4e92ff2_1771299513785.png";
import { useIsMobile } from "@/hooks/use-mobile";
import { Button } from "@/components/ui/button";
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
import { PLATFORM } from "@shared/constants";
import { createContext, useContext } from "react";

type AnchorScrollFn = (id: string) => void;
const AnchorScrollContext = createContext<AnchorScrollFn>(() => {});

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
    heading: "Core Technology",
    items: [
      { title: "Ternary Computing Engine", subtitle: "Native base-3 processing", href: "/#platform" },
      { title: "Virtual Machine", subtitle: `${PLATFORM.VM_OPCODES}-opcode ternary VM`, href: "/#architecture" },
      { title: "Binary-Ternary Gateway", subtitle: "Seamless interop layer", href: "/#architecture" },
    ],
  },
  {
    heading: "Infrastructure",
    items: [
      { title: "HPTP Timing Protocol", subtitle: "Femtosecond precision", href: "/hptp" },
      { title: "Post-Quantum Security", subtitle: "CNSA 2.0 aligned", href: "/compliance" },
      { title: "Torsion Network Stack", subtitle: "3-ary torus topology", href: "/#platform" },
    ],
  },
  {
    heading: "Data & Storage",
    items: [
      { title: "PlenumDB", subtitle: "Ternary-native database", href: "/ternarydb" },
      { title: "Compression Engine", subtitle: "Ternary encoding", href: "/compression" },
      { title: "Calendar API", subtitle: "Multi-calendar support", href: "/calendar" },
    ],
  },
];

const technologyColumns: NavColumn[] = [
  {
    heading: "Interactive Demos",
    items: [
      { title: "API Explorer", subtitle: "Try the REST API", href: "/api-demo" },
      { title: "PlenumDB Console", subtitle: "Query ternary data", href: "/ternarydb" },
      { title: "HPTP Timing Lab", subtitle: "Precision timing demo", href: "/hptp" },
      { title: "Compression Studio", subtitle: "Encode & decode", href: "/compression" },
    ],
  },
  {
    heading: "Calendar Systems",
    items: [
      { title: "Universal Calendar API", subtitle: "Cross-calendar conversion", href: "/calendar" },
      { title: "13-Moon Harmonic Calendar", subtitle: "Natural time system", href: "/13-moon" },
    ],
  },
  {
    heading: "Architecture",
    items: [
      { title: "Performance Comparison", subtitle: "Binary vs ternary", href: "/#performance" },
      { title: "5-Layer Stack", subtitle: "Full architecture view", href: "/#architecture" },
      { title: "Tribonacci 28DS", subtitle: "28-dimension symmetry", href: "/tribonacci-28ds" },
      { title: "FPGA Benchmarks", subtitle: "Yosys/Vivado synthesis", href: "/fpga-benchmarks" },
    ],
  },
];

const resourcesItems: NavLinkItem[] = [
  { title: "Whitepaper", href: "/whitepaper" },
  { title: "ISA Security Primitives", href: "/isa-security" },
  { title: "Ternary VM Demo", href: "/vm-demo" },
  { title: "Quantum Simulator", subtitle: "Qutrit FT / FIPS / QVQE", href: "/quantum-sim" },
  { title: "Documentation", href: "/docs" },
  { title: "CNSA 2.0 Compliance", href: "/compliance" },
  { title: "Module Distribution", href: "/distribution" },
  { title: "28D Agent Array", href: "/agent-array" },
];

const resourcesExternal: NavLinkItem[] = [
  { title: "GitHub Repository", href: "https://github.com/SigmaWolf-8/Ternary", external: true },
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

function MegaDropdownItem({
  item,
}: {
  item: NavLinkItem;
}) {
  const scrollToAnchor = useContext(AnchorScrollContext);
  const isAnchor = item.href.startsWith("/#");
  const anchorId = isAnchor ? item.href.slice(2) : "";
  const baseClass =
    "block select-none rounded-md p-3 leading-none no-underline outline-none transition-colors hover-elevate";

  const content = (
    <>
      <div className="text-sm font-medium leading-none">{item.title}</div>
      {item.subtitle && (
        <p className="line-clamp-2 text-xs leading-snug text-muted-foreground mt-1">
          {item.subtitle}
        </p>
      )}
    </>
  );

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

  return (
    <li>
      <NavigationMenuLink asChild>
        <Link
          href={item.href}
          className={baseClass}
          data-testid={`link-${item.title.toLowerCase().replace(/\s+/g, "-")}`}
        >
          {content}
        </Link>
      </NavigationMenuLink>
    </li>
  );
}

function MegaDropdown({ columns }: { columns: NavColumn[] }) {
  return (
    <div className="grid gap-3 p-4 md:w-[600px] lg:w-[700px] md:grid-cols-3">
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

  return (
    <li>
      <NavigationMenuLink asChild>
        <Link
          href={item.href}
          className={baseClass}
          data-testid={`link-${item.title.toLowerCase().replace(/\s+/g, "-")}`}
        >
          {item.title}
        </Link>
      </NavigationMenuLink>
    </li>
  );
}

function DesktopNav({ onOpenChange }: { onOpenChange?: (open: boolean) => void }) {
  return (
    <NavigationMenu
      className="hidden md:flex"
      delayDuration={100}
      skipDelayDuration={300}
      onValueChange={(val: string) => onOpenChange?.(val !== "")}
    >
      <NavigationMenuList>
        <NavigationMenuItem>
          <NavigationMenuTrigger data-testid="nav-trigger-platform">
            Platform
          </NavigationMenuTrigger>
          <NavigationMenuContent className="left-0">
            <MegaDropdown columns={platformColumns} />
          </NavigationMenuContent>
        </NavigationMenuItem>

        <NavigationMenuItem>
          <NavigationMenuTrigger data-testid="nav-trigger-technology">
            Technology
          </NavigationMenuTrigger>
          <NavigationMenuContent className="left-1/2 -translate-x-1/2">
            <MegaDropdown columns={technologyColumns} />
          </NavigationMenuContent>
        </NavigationMenuItem>

        <NavigationMenuItem>
          <NavigationMenuTrigger data-testid="nav-trigger-resources">
            Resources
          </NavigationMenuTrigger>
          <NavigationMenuContent className="left-0">
            <ul className="w-[220px] p-2">
              {resourcesItems.map((item) => (
                <StandardDropdownItem key={item.title} item={item} />
              ))}
              <li>
                <div className="my-1 mx-3 h-px bg-border" />
              </li>
              {resourcesExternal.map((item) => (
                <StandardDropdownItem key={item.title} item={item} />
              ))}
            </ul>
          </NavigationMenuContent>
        </NavigationMenuItem>

        <NavigationMenuItem>
          <NavigationMenuTrigger data-testid="nav-trigger-company">
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

function MobileNav({ onClose }: { onClose: () => void }) {
  const scrollToAnchor = useContext(AnchorScrollContext);
  const allPlatformItems = platformColumns.flatMap((c) => c.items);
  const allTechnologyItems = technologyColumns.flatMap((c) => c.items);
  const allResourcesItems = [...resourcesItems, ...resourcesExternal];
  const allCompanyItems = [...companyItems, ...companyLegalItems];

  return (
    <div className="flex flex-col gap-4 mt-4">
      <Accordion type="multiple" className="w-full">
        <MobileAccordionSection title="Platform" items={allPlatformItems} onNavigate={onClose} />
        <MobileAccordionSection title="Technology" items={allTechnologyItems} onNavigate={onClose} />
        <MobileAccordionSection title="Resources" items={allResourcesItems} onNavigate={onClose} />
        <MobileAccordionSection title="Company" items={allCompanyItems} onNavigate={onClose} />
      </Accordion>
      <a
        href="/#hero"
        className="w-full"
        onClick={(e) => {
          e.preventDefault();
          scrollToAnchor("hero");
          onClose();
        }}
      >
        <Button className="w-full" data-testid="mobile-button-cta">
          Get Early Access
        </Button>
      </a>
    </div>
  );
}

export function MarketingTopNav() {
  const isMobile = useIsMobile();
  const [mobileOpen, setMobileOpen] = useState(false);
  const [location, setLocation] = useLocation();

  const handleAnchorScroll = useCallback((id: string) => {
    if (location !== "/") {
      setLocation("/");
      setTimeout(() => {
        const el = document.getElementById(id);
        if (el) el.scrollIntoView({ behavior: "smooth" });
      }, 150);
    } else {
      const el = document.getElementById(id);
      if (el) el.scrollIntoView({ behavior: "smooth" });
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

  const lastScrollY = useRef(0);
  const [visible, setVisible] = useState(true);
  const [menuOpen, setMenuOpen] = useState(false);

  const handleScroll = useCallback(() => {
    if (menuOpen || mobileOpen) return;
    const currentY = window.scrollY;
    const delta = currentY - lastScrollY.current;
    if (currentY < 120) {
      setVisible(true);
    } else if (delta > 30) {
      setVisible(false);
    } else if (delta < -10) {
      setVisible(true);
    }
    lastScrollY.current = currentY;
  }, [menuOpen, mobileOpen]);

  useEffect(() => {
    window.addEventListener("scroll", handleScroll, { passive: true });
    return () => window.removeEventListener("scroll", handleScroll);
  }, [handleScroll]);

  return (
    <AnchorScrollContext.Provider value={handleAnchorScroll}>
      <a
        href="#main-content"
        className="sr-only focus:not-sr-only focus:absolute focus:z-[100] focus:top-2 focus:left-2 focus:bg-background focus:text-foreground focus:px-4 focus:py-2 focus:rounded-md focus:border"
        data-testid="link-skip-navigation"
      >
        Skip to main content
      </a>
      <header
        className={cn(
          "sticky top-0 z-[9999] w-full border-b bg-background/95 backdrop-blur-sm transition-transform duration-300",
          !visible && "-translate-y-full"
        )}
        data-testid="marketing-top-nav"
      >
        <div className="mx-auto flex h-14 max-w-7xl items-center gap-4 px-4 lg:px-6">
          <Link
            href="/"
            className="flex items-center gap-2 font-semibold text-foreground mr-2"
            data-testid="link-logo"
          >
            <img src={plenumLogo} alt="PlenumNET" className="w-4 h-4" />
            <span className="text-base">PlenumNET</span>
          </Link>

          {!isMobile && <DesktopNav onOpenChange={setMenuOpen} />}

          <div className="ml-auto flex items-center gap-2">
            <Button
              variant="ghost"
              size="icon"
              onClick={() => setTheme(theme === "light" ? "dark" : "light")}
              aria-label={`Switch to ${theme === "light" ? "dark" : "light"} mode`}
              data-testid="button-theme-toggle"
            >
              {theme === "light" ? (
                <Moon className="w-4 h-4" />
              ) : (
                <Sun className="w-4 h-4" />
              )}
            </Button>

            {!isMobile && (
              <a
                href="/#hero"
                onClick={(e) => {
                  e.preventDefault();
                  handleAnchorScroll("hero");
                }}
              >
                <Button data-testid="button-cta">Get Early Access</Button>
              </a>
            )}

            {isMobile && (
              <Sheet open={mobileOpen} onOpenChange={setMobileOpen}>
                <SheetTrigger asChild>
                  <Button
                    variant="ghost"
                    size="icon"
                    aria-label="Open menu"
                    data-testid="button-mobile-menu"
                  >
                    <Menu className="w-5 h-5" />
                  </Button>
                </SheetTrigger>
                <SheetContent side="right" className="w-[300px] sm:w-[360px]">
                  <SheetHeader>
                    <SheetTitle className="flex items-center gap-2">
                      <img src={plenumLogo} alt="PlenumNET" className="w-5 h-5" />
                      PlenumNET
                    </SheetTitle>
                  </SheetHeader>
                  <MobileNav onClose={() => setMobileOpen(false)} />
                </SheetContent>
              </Sheet>
            )}
          </div>
        </div>
      </header>
    </AnchorScrollContext.Provider>
  );
}
