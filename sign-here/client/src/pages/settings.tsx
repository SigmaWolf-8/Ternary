import { useState, useEffect } from "react";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useToast } from "@/hooks/use-toast";
import { Save, Globe, User, Clock, ShieldCheck, ChevronRight, RotateCcw } from "lucide-react";
import { resetTour } from "@/components/onboarding-tour";
import { Link } from "wouter";

const TIMEZONES = [
  { value: "America/New_York", label: "Eastern Time (ET)" },
  { value: "America/Chicago", label: "Central Time (CT)" },
  { value: "America/Denver", label: "Mountain Time (MT)" },
  { value: "America/Los_Angeles", label: "Pacific Time (PT)" },
  { value: "America/Anchorage", label: "Alaska Time (AKT)" },
  { value: "Pacific/Honolulu", label: "Hawaii Time (HT)" },
  { value: "America/Phoenix", label: "Arizona (no DST)" },
  { value: "America/Puerto_Rico", label: "Atlantic Time (AST)" },
  { value: "Europe/London", label: "GMT / London" },
  { value: "Europe/Paris", label: "Central European (CET)" },
  { value: "Europe/Helsinki", label: "Eastern European (EET)" },
  { value: "Asia/Dubai", label: "Gulf Standard (GST)" },
  { value: "Asia/Kolkata", label: "India Standard (IST)" },
  { value: "Asia/Shanghai", label: "China Standard (CST)" },
  { value: "Asia/Tokyo", label: "Japan Standard (JST)" },
  { value: "Australia/Sydney", label: "Australian Eastern (AEST)" },
  { value: "Pacific/Auckland", label: "New Zealand (NZST)" },
  { value: "UTC", label: "UTC" },
];

export function getSettings() {
  try {
    const raw = localStorage.getItem("signhere_settings");
    if (raw) return JSON.parse(raw);
  } catch {}
  return {
    displayName: "",
    email: "",
    timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
    dateFormat: "full",
  };
}

export function saveSettings(settings: Record<string, string>) {
  localStorage.setItem("signhere_settings", JSON.stringify(settings));
}

export function formatDateWithTimezone(date?: Date): string {
  const d = date || new Date();
  const settings = getSettings();
  const tz = settings.timezone || Intl.DateTimeFormat().resolvedOptions().timeZone;
  const fmt = settings.dateFormat || "full";
  try {
    if (fmt === "iso") {
      const parts = new Intl.DateTimeFormat("en-CA", {
        timeZone: tz,
        year: "numeric",
        month: "2-digit",
        day: "2-digit",
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
        hour12: false,
      }).formatToParts(d);
      const get = (t: string) => parts.find((p) => p.type === t)?.value || "";
      return `${get("year")}-${get("month")}-${get("day")}T${get("hour")}:${get("minute")}:${get("second")}`;
    }
    if (fmt === "short") {
      return new Intl.DateTimeFormat("en-US", {
        timeZone: tz,
        year: "numeric",
        month: "numeric",
        day: "numeric",
        hour: "numeric",
        minute: "2-digit",
        timeZoneName: "short",
      }).format(d);
    }
    return new Intl.DateTimeFormat("en-US", {
      timeZone: tz,
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "numeric",
      minute: "2-digit",
      second: "2-digit",
      timeZoneName: "short",
    }).format(d);
  } catch {
    return d.toLocaleString();
  }
}

