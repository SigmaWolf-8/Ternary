import { useState, useRef, useCallback, useEffect } from "react";
import { useRoute, useLocation, Link } from "wouter";
import { useQuery, useMutation } from "@tanstack/react-query";
import { Document, Page, pdfjs } from "react-pdf";
import "react-pdf/dist/Page/AnnotationLayer.css";
import "react-pdf/dist/Page/TextLayer.css";
import { PDFDocument } from "pdf-lib";
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
  Plus,
  CheckCircle2,
  Eraser,
  Shield,
  UserPlus,
  Pencil,
  Mail,
  User,
  X,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Skeleton } from "@/components/ui/skeleton";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { apiRequest, queryClient } from "@/lib/queryClient";
import { getSettings } from "@/pages/settings";
import { useToast } from "@/hooks/use-toast";
import { formatDateWithTimezone } from "@/pages/settings";
import type { Envelope, Recipient, Field as FieldType } from "@shared/schema";

const FONT_STYLES = [
  { name: "Elegant", fontFamily: "'Architects Daughter', cursive" },
  { name: "Classic", fontFamily: "'Libre Baskerville', serif" },
  { name: "Script", fontFamily: "'Lora', serif" },
  { name: "Vibrations", fontFamily: "'Great Vibes', cursive" },
  { name: "Dancing", fontFamily: "'Dancing Script', cursive" },
  { name: "Pacifico", fontFamily: "'Pacifico', cursive" },
  { name: "Sacramento", fontFamily: "'Sacramento', cursive" },
  { name: "Alex Brush", fontFamily: "'Alex Brush', cursive" },
];

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
  const [pdfVersion, setPdfVersion] = useState(0);
  const [sigDialogOpen, setSigDialogOpen] = useState(false);
  const [sigFieldId, setSigFieldId] = useState<string | null>(null);
  const [sigMode, setSigMode] = useState<"draw" | "type">("type");
  const [typedName, setTypedName] = useState("");
  const [selectedFont, setSelectedFont] = useState(0);
  const [isDrawing, setIsDrawing] = useState(false);
  const sigCanvasRef = useRef<HTMLCanvasElement>(null);
  const [textDialogOpen, setTextDialogOpen] = useState(false);
  const [textFieldId, setTextFieldId] = useState<string | null>(null);
  const [textValue, setTextValue] = useState("");
  const [clipboardField, setClipboardField] = useState<FieldType | null>(null);
  const [showAddRecipient, setShowAddRecipient] = useState(false);
  const [newRecipientName, setNewRecipientName] = useState("");
  const [newRecipientEmail, setNewRecipientEmail] = useState("");
  const [newRecipientRole, setNewRecipientRole] = useState("signer");
  const [editingRecipientId, setEditingRecipientId] = useState<string | null>(null);
  const [editRecipientName, setEditRecipientName] = useState("");
  const [editRecipientEmail, setEditRecipientEmail] = useState("");
  const [showSealWarning, setShowSealWarning] = useState(false);

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
  const pdfUrl = hasPdf ? `/api/envelopes/${envelopeId}/pdf?v=${pdfVersion}` : null;

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
      const res = await apiRequest("PUT", `/api/envelopes/${envelopeId}/fields`, { fields: fieldsData });
      return res.json() as Promise<FieldType[]>;
    },
    onSuccess: (savedFields) => {
      queryClient.invalidateQueries({ queryKey: ["/api/envelopes", envelopeId, "fields"] });
      if (savedFields && Array.isArray(savedFields)) {
        resetHistory(savedFields);
      }
      toast({ title: "Fields saved" });
    },
  });

  const addRecipientMutation = useMutation({
    mutationFn: async () => {
      await apiRequest("POST", `/api/envelopes/${envelopeId}/recipients`, {
        name: newRecipientName,
        email: newRecipientEmail,
        role: newRecipientRole,
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["/api/envelopes", envelopeId, "recipients"] });
      setNewRecipientName("");
      setNewRecipientEmail("");
      setNewRecipientRole("signer");
      setShowAddRecipient(false);
      toast({ title: "Recipient added" });
    },
    onError: (error: Error) => {
      toast({ title: "Error", description: error.message, variant: "destructive" });
    },
  });

  const updateRecipientMutation = useMutation({
    mutationFn: async ({ id, name, email }: { id: string; name: string; email: string }) => {
      await apiRequest("PATCH", `/api/recipients/${id}`, { name, email });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["/api/envelopes", envelopeId, "recipients"] });
      setEditingRecipientId(null);
      toast({ title: "Recipient updated" });
    },
    onError: (error: Error) => {
      toast({ title: "Error", description: error.message, variant: "destructive" });
    },
  });

  const deleteRecipientMutation = useMutation({
    mutationFn: async (id: string) => {
      await apiRequest("DELETE", `/api/recipients/${id}`);
      return id;
    },
    onSuccess: (deletedId) => {
      queryClient.invalidateQueries({ queryKey: ["/api/envelopes", envelopeId, "recipients"] });
      if (selectedRecipient === deletedId) {
        const remaining = recipients?.filter((r) => r.id !== deletedId);
        setSelectedRecipient(remaining && remaining.length > 0 ? remaining[0].id : "");
      }
      toast({ title: "Recipient removed" });
    },
    onError: (error: Error) => {
      toast({ title: "Error", description: error.message, variant: "destructive" });
    },
  });

  const doSend = useMutation({
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

  const handleSend = useCallback(() => {
    const lastPage = numPages || 1;
    const hasLastPageSeal = localFields.some((f) => f.page === lastPage && f.label === "seal");
    if (!hasLastPageSeal) {
      setShowSealWarning(true);
    } else {
      doSend.mutate();
    }
  }, [numPages, localFields, doSend]);

  const uploadMutation = useMutation({
    mutationFn: async (files: File[]) => {
      const buffers: ArrayBuffer[] = [];
      for (const file of files) {
        buffers.push(await file.arrayBuffer());
      }

      let finalBase64: string;
      let finalPages: number;

      if (hasPdf && buffers.length > 0) {
        const existingRes = await fetch(`/api/envelopes/${envelopeId}/pdf`, { credentials: "include" });
        if (!existingRes.ok) {
          throw new Error("Could not fetch existing PDF to append pages");
        }
        const existingBuf = await existingRes.arrayBuffer();
        const merged = await PDFDocument.create();
        const existingSrc = await PDFDocument.load(existingBuf);
        const existingCopied = await merged.copyPages(existingSrc, existingSrc.getPageIndices());
        existingCopied.forEach((p) => merged.addPage(p));
        for (const buf of buffers) {
          const src = await PDFDocument.load(buf);
          const pages = await merged.copyPages(src, src.getPageIndices());
          pages.forEach((p) => merged.addPage(p));
        }
        const mergedBytes = await merged.save();
        finalBase64 = btoa(new Uint8Array(mergedBytes).reduce((d, b) => d + String.fromCharCode(b), ""));
        finalPages = merged.getPageCount();
      } else if (buffers.length === 1) {
        finalBase64 = btoa(new Uint8Array(buffers[0]).reduce((d, b) => d + String.fromCharCode(b), ""));
        try {
          const loadingTask = pdfjs.getDocument({ data: buffers[0].slice(0) });
          const pdf = await loadingTask.promise;
          finalPages = pdf.numPages;
        } catch { finalPages = 1; }
      } else {
        const merged = await PDFDocument.create();
        for (const buf of buffers) {
          const src = await PDFDocument.load(buf);
          const pages = await merged.copyPages(src, src.getPageIndices());
          pages.forEach((p) => merged.addPage(p));
        }
        const mergedBytes = await merged.save();
        finalBase64 = btoa(new Uint8Array(mergedBytes).reduce((d, b) => d + String.fromCharCode(b), ""));
        finalPages = merged.getPageCount();
      }

      await apiRequest("POST", `/api/envelopes/${envelopeId}/upload-pdf`, {
        pdfData: finalBase64,
        pageCount: finalPages,
      });
      return finalPages;
    },
    onSuccess: (pages) => {
      setPdfVersion((v) => v + 1);
      setNumPages(pages);
      queryClient.invalidateQueries({ queryKey: ["/api/envelopes", envelopeId] });
      toast({ title: "PDF uploaded", description: `${pages} page(s) loaded` });
    },
    onError: (error: Error) => {
      toast({ title: "Upload failed", description: error.message, variant: "destructive" });
    },
  });

  const handleFileUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(e.target.files || []);
    if (!files.length) return;
    const valid = files.filter((f) => {
      if (f.type !== "application/pdf") {
        toast({ title: "Invalid file", description: `${f.name} is not a PDF`, variant: "destructive" });
        return false;
      }
      if (f.size > 30 * 1024 * 1024) {
        toast({ title: "File too large", description: `${f.name} exceeds 30MB`, variant: "destructive" });
        return false;
      }
      return true;
    });
    if (valid.length > 0) uploadMutation.mutate(valid);
    if (fileInputRef.current) fileInputRef.current.value = "";
  };

  const handlePageClick = useCallback(
    (pageNum: number, e: React.MouseEvent) => {
      if (justDraggedRef.current) {
        justDraggedRef.current = false;
        return;
      }

      const pageEl = pageRefs.current.get(pageNum);
      if (!pageEl) return;

      if (clipboardField) {
        const rect = pageEl.getBoundingClientRect();
        const scale = pdfZoom;
        const rawX = (e.clientX - rect.left) / scale;
        const rawY = (e.clientY - rect.top) / scale;
        const x = Math.max(0, Math.round(rawX - clipboardField.width / 2));
        const y = Math.max(0, Math.round(rawY - clipboardField.height / 2));
        const snappedX = Math.round(x / SNAP_GRID) * SNAP_GRID;
        const snappedY = Math.round(y / SNAP_GRID) * SNAP_GRID;

        const pastedField: FieldType = {
          ...clipboardField,
          id: `temp-${Date.now()}`,
          page: pageNum,
          x: snappedX,
          y: snappedY,
        };

        pushHistory([...localFields, pastedField]);
        setClipboardField(null);
        toast({ title: "Pasted", description: `${pastedField.type} field pasted.` });
        return;
      }

      if (!dragTool) {
        setSelectedField(null);
        return;
      }

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
        value: dragTool.type === "date" ? formatDateWithTimezone(new Date()) : null,
        required: true,
      };

      pushHistory([...localFields, newField]);
      setDragTool(null);
    },
    [dragTool, clipboardField, envelopeId, selectedRecipient, pdfZoom, localFields, pushHistory, toast]
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

  const updateFieldValue = useCallback((fieldId: string, value: string | null) => {
    pushHistory(localFields.map((f) => f.id === fieldId ? { ...f, value } : f));
  }, [localFields, pushHistory]);

  const handleFieldDoubleClick = useCallback((fieldId: string) => {
    const field = localFields.find((f) => f.id === fieldId);
    if (!field) return;
    if (field.label === "seal") return;

    if (field.type === "signature" || field.type === "initials") {
      setSigFieldId(fieldId);
      setSigDialogOpen(true);
      const r = recipients?.find((r) => r.id === field.recipientId);
      const settings = getSettings();
      setTypedName(r?.name || settings.displayName || "");
    } else if (field.type === "text") {
      setTextFieldId(fieldId);
      setTextValue(field.value || "");
      setTextDialogOpen(true);
    } else if (field.type === "checkbox") {
      updateFieldValue(fieldId, field.value === "checked" ? null : "checked");
    } else if (field.type === "date") {
      updateFieldValue(fieldId, formatDateWithTimezone(new Date()));
    }
  }, [localFields, recipients, updateFieldValue]);

  const clearSigCanvas = useCallback(() => {
    const canvas = sigCanvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.clearRect(0, 0, canvas.width, canvas.height);
  }, []);

  const startDraw = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = sigCanvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    setIsDrawing(true);
    const rect = canvas.getBoundingClientRect();
    ctx.beginPath();
    ctx.moveTo(e.clientX - rect.left, e.clientY - rect.top);
    ctx.strokeStyle = "hsl(40 65% 50%)";
    ctx.lineWidth = 2;
    ctx.lineCap = "round";
    ctx.lineJoin = "round";
  }, []);

  const onDraw = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    if (!isDrawing) return;
    const canvas = sigCanvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    const rect = canvas.getBoundingClientRect();
    ctx.lineTo(e.clientX - rect.left, e.clientY - rect.top);
    ctx.stroke();
  }, [isDrawing]);

  const endDraw = useCallback(() => setIsDrawing(false), []);

  const applySig = () => {
    if (!sigFieldId) return;
    if (sigMode === "type") {
      if (!typedName.trim()) return;
      updateFieldValue(sigFieldId, `typed:${selectedFont}:${typedName}`);
    } else {
      const canvas = sigCanvasRef.current;
      if (!canvas) return;
      const dataUrl = canvas.toDataURL();
      updateFieldValue(sigFieldId, `drawn:${dataUrl}`);
    }
    setSigDialogOpen(false);
    setSigFieldId(null);
  };

  const applyText = () => {
    if (textFieldId) {
      updateFieldValue(textFieldId, textValue || null);
      setTextDialogOpen(false);
      setTextFieldId(null);
      setTextValue("");
    }
  };

  const [sealGenerating, setSealGenerating] = useState(false);

  const generateSeal = useCallback(async () => {
    const pageFields = localFields.filter(
      (f) => f.page === activePage && f.type !== "initials" && f.type !== "checkbox" && f.label !== "seal"
    );
    if (pageFields.length === 0) {
      toast({ title: "No fields", description: "Place signature, date, or text fields on this page first.", variant: "destructive" });
      return;
    }

    setSealGenerating(true);

    try {
      let gpsText = "GPS: unavailable";
      if (navigator.geolocation) {
        try {
          const pos = await Promise.race<GeolocationPosition | null>([
            new Promise<GeolocationPosition>((resolve, reject) =>
              navigator.geolocation.getCurrentPosition(resolve, reject, { timeout: 2000 })
            ),
            new Promise<null>((resolve) => setTimeout(() => resolve(null), 2500)),
          ]);
          if (pos) {
            gpsText = `GPS: ${pos.coords.latitude.toFixed(6)}, ${pos.coords.longitude.toFixed(6)}`;
          } else {
            gpsText = "GPS: not available";
          }
        } catch {
          gpsText = "GPS: not available";
        }
      }

      const parts: string[] = [];
      for (const f of pageFields) {
        if (f.type === "signature" && f.value) {
          if (f.value.startsWith("typed:")) {
            const name = f.value.split(":").slice(2).join(":");
            parts.push(`SIG: ${name}`);
          } else if (f.value.startsWith("drawn:")) {
            parts.push("SIG: [drawn]");
          } else {
            parts.push(`SIG: ${f.value}`);
          }
        } else if (f.type === "signature") {
          parts.push("SIG: [pending]");
        } else if (f.type === "date" && f.value) {
          parts.push(`Date ${f.value}`);
        } else if (f.type === "date") {
          parts.push("DATE: [pending]");
        } else if (f.type === "text" && f.value) {
          parts.push(`TXT: ${f.value}`);
        } else if (f.type === "text") {
          parts.push("TXT: [pending]");
        }
      }

      const now = new Date();
      const baseTimestamp = formatDateWithTimezone(now);
      let highPrecFrac = "";
      if (typeof performance !== "undefined") {
        const preciseMs = performance.timeOrigin + performance.now();
        const fracSec = (preciseMs % 1000) / 1000;
        highPrecFrac = fracSec.toFixed(15).slice(1);
      } else {
        highPrecFrac = "." + String(now.getMilliseconds()).padStart(3, "0") + "000000000000";
      }
      const sealTimestamp = `${baseTimestamp} [${highPrecFrac}fs]`;

      const sealContent = `SEAL | ${parts.join(" | ")} | ${gpsText} | ${sealTimestamp}`;

      const sealX = 10;
      const sealWidth = 580;
      const sealHeight = 24;

      const pageEl = pageRefs.current.get(activePage);
      const pageHeight = pageEl ? pageEl.offsetHeight / pdfZoom : 1035;

      const sigFields = localFields.filter(
        (f) => f.page === activePage && f.type === "signature" && f.label !== "seal"
      );

      const sigGap = 8;
      const bottomMargin = 20;

      const sealY = Math.round(pageHeight - bottomMargin - sealHeight);

      const totalSigHeight = sigFields.reduce((sum, f) => sum + f.height + sigGap, 0);
      let currentY = sealY - totalSigHeight;

      const repositionedSigIds = new Set(sigFields.map((f) => f.id));

      const updatedFields: FieldType[] = [];
      for (const f of sigFields) {
        updatedFields.push({ ...f, x: sealX, y: Math.round(currentY) });
        currentY += f.height + sigGap;
      }

      const existingSealIdx = localFields.findIndex((f) => f.page === activePage && f.label === "seal");
      const existingSeal = existingSealIdx >= 0 ? localFields[existingSealIdx] : null;

      let newFields = localFields.map((f) => {
        if (repositionedSigIds.has(f.id)) {
          const updated = updatedFields.find((u) => u.id === f.id);
          return updated || f;
        }
        if (existingSeal && f.id === existingSeal.id) {
          return { ...f, value: sealContent, y: sealY, x: sealX, width: sealWidth };
        }
        return f;
      });

      if (!existingSeal) {
        const sealField: FieldType = {
          id: `temp-${Date.now()}`,
          envelopeId,
          recipientId: selectedRecipient || null,
          type: "text",
          label: "seal",
          page: activePage,
          x: sealX,
          y: sealY,
          width: sealWidth,
          height: sealHeight,
          value: sealContent,
          required: false,
        };
        newFields = [...newFields, sealField];
      }

      pushHistory(newFields);
      toast({ title: "Seal Generated", description: "Signatures repositioned above seal." });
    } finally {
      setSealGenerating(false);
    }
  }, [localFields, activePage, envelopeId, selectedRecipient, pushHistory, toast]);

  const removeField = (id: string) => {
    pushHistory(localFields.filter((f) => f.id !== id));
    if (selectedField === id) setSelectedField(null);
  };

  const getRecipientIndex = (recipientId: string | null) => {
    if (!recipients || !recipientId) return 0;
    const idx = recipients.findIndex((r) => r.id === recipientId);
    return idx < 0 ? 0 : idx;
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
            onClick={handleSend}
            disabled={doSend.isPending}
            data-testid="button-send-envelope"
          >
            <Send className="w-3 h-3" />
            {doSend.isPending ? "Sending..." : "Send"}
          </Button>
        </div>
      </div>

      <div className="flex flex-1 overflow-hidden">
        <div className="w-52 shrink-0 border-r p-3 space-y-3.5 overflow-y-auto bg-sidebar" style={{ boxShadow: 'inset 2px 2px 6px rgba(255,255,255,0.07), inset -2px -2px 6px rgba(0,0,0,0.4), inset 0 0 20px rgba(0,0,0,0.15)' }}>
          <div>
            <input
              ref={fileInputRef}
              type="file"
              accept=".pdf"
              multiple
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
              {hasPdf ? <Plus className="w-3 h-3" /> : <Upload className="w-3 h-3" />}
              {uploadMutation.isPending ? "Processing..." : hasPdf ? "Add Pages" : "Upload PDF"}
            </Button>
          </div>

          <div>
            <div className="flex items-center justify-between gap-1 mb-1.5">
              <p className="text-[9px] font-medium text-muted-foreground uppercase tracking-widest">
                Recipients
              </p>
              <Button
                size="icon"
                variant="ghost"
                className="h-5 w-5"
                onClick={() => setShowAddRecipient(!showAddRecipient)}
                data-testid="button-toggle-add-recipient"
              >
                <UserPlus className="w-3 h-3" />
              </Button>
            </div>
            {showAddRecipient && (
              <div className="space-y-1.5 mb-2 p-2 rounded-md bg-muted/50">
                <div className="relative">
                  <User className="absolute left-2 top-2 w-3 h-3 text-muted-foreground" />
                  <Input
                    placeholder="Name"
                    className="pl-7 h-7 text-[11px]"
                    value={newRecipientName}
                    onChange={(e) => setNewRecipientName(e.target.value)}
                    data-testid="input-new-recipient-name"
                  />
                </div>
                <div className="relative">
                  <Mail className="absolute left-2 top-2 w-3 h-3 text-muted-foreground" />
                  <Input
                    placeholder="Email"
                    className="pl-7 h-7 text-[11px]"
                    value={newRecipientEmail}
                    onChange={(e) => setNewRecipientEmail(e.target.value)}
                    data-testid="input-new-recipient-email"
                  />
                </div>
                <Select value={newRecipientRole} onValueChange={setNewRecipientRole}>
                  <SelectTrigger className="h-7 text-[11px]" data-testid="select-new-recipient-role">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="signer">Signer</SelectItem>
                    <SelectItem value="viewer">Viewer</SelectItem>
                    <SelectItem value="witness">Witness</SelectItem>
                  </SelectContent>
                </Select>
                <Button
                  size="sm"
                  className="w-full h-7 text-[11px]"
                  onClick={() => addRecipientMutation.mutate()}
                  disabled={!newRecipientName || !newRecipientEmail || addRecipientMutation.isPending}
                  data-testid="button-add-new-recipient"
                >
                  {addRecipientMutation.isPending ? "Adding..." : "Add Recipient"}
                </Button>
              </div>
            )}
            {recipients && recipients.length > 0 && (
              <div className="space-y-1">
                {recipients.map((r, i) => (
                  <div key={r.id} className="group">
                    {editingRecipientId === r.id ? (
                      <div className="space-y-1 p-1.5 rounded-md bg-muted/50">
                        <Input
                          className="h-6 text-[10px]"
                          value={editRecipientName}
                          onChange={(e) => setEditRecipientName(e.target.value)}
                          data-testid={`input-edit-recipient-name-${i}`}
                        />
                        <Input
                          className="h-6 text-[10px]"
                          value={editRecipientEmail}
                          onChange={(e) => setEditRecipientEmail(e.target.value)}
                          data-testid={`input-edit-recipient-email-${i}`}
                        />
                        <div className="flex gap-1">
                          <Button
                            size="sm"
                            className="flex-1 h-6 text-[10px]"
                            onClick={() => updateRecipientMutation.mutate({ id: r.id, name: editRecipientName, email: editRecipientEmail })}
                            disabled={updateRecipientMutation.isPending}
                            data-testid={`button-save-recipient-${i}`}
                          >
                            Save
                          </Button>
                          <Button
                            size="sm"
                            variant="outline"
                            className="h-6 text-[10px]"
                            onClick={() => setEditingRecipientId(null)}
                          >
                            <X className="w-2.5 h-2.5" />
                          </Button>
                        </div>
                      </div>
                    ) : (
                      <div
                        className={`flex items-center gap-1.5 px-1.5 py-1 rounded-md cursor-pointer ${
                          selectedRecipient === r.id ? "bg-muted" : ""
                        }`}
                        onClick={() => setSelectedRecipient(r.id)}
                        data-testid={`recipient-item-${i}`}
                      >
                        <span
                          className={`w-1.5 h-1.5 rounded-full shrink-0 ${
                            RECIPIENT_COLORS[i % RECIPIENT_COLORS.length].split(" ")[0].replace("border-", "bg-")
                          }`}
                        />
                        <div className="flex-1 min-w-0">
                          <p className="text-[10px] font-medium truncate">{r.name}</p>
                          <p className="text-[9px] text-muted-foreground truncate">{r.email}</p>
                        </div>
                        <div className="flex items-center gap-0.5 invisible group-hover:visible">
                          <Button
                            size="icon"
                            variant="ghost"
                            className="h-5 w-5"
                            onClick={(e) => {
                              e.stopPropagation();
                              setEditingRecipientId(r.id);
                              setEditRecipientName(r.name);
                              setEditRecipientEmail(r.email);
                            }}
                            data-testid={`button-edit-recipient-${i}`}
                          >
                            <Pencil className="w-2.5 h-2.5" />
                          </Button>
                          <Button
                            size="icon"
                            variant="ghost"
                            className="h-5 w-5"
                            onClick={(e) => {
                              e.stopPropagation();
                              deleteRecipientMutation.mutate(r.id);
                            }}
                            data-testid={`button-delete-recipient-${i}`}
                          >
                            <Trash2 className="w-2.5 h-2.5" />
                          </Button>
                        </div>
                      </div>
                    )}
                  </div>
                ))}
              </div>
            )}
            {(!recipients || recipients.length === 0) && !showAddRecipient && (
              <p className="text-[10px] text-muted-foreground">No recipients. Click + to add.</p>
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

          <Button
            variant="outline"
            size="sm"
            className="w-full"
            onClick={generateSeal}
            disabled={sealGenerating}
            data-testid="button-generate-seal"
          >
            <Shield className="w-3 h-3" />
            {sealGenerating ? "Generating..." : "Generate Seal"}
          </Button>

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

        <div
          className="flex-1 overflow-auto bg-muted/30 p-5 flex flex-col items-center gap-6"
          style={{
            boxShadow: 'inset 3px 3px 8px rgba(0,0,0,0.45), inset -3px -3px 8px rgba(255,255,255,0.05), inset 0 0 24px rgba(0,0,0,0.2)',
            borderLeft: '1px solid rgba(255,255,255,0.04)',
            borderTop: '1px solid rgba(255,255,255,0.04)',
          }}
        >
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
                    className="relative mx-auto"
                    style={{
                      width: PDF_BASE_WIDTH * pdfZoom,
                      boxShadow: '0 4px 16px rgba(0,0,0,0.5), 0 2px 4px rgba(0,0,0,0.3), 2px 2px 0 rgba(255,255,255,0.03), -1px -1px 0 rgba(0,0,0,0.2)',
                    }}
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
                        cursor: dragTool || clipboardField ? "crosshair" : undefined,
                        pointerEvents: isDragging ? "none" : undefined,
                      }}
                    >
                      {pageFields.map((f) => {
                        const ri = getRecipientIndex(f.recipientId);
                        const colorClass = RECIPIENT_COLORS[ri % RECIPIENT_COLORS.length];
                        const isSelected = selectedField === f.id;
                        const hasValue = !!f.value;

                        return (
                          <div
                            key={f.id}
                            className={`absolute rounded-sm flex flex-col items-center justify-center cursor-move ${
                              f.label === "seal"
                                ? "border border-primary/40 bg-primary/5"
                                : hasValue
                                  ? "border border-primary/30"
                                  : `border-2 border-dashed ${colorClass}`
                            } ${isSelected ? "ring-2 ring-ring" : ""}`}
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
                            onDoubleClick={(e) => {
                              e.stopPropagation();
                              handleFieldDoubleClick(f.id);
                            }}
                            onMouseDown={(e) => handleFieldMouseDown(f.id, e)}
                            onContextMenu={(e) => {
                              e.preventDefault();
                              e.stopPropagation();
                              setClipboardField({ ...f });
                              toast({ title: "Copied", description: `${f.type} field copied. Left-click on page to paste.` });
                            }}
                            data-testid={`field-canvas-${f.id}`}
                          >
                            {hasValue ? (
                              <div className="w-full h-full flex items-center justify-center overflow-hidden p-1 pointer-events-none select-none">
                                {f.value!.startsWith("typed:") ? (
                                  <span
                                    className="text-foreground truncate"
                                    style={{
                                      fontFamily: FONT_STYLES[parseInt(f.value!.split(":")[1])]?.fontFamily,
                                      fontSize: `${Math.max(10, 14 * pdfZoom)}px`,
                                    }}
                                  >
                                    {f.value!.split(":").slice(2).join(":")}
                                  </span>
                                ) : f.value!.startsWith("drawn:") ? (
                                  <img
                                    src={f.value!.replace("drawn:", "")}
                                    alt="Signature"
                                    className="max-w-full max-h-full object-contain"
                                  />
                                ) : f.type === "checkbox" && f.value === "checked" ? (
                                  <CheckCircle2 className="w-3.5 h-3.5 text-primary" />
                                ) : f.type === "date" ? (
                                  <span className="text-[9px] text-foreground truncate">
                                    <span className="font-semibold">Date </span>{f.value}
                                  </span>
                                ) : f.label === "seal" ? (
                                  <div className="w-full h-full flex items-center gap-1 px-1.5">
                                    <Shield className="w-3 h-3 shrink-0 text-primary" />
                                    <span className="text-[8px] font-mono text-foreground truncate">{f.value}</span>
                                  </div>
                                ) : (
                                  <span className="text-[10px] text-foreground truncate">{f.value}</span>
                                )}
                              </div>
                            ) : f.type === "signature" ? (
                              <>
                                <PenLine className="w-4 h-4 text-primary/50 pointer-events-none" />
                                <span className="text-[8px] font-medium text-primary/60 uppercase tracking-widest select-none pointer-events-none mt-0.5">
                                  Double-click to Sign
                                </span>
                              </>
                            ) : f.type === "initials" ? (
                              <>
                                <Hash className="w-3 h-3 text-primary/50 pointer-events-none" />
                                <span className="text-[8px] font-medium text-primary/60 uppercase tracking-widest select-none pointer-events-none mt-0.5">
                                  Double-click
                                </span>
                              </>
                            ) : f.type === "date" ? (
                              <>
                                <CalendarDays className="w-3 h-3 text-primary/50 pointer-events-none" />
                                <span className="text-[7px] font-medium text-primary/60 uppercase tracking-widest select-none pointer-events-none mt-0.5">
                                  Double-click
                                </span>
                              </>
                            ) : f.type === "checkbox" ? (
                              <CheckSquare className="w-3 h-3 text-primary/50 pointer-events-none" />
                            ) : f.type === "text" ? (
                              <>
                                <Type className="w-3 h-3 text-primary/50 pointer-events-none" />
                                <span className="text-[7px] font-medium text-primary/60 uppercase tracking-widest select-none pointer-events-none mt-0.5">
                                  Double-click
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
                    {(dragTool || clipboardField) && (
                      <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
                        <p className="text-[10px] text-muted-foreground bg-background/80 px-2.5 py-1 rounded-md">
                          {clipboardField ? `Click to paste ${clipboardField.type}` : `Click to place ${dragTool!.type}`}
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

      <Dialog open={sigDialogOpen} onOpenChange={setSigDialogOpen}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle className="text-sm">
              {sigFieldId && localFields.find((f) => f.id === sigFieldId)?.type === "initials"
                ? "Add Initials"
                : "Add Signature"}
            </DialogTitle>
          </DialogHeader>
          <Tabs value={sigMode} onValueChange={(v) => setSigMode(v as "draw" | "type")}>
            <TabsList className="w-full">
              <TabsTrigger value="type" className="flex-1" data-testid="tab-editor-type-sig">
                <Type className="w-3 h-3 mr-1" />
                <span className="text-xs">Type</span>
              </TabsTrigger>
              <TabsTrigger value="draw" className="flex-1" data-testid="tab-editor-draw-sig">
                <PenLine className="w-3 h-3 mr-1" />
                <span className="text-xs">Draw</span>
              </TabsTrigger>
            </TabsList>
            <TabsContent value="type" className="space-y-3">
              <Input
                value={typedName}
                onChange={(e) => setTypedName(e.target.value)}
                placeholder="Type your name"
                data-testid="input-editor-typed-name"
              />
              <div className="grid grid-cols-2 gap-2">
                {FONT_STYLES.map((font, i) => (
                  <div
                    key={i}
                    className={`p-3 rounded-md border cursor-pointer text-center transition-colors ${
                      selectedFont === i
                        ? "border-primary bg-primary/5"
                        : "border-border hover-elevate"
                    }`}
                    onClick={() => setSelectedFont(i)}
                    data-testid={`font-editor-${i}`}
                  >
                    <span style={{ fontFamily: font.fontFamily }} className="text-lg">
                      {typedName || "Preview"}
                    </span>
                    <p className="text-[9px] text-muted-foreground mt-1">{font.name}</p>
                  </div>
                ))}
              </div>
            </TabsContent>
            <TabsContent value="draw" className="space-y-3">
              <div className="border rounded-md overflow-hidden">
                <canvas
                  ref={sigCanvasRef}
                  width={400}
                  height={150}
                  className="w-full bg-background cursor-crosshair"
                  onMouseDown={startDraw}
                  onMouseMove={onDraw}
                  onMouseUp={endDraw}
                  onMouseLeave={endDraw}
                  data-testid="canvas-editor-draw-sig"
                />
              </div>
              <Button size="sm" variant="outline" onClick={clearSigCanvas} data-testid="button-editor-clear-sig">
                <Eraser className="w-3 h-3" />
                Clear
              </Button>
            </TabsContent>
          </Tabs>
          <DialogFooter>
            <Button onClick={applySig} data-testid="button-editor-apply-sig">
              Apply
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={textDialogOpen} onOpenChange={setTextDialogOpen}>
        <DialogContent className="max-w-sm">
          <DialogHeader>
            <DialogTitle className="text-sm">Enter Text</DialogTitle>
          </DialogHeader>
          <Input
            value={textValue}
            onChange={(e) => setTextValue(e.target.value)}
            placeholder="Enter text value"
            onKeyDown={(e) => { if (e.key === "Enter") applyText(); }}
            data-testid="input-editor-text-value"
          />
          <DialogFooter>
            <Button onClick={applyText} data-testid="button-editor-apply-text">
              Apply
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={showSealWarning} onOpenChange={setShowSealWarning}>
        <DialogContent className="max-w-sm">
          <DialogHeader>
            <DialogTitle className="text-sm">Missing Seal on Last Page</DialogTitle>
          </DialogHeader>
          <p className="text-xs text-muted-foreground">
            No seal has been placed on the last page of this document. It is recommended to generate a seal on the final page before sending. Do you want to continue anyway?
          </p>
          <DialogFooter className="gap-2">
            <Button variant="outline" size="sm" onClick={() => setShowSealWarning(false)} data-testid="button-seal-warning-cancel">
              Cancel
            </Button>
            <Button size="sm" onClick={() => { setShowSealWarning(false); doSend.mutate(); }} data-testid="button-seal-warning-send">
              Send Anyway
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
