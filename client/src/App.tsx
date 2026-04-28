/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending - All Rights Reserved
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
 *
 * SINGLE LAYOUT: Every page renders inside
 *   MarketingTopNav -> content -> MarketingFooter
 *
 * The sidebar/DashboardLayout has been removed.
 * Admin pages use an in-page AdminNav tab bar.
 */

import { Switch, Route, useLocation } from "wouter";
import { queryClient } from "./lib/queryClient";
import { QueryClientProvider } from "@tanstack/react-query";
import { Toaster } from "@/components/ui/toaster";
import { TooltipProvider } from "@/components/ui/tooltip";
import { MarketingTopNav } from "@/components/marketing-top-nav";
import { MarketingFooter } from "@/components/marketing-footer";
import { Suspense, lazy } from "react";
import { ErrorBoundary } from "@/components/error-boundary";
import { CookieConsent } from "@/components/cookie-consent";
import { usePageTitle } from "@/hooks/use-page-title";
const Landing = lazyRetry(() => import("@/pages/landing"));
import LegalPage from "@/pages/legal";
import NotFound from "@/pages/not-found";

function isRetryableChunkError(err: unknown): boolean {
  if (!(err instanceof Error)) return false;
  const msg = err.message;
  return msg.includes("Loading chunk") ||
    msg.includes("dynamically imported module") ||
    msg.includes("Importing a module script failed") ||
    msg.includes("Failed to fetch") ||
    msg.includes("ChunkLoadError");
}

function lazyRetry<T extends { default: React.ComponentType<any> }>(
  factory: () => Promise<T>,
  maxRetries = 2,
): React.LazyExoticComponent<T["default"]> {
  return lazy(async () => {
    for (let attempt = 0; attempt <= maxRetries; attempt++) {
      try {
        return await factory();
      } catch (err) {
        if (attempt < maxRetries && isRetryableChunkError(err)) {
          await new Promise((r) => setTimeout(r, 800 * (attempt + 1)));
          continue;
        }
        throw err;
      }
    }
    return factory();
  });
}

const TernaryDB = lazyRetry(() => import("@/pages/ternarydb"));
const Whitepaper = lazyRetry(() => import("@/pages/whitepaper"));
const GitHubManager = lazyRetry(() => import("@/pages/github-manager"));
const APIDemo = lazyRetry(() => import("@/pages/api-demo"));
const KongKonnect = lazyRetry(() => import("@/pages/kong-konnect"));
const Admin = lazyRetry(() => import("@/pages/admin"));
const Docs = lazyRetry(() => import("@/pages/docs"));
const CalendarPage = lazyRetry(() => import("@/pages/calendar"));
const CompliancePage = lazyRetry(() => import("@/pages/compliance"));
const HPTPDemo = lazyRetry(() => import("@/pages/hptp-demo"));
const HModalDemo = lazyRetry(() => import("@/pages/hmodal-demo"));
const ThirteenMoonPage = lazyRetry(() => import("@/pages/thirteen-moon"));
const CompressionPage = lazyRetry(() => import("@/pages/compression"));
const Tribonacci28DS = lazyRetry(() => import("@/pages/tribonacci-28ds"));
const About = lazyRetry(() => import("@/pages/about"));
const Contact = lazyRetry(() => import("@/pages/contact"));
const Distribution = lazyRetry(() => import("@/pages/distribution"));
const ISASecurityPaper = lazyRetry(() => import("@/pages/isa-security-paper"));
const VMDemo = lazyRetry(() => import("@/pages/vm-demo"));
const AgentArray = lazyRetry(() => import("@/pages/agent-array"));
const QuantumSim = lazyRetry(() => import("@/pages/quantum-sim"));
const ApiKeysPage = lazyRetry(() => import("@/pages/api-keys"));
const FPGABenchmarks = lazyRetry(() => import("@/pages/fpga-benchmarks"));
const TsaPage = lazyRetry(() => import("@/pages/tsa"));
const TerminalPage = lazyRetry(() => import("@/pages/terminal"));
const Widget = lazyRetry(() => import("@/pages/widget"));
import { LauncherProvider } from "@/components/LauncherPanel";

function LoadingSpinner() {
  return (
    <div className="flex items-center justify-center min-h-[60vh] w-full">
      <div className="text-center">
        <div className="inline-flex items-center justify-center w-12 h-12 rounded-full border-2 border-primary/20 border-t-primary animate-spin mb-4" />
        <p className="text-sm text-muted-foreground">Loading...</p>
      </div>
    </div>
  );
}

function AppRouter() {
  usePageTitle();
  return (
    <div className="min-h-screen bg-background text-foreground flex flex-col">
      <LauncherProvider>
      <MarketingTopNav />
      <main id="main-content" className="flex-1 pt-[148px]">
        <Suspense fallback={<LoadingSpinner />}>
          <Switch>
            {/* Marketing and Company */}
            <Route path="/" component={Landing} />
            <Route path="/about" component={About} />
            <Route path="/contact" component={Contact} />
            <Route path="/whitepaper" component={Whitepaper} />
            <Route path="/distribution" component={Distribution} />

            {/* Legal */}
            <Route path="/terms" component={LegalPage} />
            <Route path="/privacy" component={LegalPage} />
            <Route path="/security" component={LegalPage} />
            <Route path="/aup" component={LegalPage} />

            {/* Interactive Labs and Tools */}
            <Route path="/ternarydb" component={TernaryDB} />
            <Route path="/api-demo" component={APIDemo} />
            <Route path="/hptp" component={HPTPDemo} />
            <Route path="/hmodal" component={HModalDemo} />
            <Route path="/hmodal-demo" component={HModalDemo} />
            <Route path="/compression" component={CompressionPage} />
            <Route path="/vm-demo" component={VMDemo} />
            <Route path="/quantum-sim" component={QuantumSim} />
            <Route path="/agent-array" component={AgentArray} />
            <Route path="/terminal" component={TerminalPage} />

            {/* Calendars */}
            <Route path="/calendar" component={CalendarPage} />
            <Route path="/13-moon" component={ThirteenMoonPage} />

            {/* Reference and Documentation */}
            <Route path="/docs" component={Docs} />
            <Route path="/compliance" component={CompliancePage} />
            <Route path="/tsa" component={TsaPage} />
            <Route path="/isa-security" component={ISASecurityPaper} />
            <Route path="/tribonacci-28ds" component={Tribonacci28DS} />
            <Route path="/fpga-benchmarks" component={FPGABenchmarks} />

            {/* Admin (auth-gated at page level) */}
            <Route path="/admin" component={Admin} />
            <Route path="/github" component={GitHubManager} />
            <Route path="/kong-konnect" component={KongKonnect} />
            <Route path="/api-keys" component={ApiKeysPage} />

            {/* 404 */}
            <Route component={NotFound} />
          </Switch>
        </Suspense>
      </main>
      <MarketingFooter />
      </LauncherProvider>
    </div>
  );
}

function App() {
  const [location] = useLocation();

  if (location === "/widget") {
    return (
      <ErrorBoundary>
        <QueryClientProvider client={queryClient}>
          <Suspense fallback={<LoadingSpinner />}>
            <Widget />
          </Suspense>
        </QueryClientProvider>
      </ErrorBoundary>
    );
  }

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
