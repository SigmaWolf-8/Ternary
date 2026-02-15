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

import { useState, useEffect } from "react";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Shield } from "lucide-react";

const CONSENT_KEY = "cookie_consent";

interface ConsentPreferences {
  essential: boolean;
  functional: boolean;
  analytics: boolean;
  consentedAt: string;
}

export function CookieConsent() {
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    const stored = localStorage.getItem(CONSENT_KEY);
    if (!stored) {
      const timer = setTimeout(() => setVisible(true), 1000);
      return () => clearTimeout(timer);
    }
  }, []);

  const saveConsent = (preferences: Omit<ConsentPreferences, "essential" | "consentedAt">) => {
    const consent: ConsentPreferences = {
      essential: true,
      ...preferences,
      consentedAt: new Date().toISOString(),
    };
    localStorage.setItem(CONSENT_KEY, JSON.stringify(consent));
    setVisible(false);
  };

  if (!visible) return null;

  return (
    <div
      className="fixed bottom-0 left-0 right-0 z-50 p-4"
      data-testid="cookie-consent-banner"
    >
      <Card className="mx-auto max-w-2xl border-border/50 shadow-lg">
        <CardContent className="flex flex-col gap-3 p-4 sm:flex-row sm:items-center sm:gap-4">
          <div className="flex items-start gap-3 flex-1 min-w-0">
            <Shield className="h-5 w-5 mt-0.5 shrink-0 text-muted-foreground" />
            <p className="text-sm text-muted-foreground">
              We use essential cookies for authentication and security. Functional cookies
              remember your preferences. See our{" "}
              <a
                href="/privacy"
                className="underline underline-offset-2 text-foreground"
                data-testid="link-cookie-privacy"
              >
                Privacy Policy
              </a>{" "}
              for details.
            </p>
          </div>
          <div className="flex items-center gap-2 shrink-0 flex-wrap">
            <Button
              variant="outline"
              size="sm"
              onClick={() => saveConsent({ functional: false, analytics: false })}
              data-testid="button-cookie-reject"
            >
              Essential Only
            </Button>
            <Button
              size="sm"
              onClick={() => saveConsent({ functional: true, analytics: true })}
              data-testid="button-cookie-accept"
            >
              Accept All
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
