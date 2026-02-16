import { useLocation, Link } from "wouter";
import { LayoutDashboard, FilePlus, Shield } from "lucide-react";
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

const navItems = [
  { title: "Dashboard", url: "/", icon: LayoutDashboard },
  { title: "New Envelope", url: "/new", icon: FilePlus },
];

export function AppSidebar() {
  const [location] = useLocation();

  return (
    <Sidebar>
      <SidebarHeader className="p-4">
        <Link href="/">
          <div className="flex items-center gap-2.5 cursor-pointer" data-testid="link-home">
            <div className="flex items-center justify-center w-7 h-7 rounded-md bg-primary">
              <Shield className="w-3.5 h-3.5 text-primary-foreground" />
            </div>
            <div className="flex flex-col">
              <span className="text-base tracking-wide" style={{ fontFamily: "'Great Vibes', cursive" }}>SalviSign</span>
              <span className="text-[9px] text-muted-foreground tracking-wider leading-none">Secure Signatures</span>
            </div>
          </div>
        </Link>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel className="text-[9px] tracking-widest uppercase">Navigation</SidebarGroupLabel>
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
      <SidebarFooter className="p-4">
        <div className="flex items-center gap-2">
          <Shield className="w-2.5 h-2.5 text-muted-foreground" />
          <span className="text-[9px] text-muted-foreground tracking-wider uppercase">Quantum-Secure</span>
        </div>
      </SidebarFooter>
    </Sidebar>
  );
}
