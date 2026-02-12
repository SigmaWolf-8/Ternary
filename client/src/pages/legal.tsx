/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL
 * All Rights Reserved.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * Unauthorized copying, modification, distribution, or use of this file,
 * via any medium, is strictly prohibited without the prior written
 * permission of Capomastro Holdings Ltd.
 *
 * See LICENSE in the repository root for full terms.
 */

import { useQuery } from "@tanstack/react-query";
import { Link, useRoute } from "wouter";
import { ArrowLeft } from "lucide-react";
import { Button } from "@/components/ui/button";

const VALID_TYPES = ["terms", "privacy", "security", "aup"] as const;
type LegalType = (typeof VALID_TYPES)[number];

// Markdown parser function that converts markdown to HTML
function parseMarkdown(content: string): string {
  let html = content;

  // Escape HTML special characters first (except for our markdown markers)
  html = html.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

  // Horizontal rules (---)
  html = html.replace(/^---$/gm, "<hr class=\"my-6 border-t border-muted-foreground/20\" />");

  // Headings must be processed before paragraphs
  // H3 (###)
  html = html.replace(/^### (.+)$/gm, "<h3 class=\"text-xl font-semibold mt-6 mb-3\">$1</h3>");
  // H2 (##)
  html = html.replace(/^## (.+)$/gm, "<h2 class=\"text-2xl font-bold mt-8 mb-4\">$1</h2>");
  // H1 (#)
  html = html.replace(/^# (.+)$/gm, "<h1 class=\"text-3xl font-bold mt-8 mb-4\">$1</h1>");

  // Bold text (**text**)
  html = html.replace(/\*\*(.+?)\*\*/g, "<strong class=\"font-semibold\">$1</strong>");

  // Links [text](url)
  html = html.replace(/\[(.+?)\]\((.+?)\)/g, "<a href=\"$2\" class=\"text-primary hover:underline\" target=\"_blank\" rel=\"noopener noreferrer\">$1</a>");

  // Split into lines for paragraph and list processing
  const lines = html.split("\n");
  let processed: string[] = [];
  let inOrderedList = false;
  let inUnorderedList = false;
  let paragraphBuffer: string[] = [];

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim();

    // Handle empty lines
    if (line === "") {
      if (paragraphBuffer.length > 0) {
        processed.push(`<p class="mb-4 text-base leading-relaxed">${paragraphBuffer.join(" ")}</p>`);
        paragraphBuffer = [];
      }
      if (inOrderedList) {
        processed.push("</ol>");
        inOrderedList = false;
      }
      if (inUnorderedList) {
        processed.push("</ul>");
        inUnorderedList = false;
      }
      processed.push("");
      continue;
    }

    // Handle unordered lists (- item)
    const unorderedMatch = line.match(/^- (.+)$/);
    if (unorderedMatch) {
      if (inOrderedList) {
        processed.push("</ol>");
        inOrderedList = false;
      }
      if (!inUnorderedList) {
        processed.push("<ul class=\"list-disc list-inside mb-4 space-y-1\">");
        inUnorderedList = true;
      }
      if (paragraphBuffer.length > 0) {
        processed.push(`<p class="mb-4 text-base leading-relaxed">${paragraphBuffer.join(" ")}</p>`);
        paragraphBuffer = [];
      }
      processed.push(`<li class="text-base">${unorderedMatch[1]}</li>`);
      continue;
    }

    // Handle ordered lists (1. item, 2. item, etc.)
    const orderedMatch = line.match(/^\d+\. (.+)$/);
    if (orderedMatch) {
      if (inUnorderedList) {
        processed.push("</ul>");
        inUnorderedList = false;
      }
      if (!inOrderedList) {
        processed.push("<ol class=\"list-decimal list-inside mb-4 space-y-1\">");
        inOrderedList = true;
      }
      if (paragraphBuffer.length > 0) {
        processed.push(`<p class="mb-4 text-base leading-relaxed">${paragraphBuffer.join(" ")}</p>`);
        paragraphBuffer = [];
      }
      processed.push(`<li class="text-base">${orderedMatch[1]}</li>`);
      continue;
    }

    // Handle headings, hr, and other block elements
    if (line.startsWith("<h") || line.startsWith("<hr") || line.startsWith("</")) {
      if (paragraphBuffer.length > 0) {
        processed.push(`<p class="mb-4 text-base leading-relaxed">${paragraphBuffer.join(" ")}</p>`);
        paragraphBuffer = [];
      }
      if (inOrderedList) {
        processed.push("</ol>");
        inOrderedList = false;
      }
      if (inUnorderedList) {
        processed.push("</ul>");
        inUnorderedList = false;
      }
      processed.push(line);
      continue;
    }

    // Accumulate paragraph text
    if (line.length > 0) {
      paragraphBuffer.push(line);
    }
  }

  // Close any remaining open tags and paragraphs
  if (paragraphBuffer.length > 0) {
    processed.push(`<p class="mb-4 text-base leading-relaxed">${paragraphBuffer.join(" ")}</p>`);
  }
  if (inOrderedList) {
    processed.push("</ol>");
  }
  if (inUnorderedList) {
    processed.push("</ul>");
  }

  return processed.join("\n");
}

// Format date as "Month Day, Year"
function formatDate(date: Date): string {
  const options: Intl.DateTimeFormatOptions = { year: "numeric", month: "long", day: "numeric" };
  return date.toLocaleDateString("en-US", options);
}

export default function LegalPage() {
  const [, paramsTerms] = useRoute("/terms");
  const [, paramsPrivacy] = useRoute("/privacy");
  const [, paramsSecurity] = useRoute("/security");
  const [, paramsAup] = useRoute("/aup");

  const type: LegalType = paramsTerms
    ? "terms"
    : paramsPrivacy
      ? "privacy"
      : paramsSecurity
        ? "security"
        : paramsAup
          ? "aup"
          : "terms";

  const { data, isLoading, error } = useQuery<{ title: string; content: string }>({
    queryKey: ["/api/legal", type],
  });

  const currentDate = formatDate(new Date());
  const parsedHtml = data ? parseMarkdown(data.content) : "";

  return (
    <div className="min-h-screen bg-background" data-testid="page-legal">
      <div className="max-w-4xl mx-auto px-5 py-8">
        <div className="mb-8">
          <Button
            variant="ghost"
            size="sm"
            asChild
            data-testid="link-back-home"
            aria-label="Back to home page"
          >
            <Link href="/">
              <ArrowLeft className="w-4 h-4 mr-2" />
              Back to Home
            </Link>
          </Button>
        </div>

        {isLoading && (
          <div className="flex items-center justify-center py-20" data-testid="loading-legal">
            <div className="text-muted-foreground">Loading...</div>
          </div>
        )}

        {error && (
          <div className="text-center py-20" data-testid="error-legal">
            <p className="text-destructive">Failed to load document.</p>
          </div>
        )}

        {data && (
          <div data-testid="content-legal" className="prose dark:prose-invert prose-sm max-w-none">
            <h1 className="text-4xl font-bold mb-2 mt-0" data-testid="text-legal-title">
              {data.title}
            </h1>
            <p className="text-sm text-muted-foreground mb-8 mt-0" data-testid="text-legal-updated">
              Last Updated: {currentDate}
            </p>

            <div
              className="space-y-0 text-foreground"
              data-testid="text-legal-content"
              dangerouslySetInnerHTML={{ __html: parsedHtml }}
            />
          </div>
        )}
      </div>
    </div>
  );
}
