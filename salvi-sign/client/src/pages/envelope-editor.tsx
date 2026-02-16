import { useState, useRef, useCallback, useEffect } from "react";
import { useRoute, useLocation, Link } from "wouter";
import { useQuery, useMutation } from "@tanstack/react-query";
import { Document, Page, pdfjs } from "react-pdf";
import "react-pdf/dist/Page/AnnotationLayer.css";
import "react-pdf/dist/Page/TextLayer.css";
import {
  ArrowLeft,
  PenLine,
  CalendarDays,
  Type,
  CheckSquare,
  Hash,
  Trash2,
  Send,
  Upload,
  FileText,
  ZoomIn,
  ZoomOut,
  GripVertical,
  ChevronLeft,
  ChevronRight,
  Undo2,
  Redo2,
  Save,
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

pdfjs.GlobalWorkerOptions.workerSrc = `https://unpkg.com/pdfjs-dist@${pdfjs.version}/build/pdf.worker.min.mjs`;

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

const SNAP_GRID = 5;
const PDF_BASE_WIDTH = 800;
const DRAG_THRESHOLD = 4;
const MAX_UNDO = 50;

function useUndoRedo(initial: FieldType[]) {
  const [history, setHistory] = useState<FieldType[][]>([initial]);
  const [pointer, setPointer] = useState(0);

  const current = history[pointer];

  const push = useCallback((next: FieldType[]) => {
    setHistory((prev) => {
      const trimmed = prev.slice(0, pointer + 1);
      const updated = [...trimmed, next];
      const shifted = updated.length > MAX_UNDO;
      if (shifted) updated.shift();
      return updated;
    });
    setPointer((p) => {
      const newP = p + 1;
      return newP >= MAX_UNDO ? MAX_UNDO - 1 : newP;
    });
  }, [pointer]);

  const undo = useCallback(() => {
    setPointer((p) => Math.max(0, p - 1));
  }, []);

  const redo = useCallback(() => {
    setPointer((p) => Math.min(p + 1, history.length - 1));
  }, [history.length]);

  const reset = useCallback((fields: FieldType[]) => {
    setHistory([fields]);
    setPointer(0);
  }, []);

  return {
    fields: current,
    push,
    undo,
    redo,
    reset,
    canUndo: pointer > 0,
    canRedo: pointer < history.length - 1,
  };
}

export default function EnvelopeEditor() {
  const [, params] = useRoute("/envelope/:id/edit");
  const [, setLocation] = useLocation();
  const { toast } = useToast();
  const pageRefs = useRef<Map<number, HTMLDivElement>>(new Map());
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [selectedField, setSelectedField] = useState<string | null>(null);
  const [dragTool, setDragTool] = useState<{ type: string; w: number; h: number } | null>(null);
  const [selectedRecipient, setSelectedRecipient] = useState<string>("");
  const [initialized, setInitialized] = useState(false);
  const [numPages, setNumPages] = useState(0);
  const [pdfZoom, setPdfZoom] = useState(1);
  const [activePage, setActivePage] = useState(1);
  const [isDragging, setIsDragging] = useState(false);
  const justDraggedRef = useRef(false);
  const [reorderDragIdx, setReorderDragIdx] = useState<number | null>(null);
  const [reorderOverIdx, setReorderOverIdx] = useState<number | null>(null);

  const envelopeId = params?.id || "";

  const {
    fields: localFields,
    push: pushHistory,
    undo,
    redo,
    reset: resetHistory,
    canUndo,
    canRedo,
  } = useUndoRedo([]);

  const { data: envelope, isLoading: envLoading } = useQuery<Envelope>({
    queryKey: ["/api/envelopes", envelopeId],
  });

  const { data: recipients } = useQuery<Recipient[]>({
    queryKey: ["/api/envelopes", envelopeId, "recipients"],
  });

  const { data: existingFields } = useQuery<FieldType[]>({
    queryKey: ["/api/envelopes", envelopeId, "fields"],
  });

  const hasPdf = !!envelope?.pdfData;
  const pdfUrl = hasPdf ? `/api/envelopes/${envelopeId}/pdf` : null;

  useEffect(() => {
    if (existingFields && !initialized) {
      resetHistory(existingFields);
      setInitialized(true);
    }
  }, [existingFields, initialized, resetHistory]);

  useEffect(() => {
    if (recipients && recipients.length > 0 && !selectedRecipient) {
      setSelectedRecipient(recipients[0].id);
    }
  }, [recipients, selectedRecipient]);

  useEffect(() => {
    if (envelope?.pageCount && envelope.pageCount > 0) {
      setNumPages(envelope.pageCount);
    }
  }, [envelope]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "z") {
        e.preventDefault();
        if (e.shiftKey) {
          redo();
        } else {
          undo();
        }
      }
      if ((e.ctrlKey || e.metaKey) && e.key === "y") {
        e.preventDefault();
        redo();
      }
      if (e.key === "Delete" || e.key === "Backspace") {
        if (selectedField && document.activeElement === document.body) {
          e.preventDefault();
          const next = localFields.filter((f) => f.id !== selectedField);
          pushHistory(next);
          setSelectedField(null);
        }
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [undo, redo, selectedField, localFields, pushHistory]);

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

  const uploadMutation = useMutation({
    mutationFn: async (file: File) => {
      const arrayBuffer = await file.arrayBuffer();
      const base64 = btoa(
        new Uint8Array(arrayBuffer).reduce((data, byte) => data + String.fromCharCode(byte), "")
      );
      let pages = 1;
      try {
        const loadingTask = pdfjs.getDocument({ data: arrayBuffer.slice(0) });
        const pdf = await loadingTask.promise;
        pages = pdf.numPages;
      } catch {}
      await apiRequest("POST", `/api/envelopes/${envelopeId}/upload-pdf`, {
        pdfData: base64,
        pageCount: pages,
      });
      return pages;
    },
    onSuccess: (pages) => {
      setNumPages(pages);
      queryClient.invalidateQueries({ queryKey: ["/api/envelopes", envelopeId] });
      toast({ title: "PDF uploaded", description: `${pages} page(s) loaded` });
    },
    onError: (error: Error) => {
      toast({ title: "Upload failed", description: error.message, variant: "destructive" });
    },
  });

  const handleFileUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    if (file.type !== "application/pdf") {
      toast({ title: "Invalid file", description: "Please select a PDF", variant: "destructive" });
      return;
    }
    uploadMutation.mutate(file);
  };

  const handlePageClick = useCallback(
    (pageNum: number, e: React.MouseEvent) => {
      if (justDraggedRef.current) {
        justDraggedRef.current = false;
        return;
      }
      if (!dragTool) {
        setSelectedField(null);
        return;
      }

      const pageEl = pageRefs.current.get(pageNum);
      if (!pageEl) return;

      const rect = pageEl.getBoundingClientRect();
      const scale = pdfZoom;
      const rawX = (e.clientX - rect.left) / scale;
      const rawY = (e.clientY - rect.top) / scale;
      const x = Math.max(0, Math.round(rawX - dragTool.w / 2));
      const y = Math.max(0, Math.round(rawY - dragTool.h / 2));
      const snappedX = Math.round(x / SNAP_GRID) * SNAP_GRID;
      const snappedY = Math.round(y / SNAP_GRID) * SNAP_GRID;

      const newField: FieldType = {
        id: `temp-${Date.now()}`,
        envelopeId,
        recipientId: selectedRecipient || null,
        type: dragTool.type,
        label: null,
        page: pageNum,
        x: snappedX,
        y: snappedY,
        width: dragTool.w,
        height: dragTool.h,
        value: null,
        required: true,
      };

      pushHistory([...localFields, newField]);
      setDragTool(null);
    },
    [dragTool, envelopeId, selectedRecipient, pdfZoom, localFields, pushHistory]
  );

  const handleFieldMouseDown = useCallback(
    (fieldId: string, e: React.MouseEvent) => {
      e.stopPropagation();
      e.preventDefault();

      const field = localFields.find((f) => f.id === fieldId);
      if (!field) return;

      setSelectedField(fieldId);

      const pageEl = pageRefs.current.get(field.page);
      if (!pageEl) return;

      const startX = e.clientX;
      const startY = e.clientY;
      const origX = field.x;
      const origY = field.y;
      const scale = pdfZoom;
      let moved = false;
      let lastX = origX;
      let lastY = origY;

      const onMove = (ev: MouseEvent) => {
        const dx = ev.clientX - startX;
        const dy = ev.clientY - startY;

        if (!moved && Math.abs(dx) < DRAG_THRESHOLD && Math.abs(dy) < DRAG_THRESHOLD) {
          return;
        }

        if (!moved) {
          moved = true;
          setIsDragging(true);
        }

        const newX = Math.round(Math.max(0, origX + dx / scale) / SNAP_GRID) * SNAP_GRID;
        const newY = Math.round(Math.max(0, origY + dy / scale) / SNAP_GRID) * SNAP_GRID;
        lastX = newX;
        lastY = newY;

        const el = document.querySelector(`[data-testid="field-canvas-${fieldId}"]`) as HTMLElement;
        if (el) {
          el.style.left = `${newX * scale}px`;
          el.style.top = `${newY * scale}px`;
        }
      };

      const onUp = () => {
        document.removeEventListener("mousemove", onMove);
        document.removeEventListener("mouseup", onUp);
        setIsDragging(false);

        if (moved) {
          justDraggedRef.current = true;
          pushHistory(
            localFields.map((f) =>
              f.id === fieldId ? { ...f, x: lastX, y: lastY } : f
            )
          );
        }
      };

      document.addEventListener("mousemove", onMove);
      document.addEventListener("mouseup", onUp);
    },
    [localFields, pdfZoom, pushHistory]
  );

  const handleFieldResize = useCallback(
    (fieldId: string, e: React.MouseEvent) => {
      e.stopPropagation();
      e.preventDefault();

      const field = localFields.find((f) => f.id === fieldId);
      if (!field) return;

      const startX = e.clientX;
      const startY = e.clientY;
      const origW = field.width;
      const origH = field.height;
      const scale = pdfZoom;
      let lastW = origW;
      let lastH = origH;

      const onMove = (ev: MouseEvent) => {
        const dx = (ev.clientX - startX) / scale;
        const dy = (ev.clientY - startY) / scale;
        lastW = Math.max(20, Math.round((origW + dx) / SNAP_GRID) * SNAP_GRID);
        lastH = Math.max(20, Math.round((origH + dy) / SNAP_GRID) * SNAP_GRID);

        const el = document.querySelector(`[data-testid="field-canvas-${fieldId}"]`) as HTMLElement;
        if (el) {
          el.style.width = `${lastW * scale}px`;
          el.style.height = `${lastH * scale}px`;
        }
      };

      const onUp = () => {
        document.removeEventListener("mousemove", onMove);
        document.removeEventListener("mouseup", onUp);

        pushHistory(
          localFields.map((f) =>
            f.id === fieldId ? { ...f, width: lastW, height: lastH } : f
          )
        );
      };

      document.addEventListener("mousemove", onMove);
      document.addEventListener("mouseup", onUp);
    },
    [localFields, pdfZoom, pushHistory]
  );

  const removeField = (id: string) => {
    pushHistory(localFields.filter((f) => f.id !== id));
    if (selectedField === id) setSelectedField(null);
  };

  const getRecipientIndex = (recipientId: string | null) => {
    if (!recipients || !recipientId) return 0;
    return recipients.findIndex((r) => r.id === recipientId);
  };

  const fieldsOnPage = (page: number) => localFields.filter((f) => f.page === page);

  const handleReorderDragStart = (idx: number) => {
    setReorderDragIdx(idx);
  };

  const handleReorderDragOver = (idx: number, e: React.DragEvent) => {
    e.preventDefault();
    setReorderOverIdx(idx);
  };

  const handleReorderDrop = (dropIdx: number) => {
    if (reorderDragIdx === null || reorderDragIdx === dropIdx) {
      setReorderDragIdx(null);
      setReorderOverIdx(null);
      return;
    }
    const reordered = [...localFields];
    const [moved] = reordered.splice(reorderDragIdx, 1);
    reordered.splice(dropIdx, 0, moved);
    pushHistory(reordered);
    setReorderDragIdx(null);
    setReorderOverIdx(null);
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
            <p className="text-[10px] text-muted-foreground">
              {hasPdf ? `${numPages} page${numPages !== 1 ? "s" : ""} | Place fields on the document` : "Upload a PDF to get started"}
            </p>
          </div>
        </div>
        <div className="flex items-center gap-1.5">
          <div className="flex items-center gap-0.5 mr-1">
            <Button
              size="icon"
              variant="ghost"
              disabled={!canUndo}
              onClick={undo}
              data-testid="button-undo"
              title="Undo (Ctrl+Z)"
            >
              <Undo2 className="w-3.5 h-3.5" />
            </Button>
            <Button
              size="icon"
              variant="ghost"
              disabled={!canRedo}
              onClick={redo}
              data-testid="button-redo"
              title="Redo (Ctrl+Shift+Z)"
            >
              <Redo2 className="w-3.5 h-3.5" />
            </Button>
          </div>
          {hasPdf && (
            <div className="flex items-center gap-1 mr-2">
              <Button
                size="icon"
                variant="ghost"
                onClick={() => setPdfZoom((z) => Math.max(0.5, z - 0.1))}
                data-testid="button-zoom-out"
              >
                <ZoomOut className="w-3.5 h-3.5" />
              </Button>
              <span className="text-[10px] text-muted-foreground tabular-nums w-10 text-center">
                {Math.round(pdfZoom * 100)}%
              </span>
              <Button
                size="icon"
                variant="ghost"
                onClick={() => setPdfZoom((z) => Math.min(2, z + 0.1))}
                data-testid="button-zoom-in"
              >
                <ZoomIn className="w-3.5 h-3.5" />
              </Button>
            </div>
          )}
          <Button
            variant="outline"
            size="sm"
            onClick={() => saveMutation.mutate(localFields)}
            disabled={saveMutation.isPending}
            data-testid="button-save-fields"
          >
            <Save className="w-3 h-3" />
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
        <div className="w-52 shrink-0 border-r p-3 space-y-3.5 overflow-y-auto bg-sidebar">
          {!hasPdf && (
            <div>
              <input
                ref={fileInputRef}
                type="file"
                accept=".pdf"
                className="hidden"
                onChange={handleFileUpload}
              />
              <Button
                variant="outline"
                size="sm"
                className="w-full"
                onClick={() => fileInputRef.current?.click()}
                disabled={uploadMutation.isPending}
                data-testid="button-upload-pdf"
              >
                <Upload className="w-3 h-3" />
                {uploadMutation.isPending ? "Uploading..." : "Upload PDF"}
              </Button>
            </div>
          )}

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
                  variant={dragTool?.type === tool.type ? "secondary" : "ghost"}
                  size="sm"
                  className="w-full justify-start"
                  onClick={() =>
                    setDragTool(
                      dragTool?.type === tool.type ? null : { type: tool.type, w: tool.w, h: tool.h }
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

          {numPages > 1 && (
            <div>
              <p className="text-[9px] font-medium text-muted-foreground uppercase tracking-widest mb-1.5">
                Pages
              </p>
              <div className="flex items-center gap-1">
                <Button
                  size="icon"
                  variant="ghost"
                  disabled={activePage <= 1}
                  onClick={() => {
                    const newPage = Math.max(1, activePage - 1);
                    setActivePage(newPage);
                    const el = pageRefs.current.get(newPage);
                    el?.scrollIntoView({ behavior: "smooth", block: "center" });
                  }}
                >
                  <ChevronLeft className="w-3 h-3" />
                </Button>
                <span className="text-[10px] text-muted-foreground tabular-nums flex-1 text-center">
                  {activePage} / {numPages}
                </span>
                <Button
                  size="icon"
                  variant="ghost"
                  disabled={activePage >= numPages}
                  onClick={() => {
                    const newPage = Math.min(numPages, activePage + 1);
                    setActivePage(newPage);
                    const el = pageRefs.current.get(newPage);
                    el?.scrollIntoView({ behavior: "smooth", block: "center" });
                  }}
                >
                  <ChevronRight className="w-3 h-3" />
                </Button>
              </div>
            </div>
          )}

          {localFields.length > 0 && (
            <div>
              <p className="text-[9px] font-medium text-muted-foreground uppercase tracking-widest mb-1.5">
                Placed ({localFields.length})
              </p>
              <div className="space-y-0.5">
                {localFields.map((f, idx) => {
                  const ri = getRecipientIndex(f.recipientId);
                  const isReorderTarget = reorderOverIdx === idx && reorderDragIdx !== idx;
                  return (
                    <div
                      key={f.id}
                      draggable
                      onDragStart={() => handleReorderDragStart(idx)}
                      onDragOver={(e) => handleReorderDragOver(idx, e)}
                      onDrop={() => handleReorderDrop(idx)}
                      onDragEnd={() => {
                        setReorderDragIdx(null);
                        setReorderOverIdx(null);
                      }}
                      className={`flex items-center justify-between gap-1 p-1.5 rounded-md text-[11px] cursor-grab ${
                        selectedField === f.id ? "bg-accent" : ""
                      } ${isReorderTarget ? "border-t-2 border-primary" : ""} ${
                        reorderDragIdx === idx ? "opacity-40" : ""
                      } hover-elevate`}
                      onClick={() => {
                        setSelectedField(f.id);
                        setActivePage(f.page);
                        const el = pageRefs.current.get(f.page);
                        el?.scrollIntoView({ behavior: "smooth", block: "center" });
                      }}
                      data-testid={`field-item-${f.id}`}
                    >
                      <span className="flex items-center gap-1.5 truncate">
                        <GripVertical className="w-2.5 h-2.5 text-muted-foreground shrink-0 cursor-grab" />
                        <span
                          className={`w-1.5 h-1.5 rounded-full shrink-0 ${
                            RECIPIENT_COLORS[ri % RECIPIENT_COLORS.length].split(" ")[0].replace("border-", "bg-")
                          }`}
                        />
                        <span className="capitalize truncate">{f.type}</span>
                        <span className="text-muted-foreground">p{f.page}</span>
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

        <div className="flex-1 overflow-auto bg-muted/30 p-5 flex flex-col items-center gap-6">
          {pdfUrl ? (
            <Document
              file={pdfUrl}
              onLoadSuccess={(doc) => {
                setNumPages(doc.numPages);
              }}
              loading={
                <div className="flex items-center justify-center py-20">
                  <Skeleton className="h-[600px] w-[500px]" />
                </div>
              }
              error={
                <div className="flex flex-col items-center justify-center py-20 text-muted-foreground">
                  <FileText className="w-8 h-8 mb-2" />
                  <p className="text-xs">Failed to load PDF</p>
                </div>
              }
            >
              {Array.from({ length: numPages }, (_, i) => {
                const pageNum = i + 1;
                const pageFields = fieldsOnPage(pageNum);
                return (
                  <div
                    key={pageNum}
                    className="relative shadow-lg mx-auto"
                    style={{ width: PDF_BASE_WIDTH * pdfZoom }}
                    ref={(el) => {
                      if (el) pageRefs.current.set(pageNum, el);
                    }}
                    onClick={(e) => handlePageClick(pageNum, e)}
                    data-testid={`pdf-page-${pageNum}`}
                  >
                    <Page
                      pageNumber={pageNum}
                      width={PDF_BASE_WIDTH * pdfZoom}
                      renderAnnotationLayer={false}
                      renderTextLayer={false}
                    />
                    <div
                      className="absolute inset-0"
                      style={{
                        cursor: dragTool ? "crosshair" : undefined,
                        pointerEvents: isDragging ? "none" : undefined,
                      }}
                    >
                      {pageFields.map((f) => {
                        const ri = getRecipientIndex(f.recipientId);
                        const colorClass = RECIPIENT_COLORS[ri % RECIPIENT_COLORS.length];
                        const isSelected = selectedField === f.id;

                        return (
                          <div
                            key={f.id}
                            className={`absolute border-2 border-dashed rounded-sm flex flex-col items-center justify-center cursor-move ${colorClass} ${
                              isSelected ? "ring-2 ring-ring" : ""
                            }`}
                            style={{
                              left: f.x * pdfZoom,
                              top: f.y * pdfZoom,
                              width: f.width * pdfZoom,
                              height: f.height * pdfZoom,
                              userSelect: "none",
                            }}
                            onClick={(e) => {
                              e.stopPropagation();
                              setSelectedField(f.id);
                            }}
                            onMouseDown={(e) => handleFieldMouseDown(f.id, e)}
                            data-testid={`field-canvas-${f.id}`}
                          >
                            {f.type === "signature" ? (
                              <>
                                <PenLine className="w-4 h-4 text-primary/50 pointer-events-none" />
                                <span className="text-[8px] font-medium text-primary/60 uppercase tracking-widest select-none pointer-events-none mt-0.5">
                                  Signature Required
                                </span>
                              </>
                            ) : f.type === "initials" ? (
                              <>
                                <Hash className="w-3 h-3 text-primary/50 pointer-events-none" />
                                <span className="text-[8px] font-medium text-primary/60 uppercase tracking-widest select-none pointer-events-none mt-0.5">
                                  Initials
                                </span>
                              </>
                            ) : f.type === "date" ? (
                              <>
                                <CalendarDays className="w-3 h-3 text-primary/50 pointer-events-none" />
                                <span className="text-[7px] font-medium text-primary/60 uppercase tracking-widest select-none pointer-events-none mt-0.5">
                                  Date (HPTP)
                                </span>
                              </>
                            ) : f.type === "checkbox" ? (
                              <CheckSquare className="w-3 h-3 text-primary/50 pointer-events-none" />
                            ) : f.type === "text" ? (
                              <>
                                <Type className="w-3 h-3 text-primary/50 pointer-events-none" />
                                <span className="text-[7px] font-medium text-primary/60 uppercase tracking-widest select-none pointer-events-none mt-0.5">
                                  Text
                                </span>
                              </>
                            ) : (
                              <span className="text-[9px] font-medium capitalize opacity-60 select-none pointer-events-none tracking-wide">
                                {f.type}
                              </span>
                            )}
                            <div
                              className="absolute bottom-0 right-0 w-3 h-3 cursor-se-resize"
                              onMouseDown={(e) => handleFieldResize(f.id, e)}
                              data-testid={`resize-handle-${f.id}`}
                            >
                              <GripVertical className="w-2.5 h-2.5 text-muted-foreground rotate-[-45deg]" />
                            </div>
                          </div>
                        );
                      })}
                    </div>
                    {dragTool && (
                      <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
                        <p className="text-[10px] text-muted-foreground bg-background/80 px-2.5 py-1 rounded-md">
                          Click to place {dragTool.type}
                        </p>
                      </div>
                    )}
                    <div className="absolute bottom-2 right-3 text-[9px] text-muted-foreground/50 select-none pointer-events-none">
                      Page {pageNum}
                    </div>
                  </div>
                );
              })}
            </Document>
          ) : (
            <div className="flex-1 flex items-center justify-center">
              <Card className="max-w-sm w-full">
                <CardContent className="p-8 flex flex-col items-center text-center">
                  <div className="w-14 h-14 rounded-full bg-muted flex items-center justify-center mb-4">
                    <FileText className="w-6 h-6 text-muted-foreground" />
                  </div>
                  <h3 className="text-sm font-semibold mb-1">No document uploaded</h3>
                  <p className="text-[11px] text-muted-foreground mb-4">
                    Upload a PDF to start placing signature fields
                  </p>
                  <input
                    type="file"
                    accept=".pdf"
                    className="hidden"
                    id="editor-upload"
                    onChange={handleFileUpload}
                  />
                  <label htmlFor="editor-upload">
                    <Button
                      size="sm"
                      asChild
                      data-testid="button-upload-pdf-center"
                    >
                      <span>
                        <Upload className="w-3 h-3" />
                        Upload PDF
                      </span>
                    </Button>
                  </label>
                </CardContent>
              </Card>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
