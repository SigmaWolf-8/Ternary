import { useState, useRef, useCallback, useEffect, useMemo } from "react";
import { useIsMobile } from "@/hooks/use-mobile";
import { useRoute, useLocation, Link } from "wouter";
import { useQuery, useMutation } from "@tanstack/react-query";
import { Document, Page, pdfjs } from "react-pdf";
import "react-pdf/dist/Page/AnnotationLayer.css";
import "react-pdf/dist/Page/TextLayer.css";
import { PDFDocument } from "pdf-lib";
import {
  ArrowLeft,
  ArrowRight,
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
  ChevronDown,
  Wifi,
  WifiOff,
  LayoutTemplate,
  Stamp,
  Tag,
  Search,
  PanelLeftOpen,
  PanelLeftClose,
  MapPin,
  MapPinOff,
  Lock,
  Unlock,
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
import type { Envelope, Recipient, Field as FieldType, Template, WbsTag } from "@shared/schema";
import { useCollab, type CollabUser, type CursorPosition } from "@/lib/useCollab";
import { cacheFields, cacheEnvelope, cacheRecipients, getCachedEnvelope, getCachedFields, getCachedRecipients, addPendingOp, syncPendingOps } from "@/lib/offlineCache";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { Badge } from "@/components/ui/badge";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";

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
  { type: "signature", icon: PenLine, label: "Signature", w: 200, h: 28 },
  { type: "date", icon: CalendarDays, label: "Date", w: 140, h: 24 },
  { type: "text", icon: Type, label: "Text", w: 180, h: 28 },
  { type: "checkbox", icon: CheckSquare, label: "Checkbox", w: 28, h: 28 },
  { type: "initials", icon: Hash, label: "Initials", w: 80, h: 28 },
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
  const scrollContainerRef = useRef<HTMLDivElement>(null);
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
  const [clipboardFields, setClipboardFields] = useState<FieldType[]>([]);
  const [showAddRecipient, setShowAddRecipient] = useState(false);
  const [newRecipientName, setNewRecipientName] = useState("");
  const [newRecipientEmail, setNewRecipientEmail] = useState("");
  const [newRecipientRole, setNewRecipientRole] = useState("signer");
  const [editingRecipientId, setEditingRecipientId] = useState<string | null>(null);
  const [editRecipientName, setEditRecipientName] = useState("");
  const [editRecipientEmail, setEditRecipientEmail] = useState("");
  const [showSealWarning, setShowSealWarning] = useState(false);
  const [expandedTemplateId, setExpandedTemplateId] = useState<string | null>(null);
  const [lockedFields, setLockedFields] = useState<Set<string>>(new Set());
  const [isOffline, setIsOffline] = useState(!navigator.onLine);
  const lastCursorEmitRef = useRef(0);
  const [collabUserId] = useState(() => "user-" + Date.now());
  const [isEditingTitle, setIsEditingTitle] = useState(false);
  const [editTitle, setEditTitle] = useState("");
  const titleInputRef = useRef<HTMLInputElement>(null);
  const [showTemplatePanel, setShowTemplatePanel] = useState(false);
  const [templateSearch, setTemplateSearch] = useState("");
  const [selectedFields, setSelectedFields] = useState<Set<string>>(new Set());
  const [marquee, setMarquee] = useState<{
    pageNum: number;
    startX: number;
    startY: number;
    currentX: number;
    currentY: number;
  } | null>(null);
  const marqueeRef = useRef(marquee);
  marqueeRef.current = marquee;
  const isMobile = useIsMobile();
  const [editorPanelOpen, setEditorPanelOpen] = useState(typeof window !== "undefined" ? window.innerWidth >= 768 : true);
  const [sidebarCollapsed, setSidebarCollapsed] = useState<Record<string, boolean>>({
    recipients: false,
    tools: false,
    pages: false,
    placed: true,
  });

  useEffect(() => {
    if (isMobile) setEditorPanelOpen(false);
  }, [isMobile]);

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

  const { data: allTemplates, isLoading: templatesLoading } = useQuery<Template[]>({
    queryKey: ["/api/templates"],
    enabled: showTemplatePanel,
  });

  const { data: wbsTags } = useQuery<WbsTag[]>({
    queryKey: ["/api/wbs-tags"],
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
        if (document.activeElement === document.body) {
          const idsToDelete = selectedFields.size > 0
            ? new Set(selectedFields)
            : selectedField
              ? new Set([selectedField])
              : null;
          if (idsToDelete) {
            e.preventDefault();
            const next = localFields
              .filter((f) => !idsToDelete.has(f.id))
              .map((f) => idsToDelete.has(f.dependsOnFieldId as string) ? { ...f, dependsOnFieldId: null, dependsOnValue: null } : f);
            pushHistory(next);
            setSelectedField(null);
            setSelectedFields(new Set());
          }
        }
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [undo, redo, selectedField, selectedFields, localFields, pushHistory]);

  const collabCallbacks = useMemo(() => ({
    onRemoteFieldAdd: (field: FieldType) => {
      pushHistory([...localFields, field]);
    },
    onRemoteFieldUpdate: (field: Partial<FieldType> & { id: string }) => {
      pushHistory(localFields.map((f) => f.id === field.id ? { ...f, ...field } : f));
    },
    onRemoteFieldDelete: (fieldId: string) => {
      pushHistory(
        localFields
          .filter((f) => f.id !== fieldId)
          .map((f) => f.dependsOnFieldId === fieldId ? { ...f, dependsOnFieldId: null, dependsOnValue: null } : f)
      );
    },
    onConflict: (info: { fieldId: string; userName: string; action: string }) => {
      toast({ title: `${info.userName} ${info.action} a field` });
    },
  }), [localFields, pushHistory, toast]);

  const {
    connected: collabConnected,
    presence: collabPresence,
    cursors: collabCursors,
    emitFieldAdd,
    emitFieldUpdate,
    emitFieldDelete,
    emitCursorMove,
  } = useCollab({
    envelopeId,
    userId: collabUserId,
    userName: "Editor",
    enabled: true,
    ...collabCallbacks,
  });

  useEffect(() => {
    const goOffline = () => setIsOffline(true);
    const goOnline = () => {
      setIsOffline(false);
      syncPendingOps().then((count) => {
        if (count > 0) {
          toast({ title: "Synced", description: `${count} pending operation${count > 1 ? "s" : ""} synced` });
          queryClient.invalidateQueries({ queryKey: ["/api/envelopes", envelopeId, "fields"] });
        }
      }).catch(() => {});
    };
    window.addEventListener("online", goOnline);
    window.addEventListener("offline", goOffline);
    return () => {
      window.removeEventListener("online", goOnline);
      window.removeEventListener("offline", goOffline);
    };
  }, [envelopeId]);

  useEffect(() => {
    if (existingFields && existingFields.length > 0) {
      cacheFields(envelopeId, existingFields).catch(() => {});
    }
  }, [existingFields, envelopeId]);

  useEffect(() => {
    if (envelope) {
      cacheEnvelope(envelope).catch(() => {});
    }
  }, [envelope]);

  useEffect(() => {
    if (recipients && recipients.length > 0) {
      cacheRecipients(envelopeId, recipients).catch(() => {});
    }
  }, [recipients, envelopeId]);

  useEffect(() => {
    if (!navigator.onLine) {
      if (!envelope) {
        getCachedEnvelope(envelopeId).then((cached) => {
          if (cached) queryClient.setQueryData(["/api/envelopes", envelopeId], cached);
        }).catch(() => {});
      }
      if (!existingFields || existingFields.length === 0) {
        getCachedFields(envelopeId).then((cached) => {
          if (cached.length > 0) queryClient.setQueryData(["/api/envelopes", envelopeId, "fields"], cached);
        }).catch(() => {});
      }
      if (!recipients || recipients.length === 0) {
        getCachedRecipients(envelopeId).then((cached) => {
          if (cached.length > 0) queryClient.setQueryData(["/api/envelopes", envelopeId, "recipients"], cached);
        }).catch(() => {});
      }
    }
  }, [envelopeId, envelope, existingFields, recipients, isOffline]);

  useEffect(() => {
    if (numPages <= 1) return;
    const container = scrollContainerRef.current;
    if (!container) return;
    const pageVisibility = new Map<number, number>();
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          const pageNum = Number(entry.target.getAttribute("data-page-num"));
          if (pageNum) {
            pageVisibility.set(pageNum, entry.intersectionRatio);
          }
        });
        let bestPage = 1;
        let bestRatio = 0;
        pageVisibility.forEach((ratio, pageNum) => {
          if (ratio > bestRatio) {
            bestRatio = ratio;
            bestPage = pageNum;
          }
        });
        if (bestRatio > 0) {
          setActivePage(bestPage);
        }
      },
      { root: container, threshold: [0, 0.1, 0.25, 0.5, 0.75, 1] }
    );
    pageRefs.current.forEach((el, pageNum) => {
      el.setAttribute("data-page-num", String(pageNum));
      observer.observe(el);
    });
    return () => observer.disconnect();
  }, [numPages]);

  const handleCursorMove = useCallback((pageNum: number, e: React.MouseEvent) => {
    const now = Date.now();
    if (now - lastCursorEmitRef.current > 100) {
      lastCursorEmitRef.current = now;
      const pageEl = pageRefs.current.get(pageNum);
      if (!pageEl) return;
      const rect = pageEl.getBoundingClientRect();
      const x = (e.clientX - rect.left) / pdfZoom;
      const y = (e.clientY - rect.top) / pdfZoom;
      emitCursorMove(pageNum, x, y);
    }
  }, [pdfZoom, emitCursorMove]);


  const applyTemplate = useCallback((template: Template, mode: "current" | "all" = "current") => {
    const fieldDefs = template.fieldDefs as any[];
    if (!fieldDefs || fieldDefs.length === 0) {
      toast({ title: "No fields to apply", description: "This template has no field definitions.", variant: "destructive" });
      return;
    }
    const hasSealDef = fieldDefs.some((fd: any) => fd.type === "seal");
    const nonSealDefs = fieldDefs.filter((fd: any) => fd.type !== "seal");
    const pages = mode === "all" ? Array.from({ length: numPages }, (_, i) => i + 1) : [activePage];
    const allNewFields: FieldType[] = [];
    for (const pg of pages) {
      const oldIdToNewId: Record<string, string> = {};
      const pageFields: FieldType[] = nonSealDefs.map((fd: any) => {
        const newId = `temp-${Date.now()}-${Math.random().toString(36).slice(2, 8)}-p${pg}`;
        if (fd.id) oldIdToNewId[fd.id] = newId;
        return {
          id: newId,
          envelopeId,
          recipientId: selectedRecipient || null,
          type: fd.type,
          label: fd.label || null,
          page: pg,
          x: fd.x || 60,
          y: fd.y || 200,
          width: fd.width || 200,
          height: fd.height || 36,
          value: null,
          required: fd.required ?? true,
          dependsOnFieldId: fd.dependsOnFieldId ? (oldIdToNewId[fd.dependsOnFieldId] || null) : null,
          dependsOnValue: fd.dependsOnValue || null,
        };
      });
      pageFields.forEach((f) => {
        if (f.dependsOnFieldId && !oldIdToNewId[f.dependsOnFieldId]) {
          const origDef = nonSealDefs.find((fd: any) => oldIdToNewId[fd.id] === f.dependsOnFieldId);
          if (!origDef) f.dependsOnFieldId = null;
        }
      });
      allNewFields.push(...pageFields);
    }
    pushHistory([...localFields, ...allNewFields]);
    allNewFields.forEach((f) => emitFieldAdd(f));
    const pageLabel = mode === "all" ? `across all ${numPages} pages` : "on current page";
    toast({ title: `Applied "${template.name}"`, description: `${allNewFields.length} field${allNewFields.length !== 1 ? "s" : ""} stamped ${pageLabel}` });
    setShowTemplatePanel(false);
    setExpandedTemplateId(null);
    if (hasSealDef) {
      const sealPages = pages;
      setTimeout(() => {
        const runSeals = async () => {
          for (const pg of sealPages) {
            setActivePage(pg);
            await new Promise((r) => setTimeout(r, 100));
            if (generateSealRef.current) await generateSealRef.current();
          }
        };
        runSeals();
      }, 200);
    }
  }, [envelopeId, selectedRecipient, activePage, numPages, localFields, pushHistory, emitFieldAdd, toast]);

  const filteredTemplates = (allTemplates || []).filter((t) =>
    !templateSearch ||
    t.name.toLowerCase().includes(templateSearch.toLowerCase()) ||
    (t.description || "").toLowerCase().includes(templateSearch.toLowerCase()) ||
    (t.category || "").toLowerCase().includes(templateSearch.toLowerCase())
  );

  const renameMutation = useMutation({
    mutationFn: async (title: string) => {
      await apiRequest("PATCH", `/api/envelopes/${envelopeId}`, { title });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["/api/envelopes", envelopeId] });
      queryClient.invalidateQueries({ queryKey: ["/api/envelopes"] });
      setIsEditingTitle(false);
      toast({ title: "Envelope renamed" });
    },
    onError: () => {
      toast({ title: "Failed to rename", variant: "destructive" });
    },
  });

  const startEditingTitle = () => {
    setEditTitle(envelope?.title || "");
    setIsEditingTitle(true);
    setTimeout(() => {
      titleInputRef.current?.focus();
      titleInputRef.current?.select();
    }, 0);
  };

  const submitEditorTitle = () => {
    const trimmed = editTitle.trim();
    if (trimmed && trimmed !== envelope?.title) {
      renameMutation.mutate(trimmed);
    } else {
      setIsEditingTitle(false);
    }
  };

  const saveMutation = useMutation({
    mutationFn: async (fieldsData: FieldType[]) => {
      if (!navigator.onLine) {
        await addPendingOp({
          id: `save-${Date.now()}`,
          envelopeId,
          type: "save_fields",
          data: fieldsData,
          timestamp: Date.now(),
        });
        await cacheFields(envelopeId, fieldsData);
        return fieldsData;
      }
      const res = await apiRequest("PUT", `/api/envelopes/${envelopeId}/fields`, { fields: fieldsData });
      return res.json() as Promise<FieldType[]>;
    },
    onSuccess: (savedFields) => {
      queryClient.invalidateQueries({ queryKey: ["/api/envelopes", envelopeId, "fields"] });
      if (savedFields && Array.isArray(savedFields)) {
        resetHistory(savedFields);
      }
      toast({ title: !navigator.onLine ? "Fields saved offline (will sync when online)" : "Fields saved" });
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
      const userSettings = getSettings();
      await apiRequest("PUT", `/api/envelopes/${envelopeId}/fields`, { fields: localFields });
      await apiRequest("PATCH", `/api/envelopes/${envelopeId}`, {
        status: "sent",
        senderName: userSettings.displayName || undefined,
        senderEmail: userSettings.email || undefined,
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["/api/envelopes"] });
      toast({ title: "Envelope sent for signing" });
      setLocation(`/envelope/${envelopeId}`);
    },
  });

  const { data: envWbsTags } = useQuery<{ id: string; envelopeId: string; wbsTagId: string }[]>({
    queryKey: ["/api/envelopes", envelopeId, "wbs-tags"],
    queryFn: async () => {
      const res = await fetch(`/api/envelopes/${envelopeId}/wbs-tags`);
      return res.json();
    },
    enabled: !!envelopeId,
  });

  const envWbsTagIds = useMemo(() => (envWbsTags || []).map((t) => t.wbsTagId), [envWbsTags]);

  const wbsTagMutation = useMutation({
    mutationFn: async (tagIds: string[]) => {
      await apiRequest("PUT", `/api/envelopes/${envelopeId}/wbs-tags`, { tagIds });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["/api/envelopes", envelopeId, "wbs-tags"] });
      queryClient.invalidateQueries({ queryKey: ["/api/envelope-wbs-tags"] });
      queryClient.invalidateQueries({ queryKey: ["/api/envelopes"] });
    },
  });

  const toggleWbsTag = useCallback((tagId: string) => {
    const current = envWbsTagIds;
    const newIds = current.includes(tagId)
      ? current.filter((id) => id !== tagId)
      : [...current, tagId];
    wbsTagMutation.mutate(newIds);
  }, [envWbsTagIds, wbsTagMutation]);

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
      const pdfFiles = files.filter((f) => f.type === "application/pdf" || f.name.toLowerCase().endsWith(".pdf"));
      const nonPdfFiles = files.filter((f) => f.type !== "application/pdf" && !f.name.toLowerCase().endsWith(".pdf"));

      let lastPageCount = 1;
      if (nonPdfFiles.length > 0) {
        for (const file of nonPdfFiles) {
          const buf = await file.arrayBuffer();
          const base64 = btoa(new Uint8Array(buf).reduce((d, b) => d + String.fromCharCode(b), ""));
          const resp = await apiRequest("POST", `/api/envelopes/${envelopeId}/upload-pdf`, {
            pdfData: base64,
            pageCount: 1,
            fileName: file.name,
            fileType: file.type || file.name.split(".").pop(),
          });
          const result = await resp.json();
          if (result.pageCount) lastPageCount = result.pageCount;
        }
      }

      if (pdfFiles.length === 0) return lastPageCount;

      const buffers: ArrayBuffer[] = [];
      for (const file of pdfFiles) {
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
      toast({ title: "Document uploaded", description: `${pages} page(s) loaded` });
    },
    onError: (error: Error) => {
      toast({ title: "Upload failed", description: error.message, variant: "destructive" });
    },
  });

  const handleFileUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(e.target.files || []);
    if (!files.length) return;
    const allowedTypes = [
      "application/pdf",
      "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
      "text/csv",
      "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    ];
    const allowedExts = [".pdf", ".xlsx", ".csv", ".docx"];
    const valid = files.filter((f) => {
      const ext = f.name.toLowerCase().slice(f.name.lastIndexOf("."));
      if (!allowedTypes.includes(f.type) && !allowedExts.includes(ext)) {
        toast({ title: "Invalid file", description: `${f.name} — only PDF, XLSX, CSV, and DOCX are supported`, variant: "destructive" });
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

      if (clipboardFields.length > 0) {
        const rect = pageEl.getBoundingClientRect();
        const scale = pdfZoom;
        const rawX = (e.clientX - rect.left) / scale;
        const rawY = (e.clientY - rect.top) / scale;

        const minX = Math.min(...clipboardFields.map((f) => f.x));
        const minY = Math.min(...clipboardFields.map((f) => f.y));
        const maxX = Math.max(...clipboardFields.map((f) => f.x + f.width));
        const maxY = Math.max(...clipboardFields.map((f) => f.y + f.height));
        const groupW = maxX - minX;
        const groupH = maxY - minY;

        const anchorX = Math.max(0, Math.round(rawX - groupW / 2));
        const anchorY = Math.max(0, Math.round(rawY - groupH / 2));
        const snappedAnchorX = Math.round(anchorX / SNAP_GRID) * SNAP_GRID;
        const snappedAnchorY = Math.round(anchorY / SNAP_GRID) * SNAP_GRID;

        const pastedFields = clipboardFields.map((cf, i) => ({
          ...cf,
          id: `temp-${Date.now()}-${i}`,
          page: pageNum,
          x: snappedAnchorX + (cf.x - minX),
          y: snappedAnchorY + (cf.y - minY),
        }));

        pushHistory([...localFields, ...pastedFields]);
        setClipboardFields([]);
        toast({ title: "Pasted", description: `${pastedFields.length} field(s) pasted.` });
        return;
      }

      if (!dragTool) {
        setSelectedField(null);
        setSelectedFields(new Set());
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
        dependsOnFieldId: null,
        dependsOnValue: null,
      };

      pushHistory([...localFields, newField]);
      emitFieldAdd(newField);
      setDragTool(null);
    },
    [dragTool, clipboardFields, envelopeId, selectedRecipient, pdfZoom, localFields, pushHistory, toast, emitFieldAdd]
  );

  const toggleSidebarSection = useCallback((key: string) => {
    setSidebarCollapsed((prev) => ({ ...prev, [key]: !prev[key] }));
  }, []);

  const handleFieldMouseDown = useCallback(
    (fieldId: string, e: React.MouseEvent) => {
      e.stopPropagation();
      e.preventDefault();

      const field = localFields.find((f) => f.id === fieldId);
      if (!field) return;

      if (lockedFields.has(fieldId)) {
        setSelectedField(fieldId);
        return;
      }

      if (e.shiftKey || e.ctrlKey || e.metaKey) {
        setSelectedFields((prev) => {
          const next = new Set(prev);
          if (next.has(fieldId)) {
            next.delete(fieldId);
          } else {
            next.add(fieldId);
          }
          if (selectedField && !next.has(selectedField)) next.add(selectedField);
          return next;
        });
        setSelectedField(fieldId);
        return;
      }

      if (selectedFields.size > 0 && !selectedFields.has(fieldId)) {
        setSelectedFields(new Set());
      }

      setSelectedField(fieldId);

      const pageEl = pageRefs.current.get(field.page);
      if (!pageEl) return;

      const startX = e.clientX;
      const startY = e.clientY;
      const scale = pdfZoom;
      let moved = false;

      const movingIds = selectedFields.size > 0 && selectedFields.has(fieldId)
        ? Array.from(selectedFields)
        : [fieldId];

      const origPositions = movingIds.map((id) => {
        const f = localFields.find((lf) => lf.id === id);
        return { id, x: f?.x || 0, y: f?.y || 0 };
      });

      const lastPositions = origPositions.map((p) => ({ ...p }));

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

        origPositions.forEach((orig, i) => {
          const newX = Math.round(Math.max(0, orig.x + dx / scale) / SNAP_GRID) * SNAP_GRID;
          const newY = Math.round(Math.max(0, orig.y + dy / scale) / SNAP_GRID) * SNAP_GRID;
          lastPositions[i].x = newX;
          lastPositions[i].y = newY;

          const el = document.querySelector(`[data-testid="field-canvas-${orig.id}"]`) as HTMLElement;
          if (el) {
            el.style.left = `${newX * scale}px`;
            el.style.top = `${newY * scale}px`;
          }
        });
      };

      const onUp = () => {
        document.removeEventListener("mousemove", onMove);
        document.removeEventListener("mouseup", onUp);
        setIsDragging(false);

        if (moved) {
          justDraggedRef.current = true;
          const posMap = new Map(lastPositions.map((p) => [p.id, p]));
          pushHistory(
            localFields.map((f) => {
              const pos = posMap.get(f.id);
              return pos ? { ...f, x: pos.x, y: pos.y } : f;
            })
          );
          lastPositions.forEach((p) => {
            emitFieldUpdate({ id: p.id, x: p.x, y: p.y });
          });
        }
      };

      document.addEventListener("mousemove", onMove);
      document.addEventListener("mouseup", onUp);
    },
    [localFields, pdfZoom, pushHistory, emitFieldUpdate, selectedField, selectedFields]
  );

  const handleFieldResize = useCallback(
    (fieldId: string, e: React.MouseEvent) => {
      e.stopPropagation();
      e.preventDefault();

      const field = localFields.find((f) => f.id === fieldId);
      if (!field) return;
      if (lockedFields.has(fieldId)) return;

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
        emitFieldUpdate({ id: fieldId, width: lastW, height: lastH });
      };

      document.addEventListener("mousemove", onMove);
      document.addEventListener("mouseup", onUp);
    },
    [localFields, pdfZoom, pushHistory, emitFieldUpdate]
  );

  const handleMarqueeStart = useCallback(
    (pageNum: number, e: React.MouseEvent) => {
      if (dragTool || clipboardFields.length > 0) return;
      if ((e.target as HTMLElement).closest("[data-testid^='field-canvas-']")) return;

      const pageEl = pageRefs.current.get(pageNum);
      if (!pageEl) return;

      const rect = pageEl.getBoundingClientRect();
      const scale = pdfZoom;
      const startX = (e.clientX - rect.left) / scale;
      const startY = (e.clientY - rect.top) / scale;
      const MARQUEE_THRESHOLD = 4;

      let started = false;

      const onMove = (ev: MouseEvent) => {
        const cx = (ev.clientX - rect.left) / scale;
        const cy = (ev.clientY - rect.top) / scale;
        if (!started && Math.abs(cx - startX) < MARQUEE_THRESHOLD && Math.abs(cy - startY) < MARQUEE_THRESHOLD) return;
        started = true;
        setMarquee({ pageNum, startX, startY, currentX: cx, currentY: cy });
      };

      const onUp = () => {
        document.removeEventListener("mousemove", onMove);
        document.removeEventListener("mouseup", onUp);

        const m = marqueeRef.current;
        if (m && started) {
          const minX = Math.min(m.startX, m.currentX);
          const maxX = Math.max(m.startX, m.currentX);
          const minY = Math.min(m.startY, m.currentY);
          const maxY = Math.max(m.startY, m.currentY);

          const hits = localFields.filter((f) => {
            if (f.page !== pageNum) return false;
            return (
              f.x < maxX &&
              f.x + f.width > minX &&
              f.y < maxY &&
              f.y + f.height > minY
            );
          });

          if (hits.length > 0) {
            setSelectedFields(new Set(hits.map((f) => f.id)));
            setSelectedField(hits[0].id);
          } else {
            setSelectedFields(new Set());
            setSelectedField(null);
          }
          justDraggedRef.current = true;
        }
        setMarquee(null);
      };

      document.addEventListener("mousemove", onMove);
      document.addEventListener("mouseup", onUp);
    },
    [dragTool, clipboardFields, pdfZoom, localFields]
  );

  const updateFieldValue = useCallback((fieldId: string, value: string | null) => {
    const targetField = localFields.find((f) => f.id === fieldId);
    let updated = localFields.map((f) => f.id === fieldId ? { ...f, value } : f);

    if (targetField && targetField.type === "signature") {
      const page = targetField.page;
      const sealField = updated.find((f) => f.page === page && f.label === "seal" && f.value);
      if (sealField) {
        const pageFields = updated.filter(
          (f) => f.page === page && f.type !== "initials" && f.type !== "checkbox" && f.label !== "seal" && f.label !== "footer-line"
        );
        const parts: string[] = [];
        for (const f of pageFields) {
          if (f.type === "signature" && f.value) {
            if (f.value.startsWith("typed:")) {
              parts.push(`SIG: ${f.value.split(":").slice(2).join(":")}`);
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
        const oldVal = sealField.value!;
        const tsMatch = oldVal.match(/\|\s*(\d{4}-[^|]*(?:\[[\d.]+fs\])?)$/);
        const timestamp = tsMatch ? tsMatch[1].trim() : formatDateWithTimezone(new Date());
        const gpsMatch = oldVal.match(/\|\s*(GPS:\s*[^|]*)\s*\|/);
        const gpsText = gpsMatch ? gpsMatch[1].trim() : null;
        const newSealContent = gpsText
          ? `SEAL | ${parts.join(" | ")} | ${gpsText} | ${timestamp}`
          : `SEAL | ${parts.join(" | ")} | ${timestamp}`;
        updated = updated.map((f) => f.id === sealField.id ? { ...f, value: newSealContent } : f);
      }
    }

    pushHistory(updated);
  }, [localFields, pushHistory]);

  const handleFieldDoubleClick = useCallback((fieldId: string) => {
    const field = localFields.find((f) => f.id === fieldId);
    if (!field) return;
    if (field.label === "seal" || field.label === "footer-line") return;

    if (field.type === "signature" || field.type === "initials") {
      setSigFieldId(fieldId);
      setSigDialogOpen(true);
      const r = recipients?.find((r) => r.id === field.recipientId);
      const settings = getSettings();
      setTypedName(settings.displayName || r?.name || "");
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
  const [showGpsOnSeal, setShowGpsOnSeal] = useState(true);

  const generateSeal = useCallback(async () => {
    const pageFields = localFields.filter(
      (f) => f.page === activePage && f.type !== "initials" && f.type !== "checkbox" && f.label !== "seal" && f.label !== "footer-line"
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

      const sealContent = showGpsOnSeal
        ? `SEAL | ${parts.join(" | ")} | ${gpsText} | ${sealTimestamp}`
        : `SEAL | ${parts.join(" | ")} | ${sealTimestamp}`;

      const pageEl = pageRefs.current.get(activePage);
      const pageHeight = pageEl ? pageEl.offsetHeight / pdfZoom : 1035;
      const pageWidth = pageEl ? pageEl.offsetWidth / pdfZoom : 800;

      const sigFields = localFields.filter(
        (f) => f.page === activePage && f.type === "signature" && f.label !== "seal" && f.label !== "footer-line" && !lockedFields.has(f.id)
      );

      const bottomMargin = 20;
      const inlineGap = 4;
      const footerLineHeight = 4;
      const footerExtraLift = 24;
      const sealHeight = 24;
      const leftMargin = 40;
      const rightMargin = 40;
      const usableWidth = pageWidth - leftMargin - rightMargin;

      const sigCount = sigFields.length;
      const maxSealWidth = 470;
      const rawSigTotalWidth = sigCount > 0 ? Math.min(sigCount * 120 + (sigCount - 1) * inlineGap, Math.round(usableWidth * 0.4)) : 0;
      const sigItemWidth = sigCount > 0 ? Math.max(20, Math.floor((rawSigTotalWidth - (sigCount - 1) * inlineGap) / sigCount)) : 0;
      const sigTotalWidth = sigCount > 0 ? sigCount * sigItemWidth + (sigCount - 1) * inlineGap : 0;
      const sigRowHeight = sealHeight;

      const sealWidth = sigCount > 0
        ? Math.min(maxSealWidth, usableWidth - sigTotalWidth - inlineGap)
        : Math.min(maxSealWidth, usableWidth);
      const groupWidth = sigCount > 0 ? sigTotalWidth + inlineGap + sealWidth : sealWidth;
      const groupStartX = Math.round(leftMargin + (usableWidth - groupWidth) / 2);

      const sealRowY = Math.round(pageHeight - bottomMargin - sealHeight);
      const footerLineY = Math.round(sealRowY - footerLineHeight - 6 - footerExtraLift);

      const repositionedSigIds = new Set(sigFields.map((f) => f.id));

      const updatedFields: FieldType[] = [];
      let sigX = groupStartX;
      for (const f of sigFields) {
        updatedFields.push({ ...f, x: Math.round(sigX), y: sealRowY, width: sigItemWidth, height: sigRowHeight });
        sigX += sigItemWidth + inlineGap;
      }

      const sealX = sigCount > 0 ? Math.round(groupStartX + sigTotalWidth + inlineGap) : groupStartX;

      const existingSealIdx = localFields.findIndex((f) => f.page === activePage && f.label === "seal");
      const existingSeal = existingSealIdx >= 0 ? localFields[existingSealIdx] : null;
      const existingFooterIdx = localFields.findIndex((f) => f.page === activePage && f.label === "footer-line");
      const existingFooter = existingFooterIdx >= 0 ? localFields[existingFooterIdx] : null;

      let newFields = localFields.map((f) => {
        if (repositionedSigIds.has(f.id)) {
          const updated = updatedFields.find((u) => u.id === f.id);
          return updated || f;
        }
        if (existingSeal && f.id === existingSeal.id) {
          return { ...f, value: sealContent, y: sealRowY, x: sealX, width: sealWidth, height: sealHeight };
        }
        if (existingFooter && f.id === existingFooter.id) {
          return { ...f, y: footerLineY, x: leftMargin, width: Math.round(usableWidth) };
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
          y: sealRowY,
          width: sealWidth,
          height: sealHeight,
          value: sealContent,
          required: false,
          dependsOnFieldId: null,
          dependsOnValue: null,
        };
        newFields = [...newFields, sealField];
      }

      if (!existingFooter) {
        const footerField: FieldType = {
          id: `temp-footer-${Date.now()}`,
          envelopeId,
          recipientId: selectedRecipient || null,
          type: "text",
          label: "footer-line",
          page: activePage,
          x: leftMargin,
          y: footerLineY,
          width: Math.round(usableWidth),
          height: footerLineHeight,
          value: "─".repeat(120),
          required: false,
          dependsOnFieldId: null,
          dependsOnValue: null,
        };
        newFields = [...newFields, footerField];
      }

      pushHistory(newFields);
      toast({ title: "Seal Generated", description: "Signatures placed inline with seal below document footer line." });
    } finally {
      setSealGenerating(false);
    }
  }, [localFields, activePage, envelopeId, selectedRecipient, pushHistory, toast, showGpsOnSeal]);

  const generateSealRef = useRef(generateSeal);
  useEffect(() => { generateSealRef.current = generateSeal; }, [generateSeal]);

  const reviseSealsGps = useCallback((includeGps: boolean) => {
    const sealFields = localFields.filter((f) => f.label === "seal" && f.value);
    if (sealFields.length === 0) return;

    const gpsPattern = /\s*\|\s*GPS:\s*[^|]*/g;

    let updatedFields = localFields.map((f) => {
      if (f.label !== "seal" || !f.value) return f;
      if (!includeGps) {
        return { ...f, value: f.value.replace(gpsPattern, "") };
      }
      if (includeGps && !f.value.includes("GPS:")) {
        const lastPipe = f.value.lastIndexOf("|");
        if (lastPipe > 0) {
          const before = f.value.slice(0, lastPipe).trimEnd();
          const after = f.value.slice(lastPipe);
          return { ...f, value: `${before} | GPS: unavailable ${after}` };
        }
      }
      return f;
    });

    const sealPages = Array.from(new Set(sealFields.map((f) => f.page)));
    const inlineGap = 4;
    const sealHeight = 24;
    const leftMargin = 40;
    const rightMargin = 40;
    const maxSealWidth = 470;

    for (const page of sealPages) {
      const pageEl = pageRefs.current.get(page);
      const pageWidth = pageEl ? pageEl.offsetWidth / pdfZoom : 800;
      const usableWidth = pageWidth - leftMargin - rightMargin;

      const sigFields = updatedFields.filter(
        (f) => f.page === page && f.type === "signature" && f.label !== "seal" && f.label !== "footer-line"
      );
      const sigCount = sigFields.length;
      const rawSigTotalWidth = sigCount > 0 ? Math.min(sigCount * 120 + (sigCount - 1) * inlineGap, Math.round(usableWidth * 0.4)) : 0;
      const sigItemWidth = sigCount > 0 ? Math.max(20, Math.floor((rawSigTotalWidth - (sigCount - 1) * inlineGap) / sigCount)) : 0;
      const sigTotalWidth = sigCount > 0 ? sigCount * sigItemWidth + (sigCount - 1) * inlineGap : 0;

      const sealWidth = sigCount > 0
        ? Math.min(maxSealWidth, usableWidth - sigTotalWidth - inlineGap)
        : Math.min(maxSealWidth, usableWidth);
      const groupWidth = sigCount > 0 ? sigTotalWidth + inlineGap + sealWidth : sealWidth;
      const groupStartX = Math.round(leftMargin + (usableWidth - groupWidth) / 2);

      const sigIds = new Set(sigFields.map((sf) => sf.id));
      let sigX = groupStartX;
      updatedFields = updatedFields.map((f) => {
        if (f.page === page && sigIds.has(f.id)) {
          const newF = { ...f, x: Math.round(sigX), width: sigItemWidth, height: sealHeight };
          sigX += sigItemWidth + inlineGap;
          return newF;
        }
        if (f.page === page && f.label === "seal") {
          const sx = sigCount > 0 ? Math.round(groupStartX + sigTotalWidth + inlineGap) : groupStartX;
          return { ...f, x: sx, width: sealWidth, height: sealHeight };
        }
        return f;
      });
    }

    pushHistory(updatedFields);
  }, [localFields, pdfZoom, pushHistory]);

  const removeField = (id: string) => {
    pushHistory(
      localFields
        .filter((f) => f.id !== id)
        .map((f) => f.dependsOnFieldId === id ? { ...f, dependsOnFieldId: null, dependsOnValue: null } : f)
    );
    emitFieldDelete(id);
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
          <div className="flex items-center gap-0.5">
            <Link href={`/envelope/${envelopeId}`}>
              <Button size="icon" variant="ghost" data-testid="button-editor-back">
                <ArrowLeft className="w-3.5 h-3.5" />
              </Button>
            </Link>
            <Link href="/">
              <Button size="icon" variant="ghost" data-testid="button-editor-forward" title="Dashboard">
                <ArrowRight className="w-3.5 h-3.5" />
              </Button>
            </Link>
          </div>
          <div className="min-w-0">
            {isEditingTitle ? (
              <Input
                ref={titleInputRef}
                value={editTitle}
                onChange={(e) => setEditTitle(e.target.value)}
                onBlur={submitEditorTitle}
                onKeyDown={(e) => {
                  if (e.key === "Enter") submitEditorTitle();
                  if (e.key === "Escape") setIsEditingTitle(false);
                }}
                className="h-6 text-xs font-semibold w-48"
                data-testid="input-edit-title"
                disabled={renameMutation.isPending}
              />
            ) : (
              <button
                onClick={startEditingTitle}
                className="flex items-center gap-1 group text-left max-w-full"
                data-testid="button-edit-title"
                title="Click to rename"
              >
                <h1 className="text-xs font-semibold truncate" data-testid="text-editor-title">
                  {envelope?.title || "Untitled"}
                </h1>
                <Pencil className="w-2.5 h-2.5 text-muted-foreground invisible group-hover:visible shrink-0" />
              </button>
            )}
            <p className="text-[10px] text-muted-foreground truncate">
              {hasPdf ? `${numPages} pg${numPages !== 1 ? "s" : ""} · Place fields` : "Upload a document"}
            </p>
          </div>
          <div className="flex items-center gap-1.5 ml-2">
            <Tooltip>
              <TooltipTrigger asChild>
                <span
                  className="w-2 h-2 rounded-full shrink-0"
                  style={{ backgroundColor: collabConnected ? "#22c55e" : "#ef4444" }}
                  data-testid="indicator-collab-status"
                />
              </TooltipTrigger>
              <TooltipContent>
                <p className="text-xs">{collabConnected ? "Connected" : "Disconnected"}</p>
              </TooltipContent>
            </Tooltip>
            {collabPresence.filter((u) => u.userId !== collabUserId).map((u) => (
              <Tooltip key={u.userId}>
                <TooltipTrigger asChild>
                  <span
                    className="w-5 h-5 rounded-full flex items-center justify-center text-[9px] font-bold text-white shrink-0"
                    style={{ backgroundColor: u.color }}
                    data-testid={`presence-user-${u.userId}`}
                  >
                    {u.userName.charAt(0).toUpperCase()}
                  </span>
                </TooltipTrigger>
                <TooltipContent>
                  <p className="text-xs">{u.userName}</p>
                </TooltipContent>
              </Tooltip>
            ))}
            {isOffline && (
              <Tooltip>
                <TooltipTrigger asChild>
                  <span className="flex items-center gap-1 text-[10px] text-destructive" data-testid="indicator-offline">
                    <WifiOff className="w-3 h-3" />
                    Offline
                  </span>
                </TooltipTrigger>
                <TooltipContent>
                  <p className="text-xs">You are currently offline. Changes will sync when reconnected.</p>
                </TooltipContent>
              </Tooltip>
            )}
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
        <div
          className={`shrink-0 border-r overflow-hidden transition-all duration-200 bg-sidebar ${editorPanelOpen ? "w-48" : "w-0 border-r-0"}`}
          style={{ boxShadow: editorPanelOpen ? 'inset 2px 2px 6px rgba(255,255,255,0.07), inset -2px -2px 6px rgba(0,0,0,0.4), inset 0 0 20px rgba(0,0,0,0.15)' : 'none' }}
          data-testid="editor-sidebar-panel"
        >
          <div className={`${editorPanelOpen ? "opacity-100" : "opacity-0 pointer-events-none"} transition-opacity duration-150 w-48 h-full overflow-y-auto overflow-x-hidden p-2 space-y-1.5`}>
          <div className="flex justify-end -mb-1">
            <Button
              size="icon"
              variant="ghost"
              onClick={() => setEditorPanelOpen(false)}
              data-testid="button-close-editor-panel"
            >
              <PanelLeftClose className="w-3.5 h-3.5" />
            </Button>
          </div>
          <div>
            <input
              ref={fileInputRef}
              type="file"
              accept=".pdf,.xlsx,.csv,.docx"
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

          {wbsTags && wbsTags.length > 0 && (
            <div>
              <p className="text-[9px] font-medium text-muted-foreground uppercase tracking-widest mb-1">
                WBS Tags
              </p>
              <Popover>
                <PopoverTrigger asChild>
                  <Button variant="outline" size="sm" className="w-full justify-between text-[10px]" data-testid="dropdown-wbs-tags">
                    <span className="flex items-center gap-1.5 truncate">
                      <Tag className="w-3 h-3 shrink-0" />
                      {envWbsTagIds.length === 0
                        ? "Select tags..."
                        : `${envWbsTagIds.length} tag${envWbsTagIds.length > 1 ? "s" : ""} selected`}
                    </span>
                    <ChevronDown className="w-3 h-3 shrink-0 opacity-50" />
                  </Button>
                </PopoverTrigger>
                <PopoverContent align="start" className="w-56 max-h-64 overflow-y-auto p-1">
                  {wbsTags.map((tag) => {
                    const isActive = envWbsTagIds.includes(tag.id);
                    return (
                      <div
                        key={tag.id}
                        onClick={() => toggleWbsTag(tag.id)}
                        className="flex items-center gap-2 w-full px-2 py-1.5 rounded text-[11px] hover-elevate transition-colors cursor-pointer"
                        data-testid={`toggle-wbs-${tag.id}`}
                      >
                        <div
                          className="w-3 h-3 rounded border flex items-center justify-center shrink-0"
                          style={{
                            backgroundColor: isActive ? tag.color : "transparent",
                            borderColor: tag.color,
                          }}
                        >
                          {isActive && (
                            <CheckCircle2 className="w-2.5 h-2.5 text-white" />
                          )}
                        </div>
                        <div
                          className="w-2 h-2 rounded-full shrink-0"
                          style={{ backgroundColor: tag.color }}
                        />
                        <span className={isActive ? "font-medium" : "text-muted-foreground"}>
                          {tag.name}
                        </span>
                      </div>
                    );
                  })}
                </PopoverContent>
              </Popover>
              {envWbsTagIds.length > 0 && (
                <div className="flex flex-wrap gap-0.5 mt-1">
                  {envWbsTagIds.map((tagId) => {
                    const tag = wbsTags.find((t) => t.id === tagId);
                    if (!tag) return null;
                    return (
                      <Badge
                        key={tagId}
                        variant="outline"
                        className="text-[8px] py-0 no-default-active-elevate"
                        style={{ borderColor: tag.color, color: tag.color }}
                      >
                        {tag.name}
                      </Badge>
                    );
                  })}
                </div>
              )}
            </div>
          )}

          <div>
            <button
              className="flex items-center justify-between gap-1 w-full py-0.5"
              onClick={() => toggleSidebarSection("recipients")}
              data-testid="button-toggle-recipients-section"
            >
              <p className="text-[9px] font-medium text-muted-foreground uppercase tracking-widest">
                Recipients {recipients?.length ? `(${recipients.length})` : ""}
              </p>
              <ChevronDown className={`w-2.5 h-2.5 text-muted-foreground transition-transform ${sidebarCollapsed.recipients ? "-rotate-90" : ""}`} />
            </button>
            {!sidebarCollapsed.recipients && (
            <div className="mt-1">
            <Button
              size="icon"
              variant="ghost"
              className="h-5 w-5 mb-1"
              onClick={() => setShowAddRecipient(!showAddRecipient)}
              data-testid="button-toggle-add-recipient"
            >
              <UserPlus className="w-3 h-3" />
            </Button>
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
            )}
          </div>

          <div>
            <button
              className="flex items-center justify-between gap-1 w-full py-0.5"
              onClick={() => toggleSidebarSection("tools")}
              data-testid="button-toggle-tools-section"
            >
              <p className="text-[9px] font-medium text-muted-foreground uppercase tracking-widest">
                Field Tools
              </p>
              <ChevronDown className={`w-2.5 h-2.5 text-muted-foreground transition-transform ${sidebarCollapsed.tools ? "-rotate-90" : ""}`} />
            </button>
            {!sidebarCollapsed.tools && (
            <div className="mt-0.5 space-y-0.5">
              {FIELD_TOOLS.map((tool, idx) => (
                <div key={tool.type}>
                  <Button
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
                  {idx === 0 && (
                    <div className="flex items-center gap-1 mt-0.5 mb-0.5">
                      <Button
                        variant="outline"
                        size="sm"
                        className="flex-1"
                        onClick={generateSeal}
                        disabled={sealGenerating}
                        data-testid="button-generate-seal"
                      >
                        <Shield className="w-3 h-3" />
                        {sealGenerating ? "Generating..." : "Generate Seal"}
                      </Button>
                      <Button
                        variant="outline"
                        size="icon"
                        className={`toggle-elevate ${showGpsOnSeal ? "toggle-elevated" : ""}`}
                        onClick={() => {
                          const newGpsState = !showGpsOnSeal;
                          setShowGpsOnSeal(newGpsState);
                          reviseSealsGps(newGpsState);
                        }}
                        data-testid="button-toggle-gps"
                        title={showGpsOnSeal ? "GPS location will appear on seal" : "GPS location hidden from seal"}
                      >
                        {showGpsOnSeal ? <MapPin className="w-3.5 h-3.5" /> : <MapPinOff className="w-3.5 h-3.5" />}
                      </Button>
                    </div>
                  )}
                  {idx === FIELD_TOOLS.length - 1 && (
                    <div className="mt-0.5">
                      <Button
                        variant={showTemplatePanel ? "secondary" : "outline"}
                        size="sm"
                        className="w-full justify-start"
                        onClick={() => setShowTemplatePanel(!showTemplatePanel)}
                        data-testid="button-toggle-templates"
                      >
                        <LayoutTemplate className="w-3 h-3" />
                        <span className="text-xs">Templates</span>
                      </Button>
                      {showTemplatePanel && (
                        <div className="mt-2 space-y-1.5">
                          <div className="relative">
                            <Search className="absolute left-2 top-1/2 -translate-y-1/2 w-3 h-3 text-muted-foreground" />
                            <Input
                              placeholder="Search..."
                              value={templateSearch}
                              onChange={(e) => setTemplateSearch(e.target.value)}
                              className="pl-7 h-7 text-[10px]"
                              data-testid="input-editor-template-search"
                            />
                          </div>
                          <div className="space-y-0.5 max-h-48 overflow-y-auto">
                            {templatesLoading ? (
                              <div className="space-y-1 py-1">
                                <Skeleton className="h-8 w-full" />
                                <Skeleton className="h-8 w-full" />
                                <Skeleton className="h-8 w-full" />
                              </div>
                            ) : filteredTemplates.length === 0 ? (
                              <p className="text-[10px] text-muted-foreground text-center py-2">No templates found</p>
                            ) : (
                              filteredTemplates.map((tpl) => (
                                <div key={tpl.id} className="space-y-0.5">
                                  <div
                                    className="flex items-center gap-1.5 p-1.5 rounded-md hover-elevate cursor-pointer group/tpl"
                                    onClick={() => setExpandedTemplateId(expandedTemplateId === tpl.id ? null : tpl.id)}
                                    data-testid={`template-apply-${tpl.id}`}
                                  >
                                    <Stamp className="w-3 h-3 text-primary shrink-0" />
                                    <div className="flex-1 min-w-0">
                                      <p className="text-[10px] font-medium truncate">{tpl.name}</p>
                                      <p className="text-[8px] text-muted-foreground truncate">
                                        {(tpl.fieldDefs as any[] || []).length} fields
                                        {tpl.category ? ` · ${tpl.category}` : ""}
                                      </p>
                                    </div>
                                    <ChevronDown className={`w-2.5 h-2.5 text-muted-foreground transition-transform ${expandedTemplateId === tpl.id ? "" : "-rotate-90"}`} />
                                  </div>
                                  {expandedTemplateId === tpl.id && (
                                    <div className="flex items-center gap-1 pl-5">
                                      <Button
                                        variant="outline"
                                        size="sm"
                                        className="flex-1 text-[10px] h-6"
                                        onClick={() => applyTemplate(tpl, "current")}
                                        data-testid={`template-apply-current-${tpl.id}`}
                                      >
                                        This Page
                                      </Button>
                                      <Button
                                        variant="default"
                                        size="sm"
                                        className="flex-1 text-[10px] h-6"
                                        onClick={() => applyTemplate(tpl, "all")}
                                        data-testid={`template-apply-all-${tpl.id}`}
                                      >
                                        All Pages
                                      </Button>
                                    </div>
                                  )}
                                </div>
                              ))
                            )}
                          </div>
                        </div>
                      )}
                    </div>
                  )}
                </div>
              ))}
            </div>
            )}
          </div>

          {numPages > 1 && (
            <div>
              <p className="text-[9px] font-medium text-muted-foreground uppercase tracking-widest mb-1">
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

          {selectedFields.size > 1 && (
            <p className="text-[9px] text-primary font-medium px-0.5">
              {selectedFields.size} fields selected
            </p>
          )}

          {localFields.length > 0 && (
            <div>
              <button
                className="flex items-center justify-between gap-1 w-full py-0.5"
                onClick={() => toggleSidebarSection("placed")}
                data-testid="button-toggle-placed-section"
              >
                <p className="text-[9px] font-medium text-muted-foreground uppercase tracking-widest">
                  Placed ({localFields.length})
                </p>
                <ChevronDown className={`w-2.5 h-2.5 text-muted-foreground transition-transform ${sidebarCollapsed.placed ? "-rotate-90" : ""}`} />
              </button>
              {!sidebarCollapsed.placed && (
              <div className="space-y-0.5 mt-0.5">
                {localFields.map((f, idx) => {
                  const ri = getRecipientIndex(f.recipientId);
                  const isReorderTarget = reorderOverIdx === idx && reorderDragIdx !== idx;
                  const wouldCycle = (targetId: string, currentId: string): boolean => {
                    const visited = new Set<string>();
                    let cursor: string | null = targetId;
                    while (cursor) {
                      if (cursor === currentId) return true;
                      if (visited.has(cursor)) return false;
                      visited.add(cursor);
                      const dep = localFields.find((df) => df.id === cursor);
                      cursor = dep?.dependsOnFieldId || null;
                    }
                    return false;
                  };
                  const dependableFields = localFields.filter((cf) => cf.id !== f.id && !wouldCycle(cf.id, f.id));
                  const isMultiSelected = selectedFields.has(f.id);
                  return (
                    <div key={f.id}>
                      <div
                        draggable
                        onDragStart={() => handleReorderDragStart(idx)}
                        onDragOver={(e) => handleReorderDragOver(idx, e)}
                        onDrop={() => handleReorderDrop(idx)}
                        onDragEnd={() => {
                          setReorderDragIdx(null);
                          setReorderOverIdx(null);
                        }}
                        className={`flex items-center justify-between gap-1 p-1 rounded-md text-[10px] cursor-grab ${
                          selectedField === f.id || isMultiSelected ? "bg-accent" : ""
                        } ${isReorderTarget ? "border-t-2 border-primary" : ""} ${
                          reorderDragIdx === idx ? "opacity-40" : ""
                        } hover-elevate`}
                        onClick={(e) => {
                          if (e.shiftKey || e.ctrlKey || e.metaKey) {
                            setSelectedFields((prev) => {
                              const next = new Set(prev);
                              if (next.has(f.id)) next.delete(f.id);
                              else next.add(f.id);
                              return next;
                            });
                          } else {
                            setSelectedFields(new Set());
                          }
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
                          <span className="capitalize truncate">{f.label && f.label !== "seal" && f.label !== "footer-line" ? f.label : f.label === "footer-line" ? "Footer" : f.type}</span>
                          <span className="text-muted-foreground shrink-0">p{f.page}</span>
                        </span>
                        <span className="flex items-center gap-0.5">
                          <Button
                            size="icon"
                            variant="ghost"
                            className="w-5 h-5"
                            onClick={(e) => {
                              e.stopPropagation();
                              setLockedFields((prev) => {
                                const next = new Set(prev);
                                if (next.has(f.id)) next.delete(f.id);
                                else next.add(f.id);
                                return next;
                              });
                            }}
                            data-testid={`button-lock-field-${f.id}`}
                            title={lockedFields.has(f.id) ? "Unlock field" : "Lock field"}
                          >
                            {lockedFields.has(f.id) ? <Lock className="w-2.5 h-2.5 text-primary" /> : <Unlock className="w-2.5 h-2.5" />}
                          </Button>
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
                        </span>
                      </div>
                      <Collapsible>
                        <CollapsibleTrigger asChild>
                          <button
                            className="flex items-center gap-1 text-[9px] text-muted-foreground px-1.5 py-0.5 w-full"
                            data-testid={`button-conditional-toggle-${f.id}`}
                          >
                            <ChevronDown className="w-2.5 h-2.5" />
                            Conditional Logic
                            {f.dependsOnFieldId && <span className="text-primary ml-1">(active)</span>}
                          </button>
                        </CollapsibleTrigger>
                        <CollapsibleContent className="px-1.5 pb-1.5 space-y-1">
                          <div>
                            <label className="text-[9px] text-muted-foreground">Depends on</label>
                            <Select
                              value={f.dependsOnFieldId || "__none__"}
                              onValueChange={(val) => {
                                const newVal = val === "__none__" ? null : val;
                                pushHistory(localFields.map((lf) =>
                                  lf.id === f.id ? { ...lf, dependsOnFieldId: newVal } : lf
                                ));
                              }}
                            >
                              <SelectTrigger className="h-6 text-[10px]" data-testid={`select-depends-on-${f.id}`}>
                                <SelectValue placeholder="None" />
                              </SelectTrigger>
                              <SelectContent>
                                <SelectItem value="__none__">None</SelectItem>
                                {dependableFields.map((cf) => {
                                  const typeLabel = cf.type === "checkbox" ? "Checkbox" : cf.type === "signature" ? "Signature" : cf.type === "initials" ? "Initials" : cf.type === "date" ? "Date" : cf.type === "text" ? "Text" : cf.type;
                                  return (
                                    <SelectItem key={cf.id} value={cf.id}>
                                      {typeLabel} p{cf.page} ({cf.id.slice(0, 6)})
                                    </SelectItem>
                                  );
                                })}
                              </SelectContent>
                            </Select>
                          </div>
                          {f.dependsOnFieldId && (() => {
                            const depField = localFields.find((df) => df.id === f.dependsOnFieldId);
                            const depType = depField?.type || "checkbox";
                            return (
                              <div>
                                <label className="text-[9px] text-muted-foreground">Show when value is</label>
                                <Select
                                  value={f.dependsOnValue || (depType === "checkbox" ? "checked" : "filled")}
                                  onValueChange={(val) => {
                                    pushHistory(localFields.map((lf) =>
                                      lf.id === f.id ? { ...lf, dependsOnValue: val } : lf
                                    ));
                                  }}
                                >
                                  <SelectTrigger className="h-6 text-[10px]" data-testid={`select-depends-value-${f.id}`}>
                                    <SelectValue />
                                  </SelectTrigger>
                                  <SelectContent>
                                    {depType === "checkbox" ? (
                                      <>
                                        <SelectItem value="checked">Checked</SelectItem>
                                        <SelectItem value="unchecked">Unchecked</SelectItem>
                                      </>
                                    ) : (
                                      <>
                                        <SelectItem value="filled">Filled</SelectItem>
                                        <SelectItem value="empty">Empty</SelectItem>
                                      </>
                                    )}
                                  </SelectContent>
                                </Select>
                              </div>
                            );
                          })()}
                        </CollapsibleContent>
                      </Collapsible>
                    </div>
                  );
                })}
              </div>
              )}
            </div>
          )}
          </div>
        </div>
        {!editorPanelOpen && (
          <div className="shrink-0 flex items-start pt-2 pl-1 pr-1 border-r bg-sidebar">
            <Button
              size="icon"
              variant="ghost"
              onClick={() => setEditorPanelOpen(true)}
              data-testid="button-open-editor-panel"
            >
              <PanelLeftOpen className="w-3.5 h-3.5" />
            </Button>
          </div>
        )}

        <div
          ref={scrollContainerRef}
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
                    onMouseMove={(e) => handleCursorMove(pageNum, e)}
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
                        cursor: dragTool || clipboardFields.length > 0 ? "crosshair" : undefined,
                        pointerEvents: isDragging ? "none" : undefined,
                      }}
                      onMouseDown={(e) => handleMarqueeStart(pageNum, e)}
                    >
                      {Array.from(collabCursors.values())
                        .filter((c) => c.page === pageNum && c.userId !== collabUserId)
                        .map((c) => (
                          <div
                            key={c.userId}
                            className="absolute pointer-events-none z-50"
                            style={{
                              left: c.x * pdfZoom,
                              top: c.y * pdfZoom,
                              transform: "translate(-4px, -4px)",
                            }}
                            data-testid={`cursor-${c.userId}`}
                          >
                            <span
                              className="block w-2 h-2 rounded-full"
                              style={{ backgroundColor: c.color }}
                            />
                            <span
                              className="text-[8px] whitespace-nowrap px-1 rounded"
                              style={{ backgroundColor: c.color, color: "#fff" }}
                            >
                              {c.userName}
                            </span>
                          </div>
                        ))}
                      {pageFields.map((f) => {
                        const ri = getRecipientIndex(f.recipientId);
                        const colorClass = RECIPIENT_COLORS[ri % RECIPIENT_COLORS.length];
                        const isSelected = selectedField === f.id || selectedFields.has(f.id);
                        const hasValue = !!f.value;

                        return (
                          <div
                            key={f.id}
                            className={`absolute rounded-sm flex flex-col items-center justify-center ${lockedFields.has(f.id) ? "cursor-default" : "cursor-move"} ${
                              f.label === "footer-line"
                                ? "border-0 bg-transparent"
                                : f.label === "seal"
                                ? "border border-primary/40 bg-primary/5"
                                : hasValue
                                  ? "border border-primary/30 bg-primary/10"
                                  : `border border-primary/50 bg-primary/20`
                            } ${isSelected ? "ring-2 ring-ring" : ""} ${lockedFields.has(f.id) ? "opacity-80" : ""}`}
                            style={{
                              left: f.x * pdfZoom,
                              top: f.y * pdfZoom,
                              width: f.width * pdfZoom,
                              height: f.height * pdfZoom,
                              userSelect: "none",
                            }}
                            onClick={(e) => {
                              e.stopPropagation();
                              if (!e.shiftKey && !e.ctrlKey && !e.metaKey) {
                                setSelectedFields(new Set());
                                setSelectedField(f.id);
                              }
                            }}
                            onDoubleClick={(e) => {
                              e.stopPropagation();
                              handleFieldDoubleClick(f.id);
                            }}
                            onMouseDown={(e) => handleFieldMouseDown(f.id, e)}
                            onContextMenu={(e) => {
                              e.preventDefault();
                              e.stopPropagation();
                              const isInSelection = selectedFields.has(f.id);
                              if (isInSelection && selectedFields.size > 1) {
                                const copied = localFields.filter((lf) => selectedFields.has(lf.id)).map((lf) => ({ ...lf }));
                                setClipboardFields(copied);
                                toast({ title: "Copied", description: `${copied.length} fields copied. Left-click on page to paste.` });
                              } else {
                                setClipboardFields([{ ...f }]);
                                toast({ title: "Copied", description: `${f.type} field copied. Left-click on page to paste.` });
                              }
                            }}
                            data-testid={`field-canvas-${f.id}`}
                          >
                            {hasValue ? (
                              <div className="w-full h-full flex items-center justify-center overflow-hidden p-1 pointer-events-none select-none">
                                {f.value!.startsWith("typed:") ? (
                                  <span
                                    className="truncate"
                                    style={{
                                      color: "#1a1a1a",
                                      fontFamily: FONT_STYLES[parseInt(f.value!.split(":")[1])]?.fontFamily,
                                      fontSize: `${Math.max(10, 14 * pdfZoom)}px`,
                                      textShadow: "0.5px 0.5px 0px rgba(0,0,0,0.3), -0.3px 0.3px 0px rgba(0,0,0,0.08), 1px 1px 2px rgba(0,0,0,0.12)",
                                      letterSpacing: "-0.02em",
                                      WebkitTextStroke: "0.2px rgba(0,0,0,0.15)",
                                      paintOrder: "stroke fill",
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
                                  <CheckCircle2 className="w-3.5 h-3.5" style={{ color: "#333" }} />
                                ) : f.type === "date" ? (
                                  <span className="text-[9px] truncate" style={{ color: "#333" }}>
                                    <span className="font-semibold">Date </span>{f.value}
                                  </span>
                                ) : f.label === "footer-line" ? (
                                  <div className="w-full flex items-center justify-center" style={{ borderTop: "1.5px solid #555", marginTop: "1px" }} />
                                ) : f.label === "seal" ? (
                                  <div className="w-full h-full flex items-start gap-1 px-1.5 py-0.5 overflow-hidden">
                                    <Shield className="w-3 h-3 shrink-0 text-primary mt-0.5" />
                                    <span className="text-[7px] font-mono leading-tight break-all" style={{ color: "#333", wordBreak: "break-all" }}>{f.value}</span>
                                  </div>
                                ) : (
                                  <span className="text-[10px] truncate" style={{ color: "#333" }}>{f.value}</span>
                                )}
                              </div>
                            ) : f.type === "signature" ? (
                              <>
                                <PenLine className="w-4 h-4 pointer-events-none" style={{ color: "#333" }} />
                                <span className="text-[8px] font-semibold uppercase tracking-widest select-none pointer-events-none mt-0.5 truncate max-w-full px-1" style={{ color: "#333" }}>
                                  {f.label && f.label !== "seal" && f.label !== "footer-line" ? f.label : "Sign"}
                                </span>
                              </>
                            ) : f.type === "initials" ? (
                              <>
                                <Hash className="w-3 h-3 pointer-events-none" style={{ color: "#333" }} />
                                <span className="text-[8px] font-semibold uppercase tracking-widest select-none pointer-events-none mt-0.5 truncate max-w-full px-1" style={{ color: "#333" }}>
                                  {f.label || "Initials"}
                                </span>
                              </>
                            ) : f.type === "date" ? (
                              <>
                                <CalendarDays className="w-3 h-3 pointer-events-none" style={{ color: "#333" }} />
                                <span className="text-[7px] font-semibold uppercase tracking-widest select-none pointer-events-none mt-0.5 truncate max-w-full px-1" style={{ color: "#333" }}>
                                  {f.label || "Date"}
                                </span>
                              </>
                            ) : f.type === "checkbox" ? (
                              <>
                                <CheckSquare className="w-3 h-3 pointer-events-none" style={{ color: "#333" }} />
                                {f.label && (
                                  <span className="text-[6px] font-semibold select-none pointer-events-none mt-0.5 truncate max-w-full px-0.5" style={{ color: "#333" }}>
                                    {f.label}
                                  </span>
                                )}
                              </>
                            ) : f.type === "text" ? (
                              <>
                                <Type className="w-3 h-3 pointer-events-none" style={{ color: "#333" }} />
                                <span className="text-[7px] font-semibold uppercase tracking-widest select-none pointer-events-none mt-0.5 truncate max-w-full px-1" style={{ color: "#333" }}>
                                  {f.label || "Text"}
                                </span>
                              </>
                            ) : (
                              <span className="text-[9px] font-semibold capitalize select-none pointer-events-none tracking-wide truncate max-w-full px-1" style={{ color: "#333" }}>
                                {f.label || f.type}
                              </span>
                            )}
                            {lockedFields.has(f.id) ? (
                              <div className="absolute top-0 right-0 p-0.5">
                                <Lock className="w-2 h-2 text-primary/60" />
                              </div>
                            ) : (
                              <div
                                className="absolute bottom-0 right-0 w-3 h-3 cursor-se-resize"
                                onMouseDown={(e) => handleFieldResize(f.id, e)}
                                data-testid={`resize-handle-${f.id}`}
                              >
                                <GripVertical className="w-2.5 h-2.5 text-muted-foreground rotate-[-45deg]" />
                              </div>
                            )}
                          </div>
                        );
                      })}
                      {marquee && marquee.pageNum === pageNum && (
                        <div
                          className="absolute border border-primary bg-primary/10 pointer-events-none z-40"
                          style={{
                            left: Math.min(marquee.startX, marquee.currentX) * pdfZoom,
                            top: Math.min(marquee.startY, marquee.currentY) * pdfZoom,
                            width: Math.abs(marquee.currentX - marquee.startX) * pdfZoom,
                            height: Math.abs(marquee.currentY - marquee.startY) * pdfZoom,
                          }}
                          data-testid="marquee-selection"
                        />
                      )}
                    </div>
                    {(dragTool || clipboardFields.length > 0) && (
                      <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
                        <p className="text-[10px] text-muted-foreground bg-background/80 px-2.5 py-1 rounded-md">
                          {clipboardFields.length > 0 ? `Click to paste ${clipboardFields.length} field(s)` : `Click to place ${dragTool!.type}`}
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
                    Upload a document to start placing signature fields
                  </p>
                  <input
                    type="file"
                    accept=".pdf,.xlsx,.csv,.docx"
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
                    <span
                      className="text-lg"
                      style={{
                        fontFamily: font.fontFamily,
                        textShadow: "0.5px 0.5px 0px rgba(0,0,0,0.3), -0.3px 0.3px 0px rgba(0,0,0,0.08), 1px 1px 2px rgba(0,0,0,0.12)",
                        letterSpacing: "-0.02em",
                        WebkitTextStroke: "0.2px rgba(0,0,0,0.15)",
                        paintOrder: "stroke fill",
                      }}
                    >
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
