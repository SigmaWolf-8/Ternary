/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL
 * All Rights Reserved.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * Unauthorized copying, modification, distribution, or use of this file,
 * via any medium, is strictly prohibited without the prior written
 * permission of Capomastro Holdings Ltd.
 *
 * See LICENSE in the repository root for full terms.
 */

import { Switch, Route } from "wouter";
import { queryClient } from "./lib/queryClient";
import { QueryClientProvider } from "@tanstack/react-query";
import { Toaster } from "@/components/ui/toaster";
import { TooltipProvider } from "@/components/ui/tooltip";
import { SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar";
import { AppSidebar } from "@/components/app-sidebar";
import { MarketingTopNav } from "@/components/marketing-top-nav";
import { MarketingFooter } from "@/components/marketing-footer";
import { useState, useCallback, useRef, useEffect, Suspense, lazy } from "react";
import { Sun, Moon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ErrorBoundary } from "@/components/error-boundary";
import { CookieConsent } from "@/components/cookie-consent";
import { usePageTitle } from "@/hooks/use-page-title";
import Landing from "@/pages/landing";
import LegalPage from "@/pages/legal";
import NotFound from "@/pages/not-found";

const TernaryDB = lazy(() => import("@/pages/ternarydb"));
const Whitepaper = lazy(() => import("@/pages/whitepaper"));
const GitHubManager = lazy(() => import("@/pages/github-manager"));
const APIDemo = lazy(() => import("@/pages/api-demo"));
const KongKonnect = lazy(() => import("@/pages/kong-konnect"));
const Admin = lazy(() => import("@/pages/admin"));
const Docs = lazy(() => import("@/pages/docs"));
const CalendarPage = lazy(() => import("@/pages/calendar"));
const CompliancePage = lazy(() => import("@/pages/compliance"));
const HPTPDemo = lazy(() => import("@/pages/hptp-demo"));
const ThirteenMoonPage = lazy(() => import("@/pages/thirteen-moon"));
const CompressionPage = lazy(() => import("@/pages/compression"));
const Tribonacci28DS = lazy(() => import("@/pages/tribonacci-28ds"));
const About = lazy(() => import("@/pages/about"));
const Contact = lazy(() => import("@/pages/contact"));
const Distribution = lazy(() => import("@/pages/distribution"));
const ISASecurityPaper = lazy(() => import("@/pages/isa-security-paper"));
const VMDemo = lazy(() => import("@/pages/vm-demo"));
const AgentArray = lazy(() => import("@/pages/agent-array"));

function LoadingSpinner() {
  return (
    <div className="flex items-center justify-center min-h-screen w-full">
      <div className="text-center">
        <div className="inline-flex items-center justify-center w-12 h-12 rounded-full border-2 border-primary/20 border-t-primary animate-spin mb-4" />
        <p className="text-sm text-muted-foreground">Loading...</p>
      </div>
    </div>
  );
}

function MarketingLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="min-h-screen bg-background text-foreground flex flex-col">
      <MarketingTopNav />
      <main id="main-content" className="flex-1">
        {children}
      </main>
      <MarketingFooter />
    </div>
  );
}

function ThemeToggle() {
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

  return (
    <Button
      variant="ghost"
      size="icon"
      onClick={() => setTheme(theme === "light" ? "dark" : "light")}
      aria-label={`Switch to ${theme === "light" ? "dark" : "light"} mode`}
      data-testid="button-theme-toggle"
    >
      {theme === "light" ? <Moon className="w-4 h-4" /> : <Sun className="w-4 h-4" />}
    </Button>
  );
}

const MIN_SIDEBAR_WIDTH = 180;
const MAX_SIDEBAR_WIDTH = 400;
const DEFAULT_SIDEBAR_WIDTH = 220;
const ICON_WIDTH = "3rem";

