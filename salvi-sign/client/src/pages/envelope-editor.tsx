import { useState, useRef, useCallback, useEffect } from "react";
import { useRoute, useLocation, Link } from "wouter";
import { useQuery, useMutation } from "@tanstack/react-query";
import {
  ArrowLeft,
  PenLine,
  CalendarDays,
  Type,
  CheckSquare,
  Hash,
  Trash2,
  Send,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { apiRequest, queryClient } from "@/lib/queryClient";
import { useToast } from "@/hooks/use-toast";
import type { Envelope, Recipient, Field as FieldType } from "@shared/schema";

const FIELD_TOOLS = [
  { type: "signature", icon: PenLine, label: "Signature", w: 200, h: 60 },
  { type: "date", icon: CalendarDays, label: "Date", w: 140, h: 36 },
  { type: "text", icon: Type, label: "Text", w: 180, h: 36 },
  { type: "checkbox", icon: CheckSquare, label: "Checkbox", w: 28, h: 28 },
  { type: "initials", icon: Hash, label: "Initials", w: 80, h: 40 },
];

const RECIPIENT_COLORS = [
  "border-amber-500 bg-amber-500/10",
  "border-emerald-500 bg-emerald-500/10",
  "border-violet-500 bg-violet-500/10",
  "border-sky-500 bg-sky-500/10",
  "border-rose-500 bg-rose-500/10",
];

export default function EnvelopeEditor() {
  const [, params] = useRoute("/envelope/:id/edit");
  const [, setLocation] = useLocation();
  const { toast } = useToast();
  const canvasRef = useRef<HTMLDivElement>(null);
  const [selectedField, setSelectedField] = useState<string | null>(null);
  const [dragField, setDragField] = useState<{ type: string; w: number; h: number } | null>(null);
  const [selectedRecipient, setSelectedRecipient] = useState<string>("");
  const [localFields, setLocalFields] = useState<FieldType[]>([]);
  const [initialized, setInitialized] = useState(false);

  const envelopeId = params?.id || "";

  const { data: envelope, isLoading: envLoading } = useQuery<Envelope>({
    queryKey: ["/api/envelopes", envelopeId],
  });

  const { data: recipients } = useQuery<Recipient[]>({
    queryKey: ["/api/envelopes", envelopeId, "recipients"],
  });

  const { data: existingFields } = useQuery<FieldType[]>({
    queryKey: ["/api/envelopes", envelopeId, "fields"],
  });

  useEffect(() => {
    if (existingFields && !initialized) {
      setLocalFields(existingFields);
      setInitialized(true);
    }
  }, [existingFields, initialized]);

  useEffect(() => {
    if (recipients && recipients.length > 0 && !selectedRecipient) {
      setSelectedRecipient(recipients[0].id);
    }
  }, [recipients, selectedRecipient]);

  const saveMutation = useMutation({
    mutationFn: async (fieldsData: FieldType[]) => {
      await apiRequest("PUT", `/api/envelopes/${envelopeId}/fields`, { fields: fieldsData });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["/api/envelopes", envelopeId, "fields"] });
      toast({ title: "Fields saved" });
    },
  });

  const sendMutation = useMutation({
    mutationFn: async () => {
      await apiRequest("PUT", `/api/envelopes/${envelopeId}/fields`, { fields: localFields });
      await apiRequest("PATCH", `/api/envelopes/${envelopeId}`, { status: "sent" });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["/api/envelopes"] });
      toast({ title: "Envelope sent for signing" });
      setLocation(`/envelope/${envelopeId}`);
    },
  });

  const handleCanvasDrop = useCallback(
    (e: React.MouseEvent) => {
      if (!dragField || !canvasRef.current) return;

      const rect = canvasRef.current.getBoundingClientRect();
      const x = Math.max(0, Math.min(Math.round(e.clientX - rect.left - dragField.w / 2), rect.width - dragField.w));
      const y = Math.max(0, Math.min(Math.round(e.clientY - rect.top - dragField.h / 2), rect.height - dragField.h));

      const newField: FieldType = {
        id: `temp-${Date.now()}`,
        envelopeId,
        recipientId: selectedRecipient || null,
        type: dragField.type,
        label: null,
        page: 1,
        x,
        y,
        width: dragField.w,
        height: dragField.h,
        value: null,
        required: true,
      };

      setLocalFields((prev) => [...prev, newField]);
      setDragField(null);
    },
    [dragField, envelopeId, selectedRecipient]
  );

  const handleFieldDrag = useCallback(
    (fieldId: string, e: React.MouseEvent) => {
      e.stopPropagation();
      if (!canvasRef.current) return;

      const rect = canvasRef.current.getBoundingClientRect();
      const startX = e.clientX;
      const startY = e.clientY;
      const field = localFields.find((f) => f.id === fieldId);
      if (!field) return;

      const origX = field.x;
      const origY = field.y;

      const onMove = (ev: MouseEvent) => {
        const dx = ev.clientX - startX;
        const dy = ev.clientY - startY;
        const newX = Math.max(0, Math.min(origX + dx, rect.width - field.width));
        const newY = Math.max(0, Math.min(origY + dy, rect.height - field.height));

        setLocalFields((prev) =>
          prev.map((f) => (f.id === fieldId ? { ...f, x: newX, y: newY } : f))
        );
      };

      const onUp = () => {
        document.removeEventListener("mousemove", onMove);
        document.removeEventListener("mouseup", onUp);
      };

      document.addEventListener("mousemove", onMove);
      document.addEventListener("mouseup", onUp);
    },
    [localFields]
  );

  const removeField = (id: string) => {
    setLocalFields((prev) => prev.filter((f) => f.id !== id));
    if (selectedField === id) setSelectedField(null);
  };

  const getRecipientIndex = (recipientId: string | null) => {
    if (!recipients || !recipientId) return 0;
    return recipients.findIndex((r) => r.id === recipientId);
  };

  if (envLoading) {
    return (
      <div className="flex-1 p-5">
        <Skeleton className="h-6 w-44 mb-3" />
        <Skeleton className="h-[500px] w-full" />
      </div>
    );
  }

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      <div className="flex items-center justify-between gap-4 px-3.5 py-2 border-b flex-wrap">
        <div className="flex items-center gap-2.5">
          <Link href={`/envelope/${envelopeId}`}>
            <Button size="icon" variant="ghost" data-testid="button-editor-back">
              <ArrowLeft className="w-3.5 h-3.5" />
            </Button>
          </Link>
          <div>
            <h1 className="text-xs font-semibold" data-testid="text-editor-title">
              {envelope?.title || "Untitled"}
            </h1>
            <p className="text-[10px] text-muted-foreground">Place fields on the document</p>
          </div>
        </div>
        <div className="flex items-center gap-1.5">
          <Button
            variant="outline"
            size="sm"
            onClick={() => saveMutation.mutate(localFields)}
            disabled={saveMutation.isPending}
            data-testid="button-save-fields"
          >
            {saveMutation.isPending ? "Saving..." : "Save"}
          </Button>
          <Button
            size="sm"
            onClick={() => sendMutation.mutate()}
            disabled={sendMutation.isPending}
            data-testid="button-send-envelope"
          >
            <Send className="w-3 h-3" />
            {sendMutation.isPending ? "Sending..." : "Send"}
          </Button>
        </div>
      </div>

      <div className="flex flex-1 overflow-hidden">
        <div className="w-48 shrink-0 border-r p-3 space-y-3.5 overflow-y-auto bg-sidebar">
          <div>
            <p className="text-[9px] font-medium text-muted-foreground uppercase tracking-widest mb-1.5">
              Assign to
            </p>
            {recipients && recipients.length > 0 && (
              <Select value={selectedRecipient} onValueChange={setSelectedRecipient}>
                <SelectTrigger data-testid="select-assign-recipient">
                  <SelectValue placeholder="Select recipient" />
                </SelectTrigger>
                <SelectContent>
                  {recipients.map((r, i) => (
                    <SelectItem key={r.id} value={r.id}>
                      <span className="flex items-center gap-2">
                        <span
                          className={`w-1.5 h-1.5 rounded-full shrink-0 ${
                            RECIPIENT_COLORS[i % RECIPIENT_COLORS.length].split(" ")[0].replace("border-", "bg-")
                          }`}
                        />
                        <span className="text-xs">{r.name}</span>
                      </span>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            )}
          </div>

          <div>
            <p className="text-[9px] font-medium text-muted-foreground uppercase tracking-widest mb-1.5">
              Field Tools
            </p>
            <div className="space-y-0.5">
              {FIELD_TOOLS.map((tool) => (
                <Button
                  key={tool.type}
                  variant={dragField?.type === tool.type ? "secondary" : "ghost"}
                  size="sm"
                  className="w-full justify-start"
                  onClick={() =>
                    setDragField(
                      dragField?.type === tool.type ? null : { type: tool.type, w: tool.w, h: tool.h }
                    )
                  }
                  data-testid={`button-tool-${tool.type}`}
                >
                  <tool.icon className="w-3 h-3" />
                  <span className="text-xs">{tool.label}</span>
                </Button>
              ))}
            </div>
          </div>

          {localFields.length > 0 && (
            <div>
              <p className="text-[9px] font-medium text-muted-foreground uppercase tracking-widest mb-1.5">
                Placed ({localFields.length})
              </p>
              <div className="space-y-0.5">
                {localFields.map((f) => {
                  const ri = getRecipientIndex(f.recipientId);
                  return (
                    <div
                      key={f.id}
                      className={`flex items-center justify-between gap-1 p-1.5 rounded-md text-[11px] cursor-pointer ${
                        selectedField === f.id ? "bg-accent" : ""
                      } hover-elevate`}
                      onClick={() => setSelectedField(f.id)}
                      data-testid={`field-item-${f.id}`}
                    >
                      <span className="flex items-center gap-1.5 truncate">
                        <span
                          className={`w-1.5 h-1.5 rounded-full shrink-0 ${
                            RECIPIENT_COLORS[ri % RECIPIENT_COLORS.length].split(" ")[0].replace("border-", "bg-")
                          }`}
                        />
                        <span className="capitalize truncate">{f.type}</span>
                      </span>
                      <Button
                        size="icon"
                        variant="ghost"
                        className="w-5 h-5"
                        onClick={(e) => {
                          e.stopPropagation();
                          removeField(f.id);
                        }}
                        data-testid={`button-remove-field-${f.id}`}
                      >
                        <Trash2 className="w-2.5 h-2.5" />
                      </Button>
                    </div>
                  );
                })}
              </div>
            </div>
          )}
        </div>

        <div className="flex-1 overflow-auto bg-muted/30 p-5 flex items-start justify-center">
          <div
            ref={canvasRef}
            className={`relative bg-background border rounded-md ${
              dragField ? "cursor-crosshair" : ""
            }`}
            style={{ width: 595, height: 842 }}
            onClick={dragField ? handleCanvasDrop : undefined}
            data-testid="canvas-document"
          >
            <div className="absolute inset-0 flex flex-col items-center justify-center opacity-[0.02] pointer-events-none select-none">
              <div className="text-[72px] font-bold tracking-tighter">SALVISIGN</div>
              <div className="text-sm tracking-[0.3em] uppercase">Document</div>
            </div>

            <div className="absolute top-6 left-6 right-6 space-y-3 pointer-events-none select-none opacity-15">
              {[...Array(24)].map((_, i) => (
                <div
                  key={i}
                  className="h-1.5 bg-muted-foreground/20 rounded-sm"
                  style={{ width: `${60 + Math.sin(i * 0.8) * 30}%` }}
                />
              ))}
            </div>

            {localFields.map((f) => {
              const ri = getRecipientIndex(f.recipientId);
              const colorClass = RECIPIENT_COLORS[ri % RECIPIENT_COLORS.length];
              const isSelected = selectedField === f.id;

              return (
                <div
                  key={f.id}
                  className={`absolute border-2 border-dashed rounded-sm flex items-center justify-center cursor-move ${colorClass} ${
                    isSelected ? "ring-2 ring-ring" : ""
                  }`}
                  style={{
                    left: f.x,
                    top: f.y,
                    width: f.width,
                    height: f.height,
                  }}
                  onClick={(e) => {
                    e.stopPropagation();
                    setSelectedField(f.id);
                  }}
                  onMouseDown={(e) => handleFieldDrag(f.id, e)}
                  data-testid={`field-canvas-${f.id}`}
                >
                  <span className="text-[9px] font-medium capitalize opacity-60 select-none pointer-events-none tracking-wide">
                    {f.type}
                  </span>
                </div>
              );
            })}

            {dragField && (
              <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
                <p className="text-[10px] text-muted-foreground bg-background/80 px-2.5 py-1 rounded-md">
                  Click to place {dragField.type} field
                </p>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
