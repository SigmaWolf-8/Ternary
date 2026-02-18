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
import { useState, useEffect, useCallback, useRef } from "react";
import { useLocation } from "wouter";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox";
import { X, ChevronRight, ChevronLeft, Archive, FilePlus, Grid3X3, Tags, Shield, Settings, FileText, Pencil } from "lucide-react";

const TOUR_STORAGE_KEY = "sign-here-tour-completed";
const TOUR_DISMISSED_KEY = "sign-here-tour-dismissed";

interface TourStep {
  target: string;
  title: string;
  description: string;
  icon: React.ElementType;
  position: "bottom" | "right" | "left" | "top";
  route?: string;
}

const tourSteps: TourStep[] = [
  {
    target: '[data-testid="link-nav-file-cabinet"]',
    title: "File Cabinet",
    description: "Your home base. View all envelopes, filter by status or WBS tags, search, and manage your documents.",
    icon: Archive,
    position: "right",
  },
  {
    target: '[data-testid="text-dashboard-title"]',
    title: "Your File Cabinet",
    description: "This is where all your envelopes live. Use the search bar and status filters at the top to quickly find what you need. Click any envelope to view details or open the editor.",
    icon: FileText,
    position: "bottom",
    route: "/",
  },
  {
    target: '[data-testid="link-nav-new-envelope"]',
    title: "Create Envelopes",
    description: "Start a new document envelope. Upload PDFs, DOCX, XLSX or CSV files and add recipients with different roles.",
    icon: FilePlus,
    position: "right",
  },
  {
    target: '[data-testid="link-nav-new-envelope"]',
    title: "PDF Editor",
    description: "After creating an envelope, use the visual PDF editor to place signature fields, date fields, checkboxes, and more. Drag, resize, and snap fields exactly where you need them on any page.",
    icon: Pencil,
    position: "right",
  },
  {
    target: '[data-testid="link-nav-about"]',
    title: "About Sign Here",
    description: "Learn about our certifications, compliance standards, and advanced features like PDF Stapler and Document Converter.",
    icon: Shield,
    position: "right",
  },
  {
    target: '[data-testid="link-nav-templates"]',
    title: "Template Gallery",
    description: "Browse reusable templates with pre-configured fields. Copy shared templates to customize them, or save your own from completed envelopes.",
    icon: Grid3X3,
    position: "right",
  },
  {
    target: '[data-testid="link-nav-wbs-tags"]',
    title: "WBS Tags",
    description: "Organize envelopes with Work Breakdown Structure tags. Create color-coded tags and assign multiple tags per envelope.",
    icon: Tags,
    position: "right",
  },
  {
    target: '[data-testid="link-nav-tag-envelopes"]',
    title: "Tag Envelopes",
    description: "Bulk-assign WBS tags across all your envelopes in one place. Toggle tags on or off for quick categorization.",
    icon: Tags,
    position: "right",
  },
  {
    target: '[data-testid="link-nav-settings"]',
    title: "Settings",
    description: "Configure your timezone, date format, and profile preferences. You can restart this tour anytime from Settings.",
    icon: Settings,
    position: "right",
  },
];

function getTooltipStyle(rect: DOMRect, position: string) {
  const gap = 12;

  switch (position) {
    case "right":
      return {
        top: `${rect.top + rect.height / 2}px`,
        left: `${rect.right + gap}px`,
        transform: "translateY(-50%)",
      };
    case "left":
      return {
        top: `${rect.top + rect.height / 2}px`,
        right: `${window.innerWidth - rect.left + gap}px`,
        transform: "translateY(-50%)",
      };
    case "bottom":
      return {
        top: `${rect.bottom + gap}px`,
        left: `${rect.left + rect.width / 2}px`,
        transform: "translateX(-50%)",
      };
    case "top":
      return {
        top: `${rect.top - gap}px`,
        left: `${rect.left + rect.width / 2}px`,
        transform: "translate(-50%, -100%)",
      };
    default:
      return {
        top: `${rect.bottom + gap}px`,
        left: `${rect.left}px`,
      };
  }
}

