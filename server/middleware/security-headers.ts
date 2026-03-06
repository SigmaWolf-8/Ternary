/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL - All Rights Reserved.
 * Patent(s) Pending.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * Unauthorized copying, modification, distribution, or use of this file,
 * via any medium, is strictly prohibited without the prior written
 * permission of Capomastro Holdings Ltd.
 *
 * See LICENSE in the repository root for full terms.
 */

import helmet from "helmet";

const isDev = process.env.NODE_ENV !== "production";

export const securityHeaders = helmet({
  contentSecurityPolicy: isDev ? false : {
    directives: {
      defaultSrc: ["'self'"],
      scriptSrc: ["'self'", "'unsafe-inline'", "'unsafe-eval'"],
      styleSrc: ["'self'", "'unsafe-inline'", "https://fonts.googleapis.com"],
      fontSrc: ["'self'", "https://fonts.gstatic.com"],
      imgSrc: ["'self'", "data:", "blob:", "https:"],
      connectSrc: ["'self'", "https://api.github.com", "https://*.konghq.com", "https://*.replit.app", "https://*.replit.dev", "wss:"],
      frameSrc: ["'self'"],
      frameAncestors: ["'self'", "https://*.replit.dev", "https://*.replit.app", "https://*.repl.co", "https://*.picard.replit.dev"],
      objectSrc: ["'none'"],
      baseUri: ["'self'"],
      formAction: ["'self'"],
      upgradeInsecureRequests: [],
    },
  },
  crossOriginEmbedderPolicy: false,
  crossOriginOpenerPolicy: isDev ? false : { policy: "same-origin" },
  crossOriginResourcePolicy: { policy: "cross-origin" },
  xContentTypeOptions: true,
  xFrameOptions: isDev ? false : { action: "sameorigin" },
  xXssProtection: true,
  hsts: isDev ? false : {
    maxAge: 31536000,
    includeSubDomains: true,
    preload: true,
  },
  referrerPolicy: { policy: "strict-origin-when-cross-origin" },
});

export function additionalSecurityHeaders(_req: any, res: any, next: any) {
  res.setHeader("Permissions-Policy", "camera=(), microphone=(), geolocation=(), interest-cohort=()");
  res.setHeader("NEL", JSON.stringify({
    report_to: "default",
    max_age: 86400,
    include_subdomains: true,
  }));
  res.setHeader("Report-To", JSON.stringify({
    group: "default",
    max_age: 86400,
    endpoints: [{ url: "/api/csp-reports" }],
  }));
  next();
}
