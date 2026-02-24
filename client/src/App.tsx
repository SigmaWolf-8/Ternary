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

import { Switch, Route } from "wouter";
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
const QuantumSim = lazy(() => import("@/pages/quantum-sim"));
const ApiKeysPage = lazy(() => import("@/pages/api-keys"));
const FPGABenchmarks = lazy(() => import("@/pages/fpga-benchmarks"));
const TsaPage = lazy(() => import("@/pages/tsa"));

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
      <MarketingTopNav />
      <main id="main-content" className="flex-1">
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
            <Route path="/compression" component={CompressionPage} />
            <Route path="/vm-demo" component={VMDemo} />
            <Route path="/quantum-sim" component={QuantumSim} />
            <Route path="/agent-array" component={AgentArray} />

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
    </div>
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
