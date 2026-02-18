import { useState, useRef, useCallback, useEffect } from "react";
import { useRoute, useLocation, Link } from "wouter";
import { useQuery, useMutation } from "@tanstack/react-query";
import { Document, Page, pdfjs } from "react-pdf";
import "react-pdf/dist/Page/AnnotationLayer.css";
import "react-pdf/dist/Page/TextLayer.css";
import {
  ArrowLeft,
  ArrowRight,
  PenLine,
  CalendarDays,
  Type,
  CheckSquare,
  Hash,
  Trash2,
  Upload,
  FileText,
  ZoomIn,
  ZoomOut,
  GripVertical,
  ChevronLeft,
  ChevronRight,
  Save,
  Plus,
  ChevronDown,
  X,
  Stamp,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
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
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { Badge } from "@/components/ui/badge";
import type { Template } from "@shared/schema";

pdfjs.GlobalWorkerOptions.workerSrc = `//unpkg.com/pdfjs-dist@${pdfjs.version}/build/pdf.worker.min.mjs`;

const FIELD_TOOLS = [
  { type: "signature", label: "Signature", icon: PenLine, w: 200, h: 60 },
  { type: "seal", label: "Seal", icon: Stamp, w: 470, h: 24 },
  { type: "date", label: "Date", icon: CalendarDays, w: 150, h: 36 },
  { type: "text", label: "Text", icon: Type, w: 200, h: 36 },
  { type: "checkbox", label: "Checkbox", icon: CheckSquare, w: 28, h: 28 },
  { type: "initials", label: "Initials", icon: Hash, w: 100, h: 50 },
] as const;

const CATEGORIES = ["Legal", "HR", "Finance", "Real Estate", "Healthcare", "General"];

interface FieldDef {
  id: string;
  type: string;
  label: string;
  page: number;
  x: number;
  y: number;
  width: number;
  height: number;
  required: boolean;
}

export default function TemplateEditorPage() {
  const [, params] = useRoute("/templates/:id/edit");
  const [, setLocation] = useLocation();
  const { toast } = useToast();
  const templateId = params?.id;
  const isNew = templateId === "new";

  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [category, setCategory] = useState("General");
  const [tags, setTags] = useState<string[]>([]);
  const [tagInput, setTagInput] = useState("");
  const [fields, setFields] = useState<FieldDef[]>([]);
  const [pdfUrl, setPdfUrl] = useState<string | null>(null);
  const [numPages, setNumPages] = useState(0);
  const [activePage, setActivePage] = useState(1);
  const [scale, setScale] = useState(1);
  const [dragTool, setDragTool] = useState<{ type: string; w: number; h: number } | null>(null);
  const [selectedField, setSelectedField] = useState<string | null>(null);
  const [selectedFields, setSelectedFields] = useState<Set<string>>(new Set());
  const [dragState, setDragState] = useState<{
    ids: string[];
    startX: number;
    startY: number;
    origins: { id: string; x: number; y: number }[];
  } | null>(null);
  const [resizeState, setResizeState] = useState<{ id: string; startX: number; startY: number; startW: number; startH: number } | null>(null);
  const [marquee, setMarquee] = useState<{
    pageNum: number;
    startX: number;
    startY: number;
    currentX: number;
    currentY: number;
  } | null>(null);
  const marqueeRef = useRef(marquee);
  marqueeRef.current = marquee;
  const [editingLabel, setEditingLabel] = useState<string | null>(null);
  const [sidebarCollapsed, setSidebarCollapsed] = useState<Record<string, boolean>>({});

  const fileInputRef = useRef<HTMLInputElement>(null);
  const pageRefs = useRef<Map<number, HTMLDivElement>>(new Map());
  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const CANVAS_WIDTH = 612;
  const CANVAS_HEIGHT = 792;

  const { data: template, isLoading } = useQuery<Template>({
    queryKey: ["/api/templates", templateId],
    enabled: !isNew && !!templateId,
  });

  useEffect(() => {
    if (template && !isNew) {
      setName(template.name);
      setDescription(template.description || "");
      setCategory(template.category || "General");
      setTags((template.tags as string[]) || []);
      const defs = (template.fieldDefs as any[]) || [];
      setFields(defs.map((f: any, i: number) => ({ ...f, id: `f-${i}-${Date.now()}` })));
    }
  }, [template, isNew]);

  const toggleSidebarSection = (key: string) => {
    setSidebarCollapsed((prev) => ({ ...prev, [key]: !prev[key] }));
  };

  const handleFileUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    if (!file.name.toLowerCase().endsWith(".pdf")) {
      toast({ title: "Only PDF files are supported for template setup", variant: "destructive" });
      return;
    }
    const url = URL.createObjectURL(file);
    setPdfUrl(url);
    setActivePage(1);
    toast({ title: "PDF loaded for field placement" });
  };

  const onDocumentLoadSuccess = ({ numPages: n }: { numPages: number }) => {
    setNumPages(n);
  };

  useEffect(() => {
    if (numPages <= 1) return;
    const container = scrollContainerRef.current;
    if (!container) return;
    const observer = new IntersectionObserver(
      (entries) => {
        let bestPage = activePage;
        let bestRatio = 0;
        entries.forEach((entry) => {
          if (entry.intersectionRatio > bestRatio) {
            const p = Number(entry.target.getAttribute("data-page-num"));
            if (p) {
              bestRatio = entry.intersectionRatio;
              bestPage = p;
            }
          }
        });
        if (bestRatio > 0) setActivePage(bestPage);
      },
      { root: container, threshold: [0, 0.25, 0.5, 0.75, 1] }
    );
    pageRefs.current.forEach((el, pageNum) => {
      el.setAttribute("data-page-num", String(pageNum));
      observer.observe(el);
    });
    return () => observer.disconnect();
  }, [numPages]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Delete" || e.key === "Backspace") {
        const tag = document.activeElement?.tagName?.toLowerCase() || "";
        const isEditable = tag === "input" || tag === "textarea" || (document.activeElement as HTMLElement)?.isContentEditable;
        if (!isEditable) {
          const idsToDelete = selectedFields.size > 0
            ? new Set(selectedFields)
            : selectedField
              ? new Set([selectedField])
              : null;
          if (idsToDelete && idsToDelete.size > 0) {
            e.preventDefault();
            setFields((prev) => prev.filter((f) => !idsToDelete.has(f.id)));
            setSelectedField(null);
            setSelectedFields(new Set());
          }
        }
      }
      if (e.key === "Escape") {
        setSelectedField(null);
        setSelectedFields(new Set());
        setDragTool(null);
      }
      if ((e.ctrlKey || e.metaKey) && e.key === "a") {
        const aTag = document.activeElement?.tagName?.toLowerCase() || "";
        const aEditable = aTag === "input" || aTag === "textarea" || (document.activeElement as HTMLElement)?.isContentEditable;
        if (!aEditable) {
          e.preventDefault();
          setSelectedFields(new Set(fields.map((f) => f.id)));
          setSelectedField(null);
        }
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [selectedField, selectedFields, fields]);

  const handlePageMouseDown = (pageNum: number, e: React.MouseEvent<HTMLDivElement>) => {
    if (dragTool) return;
    if ((e.target as HTMLElement).closest("[data-field-id]")) return;
    const rect = e.currentTarget.getBoundingClientRect();
    const x = (e.clientX - rect.left) / scale;
    const y = (e.clientY - rect.top) / scale;
    setMarquee({ pageNum, startX: x, startY: y, currentX: x, currentY: y });
    if (!e.shiftKey && !e.ctrlKey && !e.metaKey) {
      setSelectedField(null);
      setSelectedFields(new Set());
    }
  };

  const handlePageClick = (pageNum: number, e: React.MouseEvent<HTMLDivElement>) => {
    if (!dragTool) {
      if (!(e.target as HTMLElement).closest("[data-field-id]") && !marqueeRef.current) {
        setSelectedField(null);
        setSelectedFields(new Set());
      }
      return;
    }
    const rect = e.currentTarget.getBoundingClientRect();
    const x = (e.clientX - rect.left) / scale;
    const tool = FIELD_TOOLS.find((t) => t.type === dragTool.type);
    if (!tool) return;
    const FIXED_Y = CANVAS_HEIGHT * 0.75;
    const newField: FieldDef = {
      id: `f-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
      type: tool.type,
      label: tool.label,
      page: pageNum,
      x: Math.max(0, x - tool.w / 2),
      y: FIXED_Y,
      width: tool.w,
      height: tool.h,
      required: true,
    };
    setFields((prev) => [...prev, newField]);
    setSelectedField(newField.id);
    setSelectedFields(new Set());
    setDragTool(null);
  };

  const handleFieldMouseDown = (fieldId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    const isMultiKey = e.shiftKey || e.ctrlKey || e.metaKey;

    let nextSelected: Set<string>;

    if (isMultiKey) {
      nextSelected = new Set(selectedFields);
      if (selectedField) nextSelected.add(selectedField);
      if (nextSelected.has(fieldId)) {
        nextSelected.delete(fieldId);
      } else {
        nextSelected.add(fieldId);
      }
      setSelectedFields(nextSelected);
      setSelectedField(null);
    } else {
      if (selectedFields.has(fieldId) && selectedFields.size > 1) {
        nextSelected = new Set(selectedFields);
      } else {
        nextSelected = new Set([fieldId]);
        setSelectedField(fieldId);
        setSelectedFields(new Set());
      }
    }

    const allDragIds = Array.from(nextSelected);
    if (allDragIds.length === 0) allDragIds.push(fieldId);

    const origins = allDragIds.map((id) => {
      const f = fields.find((ff) => ff.id === id);
      return { id, x: f?.x ?? 0, y: f?.y ?? 0 };
    });
    setDragState({
      ids: allDragIds,
      startX: e.clientX,
      startY: e.clientY,
      origins,
    });
  };

  const handleResizeMouseDown = (fieldId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    const f = fields.find((ff) => ff.id === fieldId);
    if (!f) return;
    setResizeState({
      id: fieldId,
      startX: e.clientX,
      startY: e.clientY,
      startW: f.width,
      startH: f.height,
    });
  };

  const handleMouseMove = useCallback(
    (e: MouseEvent) => {
      if (dragState) {
        const dx = (e.clientX - dragState.startX) / scale;
        const dy = (e.clientY - dragState.startY) / scale;
        const dragIds = new Set(dragState.ids);
        setFields((prev) =>
          prev.map((f) => {
            if (!dragIds.has(f.id)) return f;
            const origin = dragState.origins.find((o) => o.id === f.id);
            if (!origin) return f;
            return { ...f, x: Math.max(0, origin.x + dx), y: Math.max(0, origin.y + dy) };
          })
        );
      }
      if (resizeState) {
        const dx = (e.clientX - resizeState.startX) / scale;
        const dy = (e.clientY - resizeState.startY) / scale;
        setFields((prev) =>
          prev.map((f) =>
            f.id === resizeState.id ? { ...f, width: Math.max(20, resizeState.startW + dx), height: Math.max(16, resizeState.startH + dy) } : f
          )
        );
      }
      if (marqueeRef.current) {
        const pageEl = pageRefs.current.get(marqueeRef.current.pageNum);
        if (pageEl) {
          const rect = pageEl.getBoundingClientRect();
          const x = (e.clientX - rect.left) / scale;
          const y = (e.clientY - rect.top) / scale;
          setMarquee((prev) => prev ? { ...prev, currentX: x, currentY: y } : null);
        }
      }
    },
    [dragState, resizeState, scale]
  );

  const handleMouseUp = useCallback((e: MouseEvent) => {
    if (marqueeRef.current) {
      const m = marqueeRef.current;
      const minX = Math.min(m.startX, m.currentX);
      const maxX = Math.max(m.startX, m.currentX);
      const minY = Math.min(m.startY, m.currentY);
      const maxY = Math.max(m.startY, m.currentY);
      if (Math.abs(maxX - minX) > 3 || Math.abs(maxY - minY) > 3) {
        const hit = fields.filter(
          (f) =>
            f.page === m.pageNum &&
            f.x + f.width > minX &&
            f.x < maxX &&
            f.y + f.height > minY &&
            f.y < maxY
        );
        if (hit.length > 0) {
          const hitIds = hit.map((f) => f.id);
          const additive = e.shiftKey || e.ctrlKey || e.metaKey;
          setSelectedFields((prev) => {
            const next = additive ? new Set(prev) : new Set<string>();
            hitIds.forEach((id) => next.add(id));
            return next;
          });
          setSelectedField(null);
        }
      }
      setMarquee(null);
    }
    setDragState(null);
    setResizeState(null);
  }, [fields]);

  useEffect(() => {
    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);
    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
    };
  }, [handleMouseMove, handleMouseUp]);

  const deleteField = (id: string) => {
    setFields((prev) => prev.filter((f) => f.id !== id));
    if (selectedField === id) setSelectedField(null);
    setSelectedFields((prev) => {
      const next = new Set(prev);
      next.delete(id);
      return next;
    });
  };

  const saveMutation = useMutation({
    mutationFn: async () => {
      const fieldDefs = fields.map(({ id, ...rest }) => rest);
      const body = { name: name || "Untitled Template", description, category, tags, fieldDefs };
      if (isNew) {
        const res = await apiRequest("POST", "/api/templates", body);
        return res.json();
      } else {
        const res = await apiRequest("PATCH", `/api/templates/${templateId}`, body);
        return res.json();
      }
    },
    onSuccess: (data: Template) => {
      queryClient.invalidateQueries({ queryKey: ["/api/templates"] });
      toast({ title: isNew ? "Template created" : "Template saved" });
      if (isNew) {
        setLocation(`/templates/${data.id}/edit`);
      }
    },
    onError: () => {
      toast({ title: "Failed to save template", variant: "destructive" });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: async () => {
      await apiRequest("DELETE", `/api/templates/${templateId}`);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["/api/templates"] });
      toast({ title: "Template deleted" });
      setLocation("/templates");
    },
    onError: () => {
      toast({ title: "Failed to delete template", variant: "destructive" });
    },
  });

  const addTag = () => {
    const t = tagInput.trim();
    if (t && !tags.includes(t)) {
      setTags((prev) => [...prev, t]);
      setTagInput("");
    }
  };

  const removeTag = (tag: string) => {
    setTags((prev) => prev.filter((t) => t !== tag));
  };

  const ICON_MAP: Record<string, typeof PenLine> = {
    signature: PenLine,
    date: CalendarDays,
    text: Type,
    checkbox: CheckSquare,
    initials: Hash,
    seal: Stamp,
  };

  const renderField = (f: FieldDef) => {
    const Icon = ICON_MAP[f.type] || FileText;
    const isSelected = selectedField === f.id || selectedFields.has(f.id);
    const isDragTarget = dragState?.ids.includes(f.id);
    return (
      <div
        key={f.id}
        data-field-id={f.id}
        className={`absolute flex items-center gap-1 border-2 rounded-sm transition-shadow ${
          isSelected
            ? "border-primary bg-primary/10 shadow-md ring-2 ring-primary/30"
            : "border-primary/40 bg-primary/5 hover:border-primary/60"
        }`}
        style={{
          left: f.x * scale,
          top: f.y * scale,
          width: f.width * scale,
          height: f.height * scale,
          cursor: isDragTarget ? "grabbing" : "grab",
          zIndex: isSelected ? 20 : 10,
        }}
        onMouseDown={(e) => handleFieldMouseDown(f.id, e)}
        data-testid={`template-field-${f.id}`}
      >
        <div className="flex items-center gap-0.5 px-1 overflow-hidden" style={{ fontSize: Math.max(8, 10 * scale) }}>
          <Icon className="shrink-0" style={{ width: Math.max(8, 10 * scale), height: Math.max(8, 10 * scale) }} />
          <span className="truncate opacity-70">{f.label}</span>
        </div>
        {isSelected && selectedFields.size <= 1 && (
          <div
            className="absolute bottom-0 right-0 w-2.5 h-2.5 bg-primary rounded-tl-sm cursor-se-resize"
            onMouseDown={(e) => handleResizeMouseDown(f.id, e)}
            data-testid={`template-field-resize-${f.id}`}
          />
        )}
      </div>
    );
  };

  if (!isNew && isLoading) {
    return (
      <div className="flex-1 overflow-auto p-5">
        <div className="max-w-3xl mx-auto space-y-3">
          <Skeleton className="h-6 w-44" />
          <Skeleton className="h-96 w-full" />
        </div>
      </div>
    );
  }

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      <div className="flex items-center justify-between gap-4 px-3.5 py-2 border-b flex-wrap">
        <div className="flex items-center gap-2.5">
          <div className="flex items-center gap-0.5">
            <Link href="/templates">
              <Button size="icon" variant="ghost" data-testid="button-template-editor-back">
                <ArrowLeft className="w-3.5 h-3.5" />
              </Button>
            </Link>
            <Link href="/">
              <Button size="icon" variant="ghost" data-testid="button-template-editor-home" title="Dashboard">
                <ArrowRight className="w-3.5 h-3.5" />
              </Button>
            </Link>
          </div>
          <div className="min-w-0">
            <h1 className="text-xs font-semibold truncate" data-testid="text-template-editor-title">
              {isNew ? "New Template" : name || "Edit Template"}
            </h1>
            <p className="text-[10px] text-muted-foreground truncate">
              {fields.length} field{fields.length !== 1 ? "s" : ""} · {pdfUrl ? `${numPages} pg${numPages !== 1 ? "s" : ""}` : "No PDF loaded"}
            </p>
          </div>
        </div>
        <div className="flex items-center gap-1.5">
          <Button
            variant="outline"
            size="sm"
            onClick={() => saveMutation.mutate()}
            disabled={saveMutation.isPending || !name.trim()}
            data-testid="button-save-template"
          >
            <Save className="w-3 h-3" />
            {saveMutation.isPending ? "Saving..." : "Save"}
          </Button>
          {!isNew && !template?.isPublic && (
            <Button
              variant="ghost"
              size="sm"
              className="text-destructive"
              onClick={() => deleteMutation.mutate()}
              disabled={deleteMutation.isPending}
              data-testid="button-delete-template"
            >
              <Trash2 className="w-3 h-3" />
            </Button>
          )}
        </div>
      </div>

      <div className="flex flex-1 overflow-hidden">
        <div className="w-56 shrink-0 border-r p-2 space-y-1.5 overflow-y-auto bg-sidebar" style={{ boxShadow: 'inset 2px 2px 6px rgba(255,255,255,0.07), inset -2px -2px 6px rgba(0,0,0,0.4), inset 0 0 20px rgba(0,0,0,0.15)' }}>
          <div>
            <button
              className="flex items-center justify-between gap-1 w-full py-0.5"
              onClick={() => toggleSidebarSection("details")}
              data-testid="button-toggle-details-section"
            >
              <p className="text-[9px] font-medium text-muted-foreground uppercase tracking-widest">
                Template Details
              </p>
              <ChevronDown className={`w-2.5 h-2.5 text-muted-foreground transition-transform ${sidebarCollapsed.details ? "-rotate-90" : ""}`} />
            </button>
            {!sidebarCollapsed.details && (
              <div className="mt-1 space-y-1.5">
                <Input
                  placeholder="Template name *"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  className="h-7 text-[11px]"
                  data-testid="input-template-name"
                />
                <Input
                  placeholder="Description (optional)"
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  className="h-7 text-[11px]"
                  data-testid="input-template-description"
                />
                <Select value={category} onValueChange={setCategory}>
                  <SelectTrigger className="h-7 text-[11px]" data-testid="select-template-category">
                    <SelectValue placeholder="Category" />
                  </SelectTrigger>
                  <SelectContent>
                    {CATEGORIES.map((cat) => (
                      <SelectItem key={cat} value={cat}>{cat}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <div className="flex items-center gap-1">
                  <Input
                    placeholder="Add tag"
                    value={tagInput}
                    onChange={(e) => setTagInput(e.target.value)}
                    onKeyDown={(e) => e.key === "Enter" && (e.preventDefault(), addTag())}
                    className="h-7 text-[11px] flex-1"
                    data-testid="input-template-tag"
                  />
                  <Button size="icon" variant="ghost" className="h-7 w-7" onClick={addTag} data-testid="button-add-tag">
                    <Plus className="w-3 h-3" />
                  </Button>
                </div>
                {tags.length > 0 && (
                  <div className="flex flex-wrap gap-1">
                    {tags.map((tag) => (
                      <Badge key={tag} variant="outline" className="text-[8px] gap-0.5 pr-1">
                        {tag}
                        <button onClick={() => removeTag(tag)} className="ml-0.5">
                          <X className="w-2 h-2" />
                        </button>
                      </Badge>
                    ))}
                  </div>
                )}
              </div>
            )}
          </div>

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
              data-testid="button-upload-template-pdf"
            >
              {pdfUrl ? <Plus className="w-3 h-3" /> : <Upload className="w-3 h-3" />}
              {pdfUrl ? "Change PDF" : "Upload PDF"}
            </Button>
          </div>

          <div>
            <button
              className="flex items-center justify-between gap-1 w-full py-0.5"
              onClick={() => toggleSidebarSection("tools")}
              data-testid="button-toggle-template-tools-section"
            >
              <p className="text-[9px] font-medium text-muted-foreground uppercase tracking-widest">
                Field Tools
              </p>
              <ChevronDown className={`w-2.5 h-2.5 text-muted-foreground transition-transform ${sidebarCollapsed.tools ? "-rotate-90" : ""}`} />
            </button>
            {!sidebarCollapsed.tools && (
              <div className="mt-0.5 space-y-0.5">
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
                    data-testid={`button-template-tool-${tool.type}`}
                  >
                    <tool.icon className="w-3 h-3" />
                    <span className="text-xs">{tool.label}</span>
                  </Button>
                ))}
              </div>
            )}
          </div>

          {fields.length > 0 && (
            <div>
              <button
                className="flex items-center justify-between gap-1 w-full py-0.5"
                onClick={() => toggleSidebarSection("placed")}
                data-testid="button-toggle-template-placed-section"
              >
                <p className="text-[9px] font-medium text-muted-foreground uppercase tracking-widest">
                  Placed ({fields.length}){selectedFields.size > 1 ? ` · ${selectedFields.size} selected` : ""}
                </p>
                <ChevronDown className={`w-2.5 h-2.5 text-muted-foreground transition-transform ${sidebarCollapsed.placed ? "-rotate-90" : ""}`} />
              </button>
              {!sidebarCollapsed.placed && (
                <div className="space-y-0.5 mt-0.5">
                  {fields.map((f) => {
                    const Icon = ICON_MAP[f.type] || FileText;
                    return (
                      <div
                        key={f.id}
                        className={`flex items-center justify-between gap-1 p-1 rounded-md text-[10px] hover-elevate cursor-pointer ${selectedField === f.id || selectedFields.has(f.id) ? "bg-accent" : ""}`}
                        onClick={(e) => {
                          if (e.shiftKey || e.ctrlKey || e.metaKey) {
                            setSelectedFields((prev) => {
                              const next = new Set(prev);
                              if (next.has(f.id)) next.delete(f.id);
                              else next.add(f.id);
                              if (selectedField && !next.has(selectedField)) next.add(selectedField);
                              return next;
                            });
                            setSelectedField(null);
                          } else {
                            setSelectedField(f.id);
                            setSelectedFields(new Set());
                          }
                          setActivePage(f.page);
                        }}
                        data-testid={`template-field-item-${f.id}`}
                      >
                        <span className="flex items-center gap-1 truncate">
                          <Icon className="w-2.5 h-2.5 text-primary shrink-0" />
                          {editingLabel === f.id ? (
                            <input
                              autoFocus
                              className="bg-transparent border-b border-primary text-[10px] w-20 outline-none"
                              value={f.label}
                              onChange={(e) =>
                                setFields((prev) =>
                                  prev.map((ff) => (ff.id === f.id ? { ...ff, label: e.target.value } : ff))
                                )
                              }
                              onBlur={() => setEditingLabel(null)}
                              onKeyDown={(e) => e.key === "Enter" && setEditingLabel(null)}
                              data-testid={`input-field-label-${f.id}`}
                            />
                          ) : (
                            <span
                              className="truncate"
                              onDoubleClick={() => setEditingLabel(f.id)}
                              title="Double-click to rename"
                            >
                              {f.label}
                            </span>
                          )}
                        </span>
                        <div className="flex items-center gap-0.5 shrink-0">
                          <Button
                            size="icon"
                            variant="ghost"
                            className="h-4 w-4"
                            onClick={(e) => {
                              e.stopPropagation();
                              setFields((prev) =>
                                prev.map((ff) => (ff.id === f.id ? { ...ff, required: !ff.required } : ff))
                              );
                            }}
                            title={f.required ? "Required" : "Optional"}
                            data-testid={`button-toggle-required-${f.id}`}
                          >
                            <span className={`text-[7px] font-bold ${f.required ? "text-primary" : "text-muted-foreground"}`}>
                              {f.required ? "R" : "O"}
                            </span>
                          </Button>
                          <Button
                            size="icon"
                            variant="ghost"
                            className="h-4 w-4"
                            onClick={(e) => {
                              e.stopPropagation();
                              deleteField(f.id);
                            }}
                            data-testid={`button-delete-field-${f.id}`}
                          >
                            <Trash2 className="w-2.5 h-2.5" />
                          </Button>
                        </div>
                      </div>
                    );
                  })}
                </div>
              )}
            </div>
          )}

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
                    const p = Math.max(1, activePage - 1);
                    setActivePage(p);
                    pageRefs.current.get(p)?.scrollIntoView({ behavior: "smooth", block: "center" });
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
                    const p = Math.min(numPages, activePage + 1);
                    setActivePage(p);
                    pageRefs.current.get(p)?.scrollIntoView({ behavior: "smooth", block: "center" });
                  }}
                >
                  <ChevronRight className="w-3 h-3" />
                </Button>
              </div>
            </div>
          )}
        </div>

        <div
          ref={scrollContainerRef}
          className="flex-1 overflow-auto bg-muted/30 p-5 flex flex-col items-center gap-6"
          style={{ cursor: dragTool ? "crosshair" : "default" }}
        >
          <div className="flex items-center gap-2 mb-2">
            <Tooltip>
              <TooltipTrigger asChild>
                <Button size="icon" variant="ghost" onClick={() => setScale((s) => Math.max(0.5, s - 0.1))} data-testid="button-template-zoom-out">
                  <ZoomOut className="w-3.5 h-3.5" />
                </Button>
              </TooltipTrigger>
              <TooltipContent><p className="text-xs">Zoom Out</p></TooltipContent>
            </Tooltip>
            <span className="text-[10px] text-muted-foreground tabular-nums w-10 text-center" data-testid="text-template-zoom">
              {Math.round(scale * 100)}%
            </span>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button size="icon" variant="ghost" onClick={() => setScale((s) => Math.min(2, s + 0.1))} data-testid="button-template-zoom-in">
                  <ZoomIn className="w-3.5 h-3.5" />
                </Button>
              </TooltipTrigger>
              <TooltipContent><p className="text-xs">Zoom In</p></TooltipContent>
            </Tooltip>
          </div>

          {pdfUrl ? (
            <Document
              file={pdfUrl}
              onLoadSuccess={onDocumentLoadSuccess}
              loading={<Skeleton className="w-[612px] h-[792px]" />}
            >
              {Array.from({ length: numPages }, (_, i) => i + 1).map((pageNum) => (
                <div
                  key={pageNum}
                  ref={(el) => {
                    if (el) pageRefs.current.set(pageNum, el);
                  }}
                  className="relative mb-6 shadow-lg bg-white"
                  style={{ width: CANVAS_WIDTH * scale, height: CANVAS_HEIGHT * scale }}
                  onMouseDown={(e) => handlePageMouseDown(pageNum, e)}
                  onClick={(e) => handlePageClick(pageNum, e)}
                  data-testid={`template-page-${pageNum}`}
                >
                  <Page
                    pageNumber={pageNum}
                    width={CANVAS_WIDTH * scale}
                    renderTextLayer={false}
                    renderAnnotationLayer={false}
                  />

                  {fields
                    .filter((f) => f.page === pageNum)
                    .map((f) => renderField(f))}

                  {marquee && marquee.pageNum === pageNum && (
                    <div
                      className="absolute border border-primary/60 bg-primary/10 pointer-events-none"
                      style={{
                        left: Math.min(marquee.startX, marquee.currentX) * scale,
                        top: Math.min(marquee.startY, marquee.currentY) * scale,
                        width: Math.abs(marquee.currentX - marquee.startX) * scale,
                        height: Math.abs(marquee.currentY - marquee.startY) * scale,
                        zIndex: 50,
                      }}
                    />
                  )}
                </div>
              ))}
            </Document>
          ) : (
            <div
              ref={(el) => {
                if (el) pageRefs.current.set(1, el);
              }}
              className="relative shadow-lg bg-white"
              style={{ width: CANVAS_WIDTH * scale, height: CANVAS_HEIGHT * scale }}
              onMouseDown={(e) => handlePageMouseDown(1, e)}
              onClick={(e) => handlePageClick(1, e)}
              data-testid="template-page-blank"
            >
              <svg
                width={CANVAS_WIDTH * scale}
                height={CANVAS_HEIGHT * scale}
                className="absolute inset-0 pointer-events-none"
                xmlns="http://www.w3.org/2000/svg"
              >
                {Array.from({ length: Math.floor(CANVAS_WIDTH / 72) + 1 }, (_, i) => (
                  <line
                    key={`v-${i}`}
                    x1={i * 72 * scale}
                    y1={0}
                    x2={i * 72 * scale}
                    y2={CANVAS_HEIGHT * scale}
                    stroke={i === 0 ? "rgba(0,0,0,0.15)" : "rgba(0,0,0,0.06)"}
                    strokeWidth={i % 2 === 0 ? 0.8 : 0.4}
                  />
                ))}
                {Array.from({ length: Math.floor(CANVAS_HEIGHT / 72) + 1 }, (_, i) => (
                  <line
                    key={`h-${i}`}
                    x1={0}
                    y1={i * 72 * scale}
                    x2={CANVAS_WIDTH * scale}
                    y2={i * 72 * scale}
                    stroke={i === 0 ? "rgba(0,0,0,0.15)" : "rgba(0,0,0,0.06)"}
                    strokeWidth={i % 2 === 0 ? 0.8 : 0.4}
                  />
                ))}
                {Array.from({ length: Math.floor(CANVAS_WIDTH / 72) }, (_, i) => (
                  <text
                    key={`vl-${i}`}
                    x={(i + 1) * 72 * scale}
                    y={10 * scale}
                    textAnchor="middle"
                    fontSize={8 * scale}
                    fill="rgba(0,0,0,0.25)"
                  >
                    {i + 1}"
                  </text>
                ))}
                {Array.from({ length: Math.floor(CANVAS_HEIGHT / 72) }, (_, i) => (
                  <text
                    key={`hl-${i}`}
                    x={6 * scale}
                    y={(i + 1) * 72 * scale + 3 * scale}
                    fontSize={8 * scale}
                    fill="rgba(0,0,0,0.25)"
                  >
                    {i + 1}"
                  </text>
                ))}
                <rect
                  x={36 * scale}
                  y={36 * scale}
                  width={(CANVAS_WIDTH - 72) * scale}
                  height={(CANVAS_HEIGHT - 72) * scale}
                  fill="none"
                  stroke="rgba(0,0,0,0.08)"
                  strokeWidth={0.5}
                  strokeDasharray={`${4 * scale} ${4 * scale}`}
                />
              </svg>
              <div
                className="absolute inset-0 flex items-center justify-center pointer-events-none"
                style={{ opacity: fields.filter((f) => f.page === 1).length > 0 ? 0 : 0.15 }}
              >
                <div className="text-center">
                  <p className="text-xs font-medium text-gray-500">8.5" x 11" Canvas</p>
                  <p className="text-[10px] text-gray-400 mt-1">Select a field tool, then click to place</p>
                </div>
              </div>
              {fields
                .filter((f) => f.page === 1)
                .map((f) => renderField(f))}

              {marquee && marquee.pageNum === 1 && (
                <div
                  className="absolute border border-primary/60 bg-primary/10 pointer-events-none"
                  style={{
                    left: Math.min(marquee.startX, marquee.currentX) * scale,
                    top: Math.min(marquee.startY, marquee.currentY) * scale,
                    width: Math.abs(marquee.currentX - marquee.startX) * scale,
                    height: Math.abs(marquee.currentY - marquee.startY) * scale,
                    zIndex: 50,
                  }}
                />
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
