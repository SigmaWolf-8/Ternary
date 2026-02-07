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
                <Box className="w-5 h-5 text-primary" />
                <span className="font-bold text-lg">PlenumNET</span>
              </Link>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarHeader>

      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>Navigation</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              <Collapsible defaultOpen={isLanding} className="group/collapsible">
                <SidebarMenuItem>
                  <CollapsibleTrigger asChild>
                    <SidebarMenuButton tooltip="Platform" data-testid="button-sidebar-platform">
                      <Layers className="w-4 h-4" />
                      <span>Platform</span>
                      <ChevronRight className="ml-auto w-4 h-4 transition-transform duration-200 group-data-[state=open]/collapsible:rotate-90" />
                    </SidebarMenuButton>
                  </CollapsibleTrigger>
                  <CollapsibleContent>
                    <SidebarMenuSub>
                      <SidebarMenuSubItem>
                        <SidebarMenuSubButton onClick={() => scrollToSection("architecture")} className="cursor-pointer" data-testid="link-sidebar-architecture">
                          <Cpu className="w-4 h-4" />
                          <span>Architecture</span>
                        </SidebarMenuSubButton>
                      </SidebarMenuSubItem>
                      <SidebarMenuSubItem>
                        <SidebarMenuSubButton onClick={() => scrollToSection("components")} className="cursor-pointer" data-testid="link-sidebar-components">
                          <Network className="w-4 h-4" />
                          <span>Components</span>
                        </SidebarMenuSubButton>
                      </SidebarMenuSubItem>
                      <SidebarMenuSubItem>
                        <SidebarMenuSubButton onClick={() => scrollToSection("performance")} className="cursor-pointer" data-testid="link-sidebar-performance">
                          <Gauge className="w-4 h-4" />
                          <span>Performance</span>
                        </SidebarMenuSubButton>
                      </SidebarMenuSubItem>
                      <SidebarMenuSubItem>
                        <SidebarMenuSubButton asChild isActive={location === "/calendar"} data-testid="link-sidebar-calendar-sub">
                          <Link href="/calendar">
                            <Calendar className="w-4 h-4" />
                            <span>Calendar</span>
                          </Link>
                        </SidebarMenuSubButton>
                      </SidebarMenuSubItem>
                    </SidebarMenuSub>
                  </CollapsibleContent>
                </SidebarMenuItem>
              </Collapsible>

              <SidebarMenuItem>
                <SidebarMenuButton asChild isActive={location === "/calendar"} tooltip="Calendar API" data-testid="link-sidebar-calendar">
                  <Link href="/calendar">
                    <Globe className="w-4 h-4" />
                    <span>Calendar API</span>
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>

              <SidebarMenuItem>
                <SidebarMenuButton asChild isActive={location === "/api-demo"} tooltip="API Demo" data-testid="link-sidebar-api-demo">
                  <Link href="/api-demo">
                    <Terminal className="w-4 h-4" />
                    <span>API Demo</span>
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>

              <SidebarMenuItem>
                <SidebarMenuButton asChild isActive={location === "/whitepaper"} tooltip="Whitepaper" data-testid="link-sidebar-whitepaper">
                  <Link href="/whitepaper">
                    <FileText className="w-4 h-4" />
                    <span>Whitepaper</span>
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>

              <SidebarMenuItem>
                <SidebarMenuButton asChild isActive={location === "/docs"} tooltip="Docs" data-testid="link-sidebar-docs">
                  <Link href="/docs">
                    <BookOpen className="w-4 h-4" />
                    <span>Docs</span>
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
              <Collapsible defaultOpen className="group/collapsible">
                <SidebarMenuItem>
                  <CollapsibleTrigger asChild>
                    <SidebarMenuButton tooltip="App Links" data-testid="button-sidebar-applinks">
                      <ExternalLink className="w-4 h-4" />
                      <span>App Links</span>
                      <ChevronRight className="ml-auto w-4 h-4 transition-transform duration-200 group-data-[state=open]/collapsible:rotate-90" />
                    </SidebarMenuButton>
                  </CollapsibleTrigger>
                  <CollapsibleContent>
                    <SidebarMenuSub>
                      <SidebarMenuSubItem>
                        <SidebarMenuSubButton asChild data-testid="link-sidebar-github">
                          <a href="https://github.com/SigmaWolf-8/Ternary" target="_blank" rel="noopener noreferrer">
                            <Github className="w-4 h-4" />
                            <span>GitHub</span>
                          </a>
                        </SidebarMenuSubButton>
                      </SidebarMenuSubItem>
                      <SidebarMenuSubItem>
                        <SidebarMenuSubButton asChild data-testid="link-sidebar-kong">
                          <a href="https://cloud.konghq.com/us/gateway-manager" target="_blank" rel="noopener noreferrer">
                            <Network className="w-4 h-4" />
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

        {adminStatus?.isAdmin && (
          <>
            <SidebarSeparator />
            <SidebarGroup>
              <SidebarGroupLabel>Admin</SidebarGroupLabel>
              <SidebarGroupContent>
                <SidebarMenu>
                  <SidebarMenuItem>
                    <SidebarMenuButton asChild isActive={location === "/admin"} tooltip="Admin Dashboard" data-testid="link-sidebar-admin">
                      <Link href="/admin">
                        <Shield className="w-4 h-4" />
                        <span>Dashboard</span>
                      </Link>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                  <SidebarMenuItem>
                    <SidebarMenuButton asChild isActive={location === "/github"} tooltip="GitHub Manager" data-testid="link-sidebar-github-manager">
                      <Link href="/github">
                        <Github className="w-4 h-4" />
                        <span>GitHub Manager</span>
                      </Link>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                  <SidebarMenuItem>
                    <SidebarMenuButton asChild isActive={location === "/kong-konnect"} tooltip="Kong Konnect" data-testid="link-sidebar-kong-page">
                      <Link href="/kong-konnect">
                        <Network className="w-4 h-4" />
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
              <SidebarMenuButton disabled>
                <div className="w-4 h-4 rounded-full bg-muted animate-pulse" />
                <span className="text-muted-foreground">Loading...</span>
              </SidebarMenuButton>
            ) : isAuthenticated ? (
              <SidebarMenuButton asChild tooltip="Logout" data-testid="button-sidebar-logout">
                <a href="/api/logout">
                  <Avatar className="w-5 h-5">
                    <AvatarImage src={user?.profileImageUrl || undefined} />
                    <AvatarFallback className="text-xs">
                      {user?.firstName?.[0] || <User className="w-3 h-3" />}
                    </AvatarFallback>
                  </Avatar>
                  <span className="truncate">{user?.firstName || user?.email?.split("@")[0]}</span>
                  <LogOut className="ml-auto w-4 h-4 text-muted-foreground" />
                </a>
              </SidebarMenuButton>
            ) : (
              <SidebarMenuButton asChild tooltip="Sign In" data-testid="button-sidebar-login">
                <a href="/api/login">
                  <LogIn className="w-4 h-4" />
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
