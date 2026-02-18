import { useState, useMemo } from "react";
import { useQuery, useMutation } from "@tanstack/react-query";
import { Link } from "wouter";
import { format } from "date-fns";
import {
  FilePlus,
  FileText,
  Clock,
  CheckCircle2,
  Send,
  MoreHorizontal,
  Trash2,
  Eye,
  Pencil,
  Search,
  PanelLeftOpen,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { Badge } from "@/components/ui/badge";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { StatusBadge } from "@/components/status-badge";
import { queryClient, apiRequest } from "@/lib/queryClient";
import { useToast } from "@/hooks/use-toast";
import { useDashboardFilters } from "@/lib/dashboard-context";
import { DashboardSubSidebar } from "@/components/dashboard-sidebar";
import type { Envelope, WbsTag, EnvelopeWbsTag } from "@shared/schema";

const STATUS_ORDER: Record<string, number> = {
  draft: 0,
  sent: 1,
  signing: 2,
  completed: 3,
};

export default function Dashboard() {
  const { toast } = useToast();
  const [subSidebarOpen, setSubSidebarOpen] = useState(true);
  const {
    searchQuery,
    statusFilter,
    setStatusFilter,
    wbsFilter,
    sortField,
    sortDirection,
  } = useDashboardFilters();

  const { data: envelopes, isLoading } = useQuery<Envelope[]>({
    queryKey: ["/api/envelopes"],
  });

  const { data: wbsTags } = useQuery<WbsTag[]>({
    queryKey: ["/api/wbs-tags"],
  });

  const { data: allEnvTags } = useQuery<EnvelopeWbsTag[]>({
    queryKey: ["/api/envelope-wbs-tags"],
  });

  const envTagMap = useMemo(() => {
    const map: Record<string, string[]> = {};
    allEnvTags?.forEach((et) => {
      if (!map[et.envelopeId]) map[et.envelopeId] = [];
      map[et.envelopeId].push(et.wbsTagId);
    });
    return map;
  }, [allEnvTags]);

  const deleteMutation = useMutation({
    mutationFn: async (id: string) => {
      await apiRequest("DELETE", `/api/envelopes/${id}`);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["/api/envelopes"] });
      toast({ title: "Envelope deleted" });
    },
  });

  const sendMutation = useMutation({
    mutationFn: async (id: string) => {
      await apiRequest("PATCH", `/api/envelopes/${id}`, { status: "sent" });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["/api/envelopes"] });
      toast({ title: "Envelope sent for signing" });
    },
  });

  const stats = envelopes
    ? {
        total: envelopes.length,
        draft: envelopes.filter((e) => e.status === "draft").length,
        sent: envelopes.filter((e) => e.status === "sent" || e.status === "signing").length,
        completed: envelopes.filter((e) => e.status === "completed").length,
      }
    : { total: 0, draft: 0, sent: 0, completed: 0 };

  const wbsTagMap = useMemo(() => {
    const map: Record<string, WbsTag> = {};
    wbsTags?.forEach((t) => { map[t.id] = t; });
    return map;
  }, [wbsTags]);

  const filteredEnvelopes = useMemo(() => {
    if (!envelopes) return [];

    let result = envelopes.filter((e) => {
      const matchesSearch =
        !searchQuery ||
        e.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
        (e.description && e.description.toLowerCase().includes(searchQuery.toLowerCase()));
      const matchesStatus =
        statusFilter === "all" ||
        e.status === statusFilter ||
        (statusFilter === "sent" && e.status === "signing");
      const eTags = envTagMap[e.id] || [];
      const matchesWbs =
        wbsFilter === "all" ||
        (wbsFilter === "untagged" ? eTags.length === 0 : eTags.includes(wbsFilter));
      return matchesSearch && matchesStatus && matchesWbs;
    });

    result.sort((a, b) => {
      let cmp = 0;
      switch (sortField) {
        case "title":
          cmp = a.title.localeCompare(b.title);
          break;
        case "date":
          cmp = new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime();
          break;
        case "updated":
          cmp = new Date(a.updatedAt || a.createdAt).getTime() - new Date(b.updatedAt || b.createdAt).getTime();
          break;
        case "status":
          cmp = (STATUS_ORDER[a.status] ?? 99) - (STATUS_ORDER[b.status] ?? 99);
          break;
      }
      return sortDirection === "asc" ? cmp : -cmp;
    });

    return result;
  }, [envelopes, searchQuery, statusFilter, wbsFilter, sortField, sortDirection, envTagMap]);

  const hasActiveFilters = searchQuery || statusFilter !== "all" || wbsFilter !== "all";

  return (
    <div className="flex-1 flex overflow-hidden">
      <DashboardSubSidebar open={subSidebarOpen} onClose={() => setSubSidebarOpen(false)} />

      <div className="flex-1 overflow-auto">
        <div className="max-w-5xl mx-auto p-3 space-y-2.5">
          <div className="flex items-center justify-between gap-2 flex-wrap">
            <div className="flex items-center gap-1.5">
              {!subSidebarOpen && (
                <Button
                  size="icon"
                  variant="ghost"
                  onClick={() => setSubSidebarOpen(true)}
                  data-testid="button-open-sub-sidebar"
                >
                  <PanelLeftOpen className="w-3.5 h-3.5" />
                </Button>
              )}
              <h1 className="text-sm font-semibold tracking-tight" data-testid="text-dashboard-title">
                Dashboard
              </h1>
            </div>
            <div className="flex items-center gap-2">
              {hasActiveFilters && envelopes && (
                <span className="text-[9px] text-muted-foreground" data-testid="text-result-count">
                  {filteredEnvelopes.length} of {envelopes.length}
                </span>
              )}
              <Link href="/new">
                <Button size="sm" data-testid="button-new-envelope">
                  <FilePlus className="w-3.5 h-3.5" />
                  New Envelope
                </Button>
              </Link>
            </div>
          </div>

          <div className="grid grid-cols-4 gap-1.5">
            {[
              { label: "Total", value: stats.total, icon: FileText, color: "text-foreground", filter: "all" },
              { label: "Drafts", value: stats.draft, icon: Clock, color: "text-muted-foreground", filter: "draft" },
              { label: "In Progress", value: stats.sent, icon: Send, color: "text-primary", filter: "sent" },
              { label: "Completed", value: stats.completed, icon: CheckCircle2, color: "text-emerald-600 dark:text-emerald-400", filter: "completed" },
            ].map((stat) => (
              <Card
                key={stat.label}
                className={`cursor-pointer hover-elevate ${statusFilter === stat.filter ? "ring-1 ring-primary" : ""}`}
                onClick={() => setStatusFilter(stat.filter)}
                data-testid={`card-filter-${stat.label.toLowerCase().replace(/\s+/g, "-")}`}
              >
                <CardContent className="px-2.5 py-2">
                  <div className="flex items-center justify-between gap-1">
                    <span className="text-[9px] text-muted-foreground uppercase tracking-wider font-medium">{stat.label}</span>
                    <stat.icon className={`w-2.5 h-2.5 ${stat.color}`} />
                  </div>
                  {isLoading ? (
                    <Skeleton className="h-5 w-6 mt-0.5" />
                  ) : (
                    <p className={`text-base font-semibold mt-0.5 ${stat.color}`} data-testid={`text-stat-${stat.label.toLowerCase()}`}>
                      {stat.value}
                    </p>
                  )}
                </CardContent>
              </Card>
            ))}
          </div>

          <div>
            {isLoading ? (
              <div className="space-y-0.5">
                {[1, 2, 3].map((i) => (
                  <Card key={i}>
                    <CardContent className="px-2.5 py-1.5">
                      <div className="flex items-center gap-2">
                        <Skeleton className="h-6 w-6 rounded" />
                        <div className="flex-1 space-y-1">
                          <Skeleton className="h-3 w-36" />
                          <Skeleton className="h-2 w-20" />
                        </div>
                        <Skeleton className="h-3.5 w-12" />
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            ) : !envelopes || envelopes.length === 0 ? (
              <Card>
                <CardContent className="p-10 flex flex-col items-center justify-center text-center">
                  <div className="w-10 h-10 rounded-full bg-muted flex items-center justify-center mb-3">
                    <FileText className="w-4 h-4 text-muted-foreground" />
                  </div>
                  <h3 className="text-xs font-medium mb-0.5">No envelopes yet</h3>
                  <p className="text-[10px] text-muted-foreground mb-3">
                    Create your first envelope to get started
                  </p>
                  <Link href="/new">
                    <Button size="sm" data-testid="button-empty-new">
                      <FilePlus className="w-3.5 h-3.5" />
                      Create Envelope
                    </Button>
                  </Link>
                </CardContent>
              </Card>
            ) : filteredEnvelopes.length === 0 ? (
              <Card>
                <CardContent className="p-8 flex flex-col items-center justify-center text-center">
                  <Search className="w-5 h-5 text-muted-foreground mb-2" />
                  <h3 className="text-xs font-medium mb-0.5">No matching envelopes</h3>
                  <p className="text-[10px] text-muted-foreground mb-3">
                    Try adjusting your search or filters in the panel
                  </p>
                </CardContent>
              </Card>
            ) : (
              <div className="space-y-0.5">
                {filteredEnvelopes.map((envelope) => (
                  <Card key={envelope.id} className="hover-elevate">
                    <CardContent className="px-2.5 py-1.5">
                      <div className="flex items-center gap-2">
                        <div className="w-6 h-6 rounded bg-muted flex items-center justify-center shrink-0">
                          <FileText className="w-3 h-3 text-muted-foreground" />
                        </div>
                        <div className="flex-1 min-w-0">
                          <Link href={`/envelope/${envelope.id}/edit`}>
                            <h3
                              className="text-[11px] font-medium truncate cursor-pointer"
                              data-testid={`text-envelope-title-${envelope.id}`}
                            >
                              {envelope.title}
                            </h3>
                          </Link>
                          <p className="text-[9px] text-muted-foreground">
                            {format(new Date(envelope.createdAt), "MMM d, yyyy")}
                          </p>
                        </div>
                        <div className="flex items-center gap-1.5 shrink-0">
                          {(envTagMap[envelope.id] || []).map((tagId) => {
                            const tag = wbsTagMap[tagId];
                            if (!tag) return null;
                            return (
                              <Badge
                                key={tagId}
                                variant="outline"
                                className="text-[9px] no-default-active-elevate"
                                style={{
                                  borderColor: tag.color,
                                  color: tag.color,
                                }}
                                data-testid={`badge-wbs-${envelope.id}-${tagId}`}
                              >
                                {tag.name}
                              </Badge>
                            );
                          })}
                          <StatusBadge status={envelope.status} />
                          <DropdownMenu>
                            <DropdownMenuTrigger asChild>
                              <Button
                                size="icon"
                                variant="ghost"
                                data-testid={`button-menu-${envelope.id}`}
                              >
                                <MoreHorizontal className="w-3.5 h-3.5" />
                              </Button>
                            </DropdownMenuTrigger>
                            <DropdownMenuContent align="end">
                              <Link href={`/envelope/${envelope.id}/edit`}>
                                <DropdownMenuItem data-testid={`menu-edit-${envelope.id}`}>
                                  <Pencil className="w-3.5 h-3.5 mr-2" />
                                  Edit Fields
                                </DropdownMenuItem>
                              </Link>
                              <Link href={`/envelope/${envelope.id}`}>
                                <DropdownMenuItem data-testid={`menu-view-${envelope.id}`}>
                                  <Eye className="w-3.5 h-3.5 mr-2" />
                                  View Details
                                </DropdownMenuItem>
                              </Link>
                              {envelope.status === "draft" && (
                                <>
                                  <DropdownMenuItem
                                    onClick={() => sendMutation.mutate(envelope.id)}
                                    data-testid={`menu-send-${envelope.id}`}
                                  >
                                    <Send className="w-3.5 h-3.5 mr-2" />
                                    Send for Signing
                                  </DropdownMenuItem>
                                </>
                              )}
                              <DropdownMenuItem
                                className="text-destructive"
                                onClick={() => deleteMutation.mutate(envelope.id)}
                                data-testid={`menu-delete-${envelope.id}`}
                              >
                                <Trash2 className="w-3.5 h-3.5 mr-2" />
                                Delete
                              </DropdownMenuItem>
                            </DropdownMenuContent>
                          </DropdownMenu>
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
