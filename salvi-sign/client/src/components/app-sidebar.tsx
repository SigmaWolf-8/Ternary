import { useLocation, Link } from "wouter";
import { LayoutDashboard, FilePlus, Shield, Settings } from "lucide-react";
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
} from "@/components/ui/sidebar";
import salviLogo from "@assets/61c7a11d-25db-489e-be25-ada68c5a8504_1771266001045.jpg";

const navItems = [
  { title: "Dashboard", url: "/", icon: LayoutDashboard },
  { title: "New Envelope", url: "/new", icon: FilePlus },
];

export function AppSidebar() {
  const [location] = useLocation();

  return (
    <Sidebar>
      <SidebarHeader className="p-0">
        <Link href="/">
          <div
            className="cursor-pointer overflow-hidden"
            data-testid="link-home"
            style={{
              boxShadow: 'inset 0 6px 14px rgba(0,0,0,0.6), inset 0 -6px 14px rgba(0,0,0,0.5), inset 6px 0 14px rgba(0,0,0,0.4), inset -6px 0 14px rgba(0,0,0,0.4), inset 0 0 30px rgba(0,0,0,0.25)',
              borderTop: '3px solid rgba(0,0,0,0.35)',
              borderBottom: '3px solid rgba(255,255,255,0.08)',
              borderLeft: '3px solid rgba(0,0,0,0.2)',
              borderRight: '3px solid rgba(0,0,0,0.2)',
            }}
          >
            <img
              src={salviLogo}
              alt="SalviSign"
              className="w-full object-cover object-center"
              style={{ aspectRatio: '4/3', transform: 'scale(1.75)' }}
            />
          </div>
        </Link>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel className="text-[9px] tracking-widest uppercase">Sealed & Delivered</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {navItems.map((item) => (
                <SidebarMenuItem key={item.title}>
                  <SidebarMenuButton
                    asChild
                    isActive={location === item.url}
                  >
                    <Link href={item.url} data-testid={`link-nav-${item.title.toLowerCase().replace(/\s+/g, "-")}`}>
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
      <SidebarFooter className="p-2 space-y-2">
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton
              asChild
              isActive={location === "/settings"}
            >
              <Link href="/settings" data-testid="link-nav-settings">
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
