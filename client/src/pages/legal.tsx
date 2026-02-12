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

const VALID_TYPES = ["terms", "privacy", "security"] as const;
type LegalType = (typeof VALID_TYPES)[number];

export default function LegalPage() {
  const [, paramsTerms] = useRoute("/terms");
  const [, paramsPrivacy] = useRoute("/privacy");
  const [, paramsSecurity] = useRoute("/security");

  const type: LegalType = paramsTerms
    ? "terms"
    : paramsPrivacy
      ? "privacy"
      : paramsSecurity
        ? "security"
        : "terms";

  const { data, isLoading, error } = useQuery<{ title: string; content: string }>({
    queryKey: ["/api/legal", type],
  });

  return (
    <div className="min-h-screen bg-background" data-testid="page-legal">
      <div className="max-w-4xl mx-auto px-5 py-8">
        <div className="mb-8">
          <Button variant="ghost" size="sm" asChild data-testid="link-back-home">
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
          <div data-testid="content-legal">
            <h1 className="text-3xl font-bold mb-8" data-testid="text-legal-title">
              {data.title}
            </h1>
            <pre
              className="whitespace-pre-wrap font-sans text-sm leading-relaxed text-muted-foreground"
              data-testid="text-legal-content"
            >
              {data.content}
            </pre>
          </div>
        )}
      </div>
    </div>
  );
}
