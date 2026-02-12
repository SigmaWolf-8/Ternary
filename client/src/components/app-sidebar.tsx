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

import { useLocation, Link } from "wouter";
import { useAuth } from "@/hooks/use-auth";
import { useQuery } from "@tanstack/react-query";
import { useCallback } from "react";
import {
  Box,
  Database,
  FileText,
  BookOpen,
  ShieldCheck,
  Globe,
  Moon,
  Radio,
  Terminal,
  Github,
  Network,
  Shield,
  Archive,
  LogIn,
  LogOut,
  User,
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
  SidebarSeparator,
  useSidebar,
} from "@/components/ui/sidebar";
import { useIsMobile } from "@/hooks/use-mobile";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";

export function AppSidebar() {
  const [location] = useLocation();
  const { user, isAuthenticated, isLoading } = useAuth();
  const { setOpenMobile } = useSidebar();
  const isMobile = useIsMobile();

  const { data: adminStatus } = useQuery<{ isAdmin: boolean }>({
    queryKey: ["/api/user/admin-status"],
    enabled: isAuthenticated,
  });

  const closeMobileSidebar = useCallback(() => {
    if (isMobile) {
      setOpenMobile(false);
    }
  }, [isMobile, setOpenMobile]);

  return (
    <Sidebar collapsible="icon">
      <SidebarHeader>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton asChild size="lg" tooltip="PlenumNET">
              <Link href="/" data-testid="link-sidebar-logo" onClick={closeMobileSidebar}>
                <Box className="w-4 h-4 text-primary" />
                <span className="font-semibold text-sm">PlenumNET</span>
              </Link>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarHeader>

      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel className="text-[10px] uppercase tracking-wider">Tools</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              <SidebarMenuItem>
                <SidebarMenuButton asChild isActive={location === "/ternarydb"} tooltip="PlenumDB" data-testid="link-sidebar-ternarydb" className="text-xs">
                  <Link href="/ternarydb" onClick={closeMobileSidebar}>
                    <Database className="w-3.5 h-3.5" />
                    <span>PlenumDB</span>
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>

              <SidebarMenuItem>
                <SidebarMenuButton asChild isActive={location === "/api-demo"} tooltip="API Explorer" data-testid="link-sidebar-api-demo" className="text-xs">
                  <Link href="/api-demo" onClick={closeMobileSidebar}>
                    <Terminal className="w-3.5 h-3.5" />
                    <span>API Explorer</span>
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>

              <SidebarMenuItem>
                <SidebarMenuButton asChild isActive={location === "/hptp"} tooltip="HPTP Lab" data-testid="link-sidebar-hptp" className="text-xs">
                  <Link href="/hptp" onClick={closeMobileSidebar}>
                    <Radio className="w-3.5 h-3.5" />
                    <span>HPTP Lab</span>
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>

              <SidebarMenuItem>
                <SidebarMenuButton asChild isActive={location === "/compression"} tooltip="Compression" data-testid="link-sidebar-compression" className="text-xs">
                  <Link href="/compression" onClick={closeMobileSidebar}>
                    <Archive className="w-3.5 h-3.5" />
                    <span>Compression</span>
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>

        <SidebarSeparator />

        <SidebarGroup>
          <SidebarGroupLabel className="text-[10px] uppercase tracking-wider">Calendars</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              <SidebarMenuItem>
                <SidebarMenuButton asChild isActive={location === "/calendar"} tooltip="Universal Calendar" data-testid="link-sidebar-calendar" className="text-xs">
                  <Link href="/calendar" onClick={closeMobileSidebar}>
                    <Globe className="w-3.5 h-3.5" />
                    <span>Universal Calendar</span>
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>

              <SidebarMenuItem>
                <SidebarMenuButton asChild isActive={location === "/13-moon"} tooltip="13-Moon Calendar" data-testid="link-sidebar-13-moon" className="text-xs">
                  <Link href="/13-moon" onClick={closeMobileSidebar}>
                    <Moon className="w-3.5 h-3.5" />
                    <span>13-Moon</span>
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>

        <SidebarSeparator />

        <SidebarGroup>
          <SidebarGroupLabel className="text-[10px] uppercase tracking-wider">Reference</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              <SidebarMenuItem>
                <SidebarMenuButton asChild isActive={location === "/docs"} tooltip="Documentation" data-testid="link-sidebar-docs" className="text-xs">
                  <Link href="/docs" onClick={closeMobileSidebar}>
                    <BookOpen className="w-3.5 h-3.5" />
                    <span>Documentation</span>
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>

              <SidebarMenuItem>
                <SidebarMenuButton asChild isActive={location === "/compliance"} tooltip="CNSA 2.0" data-testid="link-sidebar-compliance" className="text-xs">
                  <Link href="/compliance" onClick={closeMobileSidebar}>
                    <ShieldCheck className="w-3.5 h-3.5" />
                    <span>CNSA 2.0</span>
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>

              <SidebarMenuItem>
                <SidebarMenuButton asChild isActive={location === "/whitepaper"} tooltip="Whitepaper" data-testid="link-sidebar-whitepaper" className="text-xs">
                  <Link href="/whitepaper" onClick={closeMobileSidebar}>
                    <FileText className="w-3.5 h-3.5" />
                    <span>Whitepaper</span>
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>
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
                      <Link href="/admin" onClick={closeMobileSidebar}>
                        <Shield className="w-3.5 h-3.5" />
                        <span>Dashboard</span>
                      </Link>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                  <SidebarMenuItem>
                    <SidebarMenuButton asChild isActive={location === "/github"} tooltip="GitHub Manager" data-testid="link-sidebar-github-manager" className="text-xs">
                      <Link href="/github" onClick={closeMobileSidebar}>
                        <Github className="w-3.5 h-3.5" />
                        <span>GitHub Manager</span>
                      </Link>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                  <SidebarMenuItem>
                    <SidebarMenuButton asChild isActive={location === "/kong-konnect"} tooltip="Kong Konnect" data-testid="link-sidebar-kong-page" className="text-xs">
                      <Link href="/kong-konnect" onClick={closeMobileSidebar}>
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
