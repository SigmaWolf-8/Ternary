import { useAuth } from "@/hooks/use-auth";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { apiRequest } from "@/lib/queryClient";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { useToast } from "@/hooks/use-toast";
import { Link } from "wouter";
import type { DeveloperSignup } from "@shared/schema";
import {
  ArrowLeft,
  Trash2,
  Mail,
  Users,
  Building2,
  Calendar,
  LogIn,
  Shield,
  Download,
} from "lucide-react";

export default function Admin() {
  const { user, isLoading: authLoading, isAuthenticated } = useAuth();
  const { toast } = useToast();
  const queryClient = useQueryClient();

  const { data, isLoading } = useQuery<{ success: boolean; signups: DeveloperSignup[] }>({
    queryKey: ["/api/admin/developer-signups"],
    enabled: isAuthenticated,
  });

  const deleteMutation = useMutation({
    mutationFn: async (id: number) => {
      await apiRequest("DELETE", `/api/admin/developer-signups/${id}`);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["/api/admin/developer-signups"] });
      toast({ title: "Removed", description: "Signup entry deleted." });
    },
    onError: () => {
      toast({ title: "Error", description: "Failed to delete entry.", variant: "destructive" });
    },
  });

  const signups = data?.signups || [];

  const exportCSV = () => {
    if (signups.length === 0) return;
    const headers = ["Email", "Name", "Company", "Interest", "Signed Up"];
    const rows = signups.map(s => [
      s.email,
      s.name || "",
      s.company || "",
      s.interest || "",
      s.createdAt ? new Date(s.createdAt).toLocaleDateString() : "",
    ]);
    const csv = [headers.join(","), ...rows.map(r => r.map(v => `"${v}"`).join(","))].join("\n");
    const blob = new Blob([csv], { type: "text/csv" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `developer-signups-${new Date().toISOString().split("T")[0]}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  };

  if (authLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center" data-testid="admin-loading">
        <div className="text-muted-foreground">Loading...</div>
      </div>
    );
  }

  if (!isAuthenticated) {
    return (
      <div className="min-h-screen flex items-center justify-center p-5">
        <Card className="max-w-md w-full p-8 text-center" data-testid="admin-login-prompt">
          <Shield className="w-12 h-12 mx-auto mb-4 text-muted-foreground" />
          <h1 className="text-2xl font-bold mb-2">Admin Access</h1>
          <p className="text-muted-foreground mb-6">
            Sign in to view developer signups.
          </p>
          <Button asChild data-testid="button-admin-login">
            <a href="/api/login">
              <LogIn className="w-4 h-4 mr-2" />
              Sign In
            </a>
          </Button>
          <div className="mt-4">
            <Link href="/" className="text-sm text-muted-foreground hover:text-primary transition-colors" data-testid="link-admin-back">
              Back to Home
            </Link>
          </div>
        </Card>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-background">
      <header className="sticky top-0 z-50 border-b bg-background/95 backdrop-blur-sm" data-testid="admin-header">
        <div className="max-w-6xl mx-auto px-5 h-14 flex items-center justify-between gap-4">
          <div className="flex items-center gap-3 flex-wrap">
            <Link href="/" data-testid="link-admin-home">
              <Button variant="ghost" size="icon">
                <ArrowLeft className="w-4 h-4" />
              </Button>
            </Link>
            <h1 className="font-semibold text-lg" data-testid="text-admin-title">Developer Signups</h1>
            <Badge variant="secondary" data-testid="badge-signup-count">{signups.length}</Badge>
          </div>
          <div className="flex items-center gap-2">
            {signups.length > 0 && (
              <Button variant="outline" size="sm" onClick={exportCSV} data-testid="button-export-csv">
                <Download className="w-4 h-4 mr-2" />
                Export CSV
              </Button>
            )}
            <span className="text-sm text-muted-foreground hidden sm:inline" data-testid="text-admin-user">
              {user?.email || user?.firstName || "Admin"}
            </span>
          </div>
        </div>
      </header>

      <main className="max-w-6xl mx-auto px-5 py-8">
        <div className="grid grid-cols-1 sm:grid-cols-3 gap-4 mb-8">
          <Card className="p-5" data-testid="stat-total-signups">
            <div className="flex items-center gap-3">
              <div className="p-2 rounded-md bg-primary/10">
                <Users className="w-5 h-5 text-primary" />
              </div>
              <div>
                <p className="text-sm text-muted-foreground">Total Signups</p>
                <p className="text-2xl font-bold">{signups.length}</p>
              </div>
            </div>
          </Card>
          <Card className="p-5" data-testid="stat-with-company">
            <div className="flex items-center gap-3">
              <div className="p-2 rounded-md bg-primary/10">
                <Building2 className="w-5 h-5 text-primary" />
              </div>
              <div>
                <p className="text-sm text-muted-foreground">With Company</p>
                <p className="text-2xl font-bold">{signups.filter(s => s.company).length}</p>
              </div>
            </div>
          </Card>
          <Card className="p-5" data-testid="stat-with-interest">
            <div className="flex items-center gap-3">
              <div className="p-2 rounded-md bg-primary/10">
                <Mail className="w-5 h-5 text-primary" />
              </div>
              <div>
                <p className="text-sm text-muted-foreground">With Interest</p>
                <p className="text-2xl font-bold">{signups.filter(s => s.interest).length}</p>
              </div>
            </div>
          </Card>
        </div>

        {isLoading ? (
          <div className="text-center py-12 text-muted-foreground" data-testid="admin-table-loading">
            Loading signups...
          </div>
        ) : signups.length === 0 ? (
          <Card className="p-12 text-center" data-testid="admin-empty-state">
            <Users className="w-12 h-12 mx-auto mb-4 text-muted-foreground" />
            <h2 className="text-xl font-semibold mb-2">No signups yet</h2>
            <p className="text-muted-foreground">Developer signups will appear here.</p>
          </Card>
        ) : (
          <Card className="overflow-hidden" data-testid="admin-signups-table">
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b bg-muted/50">
                    <th className="text-left p-3 font-medium">Email</th>
                    <th className="text-left p-3 font-medium">Name</th>
                    <th className="text-left p-3 font-medium hidden md:table-cell">Company</th>
                    <th className="text-left p-3 font-medium hidden lg:table-cell">Interest</th>
                    <th className="text-left p-3 font-medium">Date</th>
                    <th className="text-right p-3 font-medium">Actions</th>
                  </tr>
                </thead>
                <tbody>
                  {signups.map((signup) => (
                    <tr key={signup.id} className="border-b last:border-b-0" data-testid={`row-signup-${signup.id}`}>
                      <td className="p-3">
                        <div className="flex items-center gap-2">
                          <Mail className="w-4 h-4 text-muted-foreground shrink-0" />
                          <span className="font-medium truncate max-w-[200px]" data-testid={`text-email-${signup.id}`}>{signup.email}</span>
                        </div>
                      </td>
                      <td className="p-3 text-muted-foreground" data-testid={`text-name-${signup.id}`}>
                        {signup.name || "-"}
                      </td>
                      <td className="p-3 text-muted-foreground hidden md:table-cell" data-testid={`text-company-${signup.id}`}>
                        {signup.company || "-"}
                      </td>
                      <td className="p-3 text-muted-foreground hidden lg:table-cell" data-testid={`text-interest-${signup.id}`}>
                        {signup.interest || "-"}
                      </td>
                      <td className="p-3 text-muted-foreground" data-testid={`text-date-${signup.id}`}>
                        <div className="flex items-center gap-1">
                          <Calendar className="w-3 h-3 shrink-0" />
                          <span>{signup.createdAt ? new Date(signup.createdAt).toLocaleDateString() : "-"}</span>
                        </div>
                      </td>
                      <td className="p-3 text-right">
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => deleteMutation.mutate(signup.id)}
                          disabled={deleteMutation.isPending}
                          data-testid={`button-delete-${signup.id}`}
                        >
                          <Trash2 className="w-4 h-4 text-destructive" />
                        </Button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </Card>
        )}
      </main>
    </div>
  );
}
