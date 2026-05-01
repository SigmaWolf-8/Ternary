// ────────────────────────────────────────────────────────────────────
//  Rep-C bijective base-3 encoder for non-negative BigInt values.
//
//  Digit set {1, 2, 3} per the Salvi Spec v3.3.33 §3.2 / Appendix A
//  bijective base-3 (Rep-C) encoding.  This is the canonical
//  trit-native string form used everywhere wire-side fields need to
//  surface integers without leaking decimal or hex.
//
//  Used by:
//    • server/index.ts       — EAC issuer (POST /api/hmodal/issue-eac)
//    • server/routes/salvi.ts — GET /api/salvi/timing/atto-stamp
// ────────────────────────────────────────────────────────────────────

export function toBijectiveBase3(n: bigint): string {
  if (n < 0n) throw new Error("toBijectiveBase3: negative");
  if (n === 0n) return "";
  const digits: string[] = [];
  let v = n;
  while (v > 0n) {
    let r = v % 3n;
    v = v / 3n;
    if (r === 0n) {
      r = 3n;
      v -= 1n;
    }
    digits.push(r.toString());
  }
  return digits.reverse().join("");
}

export function hexToBijectiveBase3(hex: string): string {
  const clean = (hex || "").replace(/^0x/i, "");
  if (clean.length === 0) return "";
  return toBijectiveBase3(BigInt("0x" + clean));
}
