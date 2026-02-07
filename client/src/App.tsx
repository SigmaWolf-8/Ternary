import { Switch, Route } from "wouter";
import { queryClient } from "./lib/queryClient";
import { QueryClientProvider } from "@tanstack/react-query";
import { Toaster } from "@/components/ui/toaster";
import { TooltipProvider } from "@/components/ui/tooltip";
import { SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar";
import { AppSidebar } from "@/components/app-sidebar";
import Landing from "@/pages/landing";
import TernaryDB from "@/pages/ternarydb";
import Whitepaper from "@/pages/whitepaper";
import GitHubManager from "@/pages/github-manager";
import APIDemo from "@/pages/api-demo";
import KongKonnect from "@/pages/kong-konnect";
import Admin from "@/pages/admin";
import Docs from "@/pages/docs";
import CalendarPage from "@/pages/calendar";
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
      <Route component={NotFound} />
    </Switch>
  );
}

const sidebarStyle = {
  "--sidebar-width": "16rem",
  "--sidebar-width-icon": "3rem",
} as React.CSSProperties;

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <TooltipProvider>
        <SidebarProvider style={sidebarStyle}>
          <div className="flex min-h-screen w-full">
            <AppSidebar />
            <div className="flex flex-col flex-1 min-w-0">
              <header className="sticky top-0 z-50 flex items-center gap-2 border-b bg-background/95 backdrop-blur-sm px-3 h-12">
                <SidebarTrigger data-testid="button-sidebar-toggle" />
                <span className="text-sm font-medium text-muted-foreground">PlenumNET</span>
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
