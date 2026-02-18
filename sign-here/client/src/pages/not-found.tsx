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
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { AlertCircle, ArrowLeft } from "lucide-react";
import { Link } from "wouter";

export default function NotFound() {
  return (
    <div className="flex-1 flex items-center justify-center bg-background">
      <Card className="w-full max-w-md mx-4">
        <CardContent className="pt-6">
          <div className="flex mb-4 gap-2">
            <AlertCircle className="h-8 w-8 text-destructive shrink-0" />
            <h1 className="text-2xl font-bold" data-testid="text-404">404 Page Not Found</h1>
          </div>
          <p className="text-sm text-muted-foreground mb-4">
            The page you're looking for doesn't exist.
          </p>
          <Link href="/">
            <Button variant="outline" size="sm" data-testid="button-go-home">
              <ArrowLeft className="w-3 h-3" />
              Back to File Cabinet
            </Button>
          </Link>
        </CardContent>
      </Card>
    </div>
  );
}
