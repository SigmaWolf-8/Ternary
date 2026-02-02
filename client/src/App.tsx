import { Switch, Route } from "wouter";
import { queryClient } from "./lib/queryClient";
import { QueryClientProvider } from "@tanstack/react-query";
import { Toaster } from "@/components/ui/toaster";
import { TooltipProvider } from "@/components/ui/tooltip";
import Landing from "@/pages/landing";
import TernaryDB from "@/pages/ternarydb";
import Whitepaper from "@/pages/whitepaper";
import GitHubManager from "@/pages/github-manager";
import APIDemo from "@/pages/api-demo";
import NotFound from "@/pages/not-found";

function Router() {
  return (
    <Switch>
      <Route path="/" component={Landing} />
      <Route path="/ternarydb" component={TernaryDB} />
      <Route path="/whitepaper" component={Whitepaper} />
      <Route path="/github" component={GitHubManager} />
      <Route path="/api-demo" component={APIDemo} />
      <Route component={NotFound} />
    </Switch>
  );
}

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <TooltipProvider>
        <Toaster />
        <Router />
      </TooltipProvider>
    </QueryClientProvider>
  );
}

export default App;
