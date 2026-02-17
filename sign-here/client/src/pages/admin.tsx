import { useState } from "react";
import { useQuery, useMutation } from "@tanstack/react-query";
import { format } from "date-fns";
import {
  Users,
  Building2,
  FileText,
  Trash2,
  UserPlus,
  Shield,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Skeleton } from "@/components/ui/skeleton";
import { Badge } from "@/components/ui/badge";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
  DialogDescription,
} from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { queryClient, apiRequest } from "@/lib/queryClient";
import { useToast } from "@/hooks/use-toast";

interface AdminStats {
  tenants: number;
  users: number;
  envelopes: {
    total: number;
    draft: number;
    sent: number;
    completed: number;
  };
}

interface AdminUser {
  id: string;
  username: string;
  email: string;
  role: string;
  tenantId: string | null;
}

interface Tenant {
  id: string;
  name: string;
  createdAt: string;
}

export default function AdminPage() {
  const { toast } = useToast();
  const [newUserOpen, setNewUserOpen] = useState(false);
  const [newTenantOpen, setNewTenantOpen] = useState(false);
  const [newUsername, setNewUsername] = useState("");
  const [newEmail, setNewEmail] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [newRole, setNewRole] = useState("signer");
  const [newTenantId, setNewTenantId] = useState("");
  const [newTenantName, setNewTenantName] = useState("");

  const { data: stats, isLoading: statsLoading } = useQuery<AdminStats>({
    queryKey: ["/api/admin/stats"],
  });

  const { data: users, isLoading: usersLoading } = useQuery<AdminUser[]>({
    queryKey: ["/api/admin/users"],
  });

  const { data: tenants, isLoading: tenantsLoading } = useQuery<Tenant[]>({
    queryKey: ["/api/admin/tenants"],
  });

  const createUserMutation = useMutation({
    mutationFn: async () => {
      await apiRequest("POST", "/api/admin/users", {
        username: newUsername,
        email: newEmail,
        password: newPassword,
        role: newRole,
        tenantId: newTenantId && newTenantId !== "none" ? newTenantId : undefined,
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["/api/admin/users"] });
      queryClient.invalidateQueries({ queryKey: ["/api/admin/stats"] });
      toast({ title: "User created" });
      setNewUserOpen(false);
      setNewUsername("");
      setNewEmail("");
      setNewPassword("");
      setNewRole("signer");
      setNewTenantId("");
    },
    onError: (err: any) => {
      toast({ title: "Error", description: err.message, variant: "destructive" });
    },
  });

  const deleteUserMutation = useMutation({
    mutationFn: async (id: string) => {
      await apiRequest("DELETE", `/api/admin/users/${id}`);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["/api/admin/users"] });
      queryClient.invalidateQueries({ queryKey: ["/api/admin/stats"] });
      toast({ title: "User deleted" });
    },
  });

  const updateRoleMutation = useMutation({
    mutationFn: async ({ id, role }: { id: string; role: string }) => {
      await apiRequest("PATCH", `/api/admin/users/${id}`, { role });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["/api/admin/users"] });
      toast({ title: "Role updated" });
    },
  });

  const createTenantMutation = useMutation({
    mutationFn: async () => {
      await apiRequest("POST", "/api/tenants", { name: newTenantName });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["/api/admin/tenants"] });
      queryClient.invalidateQueries({ queryKey: ["/api/admin/stats"] });
      toast({ title: "Tenant created" });
      setNewTenantOpen(false);
      setNewTenantName("");
    },
    onError: (err: any) => {
      toast({ title: "Error", description: err.message, variant: "destructive" });
    },
  });

  const roleBadgeVariant = (role: string) => {
    if (role === "admin") return "default" as const;
    if (role === "signer") return "secondary" as const;
    return "outline" as const;
  };

  return (
    <div className="flex-1 overflow-auto">
      <div className="max-w-5xl mx-auto p-5 space-y-6">
        <div>
          <h1 className="text-base font-semibold tracking-tight" data-testid="text-admin-title">
            Administration
          </h1>
          <p className="text-[11px] text-muted-foreground mt-0.5 tracking-wide">
            Manage tenants, users, and platform settings
          </p>
        </div>

        <div className="grid grid-cols-2 md:grid-cols-4 gap-2.5">
          {[
            { label: "Tenants", value: stats?.tenants, icon: Building2 },
            { label: "Users", value: stats?.users, icon: Users },
            { label: "Envelopes", value: stats?.envelopes.total, icon: FileText },
            { label: "Completed", value: stats?.envelopes.completed, icon: Shield },
          ].map((stat) => (
            <Card key={stat.label}>
              <CardContent className="p-3.5">
                <div className="flex items-center justify-between gap-2">
                  <span className="text-[10px] text-muted-foreground uppercase tracking-wider font-medium">
                    {stat.label}
                  </span>
                  <stat.icon className="w-3 h-3 text-muted-foreground" />
                </div>
                {statsLoading ? (
                  <Skeleton className="h-6 w-8 mt-1.5" />
                ) : (
                  <p className="text-lg font-semibold mt-1" data-testid={`text-admin-stat-${stat.label.toLowerCase()}`}>
                    {stat.value ?? 0}
                  </p>
                )}
              </CardContent>
            </Card>
          ))}
        </div>

        <div className="grid gap-6 lg:grid-cols-2">
          <div>
            <div className="flex items-center justify-between gap-2 mb-3">
              <h2 className="text-[10px] font-medium text-muted-foreground uppercase tracking-wider">
                Tenants ({tenants?.length ?? 0})
              </h2>
              <Dialog open={newTenantOpen} onOpenChange={setNewTenantOpen}>
                <DialogTrigger asChild>
                  <Button size="sm" variant="outline" data-testid="button-add-tenant">
                    <Building2 className="w-3 h-3" />
                    Add Tenant
                  </Button>
                </DialogTrigger>
                <DialogContent>
                  <DialogHeader>
                    <DialogTitle>Create Tenant</DialogTitle>
                    <DialogDescription>Add a new organization to the platform.</DialogDescription>
                  </DialogHeader>
                  <div className="space-y-3 pt-2">
                    <div className="space-y-1.5">
                      <Label className="text-xs">Organization Name</Label>
                      <Input
                        value={newTenantName}
                        onChange={(e) => setNewTenantName(e.target.value)}
                        placeholder="Acme Corp"
                        className="h-9 text-xs"
                        data-testid="input-tenant-name"
                      />
                    </div>
                    <Button
                      size="sm"
                      onClick={() => createTenantMutation.mutate()}
                      disabled={!newTenantName || createTenantMutation.isPending}
                      data-testid="button-create-tenant"
                    >
                      {createTenantMutation.isPending ? "Creating..." : "Create Tenant"}
                    </Button>
                  </div>
                </DialogContent>
              </Dialog>
            </div>
            {tenantsLoading ? (
              <div className="space-y-1.5">
                {[1, 2].map((i) => (
                  <Card key={i}><CardContent className="p-3"><Skeleton className="h-4 w-32" /></CardContent></Card>
                ))}
              </div>
            ) : !tenants || tenants.length === 0 ? (
              <Card>
                <CardContent className="p-6 text-center">
                  <p className="text-xs text-muted-foreground">No tenants yet</p>
                </CardContent>
              </Card>
            ) : (
              <div className="space-y-1.5">
                {tenants.map((tenant) => (
                  <Card key={tenant.id} data-testid={`card-tenant-${tenant.id}`}>
                    <CardContent className="p-3 flex items-center justify-between gap-2">
                      <div className="flex items-center gap-2 min-w-0">
                        <Building2 className="w-3.5 h-3.5 text-muted-foreground shrink-0" />
                        <div className="min-w-0">
                          <p className="text-xs font-medium truncate">{tenant.name}</p>
                          <p className="text-[10px] text-muted-foreground">
                            {tenant.createdAt ? format(new Date(tenant.createdAt), "MMM d, yyyy") : "—"}
                          </p>
                        </div>
                      </div>
                      <span className="text-[9px] text-muted-foreground font-mono shrink-0">
                        {tenant.id.slice(0, 8)}
                      </span>
                    </CardContent>
                  </Card>
                ))}
              </div>
            )}
          </div>

          <div>
            <div className="flex items-center justify-between gap-2 mb-3">
              <h2 className="text-[10px] font-medium text-muted-foreground uppercase tracking-wider">
                Users ({users?.length ?? 0})
              </h2>
              <Dialog open={newUserOpen} onOpenChange={setNewUserOpen}>
                <DialogTrigger asChild>
                  <Button size="sm" variant="outline" data-testid="button-add-user">
                    <UserPlus className="w-3 h-3" />
                    Add User
                  </Button>
                </DialogTrigger>
                <DialogContent>
                  <DialogHeader>
                    <DialogTitle>Create User</DialogTitle>
                    <DialogDescription>Add a new user to the platform.</DialogDescription>
                  </DialogHeader>
                  <div className="space-y-3 pt-2">
                    <div className="space-y-1.5">
                      <Label className="text-xs">Username</Label>
                      <Input
                        value={newUsername}
                        onChange={(e) => setNewUsername(e.target.value)}
                        placeholder="john.doe"
                        className="h-9 text-xs"
                        data-testid="input-new-username"
                      />
                    </div>
                    <div className="space-y-1.5">
                      <Label className="text-xs">Email</Label>
                      <Input
                        value={newEmail}
                        onChange={(e) => setNewEmail(e.target.value)}
                        placeholder="john@company.com"
                        className="h-9 text-xs"
                        data-testid="input-new-email"
                      />
                    </div>
                    <div className="space-y-1.5">
                      <Label className="text-xs">Password</Label>
                      <Input
                        type="password"
                        value={newPassword}
                        onChange={(e) => setNewPassword(e.target.value)}
                        placeholder="Enter password"
                        className="h-9 text-xs"
                        data-testid="input-new-password"
                      />
                    </div>
                    <div className="space-y-1.5">
                      <Label className="text-xs">Role</Label>
                      <Select value={newRole} onValueChange={setNewRole}>
                        <SelectTrigger className="h-9 text-xs" data-testid="select-new-role">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="admin">Admin</SelectItem>
                          <SelectItem value="signer">Signer</SelectItem>
                          <SelectItem value="viewer">Viewer</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                    {tenants && tenants.length > 0 && (
                      <div className="space-y-1.5">
                        <Label className="text-xs">Tenant (optional)</Label>
                        <Select value={newTenantId} onValueChange={setNewTenantId}>
                          <SelectTrigger className="h-9 text-xs" data-testid="select-new-tenant">
                            <SelectValue placeholder="Select tenant" />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="none">No tenant</SelectItem>
                            {tenants.map((t) => (
                              <SelectItem key={t.id} value={t.id}>{t.name}</SelectItem>
                            ))}
                          </SelectContent>
                        </Select>
                      </div>
                    )}
                    <Button
                      size="sm"
                      onClick={() => createUserMutation.mutate()}
                      disabled={!newUsername || !newPassword || createUserMutation.isPending}
                      data-testid="button-create-user"
                    >
                      {createUserMutation.isPending ? "Creating..." : "Create User"}
                    </Button>
                  </div>
                </DialogContent>
              </Dialog>
            </div>
            {usersLoading ? (
              <div className="space-y-1.5">
                {[1, 2, 3].map((i) => (
                  <Card key={i}><CardContent className="p-3"><Skeleton className="h-4 w-32" /></CardContent></Card>
                ))}
              </div>
            ) : !users || users.length === 0 ? (
              <Card>
                <CardContent className="p-6 text-center">
                  <p className="text-xs text-muted-foreground">No users yet</p>
                </CardContent>
              </Card>
            ) : (
              <div className="space-y-1.5">
                {users.map((user) => (
                  <Card key={user.id} data-testid={`card-user-${user.id}`}>
                    <CardContent className="p-3">
                      <div className="flex items-center justify-between gap-2">
                        <div className="flex items-center gap-2 min-w-0 flex-1">
                          <Users className="w-3.5 h-3.5 text-muted-foreground shrink-0" />
                          <div className="min-w-0 flex-1">
                            <p className="text-xs font-medium truncate">{user.username}</p>
                            {user.email && (
                              <p className="text-[10px] text-muted-foreground truncate">{user.email}</p>
                            )}
                          </div>
                        </div>
                        <div className="flex items-center gap-1.5 shrink-0">
                          <Select
                            value={user.role}
                            onValueChange={(role) => updateRoleMutation.mutate({ id: user.id, role })}
                          >
                            <SelectTrigger className="h-7 text-[10px] w-20 border-0 p-1" data-testid={`select-role-${user.id}`}>
                              <Badge variant={roleBadgeVariant(user.role)} className="text-[9px]">
                                {user.role}
                              </Badge>
                            </SelectTrigger>
                            <SelectContent>
                              <SelectItem value="admin">Admin</SelectItem>
                              <SelectItem value="signer">Signer</SelectItem>
                              <SelectItem value="viewer">Viewer</SelectItem>
                            </SelectContent>
                          </Select>
                          <Button
                            size="icon"
                            variant="ghost"
                            onClick={() => deleteUserMutation.mutate(user.id)}
                            data-testid={`button-delete-user-${user.id}`}
                          >
                            <Trash2 className="w-3 h-3 text-destructive" />
                          </Button>
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            )}
          </div>
        </div>

        <Card>
          <CardHeader>
            <CardTitle className="text-sm tracking-wider uppercase">SaaS Architecture</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3 text-sm leading-relaxed text-muted-foreground">
            <p>
              Sign Here operates as a multi-tenant SaaS platform. Each organization (tenant) has complete data isolation enforced through PostgreSQL row-level security policies. Users belong to tenants and interact only with their organization's envelopes, recipients, and audit data.
            </p>
            <p>
              Tenant headers (<code className="text-[11px] bg-muted px-1 py-0.5 rounded">x-tenant-id</code>) are validated on every API request to ensure cross-tenant access is impossible. Admin users can manage tenants, create users, assign roles, and monitor platform-wide activity from this page.
            </p>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
