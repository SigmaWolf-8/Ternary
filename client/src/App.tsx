import { Switch, Route } from "wouter";
import { queryClient } from "./lib/queryClient";
import { QueryClientProvider } from "@tanstack/react-query";
import { Toaster } from "@/components/ui/toaster";
import { TooltipProvider } from "@/components/ui/tooltip";
import { SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar";
import { AppSidebar } from "@/components/app-sidebar";
import { useState, useCallback, useRef, useEffect } from "react";
import Landing from "@/pages/landing";
import TernaryDB from "@/pages/ternarydb";
import Whitepaper from "@/pages/whitepaper";
import GitHubManager from "@/pages/github-manager";
import APIDemo from "@/pages/api-demo";
import KongKonnect from "@/pages/kong-konnect";
import Admin from "@/pages/admin";
import Docs from "@/pages/docs";
import CalendarPage from "@/pages/calendar";
import CompliancePage from "@/pages/compliance";
import NotFound from "@/pages/not-found";

function Router() {
  return (
    <Switch>
      <Route path="/" component={Landing} />
      <Route path="/ternarydb" component={TernaryDB} />
      <Route path="/whitepaper" component={Whitepaper} />
      <Route path="/github" component={GitHubManager} />
      <Route path="/api-demo" component={APIDemo} />
      <Route path="/kong-konnect" component={KongKonnect} />
      <Route path="/admin" component={Admin} />
      <Route path="/docs" component={Docs} />
      <Route path="/calendar" component={CalendarPage} />
      <Route path="/compliance" component={CompliancePage} />
      <Route component={NotFound} />
    </Switch>
  );
}

const MIN_SIDEBAR_WIDTH = 180;
const MAX_SIDEBAR_WIDTH = 400;
const DEFAULT_SIDEBAR_WIDTH = 220;
const ICON_WIDTH = "3rem";

function App() {
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
    <QueryClientProvider client={queryClient}>
      <TooltipProvider>
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
              </header>
              <main className="flex-1 overflow-auto">
                <Router />
              </main>
            </div>
          </div>
        </SidebarProvider>
        <Toaster />
      </TooltipProvider>
    </QueryClientProvider>
  );
}

export default App;
