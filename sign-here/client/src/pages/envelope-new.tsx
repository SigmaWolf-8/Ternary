import { useState, useRef } from "react";
import { useLocation } from "wouter";
import { useMutation } from "@tanstack/react-query";
import { useForm, useFieldArray } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { PDFDocument } from "pdf-lib";
import {
  ArrowLeft,
  Plus,
  Trash2,
  User,
  Mail,
  Upload,
  FileText,
  X,
  GripVertical,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import { apiRequest, queryClient } from "@/lib/queryClient";
import { useToast } from "@/hooks/use-toast";
import { Link } from "wouter";

const formSchema = z.object({
  title: z.string().min(1, "Title is required"),
  description: z.string().optional(),
  recipients: z
    .array(
      z.object({
        name: z.string().min(1, "Name is required"),
        email: z.string().email("Valid email required"),
        role: z.string().default("signer"),
      })
    )
    .min(1, "At least one recipient is required"),
});

type FormValues = z.infer<typeof formSchema>;

interface PdfFileEntry {
  file: File;
  pageCount: number;
  arrayBuffer: ArrayBuffer;
}

function isPdfEntry(entry: PdfFileEntry): boolean {
  return entry.file.type === "application/pdf" || entry.file.name.toLowerCase().endsWith(".pdf");
}

async function stitchPdfs(entries: PdfFileEntry[]): Promise<{ base64: string; pageCount: number }> {
  const pdfEntries = entries.filter(isPdfEntry);
  if (pdfEntries.length === 0) {
    const base64 = btoa(
      new Uint8Array(entries[0].arrayBuffer).reduce((d, b) => d + String.fromCharCode(b), "")
    );
    return { base64, pageCount: 1 };
  }
  if (pdfEntries.length === 1) {
    const base64 = btoa(
      new Uint8Array(pdfEntries[0].arrayBuffer).reduce((d, b) => d + String.fromCharCode(b), "")
    );
    return { base64, pageCount: pdfEntries[0].pageCount };
  }

  const merged = await PDFDocument.create();
  for (const entry of pdfEntries) {
    const src = await PDFDocument.load(entry.arrayBuffer);
    const pages = await merged.copyPages(src, src.getPageIndices());
    pages.forEach((p) => merged.addPage(p));
  }
  const mergedBytes = await merged.save();
  const base64 = btoa(
    new Uint8Array(mergedBytes).reduce((d, b) => d + String.fromCharCode(b), "")
  );
  return { base64, pageCount: merged.getPageCount() };
}

export default function EnvelopeNew() {
  const [, setLocation] = useLocation();
  const { toast } = useToast();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [pdfFiles, setPdfFiles] = useState<PdfFileEntry[]>([]);
  const [isStitching, setIsStitching] = useState(false);
  const [dragIdx, setDragIdx] = useState<number | null>(null);
  const [dragOverIdx, setDragOverIdx] = useState<number | null>(null);

  const totalPages = pdfFiles.reduce((sum, f) => sum + f.pageCount, 0);

  const form = useForm<FormValues>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      title: "",
      description: "",
      recipients: [{ name: "", email: "", role: "signer" }],
    },
  });

  const { fields: recipientFields, append, remove } = useFieldArray({
    control: form.control,
    name: "recipients",
  });

  const handleFileSelect = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(e.target.files || []);
    if (!files.length) return;

    const validFiles: PdfFileEntry[] = [];
    for (const file of files) {
      const allowedTypes = [
        "application/pdf",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "text/csv",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
      ];
      const allowedExts = [".pdf", ".xlsx", ".csv", ".docx"];
      const ext = file.name.toLowerCase().slice(file.name.lastIndexOf("."));
      if (!allowedTypes.includes(file.type) && !allowedExts.includes(ext)) {
        toast({ title: "Invalid file", description: `${file.name} — only PDF, XLSX, CSV, and DOCX are supported`, variant: "destructive" });
        continue;
      }
      if (file.size > 30 * 1024 * 1024) {
        toast({ title: "File too large", description: `${file.name} exceeds 30MB`, variant: "destructive" });
        continue;
      }

      const arrayBuffer = await file.arrayBuffer();
      let pages = 1;
      const isPdf = file.type === "application/pdf" || file.name.toLowerCase().endsWith(".pdf");
      if (isPdf) {
        try {
          const { pdfjs } = await import("react-pdf");
          pdfjs.GlobalWorkerOptions.workerSrc = `https://unpkg.com/pdfjs-dist@${pdfjs.version}/build/pdf.worker.min.mjs`;
          const loadingTask = pdfjs.getDocument({ data: arrayBuffer.slice(0) });
          const pdf = await loadingTask.promise;
          pages = pdf.numPages;
        } catch {}
      }

      validFiles.push({ file, pageCount: pages, arrayBuffer });
    }

    if (validFiles.length > 0) {
      setPdfFiles((prev) => [...prev, ...validFiles]);
      if (!form.getValues("title") && validFiles.length === 1 && pdfFiles.length === 0) {
        const name = validFiles[0].file.name.replace(/\.(pdf|xlsx|csv|docx)$/i, "").replace(/[-_]/g, " ");
        form.setValue("title", name);
      }
    }

    if (fileInputRef.current) fileInputRef.current.value = "";
  };

  const removePdfFile = (idx: number) => {
    setPdfFiles((prev) => prev.filter((_, i) => i !== idx));
  };

  const handlePdfReorderDrop = (dropIdx: number) => {
    if (dragIdx === null || dragIdx === dropIdx) {
      setDragIdx(null);
      setDragOverIdx(null);
      return;
    }
    setPdfFiles((prev) => {
      const reordered = [...prev];
      const [moved] = reordered.splice(dragIdx, 1);
      reordered.splice(dropIdx, 0, moved);
      return reordered;
    });
    setDragIdx(null);
    setDragOverIdx(null);
  };

  const createMutation = useMutation({
    mutationFn: async (data: FormValues) => {
      const res = await apiRequest("POST", "/api/envelopes", data);
      const envelope = await res.json();

      if (pdfFiles.length > 0) {
        setIsStitching(true);
        try {
          const nonPdfEntries = pdfFiles.filter((e) => !isPdfEntry(e));
          for (const entry of nonPdfEntries) {
            const b64 = btoa(new Uint8Array(entry.arrayBuffer).reduce((d, b) => d + String.fromCharCode(b), ""));
            await apiRequest("POST", `/api/envelopes/${envelope.id}/upload-pdf`, {
              pdfData: b64,
              pageCount: 1,
              fileName: entry.file.name,
              fileType: entry.file.type || entry.file.name.split(".").pop(),
            });
          }
          const pdfEntries = pdfFiles.filter(isPdfEntry);
          if (pdfEntries.length > 0) {
            const { base64, pageCount } = await stitchPdfs(pdfEntries);
            await apiRequest("POST", `/api/envelopes/${envelope.id}/upload-pdf`, {
              pdfData: base64,
              pageCount,
            });
          }
        } finally {
          setIsStitching(false);
        }
      }

      return envelope;
    },
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ["/api/envelopes"] });
      toast({ title: "Envelope created" });
      setLocation(`/envelope/${data.id}/edit`);
    },
    onError: (error: Error) => {
      toast({ title: "Error", description: error.message, variant: "destructive" });
    },
  });

  const onSubmit = (data: FormValues) => {
    createMutation.mutate(data);
  };

  return (
    <div className="flex-1 overflow-auto">
      <div className="max-w-2xl mx-auto p-5 space-y-5">
        <div className="flex items-center gap-2.5">
          <Link href="/">
            <Button size="icon" variant="ghost" data-testid="button-back">
              <ArrowLeft className="w-3.5 h-3.5" />
            </Button>
          </Link>
          <div>
            <h1 className="text-sm font-semibold tracking-tight" data-testid="text-new-title">
              New Envelope
            </h1>
            <p className="text-[10px] text-muted-foreground mt-0.5 tracking-wide">
              Set up your document for signing
            </p>
          </div>
        </div>

        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <Card>
              <CardContent className="p-4 space-y-3.5">
                <h2 className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground">
                  Upload Document
                </h2>
                <input
                  ref={fileInputRef}
                  type="file"
                  accept=".pdf,.xlsx,.csv,.docx"
                  multiple
                  className="hidden"
                  onChange={handleFileSelect}
                  data-testid="input-pdf-upload"
                />
                {pdfFiles.length > 0 ? (
                  <div className="space-y-1.5">
                    {pdfFiles.map((entry, idx) => (
                      <div
                        key={`${entry.file.name}-${idx}`}
                        className={`flex items-center gap-2.5 p-2.5 rounded-md bg-muted/50 ${dragOverIdx === idx && dragIdx !== idx ? "ring-1 ring-primary" : ""}`}
                        draggable
                        onDragStart={() => setDragIdx(idx)}
                        onDragOver={(e) => { e.preventDefault(); setDragOverIdx(idx); }}
                        onDragEnd={() => { setDragIdx(null); setDragOverIdx(null); }}
                        onDrop={() => handlePdfReorderDrop(idx)}
                        data-testid={`pdf-file-${idx}`}
                      >
                        <GripVertical className="w-3 h-3 text-muted-foreground cursor-grab shrink-0" />
                        <div className="w-7 h-7 rounded-md bg-primary/10 flex items-center justify-center shrink-0">
                          <FileText className="w-3.5 h-3.5 text-primary" />
                        </div>
                        <div className="flex-1 min-w-0">
                          <p className="text-[11px] font-medium truncate" data-testid={`text-pdf-name-${idx}`}>{entry.file.name}</p>
                          <p className="text-[10px] text-muted-foreground">
                            {(entry.file.size / 1024).toFixed(0)} KB{isPdfEntry(entry) ? ` | ${entry.pageCount} page${entry.pageCount !== 1 ? "s" : ""}` : ` | ${entry.file.name.split(".").pop()?.toUpperCase()}`}
                          </p>
                        </div>
                        <Button
                          type="button"
                          size="icon"
                          variant="ghost"
                          onClick={() => removePdfFile(idx)}
                          data-testid={`button-remove-pdf-${idx}`}
                        >
                          <X className="w-3.5 h-3.5" />
                        </Button>
                      </div>
                    ))}
                    <div className="flex items-center justify-between gap-2 pt-1">
                      <p className="text-[10px] text-muted-foreground">
                        {pdfFiles.length} file{pdfFiles.length !== 1 ? "s" : ""} | {totalPages} total page{totalPages !== 1 ? "s" : ""}
                        {pdfFiles.length > 1 && " (will be stitched)"}
                      </p>
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={() => fileInputRef.current?.click()}
                        data-testid="button-add-more-pdfs"
                      >
                        <Plus className="w-3 h-3" />
                        Add More
                      </Button>
                    </div>
                  </div>
                ) : (
                  <div
                    className="border-2 border-dashed rounded-md p-8 flex flex-col items-center justify-center cursor-pointer hover-elevate"
                    onClick={() => fileInputRef.current?.click()}
                    data-testid="dropzone-pdf"
                  >
                    <Upload className="w-6 h-6 text-muted-foreground mb-2" />
                    <p className="text-xs font-medium">Click to upload PDFs</p>
                    <p className="text-[10px] text-muted-foreground mt-1">Select one or more PDF files (up to 30MB each)</p>
                  </div>
                )}
              </CardContent>
            </Card>

            <Card>
              <CardContent className="p-4 space-y-3.5">
                <h2 className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground">Document Details</h2>
                <FormField
                  control={form.control}
                  name="title"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel className="text-[11px]">Title</FormLabel>
                      <FormControl>
                        <Input
                          placeholder="e.g., Non-Disclosure Agreement"
                          {...field}
                          data-testid="input-title"
                        />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />
                <FormField
                  control={form.control}
                  name="description"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel className="text-[11px]">Description (optional)</FormLabel>
                      <FormControl>
                        <Textarea
                          placeholder="Brief description of this document..."
                          className="resize-none"
                          {...field}
                          data-testid="input-description"
                        />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />
              </CardContent>
            </Card>

            <Card>
              <CardContent className="p-4 space-y-3.5">
                <div className="flex items-center justify-between gap-2">
                  <h2 className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground">Recipients</h2>
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    onClick={() =>
                      append({ name: "", email: "", role: "signer" })
                    }
                    data-testid="button-add-recipient"
                  >
                    <Plus className="w-3 h-3" />
                    Add
                  </Button>
                </div>

                <div className="space-y-2.5">
                  {recipientFields.map((field, index) => (
                    <div
                      key={field.id}
                      className="flex items-start gap-2 p-2.5 rounded-md bg-muted/50"
                      data-testid={`recipient-row-${index}`}
                    >
                      <div className="flex-1 grid grid-cols-1 sm:grid-cols-3 gap-2">
                        <FormField
                          control={form.control}
                          name={`recipients.${index}.name`}
                          render={({ field }) => (
                            <FormItem>
                              <FormControl>
                                <div className="relative">
                                  <User className="absolute left-2.5 top-2.5 w-3 h-3 text-muted-foreground" />
                                  <Input
                                    placeholder="Name"
                                    className="pl-7"
                                    {...field}
                                    data-testid={`input-recipient-name-${index}`}
                                  />
                                </div>
                              </FormControl>
                              <FormMessage />
                            </FormItem>
                          )}
                        />
                        <FormField
                          control={form.control}
                          name={`recipients.${index}.email`}
                          render={({ field }) => (
                            <FormItem>
                              <FormControl>
                                <div className="relative">
                                  <Mail className="absolute left-2.5 top-2.5 w-3 h-3 text-muted-foreground" />
                                  <Input
                                    placeholder="Email"
                                    className="pl-7"
                                    {...field}
                                    data-testid={`input-recipient-email-${index}`}
                                  />
                                </div>
                              </FormControl>
                              <FormMessage />
                            </FormItem>
                          )}
                        />
                        <FormField
                          control={form.control}
                          name={`recipients.${index}.role`}
                          render={({ field }) => (
                            <FormItem>
                              <Select
                                onValueChange={field.onChange}
                                defaultValue={field.value}
                              >
                                <FormControl>
                                  <SelectTrigger data-testid={`select-role-${index}`}>
                                    <SelectValue placeholder="Role" />
                                  </SelectTrigger>
                                </FormControl>
                                <SelectContent>
                                  <SelectItem value="signer">Signer</SelectItem>
                                  <SelectItem value="viewer">Viewer</SelectItem>
                                  <SelectItem value="witness">Witness</SelectItem>
                                </SelectContent>
                              </Select>
                              <FormMessage />
                            </FormItem>
                          )}
                        />
                      </div>
                      {recipientFields.length > 1 && (
                        <Button
                          type="button"
                          size="icon"
                          variant="ghost"
                          onClick={() => remove(index)}
                          data-testid={`button-remove-recipient-${index}`}
                        >
                          <Trash2 className="w-3 h-3" />
                        </Button>
                      )}
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>

            <div className="flex items-center justify-end gap-1.5">
              <Link href="/">
                <Button type="button" variant="outline" size="sm" data-testid="button-cancel">
                  Cancel
                </Button>
              </Link>
              <Button
                type="submit"
                size="sm"
                disabled={createMutation.isPending || isStitching}
                data-testid="button-create"
              >
                {isStitching ? "Stitching PDFs..." : createMutation.isPending ? "Creating..." : "Continue to Editor"}
              </Button>
            </div>
          </form>
        </Form>
      </div>
    </div>
  );
}
