import { useRoute, Link, useLocation } from "wouter";
import { useQuery, useMutation } from "@tanstack/react-query";
import { format } from "date-fns";
import {
  ArrowLeft,
  Pencil,
  Send,
  Copy,
  CheckCircle2,
  User,
  Clock,
  FileText,
  ExternalLink,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { StatusBadge } from "@/components/status-badge";
import { apiRequest, queryClient } from "@/lib/queryClient";
import { useToast } from "@/hooks/use-toast";
import type { Envelope, Recipient, AuditLog } from "@shared/schema";

export default function EnvelopeDetail() {
  const [, params] = useRoute("/envelope/:id");
  const [, setLocation] = useLocation();
  const { toast } = useToast();
  const envelopeId = params?.id || "";

  const { data: envelope, isLoading } = useQuery<Envelope>({
    queryKey: ["/api/envelopes", envelopeId],
  });

  const { data: recipients } = useQuery<Recipient[]>({
    queryKey: ["/api/envelopes", envelopeId, "recipients"],
  });

  const { data: auditLogs } = useQuery<AuditLog[]>({
    queryKey: ["/api/envelopes", envelopeId, "audit"],
  });

  const sendMutation = useMutation({
    mutationFn: async () => {
      await apiRequest("PATCH", `/api/envelopes/${envelopeId}`, { status: "sent" });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["/api/envelopes", envelopeId] });
      queryClient.invalidateQueries({ queryKey: ["/api/envelopes"] });
      toast({ title: "Envelope sent for signing" });
    },
  });

  const copySignLink = (recipientId: string) => {
    const url = `${window.location.origin}/sign/${envelopeId}/${recipientId}`;
    navigator.clipboard.writeText(url);
    toast({ title: "Signing link copied to clipboard" });
  };

  if (isLoading) {
    return (
      <div className="flex-1 overflow-auto p-5">
        <div className="max-w-3xl mx-auto space-y-3">
          <Skeleton className="h-6 w-44" />
          <Skeleton className="h-28 w-full" />
          <Skeleton className="h-40 w-full" />
        </div>
      </div>
    );
  }

  if (!envelope) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <p className="text-xs text-muted-foreground">Envelope not found</p>
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-auto">
      <div className="max-w-3xl mx-auto p-5 space-y-5">
        <div className="flex items-center justify-between gap-4 flex-wrap">
          <div className="flex items-center gap-2.5">
            <Link href="/">
              <Button size="icon" variant="ghost" data-testid="button-detail-back">
                <ArrowLeft className="w-3.5 h-3.5" />
              </Button>
            </Link>
            <div>
              <div className="flex items-center gap-2">
                <h1 className="text-sm font-semibold" data-testid="text-detail-title">
                  {envelope.title}
                </h1>
                <StatusBadge status={envelope.status} />
              </div>
              {envelope.description && (
                <p className="text-[10px] text-muted-foreground mt-0.5">
                  {envelope.description}
                </p>
              )}
            </div>
          </div>
          <div className="flex items-center gap-1.5">
            {envelope.status === "draft" && (
              <>
                <Link href={`/envelope/${envelopeId}/edit`}>
                  <Button variant="outline" size="sm" data-testid="button-edit-fields">
                    <Pencil className="w-3 h-3" />
                    Edit Fields
                  </Button>
                </Link>
                <Button
                  size="sm"
                  onClick={() => sendMutation.mutate()}
                  disabled={sendMutation.isPending}
                  data-testid="button-send"
                >
                  <Send className="w-3 h-3" />
                  {sendMutation.isPending ? "Sending..." : "Send"}
                </Button>
              </>
            )}
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-2.5">
          <Card>
            <CardContent className="p-3.5">
              <div className="flex items-center gap-1.5 text-[10px] text-muted-foreground mb-1 uppercase tracking-wider">
                <Clock className="w-2.5 h-2.5" />
                Created
              </div>
              <p className="text-xs font-medium">
                {format(new Date(envelope.createdAt), "MMM d, yyyy")}
              </p>
              <p className="text-[10px] text-muted-foreground">
                {format(new Date(envelope.createdAt), "h:mm a")}
              </p>
            </CardContent>
          </Card>
          <Card>
            <CardContent className="p-3.5">
              <div className="flex items-center gap-1.5 text-[10px] text-muted-foreground mb-1 uppercase tracking-wider">
                <User className="w-2.5 h-2.5" />
                Recipients
              </div>
              <p className="text-xs font-medium">{recipients?.length || 0}</p>
              <p className="text-[10px] text-muted-foreground">
                {recipients?.filter((r) => r.status === "signed").length || 0} signed
              </p>
            </CardContent>
          </Card>
          <Card>
            <CardContent className="p-3.5">
              <div className="flex items-center gap-1.5 text-[10px] text-muted-foreground mb-1 uppercase tracking-wider">
                <FileText className="w-2.5 h-2.5" />
                Status
              </div>
              <p className="text-xs font-medium capitalize">{envelope.status}</p>
              <p className="text-[10px] text-muted-foreground">
                Updated {format(new Date(envelope.updatedAt), "MMM d")}
              </p>
            </CardContent>
          </Card>
        </div>

        <Card>
          <CardContent className="p-4">
            <h2 className="text-[10px] font-medium mb-3 uppercase tracking-wider text-muted-foreground">Recipients</h2>
            {!recipients || recipients.length === 0 ? (
              <p className="text-[10px] text-muted-foreground">No recipients added</p>
            ) : (
              <div className="space-y-1.5">
                {recipients.map((r) => (
                  <div
                    key={r.id}
                    className="flex items-center justify-between gap-3 p-2.5 rounded-md bg-muted/50"
                    data-testid={`recipient-detail-${r.id}`}
                  >
                    <div className="flex items-center gap-2.5 min-w-0">
                      <div className="w-7 h-7 rounded-full bg-muted flex items-center justify-center shrink-0">
                        <User className="w-3 h-3 text-muted-foreground" />
                      </div>
                      <div className="min-w-0">
                        <p className="text-xs font-medium truncate">{r.name}</p>
                        <p className="text-[10px] text-muted-foreground truncate">{r.email}</p>
                      </div>
                    </div>
                    <div className="flex items-center gap-1.5 shrink-0">
                      <StatusBadge status={r.status} />
                      {(envelope.status === "sent" || envelope.status === "signing") &&
                        r.status !== "signed" &&
                        r.role === "signer" && (
                          <div className="flex items-center gap-0.5">
                            <Button
                              size="icon"
                              variant="ghost"
                              onClick={() => copySignLink(r.id)}
                              data-testid={`button-copy-link-${r.id}`}
                            >
                              <Copy className="w-3 h-3" />
                            </Button>
                            <Link href={`/sign/${envelopeId}/${r.id}`}>
                              <Button
                                size="icon"
                                variant="ghost"
                                data-testid={`button-open-sign-${r.id}`}
                              >
                                <ExternalLink className="w-3 h-3" />
                              </Button>
                            </Link>
                          </div>
                        )}
                      {r.status === "signed" && r.signedAt && (
                        <span className="text-[9px] text-muted-foreground">
                          {format(new Date(r.signedAt), "MMM d, h:mm a")}
                        </span>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-4">
            <h2 className="text-[10px] font-medium mb-3 uppercase tracking-wider text-muted-foreground">Audit Trail</h2>
            {!auditLogs || auditLogs.length === 0 ? (
              <p className="text-[10px] text-muted-foreground">No activity recorded yet</p>
            ) : (
              <div className="space-y-0">
                {auditLogs.map((log, i) => (
                  <div key={log.id} className="flex gap-2.5">
                    <div className="flex flex-col items-center">
                      <div className="w-1.5 h-1.5 rounded-full bg-primary mt-1.5 shrink-0" />
                      {i < auditLogs.length - 1 && (
                        <div className="w-px flex-1 bg-border" />
                      )}
                    </div>
                    <div className="pb-3.5 min-w-0">
                      <p className="text-xs">{log.action}</p>
                      <div className="flex items-center gap-2 mt-0.5 flex-wrap">
                        <span className="text-[10px] text-muted-foreground">
                          {log.actorName}
                        </span>
                        <span className="text-[10px] text-muted-foreground">
                          {format(new Date(log.createdAt), "MMM d, h:mm a")}
                        </span>
                      </div>
                      {log.details && (
                        <p className="text-[10px] text-muted-foreground mt-0.5">
                          {log.details}
                        </p>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
