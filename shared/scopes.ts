/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * Unified Scope Registry — single source of truth for all API key scopes.
 * Both backend validation and frontend UI derive from this file.
 */

export interface ScopeEntry {
  id: string;
  label: string;
}

export interface ScopeCategory {
  id: string;
  label: string;
  scopes: ScopeEntry[];
}

export const SCOPE_REGISTRY: ScopeCategory[] = [
  {
    id: "ternary-core",
    label: "Ternary Core",
    scopes: [
      { id: "read:ternary", label: "Read Ternary" },
      { id: "write:ternary", label: "Write Ternary" },
      { id: "read:tribonacci", label: "Read Tribonacci" },
      { id: "write:tribonacci", label: "Write Tribonacci" },
    ],
  },
  {
    id: "cryptography",
    label: "Cryptography",
    scopes: [
      { id: "read:phase", label: "Read Phase Encryption" },
      { id: "write:phase", label: "Write Phase Encryption" },
      { id: "read:tl-dsa", label: "Read TL-DSA" },
      { id: "write:tl-dsa", label: "Write TL-DSA" },
      { id: "read:tl-kem", label: "Read TL-KEM" },
      { id: "write:tl-kem", label: "Write TL-KEM" },
      { id: "read:t-ae-mac", label: "Read T-AE-MAC" },
      { id: "write:t-ae-mac", label: "Write T-AE-MAC" },
      { id: "read:kernel-sponge", label: "Read Kernel Sponge" },
      { id: "read:tis-27", label: "Read TIS-27" },
      { id: "read:tis-81", label: "Read TIS-81" },
    ],
  },
  {
    id: "networking",
    label: "Networking",
    scopes: [
      { id: "read:tdns", label: "Read TDNS" },
      { id: "write:tdns", label: "Write TDNS" },
      { id: "read:hptp", label: "Read HPTP" },
      { id: "write:hptp", label: "Write HPTP" },
      { id: "read:pqti", label: "Read PQTI" },
      { id: "write:pqti", label: "Write PQTI" },
      { id: "read:tonal-field", label: "Read Tonal Field" },
      { id: "write:tonal-field", label: "Write Tonal Field" },
    ],
  },
  {
    id: "infrastructure",
    label: "Infrastructure",
    scopes: [
      { id: "read:inter-cube", label: "Read Inter-Cube" },
      { id: "write:inter-cube", label: "Write Inter-Cube" },
      { id: "read:kong", label: "Read Kong Konnect" },
      { id: "write:kong", label: "Write Kong Konnect" },
      { id: "read:sfk", label: "Read SFK Operations" },
      { id: "write:sfk", label: "Write SFK Operations" },
      { id: "read:capabilities", label: "Read Capabilities" },
      { id: "write:capabilities", label: "Write Capabilities" },
    ],
  },
  {
    id: "data-storage",
    label: "Data & Storage",
    scopes: [
      { id: "read:plenumdb", label: "Read PlenumDB" },
      { id: "write:plenumdb", label: "Write PlenumDB" },
      { id: "read:compression", label: "Read Compression" },
      { id: "write:compression", label: "Write Compression" },
      { id: "read:data-subject", label: "Read Data Subject Rights" },
      { id: "write:data-subject", label: "Write Data Subject Rights" },
    ],
  },
  {
    id: "timing-science",
    label: "Timing & Science",
    scopes: [
      { id: "read:calendar", label: "Read Calendar" },
      { id: "read:tsa", label: "Read TSA" },
      { id: "write:tsa", label: "Write TSA" },
      { id: "read:ephemeris", label: "Read Ephemeris" },
    ],
  },
  {
    id: "compute",
    label: "Compute",
    scopes: [
      { id: "read:agent-array", label: "Read Agent Array" },
      { id: "write:agent-array", label: "Write Agent Array" },
      { id: "read:quantum-sim", label: "Read Quantum Simulator" },
      { id: "write:quantum-sim", label: "Write Quantum Simulator" },
      { id: "read:xplenum", label: "Read XPlenum RISC-V" },
    ],
  },
  {
    id: "content",
    label: "Content",
    scopes: [
      { id: "read:whitepaper", label: "Read Whitepaper" },
      { id: "write:whitepaper", label: "Write Whitepaper" },
      { id: "read:signhere", label: "Read SignHere" },
      { id: "write:signhere", label: "Write SignHere" },
    ],
  },
  {
    id: "admin",
    label: "Admin",
    scopes: [
      { id: "admin:keys", label: "Manage API Keys" },
      { id: "admin:security", label: "Security Admin" },
    ],
  },
];

export function getAllScopes(): string[] {
  return SCOPE_REGISTRY.flatMap((cat) => cat.scopes.map((s) => s.id));
}

export function getScopesByCategory(): Record<string, string[]> {
  const result: Record<string, string[]> = {};
  for (const cat of SCOPE_REGISTRY) {
    result[cat.label] = cat.scopes.map((s) => s.id);
  }
  return result;
}

export function isValidScope(scope: string): boolean {
  return getAllScopes().includes(scope);
}