function DashboardLayout({ children }: { children: React.ReactNode }) {
  const [sidebarWidth, setSidebarWidth] = useState(DEFAULT_SIDEBAR_WIDTH);
  const isResizing = useRef(false);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    isResizing.current = true;
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";

    const handleMouseMove = (e: MouseEvent) => {
      if (!isResizing.current) return;
      const newWidth = Math.min(MAX_SIDEBAR_WIDTH, Math.max(MIN_SIDEBAR_WIDTH, e.clientX));
      setSidebarWidth(newWidth);
    };

    const handleMouseUp = () => {
      isResizing.current = false;
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
    };

    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);
  }, []);

  const sidebarStyle = {
    "--sidebar-width": `${sidebarWidth}px`,
    "--sidebar-width-icon": ICON_WIDTH,
  } as React.CSSProperties;

  return (
    <SidebarProvider defaultOpen={false} style={sidebarStyle}>
      <div className="flex min-h-screen w-full">
        <div className="relative flex">
          <AppSidebar />
          <div
            className="absolute top-0 right-0 w-1 h-full cursor-col-resize z-50 hover:bg-primary/20 active:bg-primary/30 transition-colors"
            onMouseDown={handleMouseDown}
            data-testid="sidebar-resize-handle"
          />
        </div>
        <div className="flex flex-col flex-1 min-w-0">
          <header className="sticky top-0 z-50 flex items-center gap-2 border-b bg-background/95 backdrop-blur-sm px-3 h-12">
            <SidebarTrigger data-testid="button-sidebar-toggle" />
            <span className="text-xs font-medium text-muted-foreground">PlenumNET</span>
            <div className="ml-auto">
              <ThemeToggle />
            </div>
          </header>
          <main id="main-content" className="flex-1 overflow-auto">
            {children}
          </main>
        </div>
      </div>
    </SidebarProvider>
  );
}

function MarketingRouter() {
  return (
    <MarketingLayout>
      <Suspense fallback={<LoadingSpinner />}>
        <Switch>
          <Route path="/" component={Landing} />
          <Route path="/about" component={About} />
          <Route path="/contact" component={Contact} />
          <Route path="/whitepaper" component={Whitepaper} />
          <Route path="/terms" component={LegalPage} />
          <Route path="/privacy" component={LegalPage} />
          <Route path="/security" component={LegalPage} />
          <Route path="/aup" component={LegalPage} />
          <Route path="/tribonacci-28ds" component={Tribonacci28DS} />
          <Route path="/distribution" component={Distribution} />
          <Route path="/isa-security" component={ISASecurityPaper} />
          <Route path="/vm-demo" component={VMDemo} />
          <Route path="/agent-array" component={AgentArray} />
        </Switch>
      </Suspense>
    </MarketingLayout>
  );
}

function DashboardRouter() {
  return (
    <DashboardLayout>
      <Suspense fallback={<LoadingSpinner />}>
        <Switch>
          <Route path="/ternarydb" component={TernaryDB} />
          <Route path="/api-demo" component={APIDemo} />
          <Route path="/hptp" component={HPTPDemo} />
          <Route path="/compression" component={CompressionPage} />
          <Route path="/calendar" component={CalendarPage} />
          <Route path="/13-moon" component={ThirteenMoonPage} />
          <Route path="/docs" component={Docs} />
          <Route path="/compliance" component={CompliancePage} />
          <Route path="/admin" component={Admin} />
          <Route path="/github" component={GitHubManager} />
          <Route path="/kong-konnect" component={KongKonnect} />
        </Switch>
      </Suspense>
    </DashboardLayout>
  );
}

const marketingPaths = ["/", "/about", "/contact", "/whitepaper", "/terms", "/privacy", "/security", "/aup", "/tribonacci-28ds", "/distribution", "/isa-security", "/vm-demo", "/agent-array"];
const dashboardPaths = ["/ternarydb", "/api-demo", "/hptp", "/compression", "/calendar", "/13-moon", "/docs", "/compliance", "/admin", "/github", "/kong-konnect"];

function AppRouter() {
  usePageTitle();
  return (
    <Switch>
      {marketingPaths.map((path) => (
        <Route key={path} path={path}>
          <MarketingRouter />
        </Route>
      ))}
      {dashboardPaths.map((path) => (
        <Route key={path} path={path}>
          <DashboardRouter />
        </Route>
      ))}
      <Route>
        <MarketingLayout>
          <NotFound />
        </MarketingLayout>
      </Route>
    </Switch>
  );
}

function App() {
  return (
    <ErrorBoundary>
      <QueryClientProvider client={queryClient}>
        <TooltipProvider>
          <AppRouter />
          <CookieConsent />
          <Toaster />
        </TooltipProvider>
      </QueryClientProvider>
    </ErrorBoundary>
  );
}

export default App;
