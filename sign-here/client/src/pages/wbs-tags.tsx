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
import {
  Plus, Pencil, Trash2, Tag, GripVertical, Sparkles, Check, Building2, Loader2,
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
  DialogFooter,
  DialogDescription,
} from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { queryClient, apiRequest } from "@/lib/queryClient";
import { useToast } from "@/hooks/use-toast";
import type { WbsTag } from "@shared/schema";
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  type DragEndEvent,
} from "@dnd-kit/core";
import {
  arrayMove,
  SortableContext,
  sortableKeyboardCoordinates,
  useSortable,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";

const PRESET_COLORS = [
  "#D4A017", "#C0392B", "#2980B9", "#27AE60", "#8E44AD",
  "#E67E22", "#1ABC9C", "#E74C3C", "#3498DB", "#2ECC71",
  "#9B59B6", "#F39C12", "#16A085",
];

type SeedStep = "industry" | "review";

function SortableTag({
  tag,
  index,
  onEdit,
  onDelete,
}: {
  tag: WbsTag;
  index: number;
  onEdit: (tag: WbsTag) => void;
  onDelete: (id: string) => void;
}) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: tag.id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
    zIndex: isDragging ? 10 : undefined,
  };

  return (
    <div ref={setNodeRef} style={style}>
      <Card className={isDragging ? "ring-1 ring-primary" : "hover-elevate"} data-testid={`card-wbs-tag-${tag.id}`}>
        <CardContent className="px-3 py-1.5">
          <div className="flex items-center gap-2">
            <div className="flex items-center gap-1.5 shrink-0">
              <button
                className="cursor-grab active:cursor-grabbing touch-none"
                {...attributes}
                {...listeners}
                data-testid={`drag-handle-${tag.id}`}
              >
                <GripVertical className="w-3 h-3 text-muted-foreground" />
              </button>
              <div
                className="w-4 h-4 rounded-full shrink-0 border border-border"
                style={{ backgroundColor: tag.color }}
              />
            </div>
            <div className="flex-1 min-w-0 flex items-center gap-1.5">
              <span className="text-xs font-medium truncate" data-testid={`text-wbs-name-${tag.id}`}>
                {tag.name}
              </span>
              <Badge
                variant="outline"
                className="text-[9px] shrink-0"
                style={{ borderColor: tag.color, color: tag.color }}
              >
                WBS-{String(index + 1).padStart(2, "0")}
              </Badge>
            </div>
            <div className="flex items-center shrink-0">
              <Button
                size="icon"
                variant="ghost"
                onClick={() => onEdit(tag)}
                data-testid={`button-edit-tag-${tag.id}`}
              >
                <Pencil className="w-3 h-3" />
              </Button>
              <Button
                size="icon"
                variant="ghost"
                onClick={() => onDelete(tag.id)}
                data-testid={`button-delete-tag-${tag.id}`}
              >
                <Trash2 className="w-3 h-3 text-destructive" />
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

export default function WbsTagsPage() {
  const { toast } = useToast();
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingTag, setEditingTag] = useState<WbsTag | null>(null);
  const [tagName, setTagName] = useState("");
  const [tagColor, setTagColor] = useState(PRESET_COLORS[0]);

  const [seedOpen, setSeedOpen] = useState(false);
  const [seedStep, setSeedStep] = useState<SeedStep>("industry");
  const [selectedIndustry, setSelectedIndustry] = useState<string | null>(null);
  const [recommendations, setRecommendations] = useState<{ name: string; color: string }[]>([]);
  const [selectedRecs, setSelectedRecs] = useState<Set<number>>(new Set());

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }),
  );

  const { data: tags, isLoading } = useQuery<WbsTag[]>({
    queryKey: ["/api/wbs-tags"],
  });

  const { data: industries } = useQuery<string[]>({
    queryKey: ["/api/wbs-tags/industries"],
  });

  const createMutation = useMutation({
    mutationFn: async (data: { name: string; color: string }) => {
      await apiRequest("POST", "/api/wbs-tags", data);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["/api/wbs-tags"] });
      toast({ title: "WBS tag created" });
      closeDialog();
    },
    onError: (error: any) => {
      toast({ title: "Error", description: error.message, variant: "destructive" });
    },
  });

  const updateMutation = useMutation({
    mutationFn: async ({ id, data }: { id: string; data: { name: string; color: string } }) => {
      await apiRequest("PATCH", `/api/wbs-tags/${id}`, data);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["/api/wbs-tags"] });
      toast({ title: "WBS tag updated" });
      closeDialog();
    },
    onError: (error: any) => {
      toast({ title: "Error", description: error.message, variant: "destructive" });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: async (id: string) => {
      await apiRequest("DELETE", `/api/wbs-tags/${id}`);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["/api/wbs-tags"] });
      toast({ title: "WBS tag deleted" });
    },
  });

  const reorderMutation = useMutation({
    mutationFn: async (orderedIds: string[]) => {
      await apiRequest("PUT", "/api/wbs-tags/reorder", { orderedIds });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["/api/wbs-tags"] });
    },
    onError: (error: any) => {
      toast({ title: "Reorder failed", description: error.message, variant: "destructive" });
      queryClient.invalidateQueries({ queryKey: ["/api/wbs-tags"] });
    },
  });

  const recommendMutation = useMutation({
    mutationFn: async (industry: string) => {
      const res = await apiRequest("POST", "/api/wbs-tags/recommend", { industry });
      return res.json() as Promise<{ name: string; color: string }[]>;
    },
    onSuccess: (data) => {
      setRecommendations(data);
      const slotsAvailable = 13 - (tags?.length || 0);
      const allIndices = new Set(data.map((_: any, i: number) => i).slice(0, slotsAvailable));
      setSelectedRecs(allIndices);
      setSeedStep("review");
    },
    onError: (error: any) => {
      toast({ title: "Error", description: error.message, variant: "destructive" });
    },
  });

  const seedMutation = useMutation({
    mutationFn: async (seedTags: { name: string; color: string }[]) => {
      await apiRequest("POST", "/api/wbs-tags/seed", { tags: seedTags });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["/api/wbs-tags"] });
      toast({ title: "WBS tags seeded", description: `${selectedRecs.size} tags created from ${selectedIndustry} template` });
      closeSeedDialog();
    },
    onError: (error: any) => {
      toast({ title: "Error", description: error.message, variant: "destructive" });
    },
  });

  function openCreate() {
    setEditingTag(null);
    setTagName("");
    setTagColor(PRESET_COLORS[(tags?.length || 0) % PRESET_COLORS.length]);
    setDialogOpen(true);
  }

  function openEdit(tag: WbsTag) {
    setEditingTag(tag);
    setTagName(tag.name);
    setTagColor(tag.color);
    setDialogOpen(true);
  }

  function closeDialog() {
    setDialogOpen(false);
    setEditingTag(null);
    setTagName("");
    setTagColor(PRESET_COLORS[0]);
  }

  function handleSave() {
    if (!tagName.trim()) return;
    if (editingTag) {
      updateMutation.mutate({ id: editingTag.id, data: { name: tagName.trim(), color: tagColor } });
    } else {
      createMutation.mutate({ name: tagName.trim(), color: tagColor });
    }
  }

  function openSeedDialog() {
    setSeedStep("industry");
    setSelectedIndustry(null);
    setRecommendations([]);
    setSelectedRecs(new Set());
    setSeedOpen(true);
  }

  function closeSeedDialog() {
    setSeedOpen(false);
    setSelectedIndustry(null);
    setRecommendations([]);
    setSelectedRecs(new Set());
    setSeedStep("industry");
  }

  function handleIndustrySelect(industry: string) {
    setSelectedIndustry(industry);
    recommendMutation.mutate(industry);
  }

  function toggleRec(index: number) {
    setSelectedRecs((prev) => {
      const next = new Set(prev);
      if (next.has(index)) {
        next.delete(index);
      } else {
        const slotsAvailable = 13 - (tags?.length || 0);
        if (next.size < slotsAvailable) {
          next.add(index);
        }
      }
      return next;
    });
  }

  function handleSeedConfirm() {
    const selected = recommendations.filter((_, i) => selectedRecs.has(i));
    if (selected.length > 0) {
      seedMutation.mutate(selected);
    }
  }

  function handleDragEnd(event: DragEndEvent) {
    const { active, over } = event;
    if (!over || active.id === over.id || !tags) return;

    const oldIndex = tags.findIndex((t) => t.id === active.id);
    const newIndex = tags.findIndex((t) => t.id === over.id);
    if (oldIndex === -1 || newIndex === -1) return;

    const reordered = arrayMove(tags, oldIndex, newIndex);
    queryClient.setQueryData(["/api/wbs-tags"], reordered);
    reorderMutation.mutate(reordered.map((t) => t.id));
  }

  const canAdd = (tags?.length || 0) < 13;
  const slotsAvailable = 13 - (tags?.length || 0);

  return (
    <div className="flex-1 overflow-auto">
      <div className="max-w-2xl mx-auto p-4 space-y-3">
        <div className="flex items-center justify-between gap-3 flex-wrap">
          <div>
            <h1 className="text-base font-semibold tracking-tight" data-testid="text-wbs-title">
              WBS Tags
            </h1>
            <p className="text-[11px] text-muted-foreground mt-0.5 tracking-wide">
              Configure up to 13 Work Breakdown Structure tags for filtering envelopes
            </p>
          </div>
          <div className="flex items-center gap-1.5">
            <Button
              size="sm"
              variant="outline"
              onClick={openSeedDialog}
              disabled={!canAdd}
              data-testid="button-seed-wbs"
            >
              <Sparkles className="w-3 h-3" />
              Seed WBS
            </Button>
            <Button
              size="sm"
              onClick={openCreate}
              disabled={!canAdd}
              data-testid="button-add-wbs-tag"
            >
              <Plus className="w-3 h-3" />
              Add Tag
            </Button>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <span className="text-[10px] text-muted-foreground uppercase tracking-wider font-medium">
            {tags?.length || 0} / 13 Tags Configured
          </span>
          {tags && tags.length > 1 && (
            <span className="text-[9px] text-muted-foreground/60 italic">
              Drag to reorder
            </span>
          )}
        </div>

        {isLoading ? (
          <div className="space-y-1">
            {[1, 2, 3].map((i) => (
              <Card key={i}>
                <CardContent className="px-3 py-1.5">
                  <div className="flex items-center gap-2">
                    <Skeleton className="h-4 w-4 rounded-full" />
                    <Skeleton className="h-3 w-40" />
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        ) : !tags || tags.length === 0 ? (
          <Card>
            <CardContent className="p-8 flex flex-col items-center justify-center text-center">
              <div className="w-8 h-8 rounded-full bg-muted flex items-center justify-center mb-2">
                <Tag className="w-3.5 h-3.5 text-muted-foreground" />
              </div>
              <h3 className="text-xs font-medium mb-0.5">No WBS tags configured</h3>
              <p className="text-[10px] text-muted-foreground mb-2">
                Create tags manually or use Seed WBS to auto-generate from an industry template
              </p>
              <div className="flex items-center gap-2">
                <Button size="sm" variant="outline" onClick={openSeedDialog} data-testid="button-empty-seed">
                  <Sparkles className="w-3.5 h-3.5" />
                  Seed WBS
                </Button>
                <Button size="sm" onClick={openCreate} data-testid="button-empty-add-tag">
                  <Plus className="w-3.5 h-3.5" />
                  Create First Tag
                </Button>
              </div>
            </CardContent>
          </Card>
        ) : (
          <DndContext
            sensors={sensors}
            collisionDetection={closestCenter}
            onDragEnd={handleDragEnd}
          >
            <SortableContext
              items={tags.map((t) => t.id)}
              strategy={verticalListSortingStrategy}
            >
              <div className="space-y-1">
                {tags.map((tag, index) => (
                  <SortableTag
                    key={tag.id}
                    tag={tag}
                    index={index}
                    onEdit={openEdit}
                    onDelete={(id) => deleteMutation.mutate(id)}
                  />
                ))}
              </div>
            </SortableContext>
          </DndContext>
        )}

        <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
          <DialogContent>
            <DialogHeader>
              <DialogTitle className="text-sm">
                {editingTag ? "Edit WBS Tag" : "Create WBS Tag"}
              </DialogTitle>
            </DialogHeader>
            <div className="space-y-4 py-2">
              <div className="space-y-1.5">
                <Label className="text-[10px] uppercase tracking-wider">Tag Name</Label>
                <Input
                  value={tagName}
                  onChange={(e) => setTagName(e.target.value)}
                  placeholder="e.g., Design Phase, Development, Testing"
                  className="text-xs"
                  data-testid="input-tag-name"
                  onKeyDown={(e) => { if (e.key === "Enter") handleSave(); }}
                />
              </div>
              <div className="space-y-1.5">
                <Label className="text-[10px] uppercase tracking-wider">Color</Label>
                <div className="flex items-center gap-1.5 flex-wrap">
                  {PRESET_COLORS.map((c) => (
                    <button
                      key={c}
                      className={`w-7 h-7 rounded-full border-2 transition-transform ${
                        tagColor === c ? "border-foreground scale-110" : "border-transparent"
                      }`}
                      style={{ backgroundColor: c }}
                      onClick={() => setTagColor(c)}
                      data-testid={`button-color-${c.replace("#", "")}`}
                    />
                  ))}
                </div>
                <div className="flex items-center gap-2 mt-2">
                  <Label className="text-[10px] text-muted-foreground">Custom:</Label>
                  <Input
                    type="color"
                    value={tagColor}
                    onChange={(e) => setTagColor(e.target.value)}
                    className="w-9 h-9 p-0.5 cursor-pointer"
                    data-testid="input-custom-color"
                  />
                  <span className="text-[10px] text-muted-foreground font-mono">{tagColor}</span>
                </div>
              </div>
              <div className="flex items-center gap-2">
                <Label className="text-[10px] uppercase tracking-wider">Preview:</Label>
                <Badge
                  style={{ backgroundColor: tagColor, color: "#fff", borderColor: tagColor }}
                  data-testid="badge-preview"
                >
                  {tagName || "Tag Name"}
                </Badge>
              </div>
            </div>
            <DialogFooter>
              <Button variant="outline" size="sm" onClick={closeDialog} data-testid="button-cancel-tag">
                Cancel
              </Button>
              <Button
                size="sm"
                onClick={handleSave}
                disabled={!tagName.trim() || createMutation.isPending || updateMutation.isPending}
                data-testid="button-save-tag"
              >
                {editingTag ? "Update" : "Create"}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>

        <Dialog open={seedOpen} onOpenChange={setSeedOpen}>
          <DialogContent className="max-w-lg">
            <DialogHeader>
              <DialogTitle className="text-sm flex items-center gap-2">
                <Sparkles className="w-4 h-4 text-primary" />
                {seedStep === "industry" ? "Seed WBS Tags" : `${selectedIndustry} — Review Tags`}
              </DialogTitle>
              <DialogDescription className="text-[11px]">
                {seedStep === "industry"
                  ? "Select your industry or purpose and we'll recommend 13 WBS categories tailored to your workflow."
                  : `Select which tags to create (${slotsAvailable} slot${slotsAvailable !== 1 ? "s" : ""} available). You can customize them later.`
                }
              </DialogDescription>
            </DialogHeader>

            {seedStep === "industry" ? (
              <div className="py-2 space-y-2 max-h-[50vh] overflow-y-auto">
                {industries ? (
                  <div className="grid grid-cols-2 gap-1.5">
                    {industries.map((ind) => (
                      <Button
                        key={ind}
                        size="sm"
                        variant={selectedIndustry === ind ? "default" : "outline"}
                        onClick={() => handleIndustrySelect(ind)}
                        disabled={recommendMutation.isPending}
                        className="justify-start text-[11px] gap-2"
                        data-testid={`button-industry-${ind.toLowerCase().replace(/[^a-z0-9]+/g, "-")}`}
                      >
                        <Building2 className="w-3 h-3 shrink-0" />
                        <span className="truncate">{ind}</span>
                        {recommendMutation.isPending && selectedIndustry === ind && (
                          <Loader2 className="w-3 h-3 animate-spin ml-auto shrink-0" />
                        )}
                      </Button>
                    ))}
                  </div>
                ) : (
                  <div className="flex items-center justify-center py-8">
                    <Loader2 className="w-5 h-5 animate-spin text-muted-foreground" />
                  </div>
                )}
              </div>
            ) : (
              <div className="py-2 space-y-1.5 max-h-[50vh] overflow-y-auto">
                {recommendations.map((rec, i) => {
                  const isSelected = selectedRecs.has(i);
                  const isDisabled = !isSelected && selectedRecs.size >= slotsAvailable;
                  return (
                    <div
                      key={i}
                      className={`flex items-center gap-2.5 px-3 py-2 rounded-md cursor-pointer transition-colors ${
                        isSelected
                          ? "bg-primary/10 border border-primary/30"
                          : isDisabled
                          ? "opacity-40 cursor-not-allowed"
                          : "border border-transparent hover-elevate"
                      }`}
                      onClick={() => !isDisabled && toggleRec(i)}
                      data-testid={`rec-tag-${i}`}
                    >
                      <div
                        className={`w-4 h-4 rounded-sm border-2 flex items-center justify-center shrink-0 ${
                          isSelected ? "bg-primary border-primary" : "border-muted-foreground/30"
                        }`}
                      >
                        {isSelected && <Check className="w-2.5 h-2.5 text-primary-foreground" />}
                      </div>
                      <div
                        className="w-3.5 h-3.5 rounded-full shrink-0"
                        style={{ backgroundColor: rec.color }}
                      />
                      <span className="text-[11px] font-medium flex-1">{rec.name}</span>
                      <Badge
                        variant="outline"
                        className="text-[8px] shrink-0"
                        style={{ borderColor: rec.color, color: rec.color }}
                      >
                        WBS-{String(i + 1).padStart(2, "0")}
                      </Badge>
                    </div>
                  );
                })}
              </div>
            )}

            <DialogFooter>
              {seedStep === "review" && (
                <>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => setSeedStep("industry")}
                    data-testid="button-seed-back"
                  >
                    Back
                  </Button>
                  <div className="flex-1" />
                  <span className="text-[10px] text-muted-foreground self-center mr-2">
                    {selectedRecs.size} selected
                  </span>
                </>
              )}
              <Button
                variant="outline"
                size="sm"
                onClick={closeSeedDialog}
                data-testid="button-seed-cancel"
              >
                Cancel
              </Button>
              {seedStep === "review" && (
                <Button
                  size="sm"
                  onClick={handleSeedConfirm}
                  disabled={selectedRecs.size === 0 || seedMutation.isPending}
                  data-testid="button-seed-confirm"
                >
                  {seedMutation.isPending ? (
                    <Loader2 className="w-3.5 h-3.5 animate-spin" />
                  ) : (
                    <Sparkles className="w-3.5 h-3.5" />
                  )}
                  Create {selectedRecs.size} Tags
                </Button>
              )}
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>
    </div>
  );
}
