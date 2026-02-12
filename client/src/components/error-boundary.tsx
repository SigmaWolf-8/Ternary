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

import React from "react";
import { Button } from "@/components/ui/button";
import { Box } from "lucide-react";
import { useLocation } from "wouter";

interface ErrorBoundaryProps {
  children: React.ReactNode;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends React.Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
    };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return {
      hasError: true,
      error,
    };
  }

  componentDidCatch(error: Error) {
    console.error("ErrorBoundary caught an error:", error);
  }

  handleReset = () => {
    this.setState({
      hasError: false,
      error: null,
    });
  };

  render() {
    if (this.state.hasError) {
      return (
        <div
          className="min-h-screen w-full flex items-center justify-center bg-background text-foreground"
          data-testid="error-boundary-fallback"
        >
          <div className="w-full max-w-md mx-4 text-center">
            <div className="flex items-center justify-center gap-2 text-primary font-bold text-xl mb-8">
              <Box className="w-6 h-6" />
              <span>PlenumNET</span>
            </div>

            <h1 className="text-3xl font-bold text-foreground mb-4">Something went wrong</h1>
            <p className="text-sm text-muted-foreground mb-2">
              An unexpected error occurred. Please try refreshing the page or returning to home.
            </p>
            {this.state.error && (
              <p className="text-xs text-destructive mb-8 font-mono bg-background/50 p-3 rounded border border-destructive/20">
                {this.state.error.message}
              </p>
            )}

            <div className="flex flex-wrap justify-center gap-3">
              <Button
                onClick={this.handleReset}
                data-testid="button-error-reset"
              >
                Try Again
              </Button>
              <Button variant="outline" asChild data-testid="button-error-home">
                <a href="/">
                  Return to Home
                </a>
              </Button>
            </div>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
