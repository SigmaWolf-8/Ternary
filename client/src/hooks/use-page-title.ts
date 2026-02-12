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
};

export function usePageTitle() {
  const [location] = useLocation();

  useEffect(() => {
    document.title = pageTitles[location] || "PlenumNET — Post-Quantum Internet Infrastructure";
  }, [location]);
}
