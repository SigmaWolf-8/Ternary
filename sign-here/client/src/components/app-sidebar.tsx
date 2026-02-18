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
import { useRef, useCallback } from "react";
import { useLocation, Link } from "wouter";
import { Archive, FilePlus, Shield, Settings, Info, Grid3X3, Tag, Tags } from "lucide-react";
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupLabel,
  SidebarGroupContent,
  SidebarMenu,
  SidebarMenuItem,
  SidebarMenuButton,
  SidebarHeader,
  SidebarFooter,
  useSidebar,
} from "@/components/ui/sidebar";
import signHereVideo from "@assets/grok-video-ea17a914-4cbc-478f-ae96-ffe0c6abb977_(1)_1771375777069.mp4";

const navItems = [
  { title: "File Cabinet", url: "/", icon: Archive },
  { title: "Tag Envelopes", url: "/wbs-tagging", icon: Tags },
  { title: "New Envelope", url: "/new", icon: FilePlus },
  { title: "About", url: "/about", icon: Info },
];

const templateItems = [
  { title: "Templates", url: "/templates", icon: Grid3X3 },
  { title: "WBS Tags", url: "/wbs-tags", icon: Tag },
];

export function AppSidebar() {
  const [location, navigate] = useLocation();
  const { isMobile, setOpenMobile } = useSidebar();
  const videoRef = useRef<HTMLVideoElement>(null);
  const playCountRef = useRef(0);
  const closeMobileSidebar = useCallback(() => {
    if (isMobile) setOpenMobile(false);
  }, [isMobile, setOpenMobile]);

  const handleLogoClick = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    navigate("/");
    closeMobileSidebar();
    const video = videoRef.current;
    if (video) {
      playCountRef.current = 0;
      video.currentTime = 0;
      video.play();
    }
  }, [navigate, closeMobileSidebar]);

  const handleVideoEnded = useCallback(() => {
    const video = videoRef.current;
    if (video && playCountRef.current < 1) {
      playCountRef.current += 1;
      video.currentTime = 0;
      video.play();
    }
  }, []);

  return (
    <Sidebar>
      <SidebarHeader className="p-0" style={{ boxShadow: "0 8px 16px -4px rgba(0,0,0,0.45), 0 3px 6px -2px rgba(0,0,0,0.3)" }}>
        <div
          className="cursor-pointer overflow-hidden flex items-center justify-center bg-black"
          data-testid="link-home"
          onClick={handleLogoClick}
          style={{
            boxShadow: 'inset 0 6px 14px rgba(0,0,0,0.6), inset 0 -6px 14px rgba(0,0,0,0.5), inset 6px 0 14px rgba(0,0,0,0.4), inset -6px 0 14px rgba(0,0,0,0.4), inset 0 0 30px rgba(0,0,0,0.25)',
            borderTop: '3px solid rgba(0,0,0,0.35)',
            borderBottom: '3px solid rgba(255,255,255,0.08)',
            borderLeft: '3px solid rgba(0,0,0,0.2)',
            borderRight: '3px solid rgba(0,0,0,0.2)',
            minHeight: isMobile ? '320px' : '200px',
            maxHeight: isMobile ? '320px' : '200px',
          }}
        >
          <video
            ref={videoRef}
            src={signHereVideo}
            autoPlay
            muted
            playsInline
            onEnded={handleVideoEnded}
            className="object-cover pointer-events-none"
            style={{ width: '100%', height: isMobile ? '320px' : '180px', objectPosition: 'center center' }}
          />
        </div>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel className="text-[9px] tracking-widest uppercase">Signed | Sealed | Delivered</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {navItems.map((item) => (
                <SidebarMenuItem key={item.title}>
                  <SidebarMenuButton
                    asChild
                    isActive={location === item.url}
                    style={{ boxShadow: "0 2px 4px -1px rgba(0,0,0,0.4), 0 1px 2px -1px rgba(0,0,0,0.3), inset 0 1px 0 rgba(255,255,255,0.08)" }}
                  >
                    <Link href={item.url} onClick={closeMobileSidebar} data-testid={`link-nav-${item.title.toLowerCase().replace(/\s+/g, "-")}`}>
                      <item.icon className="w-3.5 h-3.5" />
                      <span className="text-xs">{item.title}</span>
                    </Link>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
        <SidebarGroup>
          <SidebarGroupLabel className="text-[9px] tracking-widest uppercase">Templates</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {templateItems.map((item) => (
                <SidebarMenuItem key={item.title}>
                  <SidebarMenuButton
                    asChild
                    isActive={location === item.url}
                    style={{ boxShadow: "0 2px 4px -1px rgba(0,0,0,0.4), 0 1px 2px -1px rgba(0,0,0,0.3), inset 0 1px 0 rgba(255,255,255,0.08)" }}
                  >
                    <Link href={item.url} onClick={closeMobileSidebar} data-testid={`link-nav-${item.title.toLowerCase().replace(/\s+/g, "-")}`}>
                      <item.icon className="w-3.5 h-3.5" />
                      <span className="text-xs">{item.title}</span>
                    </Link>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter className="p-2 space-y-2" style={{ boxShadow: "inset 0 8px 16px -4px rgba(0,0,0,0.45), inset 0 3px 6px -2px rgba(0,0,0,0.3)" }}>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton
              asChild
              isActive={location === "/settings"}
            >
              <Link href="/settings" onClick={closeMobileSidebar} data-testid="link-nav-settings">
                <Settings className="w-3.5 h-3.5" />
                <span className="text-xs">Settings</span>
              </Link>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
        <div className="flex items-center gap-2 px-2 pb-1">
          <Shield className="w-2.5 h-2.5 text-muted-foreground" />
          <span className="text-[9px] text-muted-foreground tracking-wider uppercase">Quantum-Secure</span>
        </div>
      </SidebarFooter>
    </Sidebar>
  );
}
