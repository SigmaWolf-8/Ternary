import { useState, useMemo } from "react";
import { useQuery, useMutation } from "@tanstack/react-query";
import {
  Tag, Check, FileText, Search, X,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { queryClient, apiRequest } from "@/lib/queryClient";
import { useToast } from "@/hooks/use-toast";
import type { Envelope, WbsTag, EnvelopeWbsTag } from "@shared/schema";

function EnvelopeTagRow({
  envelope,
  wbsTags,
  assignedTagIds,
  onToggleTag,
  isPending,
}: {
  envelope: Envelope;
  wbsTags: WbsTag[];
  assignedTagIds: string[];
  onToggleTag: (envelopeId: string, tagId: string, isAssigned: boolean) => void;
  isPending: boolean;
}) {
  return (
    <Card className="hover-elevate" data-testid={`card-envelope-tagging-${envelope.id}`}>
      <CardContent className="p-3.5 space-y-2">
        <div className="flex items-center gap-2">
          <FileText className="w-3.5 h-3.5 text-muted-foreground shrink-0" />
          <span className="text-xs font-medium truncate flex-1" data-testid={`text-envelope-title-${envelope.id}`}>
            {envelope.title}
          </span>
          <Badge variant="outline" className="text-[9px] shrink-0">
            {envelope.status}
          </Badge>
        </div>
        <div className="flex items-center gap-1.5 flex-wrap">
          {wbsTags.map((tag) => {
            const isAssigned = assignedTagIds.includes(tag.id);
            return (
              <button
                key={tag.id}
                onClick={() => onToggleTag(envelope.id, tag.id, isAssigned)}
                disabled={isPending}
                className={`inline-flex items-center gap-1 px-2 py-1 rounded-md text-[10px] font-medium transition-all border ${
                  isAssigned
                    ? "text-white border-transparent"
                    : "bg-transparent border-border text-muted-foreground"
                }`}
                style={
                  isAssigned
                    ? { backgroundColor: tag.color, borderColor: tag.color }
                    : undefined
                }
                data-testid={`toggle-tag-${envelope.id}-${tag.id}`}
              >
                {isAssigned && <Check className="w-2.5 h-2.5" />}
                <span
                  className="w-2 h-2 rounded-full shrink-0"
                  style={{ backgroundColor: isAssigned ? "rgba(255,255,255,0.6)" : tag.color }}
                />
                {tag.name}
              </button>
            );
          })}
        </div>
      </CardContent>
    </Card>
  );
}

export default function WbsTaggingPage() {
  const { toast } = useToast();
  const [searchQuery, setSearchQuery] = useState("");
  const [filterTagId, setFilterTagId] = useState<string | null>(null);

  const { data: envelopes, isLoading: envLoading } = useQuery<Envelope[]>({
    queryKey: ["/api/envelopes"],
  });

  const { data: wbsTags, isLoading: tagsLoading } = useQuery<WbsTag[]>({
    queryKey: ["/api/wbs-tags"],
  });

  const { data: allEnvTags } = useQuery<EnvelopeWbsTag[]>({
    queryKey: ["/api/envelope-wbs-tags"],
  });

  const envTagMap = useMemo(() => {
    const map: Record<string, string[]> = {};
    allEnvTags?.forEach((et) => {
      if (!map[et.envelopeId]) map[et.envelopeId] = [];
      map[et.envelopeId].push(et.wbsTagId);
    });
    return map;
  }, [allEnvTags]);

  const toggleTagMutation = useMutation({
    mutationFn: async ({ envelopeId, tagId, isAssigned }: { envelopeId: string; tagId: string; isAssigned: boolean }) => {
      const current = envTagMap[envelopeId] || [];
      const newTagIds = isAssigned
        ? current.filter((id) => id !== tagId)
        : [...current, tagId];
      await apiRequest("PUT", `/api/envelopes/${envelopeId}/wbs-tags`, { tagIds: newTagIds });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["/api/envelope-wbs-tags"] });
      queryClient.invalidateQueries({ queryKey: ["/api/envelopes"] });
    },
    onError: (error: any) => {
      toast({ title: "Error", description: error.message, variant: "destructive" });
    },
  });

  const filteredEnvelopes = useMemo(() => {
    if (!envelopes) return [];
    return envelopes.filter((e) => {
      const matchesSearch = !searchQuery || e.title.toLowerCase().includes(searchQuery.toLowerCase());
      const tags = envTagMap[e.id] || [];
      const matchesFilter =
        !filterTagId ||
        (filterTagId === "untagged" ? tags.length === 0 : tags.includes(filterTagId));
      return matchesSearch && matchesFilter;
    });
  }, [envelopes, searchQuery, filterTagId, envTagMap]);

  const isLoading = envLoading || tagsLoading;

  return (
    <div className="flex-1 overflow-auto">
      <div className="max-w-3xl mx-auto p-5 space-y-4">
        <div>
          <h1 className="text-base font-semibold tracking-tight" data-testid="text-tagging-title">
            Tag Envelopes
          </h1>
          <p className="text-[11px] text-muted-foreground mt-0.5 tracking-wide">
            Assign multiple WBS tags to each envelope for cross-category tracking
          </p>
        </div>

        <div className="flex items-center gap-2 flex-wrap">
          <div className="relative flex-1 min-w-[180px]">
            <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground" />
            <Input
              placeholder="Search envelopes..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="pl-8 text-xs"
              data-testid="input-tagging-search"
            />
            {searchQuery && (
              <Button
                size="icon"
                variant="ghost"
                className="absolute right-0.5 top-1/2 -translate-y-1/2"
                onClick={() => setSearchQuery("")}
              >
                <X className="w-3 h-3" />
              </Button>
            )}
          </div>
        </div>

        {wbsTags && wbsTags.length > 0 && (
          <div className="flex items-center gap-1.5 flex-wrap">
            <span className="text-[9px] text-muted-foreground uppercase tracking-widest font-medium mr-1">
              Filter:
            </span>
            <Button
              size="sm"
              variant={filterTagId === null ? "default" : "outline"}
              onClick={() => setFilterTagId(null)}
              className="text-[10px]"
              data-testid="button-filter-all"
            >
              All
            </Button>
            {wbsTags.map((tag) => (
              <Button
                key={tag.id}
                size="sm"
                variant={filterTagId === tag.id ? "default" : "outline"}
                onClick={() => setFilterTagId(filterTagId === tag.id ? null : tag.id)}
                className="text-[10px] gap-1"
                data-testid={`button-filter-tag-${tag.id}`}
              >
                <span className="w-2 h-2 rounded-full shrink-0" style={{ backgroundColor: tag.color }} />
                {tag.name}
              </Button>
            ))}
            <Button
              size="sm"
              variant={filterTagId === "untagged" ? "default" : "outline"}
              onClick={() => setFilterTagId(filterTagId === "untagged" ? null : "untagged")}
              className="text-[10px]"
              data-testid="button-filter-untagged"
            >
              Untagged
            </Button>
          </div>
        )}

        {isLoading ? (
          <div className="space-y-2">
            {[1, 2, 3].map((i) => (
              <Card key={i}>
                <CardContent className="p-3.5">
                  <div className="flex items-center gap-3">
                    <Skeleton className="h-4 w-4" />
                    <Skeleton className="h-4 w-48" />
                  </div>
                  <div className="flex items-center gap-1.5 mt-2">
                    <Skeleton className="h-6 w-16" />
                    <Skeleton className="h-6 w-16" />
                    <Skeleton className="h-6 w-16" />
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        ) : !wbsTags || wbsTags.length === 0 ? (
          <Card>
            <CardContent className="p-10 flex flex-col items-center justify-center text-center">
              <div className="w-10 h-10 rounded-full bg-muted flex items-center justify-center mb-3">
                <Tag className="w-4 h-4 text-muted-foreground" />
              </div>
              <h3 className="text-xs font-medium mb-0.5">No WBS tags configured</h3>
              <p className="text-[10px] text-muted-foreground">
                Create WBS tags first in the WBS Tags page before tagging envelopes
              </p>
            </CardContent>
          </Card>
        ) : filteredEnvelopes.length === 0 ? (
          <Card>
            <CardContent className="p-10 flex flex-col items-center justify-center text-center">
              <div className="w-10 h-10 rounded-full bg-muted flex items-center justify-center mb-3">
                <FileText className="w-4 h-4 text-muted-foreground" />
              </div>
              <h3 className="text-xs font-medium mb-0.5">No envelopes found</h3>
              <p className="text-[10px] text-muted-foreground">
                {searchQuery ? "Try a different search term" : "Create some envelopes to start tagging"}
              </p>
            </CardContent>
          </Card>
        ) : (
          <div className="space-y-2">
            {filteredEnvelopes.map((env) => (
              <EnvelopeTagRow
                key={env.id}
                envelope={env}
                wbsTags={wbsTags}
                assignedTagIds={envTagMap[env.id] || []}
                onToggleTag={(eid, tid, isAssigned) => toggleTagMutation.mutate({ envelopeId: eid, tagId: tid, isAssigned })}
                isPending={toggleTagMutation.isPending}
              />
            ))}
          </div>
        )}

        <div className="text-[9px] text-muted-foreground text-center pt-2">
          {filteredEnvelopes.length} envelope{filteredEnvelopes.length !== 1 ? "s" : ""} shown
        </div>
      </div>
    </div>
  );
}
