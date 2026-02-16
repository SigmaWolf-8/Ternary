import { useState, useRef } from "react";
import { useLocation } from "wouter";
import { useMutation } from "@tanstack/react-query";
import { useForm, useFieldArray } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import {
  ArrowLeft,
  Plus,
  Trash2,
  User,
  Mail,
  Upload,
  FileText,
  X,
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

export default function EnvelopeNew() {
  const [, setLocation] = useLocation();
  const { toast } = useToast();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [pdfFile, setPdfFile] = useState<File | null>(null);
  const [pdfBase64, setPdfBase64] = useState<string | null>(null);
  const [pageCount, setPageCount] = useState(1);

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
    const file = e.target.files?.[0];
    if (!file) return;

    if (file.type !== "application/pdf") {
      toast({ title: "Invalid file", description: "Please select a PDF file", variant: "destructive" });
      return;
    }

    if (file.size > 30 * 1024 * 1024) {
      toast({ title: "File too large", description: "Maximum file size is 30MB", variant: "destructive" });
      return;
    }

    setPdfFile(file);

    const reader = new FileReader();
    reader.onload = async () => {
      const arrayBuffer = reader.result as ArrayBuffer;
      const base64 = btoa(
        new Uint8Array(arrayBuffer).reduce((data, byte) => data + String.fromCharCode(byte), "")
      );
      setPdfBase64(base64);

      try {
        const { pdfjs } = await import("react-pdf");
        pdfjs.GlobalWorkerOptions.workerSrc = `https://unpkg.com/pdfjs-dist@${pdfjs.version}/build/pdf.worker.min.mjs`;
        const loadingTask = pdfjs.getDocument({ data: arrayBuffer.slice(0) });
        const pdf = await loadingTask.promise;
        setPageCount(pdf.numPages);
      } catch {
        setPageCount(1);
      }

      if (!form.getValues("title")) {
        const name = file.name.replace(/\.pdf$/i, "").replace(/[-_]/g, " ");
        form.setValue("title", name);
      }
    };
    reader.readAsArrayBuffer(file);
  };

  const removePdf = () => {
    setPdfFile(null);
    setPdfBase64(null);
    setPageCount(1);
    if (fileInputRef.current) fileInputRef.current.value = "";
  };

  const createMutation = useMutation({
    mutationFn: async (data: FormValues) => {
      const res = await apiRequest("POST", "/api/envelopes", data);
      const envelope = await res.json();

      if (pdfBase64) {
        await apiRequest("POST", `/api/envelopes/${envelope.id}/upload-pdf`, {
          pdfData: pdfBase64,
          pageCount,
        });
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
                  accept=".pdf"
                  className="hidden"
                  onChange={handleFileSelect}
                  data-testid="input-pdf-upload"
                />
                {pdfFile ? (
                  <div className="flex items-center gap-3 p-3 rounded-md bg-muted/50">
                    <div className="w-8 h-8 rounded-md bg-primary/10 flex items-center justify-center shrink-0">
                      <FileText className="w-4 h-4 text-primary" />
                    </div>
                    <div className="flex-1 min-w-0">
                      <p className="text-xs font-medium truncate" data-testid="text-pdf-name">{pdfFile.name}</p>
                      <p className="text-[10px] text-muted-foreground">
                        {(pdfFile.size / 1024).toFixed(0)} KB | {pageCount} page{pageCount !== 1 ? "s" : ""}
                      </p>
                    </div>
                    <Button
                      type="button"
                      size="icon"
                      variant="ghost"
                      onClick={removePdf}
                      data-testid="button-remove-pdf"
                    >
                      <X className="w-3.5 h-3.5" />
                    </Button>
                  </div>
                ) : (
                  <div
                    className="border-2 border-dashed rounded-md p-8 flex flex-col items-center justify-center cursor-pointer hover-elevate"
                    onClick={() => fileInputRef.current?.click()}
                    data-testid="dropzone-pdf"
                  >
                    <Upload className="w-6 h-6 text-muted-foreground mb-2" />
                    <p className="text-xs font-medium">Click to upload PDF</p>
                    <p className="text-[10px] text-muted-foreground mt-1">PDF files up to 30MB</p>
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
                disabled={createMutation.isPending}
                data-testid="button-create"
              >
                {createMutation.isPending ? "Creating..." : "Continue to Editor"}
              </Button>
            </div>
          </form>
        </Form>
      </div>
    </div>
  );
}
