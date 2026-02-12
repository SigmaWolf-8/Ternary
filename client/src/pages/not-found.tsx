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

import { Button } from "@/components/ui/button";
import { Box, ArrowLeft, Home } from "lucide-react";
import { Link } from "wouter";

export default function NotFound() {
  return (
    <div className="min-h-screen w-full flex items-center justify-center bg-background" data-testid="page-not-found">
      <div className="w-full max-w-md mx-4 text-center">
        <div className="flex items-center justify-center gap-2 text-primary font-bold text-xl mb-8">
          <Box className="w-6 h-6" />
          <span>PlenumNET</span>
        </div>

        <h1 className="text-7xl font-bold text-primary mb-4" data-testid="text-404">404</h1>
        <h2 className="text-xl font-semibold text-foreground mb-2">Page not found</h2>
        <p className="text-sm text-muted-foreground mb-8">
          The page you're looking for doesn't exist or has been moved.
        </p>

        <div className="flex flex-wrap justify-center gap-3">
          <Button asChild data-testid="button-go-home">
            <Link href="/">
              <Home className="w-4 h-4 mr-2" />
              Back to Home
            </Link>
          </Button>
          <Button variant="outline" asChild data-testid="button-view-docs">
            <Link href="/docs">
              View Documentation
            </Link>
          </Button>
        </div>
      </div>
    </div>
  );
}
