/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
 * Patent(s) Pending.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * Unauthorized copying, modification, distribution, or use of this file,
 * via any medium, is strictly prohibited without the prior written
 * permission of Capomastro Holdings Ltd.
 *
 * See LICENSE in the repository root for full terms.
 */
import { useState } from "react";
import { useQuery, useMutation } from "@tanstack/react-query";
import { useLocation } from "wouter";
import { format } from "date-fns";
import {
  Search,
  FileText,
  Copy,
  Trash2,
  Tag,
  Grid3X3,
  Signature,
  Calendar,
  CheckSquare,
  Type,
  PenLine,
  Pencil,
  Plus,
  X,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { ScrollArea } from "@/components/ui/scroll-area";
import { apiRequest, queryClient } from "@/lib/queryClient";
import { useToast } from "@/hooks/use-toast";
import type { Template } from "@shared/schema";

const FIELD_ICONS: Record<string, typeof Signature> = {
  signature: Signature,
  date: Calendar,
  text: Type,
  checkbox: CheckSquare,
  initials: PenLine,
};

const CATEGORIES = ["Legal", "HR", "Finance", "Real Estate", "Healthcare", "General"];

export default function TemplatesPage() {
  const { toast } = useToast();
  const [, setLocation] = useLocation();
  const [search, setSearch] = useState("");
  const [categoryFilter, setCategoryFilter] = useState("all");
  const [previewTemplate, setPreviewTemplate] = useState<Template | null>(null);

  const { data: templates, isLoading } = useQuery<Template[]>({
    queryKey: ["/api/templates"],
  });

  const duplicateMutation = useMutation({
    mutationFn: async (templateId: string) => {
      const res = await apiRequest("POST", `/api/templates/${templateId}/fork`);
      return res.json();
    },
    onSuccess: (forked: Template) => {
      queryClient.invalidateQueries({ queryKey: ["/api/templates"] });
      toast({ title: `Duplicated as "${forked.name}"` });
    },
    onError: () => {
      toast({ title: "Failed to duplicate template", variant: "destructive" });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: async (templateId: string) => {
      await apiRequest("DELETE", `/api/templates/${templateId}`);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["/api/templates"] });
      toast({ title: "Template deleted" });
    },
    onError: () => {
      toast({ title: "Failed to delete template", variant: "destructive" });
    },
  });

  const filtered = (templates || []).filter((t) => {
    const matchesSearch =
      !search ||
      t.name.toLowerCase().includes(search.toLowerCase()) ||
      (t.description || "").toLowerCase().includes(search.toLowerCase()) ||
      (t.tags as string[] || []).some((tag: string) => tag.toLowerCase().includes(search.toLowerCase()));
    const matchesCategory = categoryFilter === "all" || t.category === categoryFilter;
    return matchesSearch && matchesCategory;
  });

  const categories = Array.from(new Set((templates || []).map((t) => t.category).filter(Boolean)));

  if (isLoading) {
    return (
      <div className="flex-1 overflow-auto p-5">
        <div className="max-w-5xl mx-auto space-y-4">
          <Skeleton className="h-8 w-64" />
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
            {[1, 2, 3, 4, 5, 6].map((i) => (
              <Skeleton key={i} className="h-48" />
            ))}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-auto">
      <div className="max-w-5xl mx-auto p-5 space-y-5">
        <div className="flex items-center justify-between gap-4 flex-wrap">
          <div>
            <h1 className="text-sm font-semibold" data-testid="text-templates-title">Template Gallery</h1>
            <p className="text-[10px] text-muted-foreground">
              {templates?.length || 0} template{(templates?.length || 0) !== 1 ? "s" : ""} available
              <span className="ml-1.5 text-muted-foreground/60">
                — Apply templates to envelopes from the document editor
              </span>
            </p>
          </div>
          <Button
            size="sm"
            onClick={() => setLocation("/templates/new/edit")}
            data-testid="button-new-template"
          >
            <Plus className="w-3 h-3" />
            New Template
          </Button>
        </div>

        <div className="flex items-center gap-2 flex-wrap">
          <div className="relative flex-1 min-w-[200px]">
            <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground" />
            <Input
              placeholder="Search templates..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="pl-8 h-9 text-xs"
              data-testid="input-template-search"
            />
          </div>
          <Select value={categoryFilter} onValueChange={setCategoryFilter}>
            <SelectTrigger className="w-40 h-9 text-xs" data-testid="select-template-category">
              <SelectValue placeholder="All Categories" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Categories</SelectItem>
              {CATEGORIES.map((cat) => (
                <SelectItem key={cat} value={cat}>
                  {cat}
                </SelectItem>
              ))}
              {categories
                .filter((c) => !CATEGORIES.includes(c!))
                .map((cat) => (
                  <SelectItem key={cat} value={cat!}>
                    {cat}
                  </SelectItem>
                ))}
            </SelectContent>
          </Select>
        </div>

        {filtered.length === 0 ? (
          <Card>
            <CardContent className="p-10 text-center">
              <Grid3X3 className="w-8 h-8 text-muted-foreground mx-auto mb-3" />
              <p className="text-xs font-medium mb-1">No templates found</p>
              <p className="text-[10px] text-muted-foreground">
                {search || categoryFilter !== "all"
                  ? "Try adjusting your search or category filter"
                  : "Templates will appear here once created"}
              </p>
            </CardContent>
          </Card>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
            {filtered.map((template) => {
              const fieldDefs = (template.fieldDefs as any[] || []);
              const fieldCounts: Record<string, number> = {};
              fieldDefs.forEach((f: any) => {
                fieldCounts[f.type] = (fieldCounts[f.type] || 0) + 1;
              });

              return (
                <Card
                  key={template.id}
                  className="hover-elevate cursor-pointer group"
                  onClick={() => setPreviewTemplate(template)}
                  data-testid={`card-template-${template.id}`}
                >
                  <CardContent className="p-4 flex flex-col h-full">
                    <div className="flex items-start justify-between gap-2 mb-2">
                      <div className="min-w-0 flex-1">
                        <div className="flex items-center gap-1.5 mb-1">
                          <FileText className="w-3.5 h-3.5 text-primary shrink-0" />
                          <h3
                            className="text-xs font-semibold truncate"
                            data-testid={`text-template-name-${template.id}`}
                          >
                            {template.name}
                          </h3>
                        </div>
                        {template.description && (
                          <p className="text-[10px] text-muted-foreground line-clamp-2">
                            {template.description}
                          </p>
                        )}
                      </div>
                      {template.isPublic && (
                        <Badge variant="outline" className="text-[8px] shrink-0">
                          Built-in
                        </Badge>
                      )}
                    </div>

                    <div className="flex flex-wrap gap-1 mb-3">
                      {template.category && (
                        <Badge variant="secondary" className="text-[8px]" data-testid={`badge-category-${template.id}`}>
                          {template.category}
                        </Badge>
                      )}
                      {(template.tags as string[] || []).slice(0, 3).map((tag: string) => (
                        <Badge key={tag} variant="outline" className="text-[8px]">
                          <Tag className="w-2 h-2 mr-0.5" />
                          {tag}
                        </Badge>
                      ))}
                    </div>

                    <div className="flex items-center gap-1.5 text-[9px] text-muted-foreground mb-3">
                      {Object.entries(fieldCounts).map(([type, count]) => {
                        const Icon = FIELD_ICONS[type] || FileText;
                        return (
                          <span key={type} className="flex items-center gap-0.5">
                            <Icon className="w-2.5 h-2.5" />
                            {count}
                          </span>
                        );
                      })}
                      {fieldDefs.length === 0 && <span>No fields</span>}
                      <span className="text-muted-foreground/40">|</span>
                      <span>{format(new Date(template.createdAt), "MMM d, yyyy")}</span>
                    </div>

                    <div className="flex items-center gap-1.5 mt-auto">
                      <Button
                        size="sm"
                        variant="outline"
                        className="flex-1"
                        onClick={(e) => {
                          e.stopPropagation();
                          setLocation(`/templates/${template.id}/edit`);
                        }}
                        data-testid={`button-edit-${template.id}`}
                      >
                        <Pencil className="w-3 h-3" />
                        Edit
                      </Button>
                      <Button
                        size="icon"
                        variant="ghost"
                        onClick={(e) => {
                          e.stopPropagation();
                          duplicateMutation.mutate(template.id);
                        }}
                        disabled={duplicateMutation.isPending}
                        data-testid={`button-duplicate-${template.id}`}
                      >
                        <Copy className="w-3 h-3" />
                      </Button>
                      {!template.isPublic && (
                        <Button
                          size="icon"
                          variant="ghost"
                          onClick={(e) => {
                            e.stopPropagation();
                            deleteMutation.mutate(template.id);
                          }}
                          data-testid={`button-delete-${template.id}`}
                        >
                          <Trash2 className="w-3 h-3" />
                        </Button>
                      )}
                    </div>
                  </CardContent>
                </Card>
              );
            })}
          </div>
        )}
      </div>

      <Dialog open={!!previewTemplate} onOpenChange={(open) => !open && setPreviewTemplate(null)}>
        {previewTemplate && (
          <DialogContent className="max-w-lg">
            <DialogHeader>
              <DialogTitle className="text-sm" data-testid="text-preview-title">
                {previewTemplate.name}
              </DialogTitle>
              <DialogDescription className="text-[10px]">
                {previewTemplate.description || "No description provided"}
              </DialogDescription>
            </DialogHeader>

            <div className="space-y-4">
              <div>
                <p className="text-[9px] font-medium uppercase tracking-wider text-muted-foreground mb-2">
                  Category & Tags
                </p>
                <div className="flex flex-wrap gap-1.5">
                  {previewTemplate.category && (
                    <Badge variant="secondary" className="text-[9px]">{previewTemplate.category}</Badge>
                  )}
                  {(previewTemplate.tags as string[] || []).map((tag: string) => (
                    <Badge key={tag} variant="outline" className="text-[9px]">
                      <Tag className="w-2 h-2 mr-0.5" />
                      {tag}
                    </Badge>
                  ))}
                  {!previewTemplate.category && !(previewTemplate.tags as string[] || []).length && (
                    <span className="text-[10px] text-muted-foreground">No category or tags</span>
                  )}
                </div>
              </div>

              <div>
                <p className="text-[9px] font-medium uppercase tracking-wider text-muted-foreground mb-2">
                  Fields ({(previewTemplate.fieldDefs as any[] || []).length})
                </p>
                <ScrollArea className="max-h-52">
                  {(previewTemplate.fieldDefs as any[] || []).length === 0 ? (
                    <p className="text-[10px] text-muted-foreground">No fields defined</p>
                  ) : (
                    <div className="space-y-1">
                      {(previewTemplate.fieldDefs as any[]).map((f: any, i: number) => {
                        const Icon = FIELD_ICONS[f.type] || FileText;
                        return (
                          <div key={i} className="flex items-center gap-2 p-1.5 rounded-md bg-muted/50" data-testid={`preview-field-${i}`}>
                            <Icon className="w-3 h-3 text-primary shrink-0" />
                            <span className="text-[11px] capitalize">{f.label || f.type}</span>
                            <span className="text-[9px] text-muted-foreground ml-auto">
                              Page {f.page}
                            </span>
                            {f.required && (
                              <Badge variant="outline" className="text-[7px] px-1 py-0">
                                Required
                              </Badge>
                            )}
                          </div>
                        );
                      })}
                    </div>
                  )}
                </ScrollArea>
              </div>

              <div className="flex items-center gap-2 text-[9px] text-muted-foreground">
                <span>Created {format(new Date(previewTemplate.createdAt), "MMM d, yyyy")}</span>
                {previewTemplate.forkedFromId && (
                  <>
                    <span className="text-muted-foreground/40">|</span>
                    <span className="flex items-center gap-0.5">
                      <Copy className="w-2 h-2" /> Duplicated
                    </span>
                  </>
                )}
              </div>

              <div className="flex items-center gap-2 pt-2">
                <Button
                  size="sm"
                  className="flex-1"
                  onClick={() => {
                    setLocation(`/templates/${previewTemplate.id}/edit`);
                    setPreviewTemplate(null);
                  }}
                  data-testid="button-preview-edit"
                >
                  <Pencil className="w-3 h-3" />
                  Edit Template
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  className="flex-1"
                  onClick={() => {
                    duplicateMutation.mutate(previewTemplate.id);
                    setPreviewTemplate(null);
                  }}
                  disabled={duplicateMutation.isPending}
                  data-testid="button-preview-duplicate"
                >
                  <Copy className="w-3 h-3" />
                  Duplicate
                </Button>
              </div>

              {!previewTemplate.isPublic && (
                <Button
                  variant="ghost"
                  size="sm"
                  className="w-full text-destructive"
                  onClick={() => {
                    deleteMutation.mutate(previewTemplate.id);
                    setPreviewTemplate(null);
                  }}
                  data-testid="button-preview-delete"
                >
                  <Trash2 className="w-3 h-3" />
                  Delete Template
                </Button>
              )}

              <p className="text-[10px] text-muted-foreground text-center border-t pt-3">
                Click Edit to set up fields with a PDF viewer, or apply this template from the envelope editor sidebar.
              </p>
            </div>
          </DialogContent>
        )}
      </Dialog>
    </div>
  );
}
