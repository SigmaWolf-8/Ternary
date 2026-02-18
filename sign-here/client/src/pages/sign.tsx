import { useState, useRef, useCallback, useEffect } from "react";
import { useRoute, useLocation } from "wouter";
import { useQuery, useMutation } from "@tanstack/react-query";
import { Document, Page, pdfjs } from "react-pdf";
import "react-pdf/dist/Page/AnnotationLayer.css";
import "react-pdf/dist/Page/TextLayer.css";
import confetti from "canvas-confetti";
import {
  CheckCircle2,
  PenLine,
  Type,
  Eraser,
  Shield,
  FileText,
  ZoomIn,
  ZoomOut,
  CalendarDays,
  CheckSquare,
  Hash,
  ShieldCheck,
  ChevronDown,
  X,
  Undo2,
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
import { apiRequest, queryClient } from "@/lib/queryClient";
import { useToast } from "@/hooks/use-toast";
import { useIsMobile } from "@/hooks/use-mobile";
import { formatDateWithTimezone, getSettings } from "@/pages/settings";
import type { Envelope, Recipient, Field as FieldType } from "@shared/schema";

pdfjs.GlobalWorkerOptions.workerSrc = `https://unpkg.com/pdfjs-dist@${pdfjs.version}/build/pdf.worker.min.mjs`;

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

const PDF_BASE_WIDTH = 800;

function hapticTap() { navigator.vibrate?.(12); }
function hapticStroke() { navigator.vibrate?.(8); }
function hapticSuccess() { navigator.vibrate?.([30, 20, 60, 20, 90]); }
function hapticCompletion() { navigator.vibrate?.([50, 30, 80, 30, 120, 50, 200]); }
function hapticBump() { navigator.vibrate?.(20); }

function fireGoldConfetti() {
  confetti({
    particleCount: 100,
    spread: 70,
    colors: ["#FFD700", "#D4AF37", "#FFCC00"],
    gravity: 0.5,
    startVelocity: 30,
    ticks: 300,
    origin: { y: 0.6 },
  });
}

export default function Sign() {
  const [, params] = useRoute("/sign/:envelopeId/:recipientId");
  const [, setLocation] = useLocation();
  const { toast } = useToast();
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const isMobile = useIsMobile();
  const containerRef = useRef<HTMLDivElement>(null);
  const [containerWidth, setContainerWidth] = useState(0);

  const envelopeId = params?.envelopeId || "";
  const recipientId = params?.recipientId || "";

  const [sigDialogOpen, setSigDialogOpen] = useState(false);
  const [activeFieldId, setActiveFieldId] = useState<string | null>(null);
  const [sigMode, setSigMode] = useState<"draw" | "type">("type");
  const [typedName, setTypedName] = useState("");
  const [selectedFont, setSelectedFont] = useState(0);
  const [isDrawing, setIsDrawing] = useState(false);
  const [fieldValues, setFieldValues] = useState<Record<string, string>>({});
  const [completed, setCompleted] = useState(false);
  const [numPages, setNumPages] = useState(0);
  const [pdfZoom, setPdfZoom] = useState(1);
  const [textEditField, setTextEditField] = useState<string | null>(null);
  const [textEditValue, setTextEditValue] = useState("");
  const [drawStrokes, setDrawStrokes] = useState<ImageData[]>([]);
  const [mobileInkColor, setMobileInkColor] = useState("#FFD700");
  const [highlightFieldId, setHighlightFieldId] = useState<string | null>(null);
  const [mobileCanvasSize, setMobileCanvasSize] = useState({ w: 360, h: 200 });

  useEffect(() => {
    if (!isMobile || !containerRef.current) return;
    const measure = () => {
      if (containerRef.current) {
        setContainerWidth(containerRef.current.clientWidth);
      }
    };
    measure();
    const ro = new ResizeObserver(measure);
    ro.observe(containerRef.current);
    return () => ro.disconnect();
  }, [isMobile]);

  useEffect(() => {
    if (!isMobile) return;
    const update = () => {
      setMobileCanvasSize({
        w: Math.min(window.innerWidth - 32, 600),
        h: Math.min(window.innerHeight - 280, 300),
      });
    };
    update();
    window.addEventListener("resize", update);
    window.addEventListener("orientationchange", update);
    return () => {
      window.removeEventListener("resize", update);
      window.removeEventListener("orientationchange", update);
    };
  }, [isMobile]);

  const mobilePdfWidth = containerWidth > 0 ? containerWidth : 360;
  const mobileZoom = mobilePdfWidth / PDF_BASE_WIDTH;
  const effectiveZoom = isMobile ? mobileZoom : pdfZoom;
  const effectiveWidth = isMobile ? mobilePdfWidth : PDF_BASE_WIDTH * pdfZoom;

  const { data: envelope, isLoading: envLoading } = useQuery<Envelope>({
    queryKey: ["/api/envelopes", envelopeId],
  });

  const { data: recipient } = useQuery<Recipient>({
    queryKey: ["/api/recipients", recipientId],
  });

  const { data: fields } = useQuery<FieldType[]>({
    queryKey: ["/api/envelopes", envelopeId, "fields"],
  });

  const { data: allRecipients } = useQuery<Recipient[]>({
    queryKey: ["/api/envelopes", envelopeId, "recipients"],
  });

  const hasPdf = !!envelope?.pdfData;
  const pdfUrl = hasPdf ? `/api/envelopes/${envelopeId}/pdf` : null;
  const activeRecipientIds = new Set((allRecipients || []).map((r) => r.id));
  const myFields = fields?.filter((f) => {
    if (f.recipientId === recipientId) return true;
    if (f.recipientId && !activeRecipientIds.has(f.recipientId)) return true;
    return false;
  }) || [];
  const otherFields = fields?.filter((f) => f.recipientId !== recipientId && f.recipientId && activeRecipientIds.has(f.recipientId)) || [];

  useEffect(() => {
    if (envelope?.pageCount) setNumPages(envelope.pageCount);
  }, [envelope]);

  const myFieldIds = myFields.map((f) => f.id).join(",");
  useEffect(() => {
    if (myFields.length > 0) {
      setFieldValues((prev) => {
        const next = { ...prev };
        let changed = false;
        myFields.forEach((f) => {
          if (f.type === "date" && !next[f.id]) {
            next[f.id] = formatDateWithTimezone(new Date());
            changed = true;
          } else if (f.value && !next[f.id]) {
            next[f.id] = f.value;
            changed = true;
          }
        });
        return changed ? next : prev;
      });
    }
  }, [myFieldIds]);

  const signMutation = useMutation({
    mutationFn: async (data: { fieldValues: Record<string, string> }) => {
      const res = await apiRequest("POST", `/api/envelopes/${envelopeId}/sign`, {
        recipientId,
        fieldValues: data.fieldValues,
      });
      return res.json();
    },
    onSuccess: (data: any) => {
      setCompleted(true);
      queryClient.invalidateQueries({ queryKey: ["/api/envelopes", envelopeId] });
      if (isMobile) {
        hapticCompletion();
        setTimeout(fireGoldConfetti, 300);
      }
      if (data?.certified) {
        toast({ title: "Document signed and certified", description: `All ${data.totalSigners}/${data.totalSigners} parties completed` });
      } else {
        toast({ title: "Document signed successfully", description: data?.signedCount ? `${data.signedCount}/${data.totalSigners} parties completed` : undefined });
      }
    },
    onError: (error: Error) => {
      toast({ title: "Error", description: error.message, variant: "destructive" });
    },
  });

  const clearCanvas = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    if (isMobile) setDrawStrokes([]);
  }, [isMobile]);

  const undoStroke = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    if (drawStrokes.length > 0) {
      const prev = drawStrokes.slice(0, -1);
      setDrawStrokes(prev);
      if (prev.length === 0) {
        ctx.clearRect(0, 0, canvas.width, canvas.height);
      } else {
        ctx.putImageData(prev[prev.length - 1], 0, 0);
      }
      if (isMobile) hapticTap();
    }
  }, [drawStrokes, isMobile]);

  const startDraw = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    if (isMobile) {
      const imgData = ctx.getImageData(0, 0, canvas.width, canvas.height);
      setDrawStrokes((prev) => [...prev, imgData]);
      hapticStroke();
    }
    setIsDrawing(true);
    const rect = canvas.getBoundingClientRect();
    ctx.beginPath();
    ctx.moveTo(e.clientX - rect.left, e.clientY - rect.top);
    ctx.strokeStyle = isMobile ? mobileInkColor : "hsl(40 65% 50%)";
    ctx.lineWidth = isMobile ? 3 : 2;
    ctx.lineCap = "round";
    ctx.lineJoin = "round";
  }, [isMobile, mobileInkColor]);

  const draw = useCallback(
    (e: React.MouseEvent<HTMLCanvasElement>) => {
      if (!isDrawing) return;
      const canvas = canvasRef.current;
      if (!canvas) return;
      const ctx = canvas.getContext("2d");
      if (!ctx) return;
      const rect = canvas.getBoundingClientRect();
      ctx.lineTo(e.clientX - rect.left, e.clientY - rect.top);
      ctx.stroke();
    },
    [isDrawing]
  );

  const endDraw = useCallback(() => {
    setIsDrawing(false);
  }, []);

  const startDrawTouch = useCallback((e: React.TouchEvent<HTMLCanvasElement>) => {
    e.preventDefault();
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    const imgData = ctx.getImageData(0, 0, canvas.width, canvas.height);
    setDrawStrokes((prev) => [...prev, imgData]);
    setIsDrawing(true);
    const rect = canvas.getBoundingClientRect();
    const touch = e.touches[0];
    ctx.beginPath();
    ctx.moveTo(touch.clientX - rect.left, touch.clientY - rect.top);
    ctx.strokeStyle = mobileInkColor;
    ctx.lineWidth = 3;
    ctx.lineCap = "round";
    ctx.lineJoin = "round";
    hapticStroke();
  }, [mobileInkColor]);

  const drawTouch = useCallback(
    (e: React.TouchEvent<HTMLCanvasElement>) => {
      e.preventDefault();
      if (!isDrawing) return;
      const canvas = canvasRef.current;
      if (!canvas) return;
      const ctx = canvas.getContext("2d");
      if (!ctx) return;
      const rect = canvas.getBoundingClientRect();
      const touch = e.touches[0];
      ctx.lineTo(touch.clientX - rect.left, touch.clientY - rect.top);
      ctx.stroke();
    },
    [isDrawing]
  );

  const endDrawTouch = useCallback((e: React.TouchEvent<HTMLCanvasElement>) => {
    e.preventDefault();
    setIsDrawing(false);
  }, []);

  const applySig = () => {
    if (!activeFieldId) return;

    if (sigMode === "type") {
      if (!typedName.trim()) {
        if (isMobile) toast({ title: "Enter your name first" });
        return;
      }
      setFieldValues((prev) => ({
        ...prev,
        [activeFieldId]: `typed:${selectedFont}:${typedName}`,
      }));
    } else {
      const canvas = canvasRef.current;
      if (!canvas) return;
      if (isMobile) {
        const ctx = canvas.getContext("2d");
        if (ctx) {
          const imgData = ctx.getImageData(0, 0, canvas.width, canvas.height);
          const isEmpty = !imgData.data.some((ch, i) => i % 4 === 3 && ch > 0);
          if (isEmpty) {
            toast({ title: "Draw your signature first" });
            return;
          }
        }
      }
      const dataUrl = canvas.toDataURL();
      setFieldValues((prev) => ({ ...prev, [activeFieldId]: `drawn:${dataUrl}` }));
    }

    if (isMobile) hapticSuccess();
    setSigDialogOpen(false);
    setActiveFieldId(null);
    if (isMobile) setDrawStrokes([]);
  };

  const isFieldVisible = useCallback((field: FieldType): boolean => {
    if (!field.dependsOnFieldId) return true;
    const depValue = fieldValues[field.dependsOnFieldId] || "";
    const expectedValue = field.dependsOnValue || "checked";
    if (expectedValue === "checked") return depValue === "checked";
    if (expectedValue === "unchecked") return depValue !== "checked";
    if (expectedValue === "filled") return depValue.length > 0;
    if (expectedValue === "empty") return depValue.length === 0;
    return depValue === expectedValue;
  }, [fieldValues]);

  const visibleMyFields = myFields.filter(isFieldVisible);
  const visibleOtherFields = otherFields.filter(isFieldVisible);
  const completedCount = visibleMyFields.filter((f) => !!fieldValues[f.id]).length;
  const totalFields = visibleMyFields.length;
  const allFieldsComplete = totalFields > 0 && completedCount === totalFields;

  const handleFieldClick = (field: FieldType) => {
    if (isMobile) {
      hapticTap();
      setHighlightFieldId(field.id);
      setTimeout(() => setHighlightFieldId(null), 400);
    }

    if (field.type === "signature" || field.type === "initials") {
      setActiveFieldId(field.id);
      if (isMobile) {
        setTimeout(() => setSigDialogOpen(true), isMobile ? 200 : 0);
      } else {
        setSigDialogOpen(true);
      }
      const settings = getSettings();
      setTypedName(settings.displayName || recipient?.name || "");
    } else if (field.type === "date") {
      setFieldValues((prev) => ({
        ...prev,
        [field.id]: formatDateWithTimezone(new Date()),
      }));
    } else if (field.type === "checkbox") {
      setFieldValues((prev) => ({
        ...prev,
        [field.id]: prev[field.id] === "checked" ? "" : "checked",
      }));
    } else if (field.type === "text") {
      setTextEditField(field.id);
      setTextEditValue(fieldValues[field.id] || "");
    }
  };

  const applyTextValue = () => {
    if (textEditField) {
      setFieldValues((prev) => ({ ...prev, [textEditField]: textEditValue }));
      setTextEditField(null);
      setTextEditValue("");
      if (isMobile) hapticTap();
    }
  };

  const handleSubmit = () => {
    const requiredFields = visibleMyFields.filter((f) => f.required);
    const missingFields = requiredFields.filter((f) => !fieldValues[f.id]);

    if (missingFields.length > 0) {
      toast({
        title: "Missing required fields",
        description: `Please complete all ${missingFields.length} required field(s)`,
        variant: "destructive",
      });
      return;
    }

    const finalValues = { ...fieldValues };
    visibleMyFields.forEach((f) => {
      if (f.type === "date") {
        finalValues[f.id] = formatDateWithTimezone(new Date());
      }
    });
    setFieldValues(finalValues);

    signMutation.mutate({ fieldValues: finalValues });
  };

  const scrollToNextField = useCallback(() => {
    const nextField = visibleMyFields.find((f) => !fieldValues[f.id]);
    if (nextField) {
      const el = document.querySelector(`[data-testid="sign-field-${nextField.id}"]`);
      if (el) {
        el.scrollIntoView({ behavior: "smooth", block: "center" });
        setHighlightFieldId(nextField.id);
        setTimeout(() => setHighlightFieldId(null), 600);
        hapticBump();
      }
    }
  }, [visibleMyFields, fieldValues]);

  const fieldsOnPage = (page: number) => visibleMyFields.filter((f) => f.page === page);
  const otherFieldsOnPage = (page: number) => visibleOtherFields.filter((f) => f.page === page);

  const renderFieldValue = (f: FieldType, value: string, zoom: number) => {
    if (value.startsWith("typed:")) {
      const fontIdx = parseInt(value.split(":")[1]) || 0;
      const text = value.split(":").slice(2).join(":");
      return (
        <span
          className="truncate"
          style={{
            color: "#1a1a1a",
            fontFamily: FONT_STYLES[fontIdx]?.fontFamily,
            fontSize: `${Math.max(10, 14 * zoom)}px`,
            textShadow: "0.5px 0.5px 0px rgba(0,0,0,0.3), -0.3px 0.3px 0px rgba(0,0,0,0.08), 1px 1px 2px rgba(0,0,0,0.12)",
            letterSpacing: "-0.02em",
            WebkitTextStroke: "0.2px rgba(0,0,0,0.15)",
            paintOrder: "stroke fill",
          }}
        >
          {text}
        </span>
      );
    }
    if (value.startsWith("drawn:")) {
      return <img src={value.replace("drawn:", "")} alt="Signature" className="max-w-full max-h-full object-contain" />;
    }
    if (f.type === "checkbox") {
      return <CheckCircle2 className="w-3.5 h-3.5 text-emerald-500" />;
    }
    return <span className="text-[10px]">{value}</span>;
  };

  const renderFieldPlaceholder = (f: FieldType, mobile: boolean) => {
    const iconClass = mobile ? "w-5 h-5" : "w-4 h-4";
    const smallIconClass = mobile ? "w-4 h-4" : "w-3 h-3";
    const labelClass = mobile
      ? "text-[10px] font-semibold text-amber-400/80 uppercase tracking-widest"
      : "text-[8px] font-medium text-primary/60 uppercase tracking-widest";
    const smallLabelClass = mobile
      ? "text-[9px] font-semibold text-amber-400/80 uppercase tracking-widest"
      : "text-[7px] font-medium text-primary/60 uppercase tracking-widest";

    if (f.type === "signature") {
      return (
        <>
          <PenLine className={`${iconClass} ${mobile ? "text-amber-400/60" : "text-primary/50"}`} />
          <span className={labelClass}>{mobile ? "Tap to Sign" : "Click to Sign"}</span>
        </>
      );
    }
    if (f.type === "initials") {
      return (
        <>
          <Hash className={`${smallIconClass} ${mobile ? "text-amber-400/60" : "text-primary/50"}`} />
          <span className={smallLabelClass}>Initials</span>
        </>
      );
    }
    if (f.type === "date") {
      return (
        <>
          <CalendarDays className={`${smallIconClass} ${mobile ? "text-amber-400/60" : "text-primary/50"}`} />
          <span className={smallLabelClass}>Date (HPTP)</span>
        </>
      );
    }
    if (f.type === "checkbox") {
      return <CheckSquare className={`${mobile ? "w-5 h-5 text-amber-400/50" : "w-3.5 h-3.5 text-primary/40"}`} />;
    }
    return (
      <>
        <Type className={`${smallIconClass} ${mobile ? "text-amber-400/60" : "text-primary/50"}`} />
        <span className={smallLabelClass}>{mobile ? "Tap to Enter" : "Enter Text"}</span>
      </>
    );
  };

  const getFieldStyle = (f: FieldType, zoom: number, mobile: boolean) => {
    const rawW = f.width * zoom;
    const rawH = f.height * zoom;
    const minSize = mobile ? 44 : 0;
    const w = Math.max(rawW, minSize);
    const h = Math.max(rawH, minSize);
    const offsetX = (w - rawW) / 2;
    const offsetY = (h - rawH) / 2;
    return {
      left: f.x * zoom - offsetX,
      top: f.y * zoom - offsetY,
      width: w,
      height: h,
    };
  };

  if (envLoading) {
    return (
      <div className={`min-h-screen flex items-center justify-center ${isMobile ? "bg-zinc-950" : "bg-background"}`}>
        <div className="space-y-3 w-full max-w-lg p-5">
          <Skeleton className="h-6 w-44" />
          <Skeleton className="h-[500px] w-full" />
        </div>
      </div>
    );
  }

  if (completed) {
    if (isMobile) {
      return (
        <div
          className="fixed inset-0 z-50 flex flex-col items-center justify-center px-6"
          style={{
            background: "linear-gradient(135deg, #0a0a0a 0%, #1a1a2e 50%, #0a0a0a 100%)",
            minHeight: "100dvh",
          }}
        >
          <div className="animate-in fade-in slide-in-from-bottom-4 duration-500 flex flex-col items-center text-center">
            <div
              className="w-20 h-20 rounded-full flex items-center justify-center mb-5"
              style={{
                background: "radial-gradient(circle, rgba(255,215,0,0.15), transparent 70%)",
                boxShadow: "0 0 40px rgba(255,215,0,0.2)",
              }}
            >
              <ShieldCheck className="w-10 h-10 text-amber-400" />
            </div>
            <h1
              className="text-2xl font-bold text-white mb-2 tracking-tight"
              style={{
                textShadow: "0 0 20px rgba(255,215,0,0.3)",
              }}
              data-testid="text-sign-complete"
            >
              Document Sealed
            </h1>
            <p className="text-sm text-zinc-400 mb-8 leading-relaxed max-w-xs">
              Your signature has been eternally bound with femtosecond precision.
            </p>
            <div className="flex flex-col gap-3 w-full max-w-xs">
              <Button
                onClick={() => window.close()}
                className="h-12 rounded-full text-sm font-semibold"
                style={{
                  background: "linear-gradient(135deg, #FFD700, #D4AF37)",
                  color: "#0a0a0a",
                  boxShadow: "0 4px 16px rgba(255,215,0,0.3)",
                }}
                data-testid="button-return-sanctuary"
              >
                Return to Sanctuary
              </Button>
              {envelope?.status === "completed" && (
                <Button
                  variant="outline"
                  onClick={() => setLocation(`/envelope/${envelopeId}/certificate`)}
                  className="h-12 rounded-full text-sm border-amber-500/30 text-amber-400"
                  data-testid="button-claim-cert"
                >
                  Claim Your Certified Copy
                </Button>
              )}
            </div>
            <div className="flex items-center gap-1.5 mt-8 text-[10px] text-zinc-500 uppercase tracking-widest">
              <Shield className="w-3 h-3" />
              <span>Quantum-Secure Verified</span>
            </div>
          </div>
        </div>
      );
    }

    return (
      <div className="min-h-screen bg-background flex items-center justify-center">
        <Card className="max-w-sm w-full mx-4">
          <CardContent className="p-8 flex flex-col items-center text-center">
            <div className="w-12 h-12 rounded-full bg-emerald-500/10 flex items-center justify-center mb-3">
              <CheckCircle2 className="w-5 h-5 text-emerald-500" />
            </div>
            <h1 className="text-sm font-semibold mb-1.5" data-testid="text-sign-complete">
              Document Signed
            </h1>
            <p className="text-[11px] text-muted-foreground mb-5 leading-relaxed">
              Your signature has been recorded and secured. You may close this page.
            </p>
            <div className="flex items-center gap-1.5 text-[9px] text-muted-foreground uppercase tracking-widest">
              <Shield className="w-2.5 h-2.5" />
              <span>Quantum-secure verified</span>
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  if (recipient?.status === "signed") {
    return (
      <div className={`min-h-screen flex items-center justify-center ${isMobile ? "bg-zinc-950" : "bg-background"}`}>
        <Card className={`max-w-sm w-full mx-4 ${isMobile ? "bg-zinc-900 border-zinc-800" : ""}`}>
          <CardContent className="p-8 flex flex-col items-center text-center">
            <div className="w-12 h-12 rounded-full bg-emerald-500/10 flex items-center justify-center mb-3">
              <CheckCircle2 className="w-5 h-5 text-emerald-500" />
            </div>
            <h1 className={`text-sm font-semibold mb-1.5 ${isMobile ? "text-white" : ""}`}>Already Signed</h1>
            <p className={`text-[11px] ${isMobile ? "text-zinc-400" : "text-muted-foreground"}`}>
              This document has already been signed.
            </p>
          </CardContent>
        </Card>
      </div>
    );
  }

  const mobileSignaturePad = sigDialogOpen && isMobile && (
    <div
      className="fixed inset-0 z-[60] flex flex-col"
      style={{
        background: "#0a0a0a",
        paddingTop: "env(safe-area-inset-top)",
        paddingBottom: "env(safe-area-inset-bottom)",
      }}
    >
      <div className="flex items-center justify-between px-4 py-3 border-b border-zinc-800">
        <span className="text-sm font-semibold text-amber-400 tracking-wide">
          {activeFieldId && myFields.find((f) => f.id === activeFieldId)?.type === "initials"
            ? "Add Initials"
            : "Sign Here"}
        </span>
        <Button
          size="icon"
          variant="ghost"
          onClick={() => { setSigDialogOpen(false); setActiveFieldId(null); setDrawStrokes([]); }}
          className="text-zinc-400"
          data-testid="button-close-mobile-sig"
        >
          <X className="w-5 h-5" />
        </Button>
      </div>

      <Tabs value={sigMode} onValueChange={(v) => setSigMode(v as "draw" | "type")} className="flex-1 flex flex-col">
        <TabsList className="mx-4 mt-3 bg-zinc-900 border border-zinc-800">
          <TabsTrigger value="type" className="flex-1 text-sm data-[state=active]:bg-amber-500/20 data-[state=active]:text-amber-400" data-testid="tab-type-sig">
            <Type className="w-4 h-4 mr-1.5" />
            Type
          </TabsTrigger>
          <TabsTrigger value="draw" className="flex-1 text-sm data-[state=active]:bg-amber-500/20 data-[state=active]:text-amber-400" data-testid="tab-draw-sig">
            <PenLine className="w-4 h-4 mr-1.5" />
            Draw
          </TabsTrigger>
        </TabsList>

        <TabsContent value="type" className="flex-1 overflow-y-auto px-4 pt-4 space-y-4">
          <Input
            value={typedName}
            onChange={(e) => setTypedName(e.target.value)}
            placeholder="Type your name"
            className="h-12 text-base bg-zinc-900 border-zinc-700 text-white placeholder:text-zinc-500"
            style={{ fontSize: "16px" }}
            data-testid="input-typed-name"
          />
          <div className="grid grid-cols-2 gap-2.5">
            {FONT_STYLES.map((font, i) => (
              <div
                key={font.name}
                className={`p-3 rounded-xl border cursor-pointer text-center transition-all ${
                  selectedFont === i
                    ? "border-amber-500/60 bg-amber-500/10"
                    : "border-zinc-700 bg-zinc-900"
                }`}
                onClick={() => { setSelectedFont(i); hapticTap(); }}
                data-testid={`font-option-${i}`}
              >
                <span
                  className="text-base text-white"
                  style={{
                    fontFamily: font.fontFamily,
                    textShadow: "0.5px 0.5px 0px rgba(0,0,0,0.3)",
                  }}
                >
                  {typedName || "Your Name"}
                </span>
                <p className="text-[9px] text-zinc-500 mt-1 tracking-wider uppercase">{font.name}</p>
              </div>
            ))}
          </div>
        </TabsContent>

        <TabsContent value="draw" className="flex-1 flex flex-col px-4 pt-3">
          <div className="flex items-center gap-2 mb-3">
            {["#FFD700", "#1a1a2e", "#3B82F6", "#EF4444"].map((c) => (
              <button
                key={c}
                className={`w-8 h-8 rounded-full border-2 transition-all ${mobileInkColor === c ? "border-white scale-110" : "border-zinc-600"}`}
                style={{ backgroundColor: c }}
                onClick={() => { setMobileInkColor(c); hapticTap(); }}
                data-testid={`ink-color-${c.replace("#", "")}`}
              />
            ))}
            <span className="text-[10px] text-zinc-500 ml-auto uppercase tracking-wider">Ink Color</span>
          </div>
          <div
            className="flex-1 relative rounded-xl border border-zinc-700 overflow-hidden"
            style={{ touchAction: "none" }}
          >
            <canvas
              ref={canvasRef}
              width={mobileCanvasSize.w}
              height={mobileCanvasSize.h}
              className="w-full h-full cursor-crosshair"
              style={{ background: "#0a0a0a" }}
              onMouseDown={startDraw}
              onMouseMove={draw}
              onMouseUp={endDraw}
              onMouseLeave={endDraw}
              onTouchStart={startDrawTouch}
              onTouchMove={drawTouch}
              onTouchEnd={endDrawTouch}
              data-testid="canvas-draw-sig"
            />
          </div>
          <p className="text-[11px] text-zinc-500 text-center mt-2 tracking-wide">
            Draw your signature above
          </p>
        </TabsContent>
      </Tabs>

      <div
        className="flex items-center justify-between gap-3 px-4 py-3 border-t border-zinc-800"
        style={{ paddingBottom: "max(12px, env(safe-area-inset-bottom))" }}
      >
        <Button
          variant="ghost"
          onClick={clearCanvas}
          className="text-zinc-400 h-12"
          data-testid="button-clear-canvas"
        >
          <Eraser className="w-4 h-4 mr-1.5" />
          Clear
        </Button>
        <Button
          onClick={applySig}
          className="h-12 px-8 rounded-full text-sm font-semibold"
          style={{
            background: "linear-gradient(135deg, #FFD700, #D4AF37)",
            color: "#0a0a0a",
            boxShadow: "0 4px 12px rgba(255,215,0,0.25), inset 0 2px 4px rgba(255,255,255,0.2)",
          }}
          data-testid="button-apply-sig"
        >
          Seal It
        </Button>
        <Button
          variant="ghost"
          onClick={undoStroke}
          disabled={drawStrokes.length === 0}
          className="text-zinc-400 h-12"
          data-testid="button-undo-stroke"
        >
          <Undo2 className="w-4 h-4 mr-1.5" />
          Undo
        </Button>
      </div>
    </div>
  );

  const desktopSignatureDialog = !isMobile && (
    <Dialog open={sigDialogOpen} onOpenChange={setSigDialogOpen}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle className="text-sm">
            {activeFieldId && myFields.find((f) => f.id === activeFieldId)?.type === "initials"
              ? "Add Initials"
              : "Add Signature"}
          </DialogTitle>
        </DialogHeader>

        <Tabs value={sigMode} onValueChange={(v) => setSigMode(v as "draw" | "type")}>
          <TabsList className="w-full">
            <TabsTrigger value="type" className="flex-1" data-testid="tab-type-sig">
              <Type className="w-3 h-3 mr-1" />
              <span className="text-xs">Type</span>
            </TabsTrigger>
            <TabsTrigger value="draw" className="flex-1" data-testid="tab-draw-sig">
              <PenLine className="w-3 h-3 mr-1" />
              <span className="text-xs">Draw</span>
            </TabsTrigger>
          </TabsList>

          <TabsContent value="type" className="space-y-2.5">
            <Input
              value={typedName}
              onChange={(e) => setTypedName(e.target.value)}
              placeholder="Type your name"
              data-testid="input-typed-name"
            />
            <div className="grid grid-cols-2 gap-2">
              {FONT_STYLES.map((font, i) => (
                <div
                  key={font.name}
                  className={`p-2.5 rounded-md border cursor-pointer text-center transition-colors ${
                    selectedFont === i
                      ? "border-primary bg-primary/5"
                      : "border-border hover-elevate"
                  }`}
                  onClick={() => setSelectedFont(i)}
                  data-testid={`font-option-${i}`}
                >
                  <span
                    className="text-base"
                    style={{
                      fontFamily: font.fontFamily,
                      textShadow: "0.5px 0.5px 0px rgba(0,0,0,0.3), -0.3px 0.3px 0px rgba(0,0,0,0.08), 1px 1px 2px rgba(0,0,0,0.12)",
                      letterSpacing: "-0.02em",
                      WebkitTextStroke: "0.2px rgba(0,0,0,0.15)",
                      paintOrder: "stroke fill",
                    }}
                  >
                    {typedName || "Your Name"}
                  </span>
                  <p className="text-[9px] text-muted-foreground mt-0.5 tracking-wider uppercase">{font.name}</p>
                </div>
              ))}
            </div>
          </TabsContent>

          <TabsContent value="draw" className="space-y-2.5">
            <div className="relative border rounded-md bg-muted/30">
              <canvas
                ref={canvasRef}
                width={400}
                height={160}
                className="w-full cursor-crosshair"
                onMouseDown={startDraw}
                onMouseMove={draw}
                onMouseUp={endDraw}
                onMouseLeave={endDraw}
                data-testid="canvas-draw-sig"
              />
              <Button
                size="icon"
                variant="ghost"
                className="absolute top-1 right-1"
                onClick={clearCanvas}
                data-testid="button-clear-canvas"
              >
                <Eraser className="w-3 h-3" />
              </Button>
            </div>
            <p className="text-[10px] text-muted-foreground text-center tracking-wide">
              Draw your signature above
            </p>
          </TabsContent>
        </Tabs>

        <DialogFooter>
          <Button variant="outline" size="sm" onClick={() => setSigDialogOpen(false)} data-testid="button-cancel-sig">
            Cancel
          </Button>
          <Button size="sm" onClick={applySig} data-testid="button-apply-sig">
            Apply
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );

  const renderPageFields = (pageNum: number) => {
    const pageFields = fieldsOnPage(pageNum);
    const otherPageFields = otherFieldsOnPage(pageNum);

    return (
      <div className="absolute inset-0 pointer-events-none">
        {otherPageFields.map((f) => (
          <div
            key={f.id}
            className="absolute rounded-sm flex items-center justify-center opacity-40"
            style={{
              left: f.x * effectiveZoom,
              top: f.y * effectiveZoom,
              width: f.width * effectiveZoom,
              height: f.height * effectiveZoom,
            }}
          >
            {f.value ? (
              <div className="w-full h-full flex items-center justify-center overflow-hidden p-1">
                {renderFieldValue(f, f.value, effectiveZoom)}
              </div>
            ) : (
              <div className="border border-muted-foreground/30 border-dashed rounded-sm w-full h-full" />
            )}
          </div>
        ))}
        {pageFields.map((f) => {
          const value = fieldValues[f.id];
          const hasValue = !!value;
          const isHighlighted = highlightFieldId === f.id;
          const style = getFieldStyle(f, effectiveZoom, isMobile);
          const isNextField = !hasValue && visibleMyFields.find((mf) => !fieldValues[mf.id])?.id === f.id;

          return (
            <div
              key={f.id}
              className={`absolute rounded-sm flex items-center justify-center cursor-pointer pointer-events-auto transition-all ${
                hasValue
                  ? "border-2 border-emerald-500 bg-emerald-500/5"
                  : isMobile
                    ? `border-2 border-amber-500/60 border-dashed bg-amber-500/10 ${isNextField ? "animate-pulse" : ""}`
                    : "border-2 border-primary/60 border-dashed bg-primary/5 hover-elevate"
              }`}
              style={{
                ...style,
                ...(isMobile && isHighlighted && !hasValue ? { boxShadow: "0 0 16px rgba(255,215,0,0.5)" } : {}),
                ...(isMobile && !hasValue ? { borderWidth: "3px" } : {}),
              }}
              onClick={() => handleFieldClick(f)}
              data-testid={`sign-field-${f.id}`}
            >
              {hasValue ? (
                <div className="w-full h-full flex items-center justify-center overflow-hidden p-1">
                  {renderFieldValue(f, value, effectiveZoom)}
                </div>
              ) : (
                <div className="flex flex-col items-center justify-center gap-0.5 select-none pointer-events-none">
                  {renderFieldPlaceholder(f, isMobile)}
                </div>
              )}
            </div>
          );
        })}
      </div>
    );
  };

  return (
    <div
      className={isMobile ? "min-h-[100dvh] bg-zinc-950" : "min-h-screen bg-background"}
      style={isMobile ? { overscrollBehavior: "none", touchAction: "manipulation" } : undefined}
    >
      {isMobile ? (
        <div
          className="sticky top-0 z-50 border-b border-zinc-800 px-3 py-2.5"
          style={{
            background: "rgba(10,10,10,0.85)",
            backdropFilter: "blur(12px)",
            WebkitBackdropFilter: "blur(12px)",
            paddingTop: "max(10px, env(safe-area-inset-top))",
          }}
        >
          <div className="flex items-center justify-between gap-2">
            <div className="min-w-0 flex-1">
              <h1 className="text-sm font-semibold text-white truncate" data-testid="text-sign-title">
                {envelope?.title}
              </h1>
              <p className="text-[10px] text-zinc-400">
                Signing as <span className="font-medium text-amber-400">{recipient?.name}</span>
              </p>
            </div>
            <div className="flex items-center gap-2 shrink-0">
              <div
                className="relative w-9 h-9 flex items-center justify-center"
                style={{
                  background: `conic-gradient(#FFD700 ${(completedCount / Math.max(totalFields, 1)) * 360}deg, #333 0deg)`,
                  borderRadius: "50%",
                }}
              >
                <div className="w-7 h-7 rounded-full bg-zinc-950 flex items-center justify-center">
                  <span className="text-[9px] text-amber-400 font-bold tabular-nums">
                    {completedCount}/{totalFields}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>
      ) : (
        <div className="border-b sticky top-0 z-50 bg-background">
          <div className="max-w-5xl mx-auto px-4 py-3 flex items-center justify-between gap-4 flex-wrap">
            <div>
              <h1 className="text-sm font-semibold" data-testid="text-sign-title">
                {envelope?.title}
              </h1>
              <p className="text-[10px] text-muted-foreground">
                Signing as <span className="font-medium">{recipient?.name}</span>
              </p>
            </div>
            <div className="flex items-center gap-2">
              {hasPdf && (
                <div className="flex items-center gap-1 mr-2">
                  <Button
                    size="icon"
                    variant="ghost"
                    onClick={() => setPdfZoom((z) => Math.max(0.5, z - 0.1))}
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
                  >
                    <ZoomIn className="w-3.5 h-3.5" />
                  </Button>
                </div>
              )}
              <span className="text-[10px] text-muted-foreground tabular-nums">
                {Object.keys(fieldValues).length}/{myFields.length} completed
              </span>
              <Button
                size="sm"
                onClick={handleSubmit}
                disabled={signMutation.isPending}
                data-testid="button-finish-signing"
              >
                {signMutation.isPending ? "Submitting..." : "Finish Signing"}
              </Button>
            </div>
          </div>
        </div>
      )}

      <div
        ref={containerRef}
        className={isMobile ? "px-0 py-3 flex flex-col items-center gap-4 overflow-x-hidden" : "max-w-5xl mx-auto p-5 flex flex-col items-center gap-6"}
      >
        {pdfUrl ? (
          <Document
            file={pdfUrl}
            onLoadSuccess={(doc) => setNumPages(doc.numPages)}
            loading={
              <div className="flex items-center justify-center py-20">
                <Skeleton className={isMobile ? "h-[400px] w-full mx-4" : "h-[600px] w-[500px]"} />
              </div>
            }
            error={
              <div className="flex flex-col items-center justify-center py-20 text-muted-foreground">
                <FileText className="w-8 h-8 mb-2" />
                <p className="text-xs">Failed to load document</p>
              </div>
            }
          >
            {Array.from({ length: numPages }, (_, i) => {
              const pageNum = i + 1;
              return (
                <div
                  key={pageNum}
                  className={`relative mx-auto ${isMobile ? "shadow-xl" : "shadow-lg"}`}
                  style={{ width: effectiveWidth }}
                  data-testid={`sign-page-${pageNum}`}
                >
                  <Page
                    pageNumber={pageNum}
                    width={effectiveWidth}
                    renderAnnotationLayer={false}
                    renderTextLayer={false}
                  />
                  {renderPageFields(pageNum)}
                  <div className={`absolute bottom-2 right-3 select-none pointer-events-none ${isMobile ? "text-[10px] text-zinc-500" : "text-[9px] text-muted-foreground/50"}`}>
                    Page {pageNum}
                  </div>
                </div>
              );
            })}
          </Document>
        ) : (
          <div
            className={`relative border rounded-md ${isMobile ? "bg-zinc-900 border-zinc-700" : "bg-background"}`}
            style={{ width: isMobile ? mobilePdfWidth : 595, height: isMobile ? mobilePdfWidth * 1.414 : 842 }}
            data-testid="canvas-sign-document"
          >
            <div className="absolute inset-0 flex flex-col items-center justify-center opacity-[0.02] pointer-events-none select-none">
              <div className="text-[72px] font-bold tracking-tighter">SIGN HERE</div>
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
            {myFields.map((f) => {
              const value = fieldValues[f.id];
              const hasValue = !!value;
              const scaleFactor = isMobile ? mobilePdfWidth / 595 : 1;
              const style = {
                left: f.x * scaleFactor,
                top: f.y * scaleFactor,
                width: Math.max(f.width * scaleFactor, isMobile ? 44 : 0),
                height: Math.max(f.height * scaleFactor, isMobile ? 44 : 0),
              };
              return (
                <div
                  key={f.id}
                  className={`absolute border-2 rounded-sm flex items-center justify-center cursor-pointer transition-colors ${
                    hasValue
                      ? "border-emerald-500 bg-emerald-500/5"
                      : isMobile
                        ? "border-amber-500/60 border-dashed bg-amber-500/10"
                        : "border-primary/60 border-dashed bg-primary/5 hover-elevate"
                  }`}
                  style={style}
                  onClick={() => handleFieldClick(f)}
                  data-testid={`sign-field-${f.id}`}
                >
                  {hasValue ? (
                    <div className="w-full h-full flex items-center justify-center overflow-hidden p-1">
                      {renderFieldValue(f, value, scaleFactor)}
                    </div>
                  ) : (
                    <div className="flex flex-col items-center justify-center gap-0.5 select-none pointer-events-none">
                      {renderFieldPlaceholder(f, isMobile)}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>

      {isMobile && !sigDialogOpen && (
        <div
          className="fixed z-40"
          style={{
            bottom: "max(24px, env(safe-area-inset-bottom, 24px))",
            right: "16px",
          }}
        >
          {allFieldsComplete ? (
            <Button
              onClick={handleSubmit}
              disabled={signMutation.isPending}
              className="h-14 px-6 rounded-full text-sm font-semibold shadow-2xl"
              style={{
                background: "linear-gradient(135deg, #FFD700, #D4AF37)",
                color: "#0a0a0a",
                boxShadow: "0 4px 20px rgba(255,215,0,0.4)",
              }}
              data-testid="button-finish-signing"
            >
              {signMutation.isPending ? "Sealing..." : "Finish & Submit"}
            </Button>
          ) : (
            <Button
              onClick={scrollToNextField}
              className="h-12 px-5 rounded-full text-sm font-medium shadow-xl"
              style={{
                background: "linear-gradient(135deg, #FFD700, #D4AF37)",
                color: "#0a0a0a",
                boxShadow: "0 4px 16px rgba(255,215,0,0.3)",
              }}
              data-testid="button-next-field"
            >
              <ChevronDown className="w-4 h-4 mr-1.5" />
              Next Field ({totalFields - completedCount} left)
            </Button>
          )}
        </div>
      )}

      {mobileSignaturePad}
      {desktopSignatureDialog}

      <Dialog open={!!textEditField} onOpenChange={(open) => { if (!open) setTextEditField(null); }}>
        <DialogContent className={isMobile ? "max-w-[calc(100vw-32px)] bg-zinc-900 border-zinc-700" : "max-w-sm"}>
          <DialogHeader>
            <DialogTitle className={`text-sm ${isMobile ? "text-white" : ""}`}>Enter Text</DialogTitle>
          </DialogHeader>
          <Input
            value={textEditValue}
            onChange={(e) => setTextEditValue(e.target.value)}
            placeholder="Type your text..."
            autoFocus
            className={isMobile ? "h-12 text-base bg-zinc-800 border-zinc-600 text-white" : ""}
            style={isMobile ? { fontSize: "16px" } : undefined}
            data-testid="input-text-value"
            onKeyDown={(e) => {
              if (e.key === "Enter") applyTextValue();
            }}
          />
          <DialogFooter>
            <Button variant="outline" size={isMobile ? "default" : "sm"} onClick={() => setTextEditField(null)} className={isMobile ? "h-11 border-zinc-600 text-zinc-300" : ""}>
              Cancel
            </Button>
            <Button
              size={isMobile ? "default" : "sm"}
              onClick={applyTextValue}
              className={isMobile ? "h-11" : ""}
              style={isMobile ? { background: "linear-gradient(135deg, #FFD700, #D4AF37)", color: "#0a0a0a" } : undefined}
              data-testid="button-apply-text"
            >
              Done
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
