import { useState, useRef, useCallback, useEffect } from "react";
import { useRoute, useLocation } from "wouter";
import { useQuery, useMutation } from "@tanstack/react-query";
import { Document, Page, pdfjs } from "react-pdf";
import "react-pdf/dist/Page/AnnotationLayer.css";
import "react-pdf/dist/Page/TextLayer.css";
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

export default function Sign() {
  const [, params] = useRoute("/sign/:envelopeId/:recipientId");
  const [, setLocation] = useLocation();
  const { toast } = useToast();
  const canvasRef = useRef<HTMLCanvasElement>(null);

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
  }, []);

  const startDraw = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
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

  const applySig = () => {
    if (!activeFieldId) return;

    if (sigMode === "type") {
      if (!typedName.trim()) return;
      setFieldValues((prev) => ({
        ...prev,
        [activeFieldId]: `typed:${selectedFont}:${typedName}`,
      }));
    } else {
      const canvas = canvasRef.current;
      if (!canvas) return;
      const dataUrl = canvas.toDataURL();
      setFieldValues((prev) => ({ ...prev, [activeFieldId]: `drawn:${dataUrl}` }));
    }

    setSigDialogOpen(false);
    setActiveFieldId(null);
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

  const handleFieldClick = (field: FieldType) => {
    if (field.type === "signature" || field.type === "initials") {
      setActiveFieldId(field.id);
      setSigDialogOpen(true);
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

  const fieldsOnPage = (page: number) => visibleMyFields.filter((f) => f.page === page);
  const otherFieldsOnPage = (page: number) => visibleOtherFields.filter((f) => f.page === page);

  if (envLoading) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center">
        <div className="space-y-3 w-full max-w-lg p-5">
          <Skeleton className="h-6 w-44" />
          <Skeleton className="h-[500px] w-full" />
        </div>
      </div>
    );
  }

  if (completed) {
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
      <div className="min-h-screen bg-background flex items-center justify-center">
        <Card className="max-w-sm w-full mx-4">
          <CardContent className="p-8 flex flex-col items-center text-center">
            <div className="w-12 h-12 rounded-full bg-emerald-500/10 flex items-center justify-center mb-3">
              <CheckCircle2 className="w-5 h-5 text-emerald-500" />
            </div>
            <h1 className="text-sm font-semibold mb-1.5">Already Signed</h1>
            <p className="text-[11px] text-muted-foreground">
              This document has already been signed.
            </p>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-background">
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

      <div className="max-w-5xl mx-auto p-5 flex flex-col items-center gap-6">
        {pdfUrl ? (
          <Document
            file={pdfUrl}
            onLoadSuccess={(doc) => setNumPages(doc.numPages)}
            loading={
              <div className="flex items-center justify-center py-20">
                <Skeleton className="h-[600px] w-[500px]" />
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
              const pageFields = fieldsOnPage(pageNum);
              const otherPageFields = otherFieldsOnPage(pageNum);
              return (
                <div
                  key={pageNum}
                  className="relative shadow-lg mx-auto"
                  style={{ width: PDF_BASE_WIDTH * pdfZoom }}
                  data-testid={`sign-page-${pageNum}`}
                >
                  <Page
                    pageNumber={pageNum}
                    width={PDF_BASE_WIDTH * pdfZoom}
                    renderAnnotationLayer={false}
                    renderTextLayer={false}
                  />
                  <div className="absolute inset-0 pointer-events-none">
                    {otherPageFields.map((f) => (
                      <div
                        key={f.id}
                        className="absolute rounded-sm flex items-center justify-center opacity-40"
                        style={{
                          left: f.x * pdfZoom,
                          top: f.y * pdfZoom,
                          width: f.width * pdfZoom,
                          height: f.height * pdfZoom,
                        }}
                      >
                        {f.value ? (
                          <div className="w-full h-full flex items-center justify-center overflow-hidden p-1">
                            {f.value.startsWith("typed:") ? (
                              <span
                                className="truncate"
                                style={{
                                  color: "#1a1a1a",
                                  fontFamily: FONT_STYLES[parseInt(f.value.split(":")[1]) || 0]?.fontFamily,
                                  fontSize: `${Math.max(10, 14 * pdfZoom)}px`,
                                  textShadow: "0.5px 0.5px 0px rgba(0,0,0,0.3), -0.3px 0.3px 0px rgba(0,0,0,0.08), 1px 1px 2px rgba(0,0,0,0.12)",
                                  letterSpacing: "-0.02em",
                                  WebkitTextStroke: "0.2px rgba(0,0,0,0.15)",
                                  paintOrder: "stroke fill",
                                }}
                              >
                                {f.value.split(":").slice(2).join(":")}
                              </span>
                            ) : (
                              <span className="text-[10px] text-muted-foreground">{f.value}</span>
                            )}
                          </div>
                        ) : (
                          <div className="border border-muted-foreground/30 border-dashed rounded-sm w-full h-full" />
                        )}
                      </div>
                    ))}
                    {pageFields.map((f) => {
                      const value = fieldValues[f.id];
                      const hasValue = !!value;

                      return (
                        <div
                          key={f.id}
                          className={`absolute border-2 rounded-sm flex items-center justify-center cursor-pointer pointer-events-auto transition-colors ${
                            hasValue
                              ? "border-emerald-500 bg-emerald-500/5"
                              : "border-primary/60 border-dashed bg-primary/5 hover-elevate"
                          }`}
                          style={{
                            left: f.x * pdfZoom,
                            top: f.y * pdfZoom,
                            width: f.width * pdfZoom,
                            height: f.height * pdfZoom,
                          }}
                          onClick={() => handleFieldClick(f)}
                          data-testid={`sign-field-${f.id}`}
                        >
                          {hasValue ? (
                            <div className="w-full h-full flex items-center justify-center overflow-hidden p-1">
                              {value.startsWith("typed:") ? (
                                <span
                                  className="truncate"
                                  style={{
                                    color: "#1a1a1a",
                                    fontFamily: FONT_STYLES[parseInt(value.split(":")[1])].fontFamily,
                                    fontSize: `${Math.max(10, 14 * pdfZoom)}px`,
                                    textShadow: "0.5px 0.5px 0px rgba(0,0,0,0.3), -0.3px 0.3px 0px rgba(0,0,0,0.08), 1px 1px 2px rgba(0,0,0,0.12)",
                                    letterSpacing: "-0.02em",
                                    WebkitTextStroke: "0.2px rgba(0,0,0,0.15)",
                                    paintOrder: "stroke fill",
                                  }}
                                >
                                  {value.split(":").slice(2).join(":")}
                                </span>
                              ) : value.startsWith("drawn:") ? (
                                <img
                                  src={value.replace("drawn:", "")}
                                  alt="Signature"
                                  className="max-w-full max-h-full object-contain"
                                />
                              ) : f.type === "checkbox" ? (
                                <CheckCircle2 className="w-3.5 h-3.5 text-emerald-500" />
                              ) : (
                                <span className="text-[10px]">{value}</span>
                              )}
                            </div>
                          ) : (
                            <div className="flex flex-col items-center justify-center gap-0.5 select-none pointer-events-none">
                              {f.type === "signature" ? (
                                <>
                                  <PenLine className="w-4 h-4 text-primary/50" />
                                  <span className="text-[8px] font-medium text-primary/60 uppercase tracking-widest">
                                    Click to Sign
                                  </span>
                                </>
                              ) : f.type === "initials" ? (
                                <>
                                  <Hash className="w-3 h-3 text-primary/50" />
                                  <span className="text-[8px] font-medium text-primary/60 uppercase tracking-widest">
                                    Initials
                                  </span>
                                </>
                              ) : f.type === "date" ? (
                                <>
                                  <CalendarDays className="w-3 h-3 text-primary/50" />
                                  <span className="text-[7px] font-medium text-primary/60 uppercase tracking-widest">
                                    Date (HPTP)
                                  </span>
                                </>
                              ) : f.type === "checkbox" ? (
                                <CheckSquare className="w-3.5 h-3.5 text-primary/40" />
                              ) : (
                                <>
                                  <Type className="w-3 h-3 text-primary/50" />
                                  <span className="text-[7px] font-medium text-primary/60 uppercase tracking-widest">
                                    Enter Text
                                  </span>
                                </>
                              )}
                            </div>
                          )}
                        </div>
                      );
                    })}
                  </div>
                  <div className="absolute bottom-2 right-3 text-[9px] text-muted-foreground/50 select-none pointer-events-none">
                    Page {pageNum}
                  </div>
                </div>
              );
            })}
          </Document>
        ) : (
          <div
            className="relative bg-background border rounded-md"
            style={{ width: 595, height: 842 }}
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
              return (
                <div
                  key={f.id}
                  className={`absolute border-2 rounded-sm flex items-center justify-center cursor-pointer transition-colors ${
                    hasValue
                      ? "border-emerald-500 bg-emerald-500/5"
                      : "border-primary/60 border-dashed bg-primary/5 hover-elevate"
                  }`}
                  style={{
                    left: f.x,
                    top: f.y,
                    width: f.width,
                    height: f.height,
                  }}
                  onClick={() => handleFieldClick(f)}
                  data-testid={`sign-field-${f.id}`}
                >
                  {hasValue ? (
                    <div className="w-full h-full flex items-center justify-center overflow-hidden p-1">
                      {value.startsWith("typed:") ? (
                        <span
                          className="truncate"
                          style={{
                            color: "#1a1a1a",
                            fontFamily: FONT_STYLES[parseInt(value.split(":")[1])].fontFamily,
                            fontSize: f.type === "initials" ? "12px" : "14px",
                            textShadow: "0.5px 0.5px 0px rgba(0,0,0,0.3), -0.3px 0.3px 0px rgba(0,0,0,0.08), 1px 1px 2px rgba(0,0,0,0.12)",
                            letterSpacing: "-0.02em",
                            WebkitTextStroke: "0.2px rgba(0,0,0,0.15)",
                            paintOrder: "stroke fill",
                          }}
                        >
                          {value.split(":").slice(2).join(":")}
                        </span>
                      ) : value.startsWith("drawn:") ? (
                        <img
                          src={value.replace("drawn:", "")}
                          alt="Signature"
                          className="max-w-full max-h-full object-contain"
                        />
                      ) : f.type === "checkbox" ? (
                        <CheckCircle2 className="w-3.5 h-3.5 text-emerald-500" />
                      ) : (
                        <span className="text-[10px]">{value}</span>
                      )}
                    </div>
                  ) : (
                    <div className="flex flex-col items-center justify-center gap-0.5 select-none pointer-events-none">
                      {f.type === "signature" ? (
                        <>
                          <PenLine className="w-4 h-4 text-primary/50" />
                          <span className="text-[8px] font-medium text-primary/60 uppercase tracking-widest">
                            Click to Sign
                          </span>
                        </>
                      ) : f.type === "initials" ? (
                        <>
                          <Hash className="w-3 h-3 text-primary/50" />
                          <span className="text-[8px] font-medium text-primary/60 uppercase tracking-widest">
                            Initials
                          </span>
                        </>
                      ) : f.type === "date" ? (
                        <>
                          <CalendarDays className="w-3 h-3 text-primary/50" />
                          <span className="text-[7px] font-medium text-primary/60 uppercase tracking-widest">
                            Date (HPTP)
                          </span>
                        </>
                      ) : f.type === "checkbox" ? (
                        <CheckSquare className="w-3.5 h-3.5 text-primary/40" />
                      ) : (
                        <>
                          <Type className="w-3 h-3 text-primary/50" />
                          <span className="text-[7px] font-medium text-primary/60 uppercase tracking-widest">
                            Enter Text
                          </span>
                        </>
                      )}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>

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

      <Dialog open={!!textEditField} onOpenChange={(open) => { if (!open) setTextEditField(null); }}>
        <DialogContent className="max-w-sm">
          <DialogHeader>
            <DialogTitle className="text-sm">Enter Text</DialogTitle>
          </DialogHeader>
          <Input
            value={textEditValue}
            onChange={(e) => setTextEditValue(e.target.value)}
            placeholder="Type your text..."
            autoFocus
            data-testid="input-text-value"
            onKeyDown={(e) => {
              if (e.key === "Enter") applyTextValue();
            }}
          />
          <DialogFooter>
            <Button variant="outline" size="sm" onClick={() => setTextEditField(null)}>
              Cancel
            </Button>
            <Button size="sm" onClick={applyTextValue} data-testid="button-apply-text">
              Apply
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