export function OnboardingTour() {
  const [active, setActive] = useState(false);
  const [currentStep, setCurrentStep] = useState(0);
  const [targetRect, setTargetRect] = useState<DOMRect | null>(null);
  const [dontShowAgain, setDontShowAgain] = useState(false);
  const [location, navigate] = useLocation();
  const overlayRef = useRef<HTMLDivElement>(null);

  const isStandalonePage = location.startsWith("/sign/") || location.startsWith("/share/");

  useEffect(() => {
    if (isStandalonePage) return;
    const completed = localStorage.getItem(TOUR_STORAGE_KEY);
    const dismissed = localStorage.getItem(TOUR_DISMISSED_KEY);
    if (!completed && !dismissed) {
      const timer = setTimeout(() => setActive(true), 1500);
      return () => clearTimeout(timer);
    }
  }, [isStandalonePage]);

  const updateTargetRect = useCallback(() => {
    const step = tourSteps[currentStep];
    if (!step) return;
    const el = document.querySelector(step.target);
    if (el) {
      setTargetRect(el.getBoundingClientRect());
    } else {
      setTargetRect(null);
    }
  }, [currentStep]);

  useEffect(() => {
    if (!active) return;
    updateTargetRect();
    const interval = setInterval(updateTargetRect, 300);
    window.addEventListener("resize", updateTargetRect);
    window.addEventListener("scroll", updateTargetRect, true);
    return () => {
      clearInterval(interval);
      window.removeEventListener("resize", updateTargetRect);
      window.removeEventListener("scroll", updateTargetRect, true);
    };
  }, [active, currentStep, updateTargetRect]);

  const endTour = useCallback((completed: boolean) => {
    setActive(false);
    if (completed || dontShowAgain) {
      localStorage.setItem(TOUR_STORAGE_KEY, "true");
    } else {
      localStorage.setItem(TOUR_DISMISSED_KEY, "true");
    }
  }, [dontShowAgain]);

  const nextStep = useCallback(() => {
    if (currentStep < tourSteps.length - 1) {
      const nextRoute = tourSteps[currentStep + 1].route;
      if (nextRoute) navigate(nextRoute);
      setCurrentStep((s) => s + 1);
    } else {
      endTour(true);
    }
  }, [currentStep, endTour, navigate]);

  const prevStep = useCallback(() => {
    if (currentStep > 0) {
      const prevRoute = tourSteps[currentStep - 1].route;
      if (prevRoute) navigate(prevRoute);
      setCurrentStep((s) => s - 1);
    }
  }, [currentStep, navigate]);

  useEffect(() => {
    if (active && isStandalonePage) {
      setActive(false);
    }
  }, [active, isStandalonePage]);

  if (!active || isStandalonePage) return null;

  const step = tourSteps[currentStep];
  const StepIcon = step.icon;
  const isLast = currentStep === tourSteps.length - 1;
  const isFirst = currentStep === 0;

  const tooltipStyle = targetRect
    ? { ...getTooltipStyle(targetRect, step.position), width: "320px", maxWidth: "calc(100vw - 24px)" }
    : { top: "50%", left: "50%", transform: "translate(-50%, -50%)", width: "320px", maxWidth: "calc(100vw - 24px)" };

  return (
    <div ref={overlayRef} className="fixed inset-0 z-[9999]" data-testid="onboarding-tour-overlay">
      <svg className="absolute inset-0 w-full h-full" style={{ pointerEvents: "none" }}>
        <defs>
          <mask id="tour-mask">
            <rect x="0" y="0" width="100%" height="100%" fill="white" />
            {targetRect && (
              <rect
                x={targetRect.left - 4}
                y={targetRect.top - 4}
                width={targetRect.width + 8}
                height={targetRect.height + 8}
                rx="6"
                fill="black"
              />
            )}
          </mask>
        </defs>
        <rect
          x="0"
          y="0"
          width="100%"
          height="100%"
          fill="rgba(0,0,0,0.6)"
          mask="url(#tour-mask)"
          style={{ pointerEvents: "auto" }}
          onClick={(e) => e.stopPropagation()}
        />
      </svg>

      {targetRect && (
        <div
          className="absolute rounded-md pointer-events-none"
          style={{
            top: targetRect.top - 4,
            left: targetRect.left - 4,
            width: targetRect.width + 8,
            height: targetRect.height + 8,
            boxShadow: "0 0 0 2px hsl(var(--primary)), 0 0 12px 2px hsl(var(--primary) / 0.4)",
          }}
        />
      )}

      <div
        className="absolute z-[10000]"
        style={tooltipStyle}
      >
        <Card className="border-primary/30 shadow-lg">
          <CardContent className="p-4 space-y-3">
            <div className="flex items-start justify-between gap-2">
              <div className="flex items-center gap-2">
                <div className="w-7 h-7 rounded-md bg-primary/10 flex items-center justify-center shrink-0">
                  <StepIcon className="w-3.5 h-3.5 text-primary" />
                </div>
                <h3 className="text-sm font-semibold">{step.title}</h3>
              </div>
              <Button
                size="icon"
                variant="ghost"
                className="h-6 w-6 shrink-0"
                onClick={() => endTour(false)}
                data-testid="button-tour-close"
              >
                <X className="w-3 h-3" />
              </Button>
            </div>
            <p className="text-xs leading-relaxed text-muted-foreground">
              {step.description}
            </p>
            <div className="flex items-center gap-2 pt-0.5">
              <Checkbox
                id="tour-dont-show"
                checked={dontShowAgain}
                onCheckedChange={(checked) => setDontShowAgain(checked === true)}
                data-testid="checkbox-dont-show-tour"
              />
              <label htmlFor="tour-dont-show" className="text-[10px] text-muted-foreground cursor-pointer select-none">
                Don't show this again
              </label>
            </div>
            <div className="flex items-center justify-between gap-2 pt-1">
              <span className="text-[10px] text-muted-foreground">
                {currentStep + 1} of {tourSteps.length}
              </span>
              <div className="flex items-center gap-1.5">
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() => endTour(false)}
                  data-testid="button-tour-skip"
                >
                  <span className="text-xs">Skip</span>
                </Button>
                {!isFirst && (
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={prevStep}
                    data-testid="button-tour-prev"
                  >
                    <ChevronLeft className="w-3 h-3" />
                  </Button>
                )}
                <Button
                  size="sm"
                  onClick={nextStep}
                  data-testid="button-tour-next"
                >
                  <span className="text-xs">{isLast ? "Finish" : "Next"}</span>
                  {!isLast && <ChevronRight className="w-3 h-3" />}
                </Button>
              </div>
            </div>
            <div className="flex gap-1 justify-center">
              {tourSteps.map((_, i) => (
                <div
                  key={i}
                  className={`w-1.5 h-1.5 rounded-full transition-colors ${
                    i === currentStep ? "bg-primary" : i < currentStep ? "bg-primary/40" : "bg-muted-foreground/20"
                  }`}
                />
              ))}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

export function resetTour() {
  localStorage.removeItem(TOUR_STORAGE_KEY);
  localStorage.removeItem(TOUR_DISMISSED_KEY);
}
