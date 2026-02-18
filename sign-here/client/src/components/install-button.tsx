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
import { useEffect, useState, useCallback } from "react";
import { Button } from "@/components/ui/button";
import { Download } from "lucide-react";
import { MobileInstallModal } from "@/components/mobile-install-modal";

interface BeforeInstallPromptEvent extends Event {
  prompt: () => Promise<void>;
  userChoice: Promise<{ outcome: "accepted" | "dismissed" }>;
}

export function InstallButton() {
  const [deferredPrompt, setDeferredPrompt] = useState<BeforeInstallPromptEvent | null>(null);
  const [showAndroid, setShowAndroid] = useState(false);
  const [showIOS, setShowIOS] = useState(false);
  const [iosModalOpen, setIosModalOpen] = useState(false);

  const isStandalone = typeof window !== "undefined" &&
    (window.matchMedia("(display-mode: standalone)").matches ||
     (navigator as any).standalone === true);

  const isIOS = typeof navigator !== "undefined" &&
    /iPad|iPhone|iPod/.test(navigator.userAgent) &&
    !(window as any).MSStream;

  useEffect(() => {
    if (isStandalone) return;

    const handler = (e: Event) => {
      e.preventDefault();
      setDeferredPrompt(e as BeforeInstallPromptEvent);
      setShowAndroid(true);
    };

    window.addEventListener("beforeinstallprompt", handler);

    if (isIOS && window.innerWidth <= 1024) {
      setShowIOS(true);
    }

    return () => {
      window.removeEventListener("beforeinstallprompt", handler);
    };
  }, [isStandalone, isIOS]);

  const handleInstall = useCallback(async () => {
    if (typeof navigator !== "undefined" && navigator.vibrate) {
      navigator.vibrate(12);
    }

    if (deferredPrompt) {
      deferredPrompt.prompt();
      const { outcome } = await deferredPrompt.userChoice;
      if (outcome === "accepted") {
        setShowAndroid(false);
      }
      setDeferredPrompt(null);
      return;
    }

    if (isIOS) {
      setIosModalOpen(true);
    }
  }, [deferredPrompt, isIOS]);

  if (isStandalone) return null;
  if (!showAndroid && !showIOS) return null;

  return (
    <>
      <Button
        size="icon"
        variant="default"
        onClick={handleInstall}
        className="fixed bottom-6 right-6 z-50 rounded-full w-14 h-14 shadow-2xl"
        style={{
          background: "linear-gradient(135deg, #d97706, #f59e0b)",
          boxShadow: "0 8px 32px rgba(217, 119, 6, 0.4)",
        }}
        aria-label="Install Sign Here"
        data-testid="button-install-pwa"
      >
        <Download className="h-6 w-6 text-black" />
      </Button>

      <MobileInstallModal open={iosModalOpen} onOpenChange={setIosModalOpen} />
    </>
  );
}
