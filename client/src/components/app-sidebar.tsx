import { useLocation, Link } from "wouter";
import { useAuth } from "@/hooks/use-auth";
import { useQuery } from "@tanstack/react-query";
import { useCallback } from "react";
import {
  Box,
  Layers,
  Network,
  Cpu,
  Gauge,
  Calendar,
  Globe,
  Terminal,
  FileText,
  BookOpen,
  ShieldCheck,
  Github,
  ExternalLink,
  LogIn,
  LogOut,
  User,
  Shield,
  ChevronRight,
} from "lucide-react";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
  SidebarSeparator,
} from "@/components/ui/sidebar";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";

export function AppSidebar() {
  const [location] = useLocation();
  const { user, isAuthenticated, isLoading } = useAuth();

  const { data: adminStatus } = useQuery<{ isAdmin: boolean }>({
    queryKey: ["/api/user/admin-status"],
    enabled: isAuthenticated,
  });

  const isLanding = location === "/";
  const [, navigate] = useLocation();

  const scrollToSection = useCallback((sectionId: string) => {
    if (isLanding) {
      const el = document.getElementById(sectionId);
      if (el) el.scrollIntoView({ behavior: "smooth", block: "start" });
    } else {
      navigate("/");
      setTimeout(() => {
        const el = document.getElementById(sectionId);
        if (el) el.scrollIntoView({ behavior: "smooth", block: "start" });
      }, 300);
    }
  }, [isLanding, navigate]);

  return (
    <Sidebar collapsible="icon">
      <SidebarHeader>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton asChild size="lg" tooltip="PlenumNET">
              <Link href="/" data-testid="link-sidebar-logo">
                <Box className="w-4 h-4 text-primary" />
                <span className="font-semibold text-sm">PlenumNET</span>
              </Link>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarHeader>

      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel className="text-[10px] uppercase tracking-wider">Navigation</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              <Collapsible defaultOpen={false} className="group/collapsible">
                <SidebarMenuItem>
                  <CollapsibleTrigger asChild>
                    <SidebarMenuButton tooltip="Platform" data-testid="button-sidebar-platform" className="text-xs">
                      <Layers className="w-3.5 h-3.5" />
                      <span>Platform</span>
                      <ChevronRight className="ml-auto w-3.5 h-3.5 transition-transform duration-200 group-data-[state=open]/collapsible:rotate-90" />
                    </SidebarMenuButton>
                  </CollapsibleTrigger>
                  <CollapsibleContent>
                    <SidebarMenuSub>
                      <SidebarMenuSubItem>
                        <SidebarMenuSubButton onClick={() => scrollToSection("architecture")} className="cursor-pointer text-xs" data-testid="link-sidebar-architecture">
                          <Cpu className="w-3.5 h-3.5" />
                          <span>Architecture</span>
                        </SidebarMenuSubButton>
                      </SidebarMenuSubItem>
                      <SidebarMenuSubItem>
                        <SidebarMenuSubButton onClick={() => scrollToSection("components")} className="cursor-pointer text-xs" data-testid="link-sidebar-components">
                          <Network className="w-3.5 h-3.5" />
                          <span>Components</span>
                        </SidebarMenuSubButton>
                      </SidebarMenuSubItem>
                      <SidebarMenuSubItem>
                        <SidebarMenuSubButton onClick={() => scrollToSection("performance")} className="cursor-pointer text-xs" data-testid="link-sidebar-performance">
                          <Gauge className="w-3.5 h-3.5" />
                          <span>Performance</span>
                        </SidebarMenuSubButton>
                      </SidebarMenuSubItem>
                      <SidebarMenuSubItem>
                        <SidebarMenuSubButton asChild isActive={location === "/calendar"} className="text-xs" data-testid="link-sidebar-calendar-sub">
                          <Link href="/calendar">
                            <Calendar className="w-3.5 h-3.5" />
                            <span>Calendar</span>
                          </Link>
                        </SidebarMenuSubButton>
                      </SidebarMenuSubItem>
                    </SidebarMenuSub>
                  </CollapsibleContent>
                </SidebarMenuItem>
              </Collapsible>

              <SidebarMenuItem>
                <SidebarMenuButton asChild isActive={location === "/calendar"} tooltip="Calendar API" data-testid="link-sidebar-calendar" className="text-xs">
                  <Link href="/calendar">
                    <Globe className="w-3.5 h-3.5" />
                    <span>Calendar API</span>
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>

              <SidebarMenuItem>
                <SidebarMenuButton asChild isActive={location === "/api-demo"} tooltip="API Demo" data-testid="link-sidebar-api-demo" className="text-xs">
                  <Link href="/api-demo">
                    <Terminal className="w-3.5 h-3.5" />
                    <span>API Demo</span>
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>

              <SidebarMenuItem>
                <SidebarMenuButton asChild isActive={location === "/whitepaper"} tooltip="Whitepaper" data-testid="link-sidebar-whitepaper" className="text-xs">
                  <Link href="/whitepaper">
                    <FileText className="w-3.5 h-3.5" />
                    <span>Whitepaper</span>
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>

              <SidebarMenuItem>
                <SidebarMenuButton asChild isActive={location === "/docs"} tooltip="Docs" data-testid="link-sidebar-docs" className="text-xs">
                  <Link href="/docs">
                    <BookOpen className="w-3.5 h-3.5" />
                    <span>Docs</span>
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>

              <SidebarMenuItem>
                <SidebarMenuButton asChild isActive={location === "/compliance"} tooltip="CNSA 2.0" data-testid="link-sidebar-compliance" className="text-xs">
                  <Link href="/compliance">
                    <ShieldCheck className="w-3.5 h-3.5" />
                    <span>CNSA 2.0</span>
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>

        <SidebarSeparator />

        <SidebarGroup>
          <SidebarGroupContent>
            <SidebarMenu>
              <Collapsible defaultOpen={false} className="group/collapsible">
                <SidebarMenuItem>
                  <CollapsibleTrigger asChild>
                    <SidebarMenuButton tooltip="App Links" data-testid="button-sidebar-applinks" className="text-xs">
                      <ExternalLink className="w-3.5 h-3.5" />
                      <span>App Links</span>
                      <ChevronRight className="ml-auto w-3.5 h-3.5 transition-transform duration-200 group-data-[state=open]/collapsible:rotate-90" />
                    </SidebarMenuButton>
                  </CollapsibleTrigger>
                  <CollapsibleContent>
                    <SidebarMenuSub>
                      <SidebarMenuSubItem>
                        <SidebarMenuSubButton asChild className="text-xs" data-testid="link-sidebar-github">
                          <a href="https://github.com/SigmaWolf-8/Ternary" target="_blank" rel="noopener noreferrer">
                            <Github className="w-3.5 h-3.5" />
                            <span>GitHub</span>
                          </a>
                        </SidebarMenuSubButton>
                      </SidebarMenuSubItem>
                      <SidebarMenuSubItem>
                        <SidebarMenuSubButton asChild className="text-xs" data-testid="link-sidebar-kong">
                          <a href="https://cloud.konghq.com/us/gateway-manager" target="_blank" rel="noopener noreferrer">
                            <Network className="w-3.5 h-3.5" />
                            <span>Kong Konnect</span>
                          </a>
                        </SidebarMenuSubButton>
                      </SidebarMenuSubItem>
                    </SidebarMenuSub>
                  </CollapsibleContent>
                </SidebarMenuItem>
              </Collapsible>
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>

        <div className="flex-1" />

        {adminStatus?.isAdmin && (
          <>
            <SidebarSeparator />
            <SidebarGroup>
              <SidebarGroupLabel className="text-[10px] uppercase tracking-wider">Admin</SidebarGroupLabel>
              <SidebarGroupContent>
                <SidebarMenu>
                  <SidebarMenuItem>
                    <SidebarMenuButton asChild isActive={location === "/admin"} tooltip="Admin Dashboard" data-testid="link-sidebar-admin" className="text-xs">
                      <Link href="/admin">
                        <Shield className="w-3.5 h-3.5" />
                        <span>Dashboard</span>
                      </Link>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                  <SidebarMenuItem>
                    <SidebarMenuButton asChild isActive={location === "/github"} tooltip="GitHub Manager" data-testid="link-sidebar-github-manager" className="text-xs">
                      <Link href="/github">
                        <Github className="w-3.5 h-3.5" />
                        <span>GitHub Manager</span>
                      </Link>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                  <SidebarMenuItem>
                    <SidebarMenuButton asChild isActive={location === "/kong-konnect"} tooltip="Kong Konnect" data-testid="link-sidebar-kong-page" className="text-xs">
                      <Link href="/kong-konnect">
                        <Network className="w-3.5 h-3.5" />
                        <span>Kong Konnect</span>
                      </Link>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                </SidebarMenu>
              </SidebarGroupContent>
            </SidebarGroup>
          </>
        )}
      </SidebarContent>

      <SidebarFooter>
        <SidebarMenu>
          <SidebarMenuItem>
            {isLoading ? (
              <SidebarMenuButton disabled className="text-xs">
                <div className="w-3.5 h-3.5 rounded-full bg-muted animate-pulse" />
                <span className="text-muted-foreground">Loading...</span>
              </SidebarMenuButton>
            ) : isAuthenticated ? (
              <SidebarMenuButton asChild tooltip="Logout" data-testid="button-sidebar-logout" className="text-xs">
                <a href="/api/logout">
                  <Avatar className="w-5 h-5">
                    <AvatarImage src={user?.profileImageUrl || undefined} />
                    <AvatarFallback className="text-[10px]">
                      {user?.firstName?.[0] || <User className="w-3 h-3" />}
                    </AvatarFallback>
                  </Avatar>
                  <span className="truncate">{user?.firstName || user?.email?.split("@")[0]}</span>
                  <LogOut className="ml-auto w-3.5 h-3.5 text-muted-foreground" />
                </a>
              </SidebarMenuButton>
            ) : (
              <SidebarMenuButton asChild tooltip="Sign In" data-testid="button-sidebar-login" className="text-xs">
                <a href="/api/login">
                  <LogIn className="w-3.5 h-3.5" />
                  <span>Sign In</span>
                </a>
              </SidebarMenuButton>
            )}
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarFooter>
    </Sidebar>
  );
}
