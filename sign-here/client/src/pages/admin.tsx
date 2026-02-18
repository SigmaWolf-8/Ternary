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
  Crown,
  Settings2,
  Zap,
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
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@/components/ui/tabs";
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
  isPlatformCreator: boolean;
}

interface Tenant {
  id: string;
  name: string;
  createdAt: string;
}

interface SaaSSettings {
  pricingTier: string;
  maxEnvelopesPerTenant: number;
  maxUsersPerTenant: number;
  features: Record<string, boolean>;
  platform: {
    version: string;
    plenumVersion: string;
    phase: string;
    encryption: string;
  };
}

const ROLE_LABELS: Record<string, string> = {
  sadmin: "SAdmin",
  admin: "Admin",
  manager: "Manager",
  signer: "Signer",
  viewer: "Viewer",
};

const ROLE_DESCRIPTIONS: Record<string, string> = {
  sadmin: "Platform Creator — full SaaS control",
  admin: "Tenant owner — manage users & templates",
  manager: "Prepare, send, and oversee envelopes",
  signer: "Sign assigned documents only",
  viewer: "Read-only access for compliance",
};

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

  const { data: saasSettings } = useQuery<SaaSSettings>({
    queryKey: ["/api/saas/settings"],
  });

  const currentUser = users?.find((u) => u.isPlatformCreator && u.role === "sadmin");
  const isSAdmin = !!currentUser;

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
    onError: (err: any) => {
      toast({ title: "Error", description: err.message, variant: "destructive" });
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
    onError: (err: any) => {
      toast({ title: "Error", description: err.message, variant: "destructive" });
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
    if (role === "sadmin") return "default" as const;
    if (role === "admin") return "default" as const;
    if (role === "manager") return "secondary" as const;
    if (role === "signer") return "secondary" as const;
    return "outline" as const;
  };

  const roleBadgeClass = (role: string) => {
    if (role === "sadmin") return "bg-amber-600/80 text-white border-amber-600";
    return "";
  };

  return (
    <div className="flex-1 overflow-auto">
      <div className="max-w-5xl mx-auto p-5 space-y-6">
        <div className="flex items-center justify-between gap-2 flex-wrap">
          <div>
            <h1 className="text-base font-semibold tracking-tight" data-testid="text-admin-title">
              Administration
            </h1>
            <p className="text-[11px] text-muted-foreground mt-0.5 tracking-wide">
              Manage tenants, users, roles, and platform settings
            </p>
          </div>
          {isSAdmin && (
            <Badge variant="default" className="bg-amber-600/80 text-white border-amber-600 text-[10px]" data-testid="badge-sadmin">
              <Crown className="w-3 h-3 mr-1" />
              SAdmin — Platform Creator
            </Badge>
          )}
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

        <Tabs defaultValue="users" className="w-full">
          <TabsList className="grid w-full grid-cols-4 h-9">
            <TabsTrigger value="users" className="text-xs" data-testid="tab-users">Users</TabsTrigger>
            <TabsTrigger value="tenants" className="text-xs" data-testid="tab-tenants">Tenants</TabsTrigger>
            <TabsTrigger value="roles" className="text-xs" data-testid="tab-roles">Roles</TabsTrigger>
            <TabsTrigger value="saas" className="text-xs" data-testid="tab-saas">
              <Crown className="w-3 h-3 mr-1" />
              SaaS
            </TabsTrigger>
          </TabsList>

          <TabsContent value="users" className="mt-4">
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
                          <SelectItem value="manager">Manager</SelectItem>
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
                          {user.role === "sadmin" ? (
                            <Crown className="w-3.5 h-3.5 text-amber-500 shrink-0" />
                          ) : (
                            <Users className="w-3.5 h-3.5 text-muted-foreground shrink-0" />
                          )}
                          <div className="min-w-0 flex-1">
                            <div className="flex items-center gap-1.5">
                              <p className="text-xs font-medium truncate">{user.username}</p>
                              {user.isPlatformCreator && (
                                <Badge variant="outline" className="text-[8px] px-1 py-0 border-amber-500/50 text-amber-500">
                                  Creator
                                </Badge>
                              )}
                            </div>
                            {user.email && (
                              <p className="text-[10px] text-muted-foreground truncate">{user.email}</p>
                            )}
                          </div>
                        </div>
                        <div className="flex items-center gap-1.5 shrink-0">
                          {user.role === "sadmin" && user.isPlatformCreator ? (
                            <Badge className={`text-[9px] ${roleBadgeClass("sadmin")}`} data-testid={`badge-role-${user.id}`}>
                              {ROLE_LABELS.sadmin}
                            </Badge>
                          ) : (
                            <Select
                              value={user.role}
                              onValueChange={(role) => updateRoleMutation.mutate({ id: user.id, role })}
                            >
                              <SelectTrigger className="h-7 text-[10px] w-24 border-0 p-1" data-testid={`select-role-${user.id}`}>
                                <Badge variant={roleBadgeVariant(user.role)} className={`text-[9px] ${roleBadgeClass(user.role)}`}>
                                  {ROLE_LABELS[user.role] || user.role}
                                </Badge>
                              </SelectTrigger>
                              <SelectContent>
                                <SelectItem value="admin">Admin</SelectItem>
                                <SelectItem value="manager">Manager</SelectItem>
                                <SelectItem value="signer">Signer</SelectItem>
                                <SelectItem value="viewer">Viewer</SelectItem>
                              </SelectContent>
                            </Select>
                          )}
                          {!(user.role === "sadmin" && user.isPlatformCreator) && (
                            <Button
                              size="icon"
                              variant="ghost"
                              onClick={() => deleteUserMutation.mutate(user.id)}
                              data-testid={`button-delete-user-${user.id}`}
                            >
                              <Trash2 className="w-3 h-3 text-destructive" />
                            </Button>
                          )}
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            )}
          </TabsContent>

          <TabsContent value="tenants" className="mt-4">
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
                            {tenant.createdAt ? format(new Date(tenant.createdAt), "MMM d, yyyy") : "---"}
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
          </TabsContent>

          <TabsContent value="roles" className="mt-4">
            <Card>
              <CardHeader>
                <CardTitle className="text-sm tracking-wider uppercase">Role Permissions Matrix</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="overflow-x-auto">
                  <table className="w-full text-[11px]">
                    <thead>
                      <tr className="border-b">
                        <th className="text-left py-2 pr-3 text-muted-foreground font-medium">Role</th>
                        <th className="text-center py-2 px-2 text-muted-foreground font-medium">Create</th>
                        <th className="text-center py-2 px-2 text-muted-foreground font-medium">Edit</th>
                        <th className="text-center py-2 px-2 text-muted-foreground font-medium">Send</th>
                        <th className="text-center py-2 px-2 text-muted-foreground font-medium">Sign</th>
                        <th className="text-center py-2 px-2 text-muted-foreground font-medium">View All</th>
                        <th className="text-center py-2 px-2 text-muted-foreground font-medium">Templates</th>
                        <th className="text-center py-2 px-2 text-muted-foreground font-medium">Audit</th>
                        <th className="text-center py-2 px-2 text-muted-foreground font-medium">Invite</th>
                        <th className="text-center py-2 px-2 text-muted-foreground font-medium">SaaS</th>
                      </tr>
                    </thead>
                    <tbody>
                      {[
                        { role: "sadmin", perms: [true, true, true, true, true, true, true, true, true] },
                        { role: "admin", perms: [true, true, true, true, true, true, true, true, false] },
                        { role: "manager", perms: [true, true, true, true, true, true, true, false, false] },
                        { role: "signer", perms: [false, false, false, true, false, false, false, false, false] },
                        { role: "viewer", perms: [false, false, false, false, true, false, true, false, false] },
                      ].map(({ role, perms }) => (
                        <tr key={role} className="border-b last:border-0">
                          <td className="py-2.5 pr-3">
                            <div className="flex items-center gap-1.5">
                              <Badge variant={roleBadgeVariant(role)} className={`text-[9px] ${roleBadgeClass(role)}`}>
                                {ROLE_LABELS[role]}
                              </Badge>
                            </div>
                            <p className="text-[9px] text-muted-foreground mt-0.5">{ROLE_DESCRIPTIONS[role]}</p>
                          </td>
                          {perms.map((p, i) => (
                            <td key={i} className="text-center py-2.5 px-2">
                              {p ? (
                                <span className="text-green-500">Yes</span>
                              ) : (
                                <span className="text-muted-foreground/40">---</span>
                              )}
                            </td>
                          ))}
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="saas" className="mt-4">
            {!isSAdmin ? (
              <Card>
                <CardContent className="p-8 text-center space-y-3">
                  <Shield className="w-8 h-8 text-muted-foreground mx-auto" />
                  <p className="text-sm font-medium">SAdmin Access Required</p>
                  <p className="text-xs text-muted-foreground">
                    Only the platform creator (SAdmin) can access SaaS management controls.
                  </p>
                </CardContent>
              </Card>
            ) : (
              <div className="space-y-4">
                <div className="grid gap-4 lg:grid-cols-2">
                  <Card data-testid="card-saas-customers">
                    <CardHeader className="pb-3">
                      <CardTitle className="text-sm flex items-center gap-2">
                        <Building2 className="w-4 h-4" />
                        Customers & Tenants
                      </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-3">
                      {tenants && tenants.length > 0 ? (
                        <div className="space-y-2 max-h-48 overflow-y-auto">
                          {tenants.map((tenant) => (
                            <div key={tenant.id} className="flex items-center justify-between gap-2 p-2.5 rounded-md bg-muted/50">
                              <div className="min-w-0">
                                <p className="text-xs font-medium truncate">{tenant.name}</p>
                                <p className="text-[10px] text-muted-foreground">
                                  {users?.filter((u) => u.tenantId === tenant.id).length ?? 0} users
                                </p>
                              </div>
                              <span className="text-[9px] text-muted-foreground font-mono">{tenant.id.slice(0, 8)}</span>
                            </div>
                          ))}
                        </div>
                      ) : (
                        <p className="text-xs text-muted-foreground text-center py-4">No customers yet</p>
                      )}
                      <div className="pt-2 border-t">
                        <p className="text-[10px] font-medium text-muted-foreground uppercase tracking-wider mb-2">Invite New Customer</p>
                        <div className="flex gap-2">
                          <Input placeholder="customer@email.com" className="h-9 text-xs flex-1" data-testid="input-invite-customer" />
                          <Button size="sm" data-testid="button-invite-customer">
                            Send Invite
                          </Button>
                        </div>
                      </div>
                    </CardContent>
                  </Card>

                  <Card data-testid="card-saas-settings">
                    <CardHeader className="pb-3">
                      <CardTitle className="text-sm flex items-center gap-2">
                        <Settings2 className="w-4 h-4" />
                        SaaS Settings
                      </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-4">
                      <div className="space-y-1.5">
                        <Label className="text-xs text-muted-foreground">Pricing Tier</Label>
                        <Select defaultValue={saasSettings?.pricingTier || "enterprise"}>
                          <SelectTrigger className="h-9 text-xs" data-testid="select-pricing-tier">
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="freemium">Freemium</SelectItem>
                            <SelectItem value="pro">Pro ($49/mo)</SelectItem>
                            <SelectItem value="enterprise">Enterprise (custom)</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                      <div className="space-y-1.5">
                        <Label className="text-xs text-muted-foreground">Max Envelopes per Tenant</Label>
                        <Input type="number" defaultValue={saasSettings?.maxEnvelopesPerTenant || 500} className="h-9 text-xs" data-testid="input-max-envelopes" />
                      </div>
                      <div className="space-y-1.5">
                        <Label className="text-xs text-muted-foreground">Max Users per Tenant</Label>
                        <Input type="number" defaultValue={saasSettings?.maxUsersPerTenant || 50} className="h-9 text-xs" data-testid="input-max-users" />
                      </div>
                      <Button size="sm" className="w-full" data-testid="button-save-saas-settings">
                        Save SaaS Settings
                      </Button>
                    </CardContent>
                  </Card>
                </div>

                <Card data-testid="card-platform-features">
                  <CardHeader className="pb-3">
                    <CardTitle className="text-sm flex items-center gap-2">
                      <Zap className="w-4 h-4" />
                      Platform Features
                    </CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="grid grid-cols-2 md:grid-cols-4 gap-2">
                      {saasSettings?.features && Object.entries(saasSettings.features).map(([key, enabled]) => (
                        <div key={key} className="flex items-center gap-2 p-2 rounded-md bg-muted/50">
                          <div className={`w-2 h-2 rounded-full ${enabled ? "bg-green-500" : "bg-muted-foreground/30"}`} />
                          <span className="text-[11px] capitalize">{key.replace(/([A-Z])/g, " $1").trim()}</span>
                        </div>
                      ))}
                    </div>
                    {saasSettings?.platform && (
                      <div className="flex flex-wrap gap-2 mt-3 pt-3 border-t">
                        <Badge variant="outline" className="text-[9px]">v{saasSettings.platform.version}</Badge>
                        <Badge variant="outline" className="text-[9px]">PlenumNET v{saasSettings.platform.plenumVersion}</Badge>
                        <Badge variant="outline" className="text-[9px]">Phase {saasSettings.platform.phase}</Badge>
                        <Badge variant="outline" className="text-[9px]">{saasSettings.platform.encryption}</Badge>
                      </div>
                    )}
                  </CardContent>
                </Card>
              </div>
            )}
          </TabsContent>
        </Tabs>

        <Card>
          <CardHeader>
            <CardTitle className="text-sm tracking-wider uppercase">SaaS Architecture</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3 text-sm leading-relaxed text-muted-foreground">
            <p>
              Sign Here operates as a multi-tenant SaaS platform. Each organization (tenant) has complete data isolation enforced through PostgreSQL row-level security policies. Users belong to tenants and interact only with their organization's envelopes, recipients, and audit data.
            </p>
            <p>
              The platform uses a 5-tier role model: <strong className="text-foreground">SAdmin</strong> (platform creator with exclusive SaaS controls), <strong className="text-foreground">Admin</strong> (tenant owners), <strong className="text-foreground">Manager</strong> (envelope oversight), <strong className="text-foreground">Signer</strong> (document signing), and <strong className="text-foreground">Viewer</strong> (read-only compliance). Tenant headers (<code className="text-[11px] bg-muted px-1 py-0.5 rounded">x-tenant-id</code>) are validated on every API request to ensure cross-tenant access is impossible.
            </p>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
