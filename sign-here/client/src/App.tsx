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
import { Switch, Route, useLocation } from "wouter";
import { queryClient } from "./lib/queryClient";
import { QueryClientProvider } from "@tanstack/react-query";
import { Toaster } from "@/components/ui/toaster";
import { TooltipProvider } from "@/components/ui/tooltip";
import { SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar";
import { AppSidebar } from "@/components/app-sidebar";
import { ThemeProvider } from "@/components/theme-provider";
import { ThemeToggle } from "@/components/theme-toggle";
import { ZoomProvider, ZoomControl } from "@/components/zoom-control";
import { DashboardFilterProvider } from "@/lib/dashboard-context";
import { InstallButton } from "@/components/install-button";
import { OnboardingTour } from "@/components/onboarding-tour";
import NotFound from "@/pages/not-found";
import Dashboard from "@/pages/dashboard";
import EnvelopeNew from "@/pages/envelope-new";
import EnvelopeDetail from "@/pages/envelope-detail";
import EnvelopeEditor from "@/pages/envelope-editor";
import Sign from "@/pages/sign";
import SharePage from "@/pages/share";
import SettingsPage from "@/pages/settings";
import AboutPage from "@/pages/about";
import AdminPage from "@/pages/admin";
import CertificatePage from "@/pages/certificate";
import TemplatesPage from "@/pages/templates";
import TemplateEditorPage from "@/pages/template-editor";
import WbsTagsPage from "@/pages/wbs-tags";
import WbsTaggingPage from "@/pages/wbs-tagging";

function AppLayout({ children }: { children: React.ReactNode }) {
  const style = {
    "--sidebar-width": "10rem",
    "--sidebar-width-icon": "3rem",
  };

  return (
    <DashboardFilterProvider>
      <SidebarProvider style={style as React.CSSProperties}>
        <div className="flex h-screen w-full">
          <AppSidebar />
          <div className="flex flex-col flex-1 min-w-0">
            <header className="flex items-center justify-between gap-2 px-3 py-1.5 border-b shrink-0 bg-background sticky top-0 z-50">
              <SidebarTrigger data-testid="button-sidebar-toggle" />
              <div className="flex items-center gap-1">
                <span className="text-[9px] text-muted-foreground tracking-wider uppercase mr-2 hidden sm:inline" data-testid="text-version">
                  v1.1.2 | PlenumNET v2.1
                </span>
                <ZoomControl />
                <div className="w-px h-4 bg-border mx-1" />
                <ThemeToggle />
              </div>
            </header>
            <main className="flex-1 overflow-hidden flex flex-col">
              {children}
            </main>
          </div>
        </div>
      </SidebarProvider>
    </DashboardFilterProvider>
  );
}

function MainRouter() {
  return (
    <AppLayout>
      <Switch>
        <Route path="/" component={Dashboard} />
        <Route path="/new" component={EnvelopeNew} />
        <Route path="/envelope/:id" component={EnvelopeDetail} />
        <Route path="/envelope/:id/edit" component={EnvelopeEditor} />
        <Route path="/envelope/:id/certificate" component={CertificatePage} />
        <Route path="/templates" component={TemplatesPage} />
        <Route path="/templates/:id/edit" component={TemplateEditorPage} />
        <Route path="/wbs-tags" component={WbsTagsPage} />
        <Route path="/wbs-tagging" component={WbsTaggingPage} />
        <Route path="/about" component={AboutPage} />
        <Route path="/admin" component={AdminPage} />
        <Route path="/settings" component={SettingsPage} />
        <Route component={NotFound} />
      </Switch>
    </AppLayout>
  );
}

function AppRoutes() {
  const [location] = useLocation();
  const isSignPage = location.startsWith("/sign/");
  const isSharePage = location.startsWith("/share/");

  if (isSignPage) {
    return (
      <Switch>
        <Route path="/sign/:envelopeId/:recipientId" component={Sign} />
      </Switch>
    );
  }

  if (isSharePage) {
    return (
      <Switch>
        <Route path="/share/:id" component={SharePage} />
      </Switch>
    );
  }

  return <MainRouter />;
}

function App() {
  return (
    <ThemeProvider>
      <ZoomProvider>
        <QueryClientProvider client={queryClient}>
          <TooltipProvider>
            <AppRoutes />
            <InstallButton />
            <OnboardingTour />
            <Toaster />
          </TooltipProvider>
        </QueryClientProvider>
      </ZoomProvider>
    </ThemeProvider>
  );
}

export default App;