export default function Settings() {
  const { toast } = useToast();
  const [displayName, setDisplayName] = useState("");
  const [email, setEmail] = useState("");
  const [timezone, setTimezone] = useState(Intl.DateTimeFormat().resolvedOptions().timeZone);
  const [dateFormat, setDateFormat] = useState("full");
  const [role, setRole] = useState("signer");
  const [currentTime, setCurrentTime] = useState("");

  useEffect(() => {
    const settings = getSettings();
    setDisplayName(settings.displayName || "");
    setEmail(settings.email || "");
    setTimezone(settings.timezone || Intl.DateTimeFormat().resolvedOptions().timeZone);
    setDateFormat(settings.dateFormat || "full");
    setRole(settings.role || "signer");
  }, []);

  useEffect(() => {
    const update = () => {
      try {
        setCurrentTime(
          new Intl.DateTimeFormat("en-US", {
            timeZone: timezone,
            year: "numeric",
            month: "short",
            day: "numeric",
            hour: "numeric",
            minute: "2-digit",
            second: "2-digit",
            timeZoneName: "short",
          }).format(new Date())
        );
      } catch {
        setCurrentTime(new Date().toLocaleString());
      }
    };
    update();
    const interval = setInterval(update, 1000);
    return () => clearInterval(interval);
  }, [timezone]);

  const handleSave = () => {
    saveSettings({ displayName, email, timezone, dateFormat, role });
    toast({ title: "Settings saved", description: "Your preferences have been updated" });
    window.history.back();
  };

  return (
    <div className="flex-1 overflow-auto p-5">
      <div className="max-w-lg mx-auto space-y-5">
        <div>
          <h1 className="text-sm font-semibold" data-testid="text-settings-title">Settings</h1>
          <p className="text-[10px] text-muted-foreground mt-0.5">
            Configure your Sign Here preferences
          </p>
        </div>

        <Card>
          <CardContent className="p-4 space-y-4">
            <div className="flex items-center gap-2 mb-1">
              <User className="w-3.5 h-3.5 text-muted-foreground" />
              <span className="text-[10px] font-medium uppercase tracking-widest text-muted-foreground">Profile</span>
            </div>

            <div className="space-y-1.5">
              <label className="text-[10px] font-medium text-muted-foreground uppercase tracking-wider">Display Name</label>
              <Input
                value={displayName}
                onChange={(e) => setDisplayName(e.target.value)}
                placeholder="Your name"
                data-testid="input-display-name"
              />
            </div>

            <div className="space-y-1.5">
              <label className="text-[10px] font-medium text-muted-foreground uppercase tracking-wider">Email</label>
              <Input
                type="email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                placeholder="you@example.com"
                data-testid="input-email"
              />
            </div>

            <div className="space-y-1.5">
              <label className="text-[10px] font-medium text-muted-foreground uppercase tracking-wider">Role</label>
              <Select value={role} onValueChange={setRole}>
                <SelectTrigger data-testid="select-role">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="admin">Admin</SelectItem>
                  <SelectItem value="signer">Signer</SelectItem>
                  <SelectItem value="viewer">Viewer</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-4 space-y-4">
            <div className="flex items-center gap-2 mb-1">
              <Globe className="w-3.5 h-3.5 text-muted-foreground" />
              <span className="text-[10px] font-medium uppercase tracking-widest text-muted-foreground">Timezone & Date</span>
            </div>

            <div className="space-y-1.5">
              <label className="text-[10px] font-medium text-muted-foreground uppercase tracking-wider">Timezone</label>
              <Select value={timezone} onValueChange={setTimezone}>
                <SelectTrigger data-testid="select-timezone">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {TIMEZONES.map((tz) => (
                    <SelectItem key={tz.value} value={tz.value}>
                      {tz.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-1.5">
              <label className="text-[10px] font-medium text-muted-foreground uppercase tracking-wider">Date Format</label>
              <Select value={dateFormat} onValueChange={setDateFormat}>
                <SelectTrigger data-testid="select-date-format">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="full">Full (Jan 15, 2026, 3:45:00 PM EST)</SelectItem>
                  <SelectItem value="short">Short (1/15/2026 3:45 PM)</SelectItem>
                  <SelectItem value="iso">ISO (2026-01-15T15:45:00)</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className="flex items-center gap-2 p-2.5 rounded-md bg-muted/50">
              <Clock className="w-3.5 h-3.5 text-primary" />
              <div>
                <span className="text-[9px] text-muted-foreground uppercase tracking-wider block">Current Time Preview</span>
                <span className="text-xs font-medium" data-testid="text-time-preview">{currentTime}</span>
              </div>
            </div>
          </CardContent>
        </Card>

        {role === "admin" && (
          <Card>
            <CardContent className="p-4 space-y-3">
              <div className="flex items-center gap-2 mb-1">
                <ShieldCheck className="w-3.5 h-3.5 text-muted-foreground" />
                <span className="text-[10px] font-medium uppercase tracking-widest text-muted-foreground">Administration</span>
              </div>
              <Link href="/admin">
                <Button variant="outline" className="w-full justify-between" data-testid="link-admin-panel">
                  <span className="flex items-center gap-2">
                    <ShieldCheck className="w-3.5 h-3.5" />
                    Admin Panel
                  </span>
                  <ChevronRight className="w-3.5 h-3.5 opacity-50" />
                </Button>
              </Link>
              <p className="text-[9px] text-muted-foreground">
                Manage tenants, users, roles, and view platform statistics
              </p>
            </CardContent>
          </Card>
        )}

        <Card>
          <CardContent className="p-4 space-y-3">
            <div className="flex items-center gap-2 mb-1">
              <RotateCcw className="w-3.5 h-3.5 text-muted-foreground" />
              <span className="text-[10px] font-medium uppercase tracking-widest text-muted-foreground">Onboarding</span>
            </div>
            <Button
              variant="outline"
              className="w-full"
              onClick={() => {
                resetTour();
                toast({ title: "Tour reset", description: "The onboarding tour will appear next time you visit" });
              }}
              data-testid="button-restart-tour"
            >
              <RotateCcw className="w-3.5 h-3.5" />
              Restart Onboarding Tour
            </Button>
            <p className="text-[9px] text-muted-foreground">
              Re-enable the guided walkthrough of Sign Here features
            </p>
          </CardContent>
        </Card>

        <Button onClick={handleSave} className="w-full" data-testid="button-save-settings">
          <Save className="w-3.5 h-3.5" />
          Save Settings
        </Button>
      </div>
    </div>
  );
}
