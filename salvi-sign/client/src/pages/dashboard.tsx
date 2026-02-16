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
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { StatusBadge } from "@/components/status-badge";
import { queryClient, apiRequest } from "@/lib/queryClient";
import { useToast } from "@/hooks/use-toast";
import type { Envelope } from "@shared/schema";

export default function Dashboard() {
  const { toast } = useToast();

  const { data: envelopes, isLoading } = useQuery<Envelope[]>({
    queryKey: ["/api/envelopes"],
  });

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

  return (
    <div className="flex-1 overflow-auto">
      <div className="max-w-4xl mx-auto p-5 space-y-5">
        <div className="flex items-center justify-between gap-4 flex-wrap">
          <div>
            <h1 className="text-base font-semibold tracking-tight" data-testid="text-dashboard-title">
              Dashboard
            </h1>
            <p className="text-[11px] text-muted-foreground mt-0.5 tracking-wide">
              Manage your documents and signatures
            </p>
          </div>
          <Link href="/new">
            <Button size="sm" data-testid="button-new-envelope">
              <FilePlus className="w-3.5 h-3.5" />
              New Envelope
            </Button>
          </Link>
        </div>

        <div className="grid grid-cols-2 md:grid-cols-4 gap-2.5">
          {[
            { label: "Total", value: stats.total, icon: FileText, color: "text-foreground" },
            { label: "Drafts", value: stats.draft, icon: Clock, color: "text-muted-foreground" },
            { label: "In Progress", value: stats.sent, icon: Send, color: "text-primary" },
            { label: "Completed", value: stats.completed, icon: CheckCircle2, color: "text-emerald-600 dark:text-emerald-400" },
          ].map((stat) => (
            <Card key={stat.label}>
              <CardContent className="p-3.5">
                <div className="flex items-center justify-between gap-2">
                  <span className="text-[10px] text-muted-foreground uppercase tracking-wider font-medium">{stat.label}</span>
                  <stat.icon className={`w-3 h-3 ${stat.color}`} />
                </div>
                {isLoading ? (
                  <Skeleton className="h-6 w-8 mt-1.5" />
                ) : (
                  <p className={`text-lg font-semibold mt-1 ${stat.color}`} data-testid={`text-stat-${stat.label.toLowerCase()}`}>
                    {stat.value}
                  </p>
                )}
              </CardContent>
            </Card>
          ))}
        </div>

        <div>
          <h2 className="text-[10px] font-medium text-muted-foreground mb-2.5 uppercase tracking-wider">Recent Envelopes</h2>
          {isLoading ? (
            <div className="space-y-1.5">
              {[1, 2, 3].map((i) => (
                <Card key={i}>
                  <CardContent className="p-3.5">
                    <div className="flex items-center gap-3">
                      <Skeleton className="h-8 w-8 rounded-md" />
                      <div className="flex-1 space-y-1.5">
                        <Skeleton className="h-3.5 w-44" />
                        <Skeleton className="h-2.5 w-28" />
                      </div>
                      <Skeleton className="h-4 w-14" />
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
          ) : (
            <div className="space-y-1.5">
              {envelopes.map((envelope) => (
                <Card key={envelope.id} className="hover-elevate">
                  <CardContent className="p-3.5">
                    <div className="flex items-center gap-3">
                      <div className="w-8 h-8 rounded-md bg-muted flex items-center justify-center shrink-0">
                        <FileText className="w-3.5 h-3.5 text-muted-foreground" />
                      </div>
                      <div className="flex-1 min-w-0">
                        <Link href={`/envelope/${envelope.id}`}>
                          <h3
                            className="text-xs font-medium truncate cursor-pointer"
                            data-testid={`text-envelope-title-${envelope.id}`}
                          >
                            {envelope.title}
                          </h3>
                        </Link>
                        <p className="text-[10px] text-muted-foreground mt-0.5">
                          {format(new Date(envelope.createdAt), "MMM d, yyyy")}
                        </p>
                      </div>
                      <div className="flex items-center gap-1.5 shrink-0">
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
                            <Link href={`/envelope/${envelope.id}`}>
                              <DropdownMenuItem data-testid={`menu-view-${envelope.id}`}>
                                <Eye className="w-3.5 h-3.5 mr-2" />
                                View Details
                              </DropdownMenuItem>
                            </Link>
                            {envelope.status === "draft" && (
                              <>
                                <Link href={`/envelope/${envelope.id}/edit`}>
                                  <DropdownMenuItem data-testid={`menu-edit-${envelope.id}`}>
                                    <Pencil className="w-3.5 h-3.5 mr-2" />
                                    Edit Fields
                                  </DropdownMenuItem>
                                </Link>
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
  );
}
