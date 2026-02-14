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

import { useEffect } from "react";
import { useLocation } from "wouter";

const pageTitles: Record<string, string> = {
  "/": "PlenumNET — Post-Quantum Internet Infrastructure",
  "/about": "About — PlenumNET",
  "/contact": "Contact — PlenumNET",
  "/whitepaper": "Whitepaper — PlenumNET",
  "/terms": "Terms of Service — PlenumNET",
  "/privacy": "Privacy Policy — PlenumNET",
  "/security": "Security Policy — PlenumNET",
  "/aup": "Acceptable Use Policy — PlenumNET",
  "/ternarydb": "PlenumDB Console — PlenumNET",
  "/api-demo": "API Explorer — PlenumNET",
  "/hptp": "HPTP Timing Lab — PlenumNET",
  "/compression": "Compression Studio — PlenumNET",
  "/calendar": "Universal Calendar API — PlenumNET",
  "/13-moon": "13-Moon Harmonic Calendar — PlenumNET",
  "/docs": "Documentation — PlenumNET",
  "/compliance": "CNSA 2.0 Compliance — PlenumNET",
  "/admin": "Admin Dashboard — PlenumNET",
  "/github": "GitHub Manager — PlenumNET",
  "/kong-konnect": "Kong Konnect — PlenumNET",
  "/tribonacci-28ds": "Tribonacci 28-Dimension Symmetry — PlenumNET",
  "/distribution": "Salvi Framework Distribution — PlenumNET",
  "/isa-security": "ISA Security Primitives — PlenumNET",
};

export function usePageTitle() {
  const [location] = useLocation();

  useEffect(() => {
    document.title = pageTitles[location] || "PlenumNET — Post-Quantum Internet Infrastructure";
  }, [location]);
}
