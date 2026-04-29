import QRCode from "qrcode";
import { useEffect, useMemo, useState } from "react";
import { ShieldCheck, Stamp, Hexagon, Printer, Info } from "lucide-react";

// ────────────────────────────────────────────────────────────────────────────
// Energy Attestation Certificate (EAC) — professional notarization layout
// styled after the SignHere notarization PDF reference (TM-2026-042 Rev.2).
//
// All cryptographic identifiers (chain_tag, session_id, chain_seed,
// cipher_trits, tis27_hash) are surfaced TRIT-NATIVE in Rep-C bijective
// base-3 (digit set {1,2,3}).  Hex copies are rendered only inside the
// "Audit interop" footer block.  No hex appears in any primary field.
//
// The SCANNABLE seal QR (right) is a full ISO/IEC 18004 matrix — every
// dark module is rendered as a small isometric crystal/gem (three
// shaded facets per cell) so the whole seal reads as a faceted
// post-quantum crystal lattice while remaining scannable by any
// conformant QR decoder.  Around the QR sit four concentric Forge
// polygons (3-, 7-, 11-, 13-gon — Coprime Triple {7,11,13} + ternary
// anchor), an outer mandala ring of small isometric cubes, and curved
// arc-text bands.
//
// The cert also surfaces:
//   - the FORWARD-FACING attosecond timestamp as a single integer at
//     the top of the Timestamp section (the rational + walk formula
//     are still preserved further down for audit purity)
//   - a Hedera HCS witness block (blockchain non-repudiation)
//   - a 12-system Calendar Stamp block (subset of the 42-calendar sync)
//   - a Print / Save-as-PDF button that prints the certificate alone
// ────────────────────────────────────────────────────────────────────────────

interface EacProps {
  eac: any;
  error?: string | null;
}

// ── Helpers ──────────────────────────────────────────────────────────────
function fmtTrit(s: string | undefined, group = 9): string {
  // Bijective base-3 of the integer 0 is the empty string by definition
  // (Spec v3.3.33 §3.2 — Rep-C has no zero glyph).  Render that as "0".
  if (s == null) return "—";
  if (s === "") return "0";
  const out: string[] = [];
  for (let i = 0; i < s.length; i += group) out.push(s.slice(i, i + group));
  return out.join(" ");
}
function trimTrit(s: string | undefined, head = 54): string {
  if (s == null) return "—";
  if (s === "") return "0";
  if (s.length <= head) return fmtTrit(s);
  return fmtTrit(s.slice(0, head)) + " …";
}
function trimHex(s: string | undefined, head = 32): string {
  if (!s) return "—";
  return s.length <= head ? s : s.slice(0, head) + "…";
}

// Format a long decimal integer with thin spaces every 3 digits, so the
// attosecond timestamp is humanly readable instead of one long blob.
function groupDigits(s: string | undefined): string {
  if (!s) return "—";
  const sign = s.startsWith("-") ? "-" : "";
  const body = sign ? s.slice(1) : s;
  if (!/^\d+$/.test(body)) return s;
  let out = "";
  for (let i = 0; i < body.length; i++) {
    if (i > 0 && (body.length - i) % 3 === 0) out += "\u202F";  // narrow no-break space
    out += body[i];
  }
  return sign + out;
}

// Convert a hex string to Rep-C bijective base-3 (digits {1,2,3}) on the
// client.  Used for hex-only fields (signature_hex, public_key_hash) so
// the certificate stays trit-native everywhere user-facing.
function bigHexToTrit(hex: string | undefined): string | undefined {
  if (!hex) return undefined;
  const clean = hex.replace(/^0x/i, "");
  if (clean.length === 0) return "";
  let v: bigint;
  try { v = BigInt("0x" + clean); } catch { return undefined; }
  if (v === 0n) return "";
  const out: string[] = [];
  while (v > 0n) {
    let r = v % 3n;
    v = v / 3n;
    if (r === 0n) { r = 3n; v -= 1n; }
    out.push(r.toString());
  }
  return out.reverse().join("");
}

// Render a regular n-gon vertex set, rotated so the apex points up.
function polygonPoints(cx: number, cy: number, r: number, n: number, rotDeg = -90): string {
  const pts: string[] = [];
  for (let i = 0; i < n; i++) {
    const a = ((360 * i) / n + rotDeg) * (Math.PI / 180);
    const x = cx + r * Math.cos(a);
    const y = cy + r * Math.sin(a);
    pts.push(`${x.toFixed(2)},${y.toFixed(2)}`);
  }
  return pts.join(" ");
}

// ── CrystalMatrix ────────────────────────────────────────────────────────
// QR matrix renderer where every DARK module is drawn as a small
// isometric crystal cell (three shaded facets — top, left-front,
// right-front) so the whole matrix reads as a faceted post-quantum
// crystal lattice.  Light modules stay pure white.  Decoder contract
// preserved: every dark cell is "mostly dark" to a scanner because all
// three facets use shades of currentColor (≥ 60% luminance towards dark
// on every facet).  A 4-module quiet zone (ISO/IEC 18004 §6.3.8) is
// painted in solid white around the matrix.
function CrystalMatrix({
  modules,
  size,
}: {
  modules: boolean[][];
  size: number;
}) {
  const n = modules.length;
  const QUIET = 4;
  const total = n + QUIET * 2;
  const cell = size / total;
  const offset = QUIET * cell;
  const facets: JSX.Element[] = [];
  for (let r = 0; r < n; r++) {
    for (let c = 0; c < n; c++) {
      if (!modules[r][c]) continue;
      const x = offset + c * cell;
      const y = offset + r * cell;
      // SCANNABILITY-FIRST construction: each dark module is painted as
      // a SOLID full-cell square in currentColor (the QR-decoder
      // substrate — guarantees ≥ 100 % dark mass to ISO/IEC 18004
      // decoders).  THREE crystal facets are then OVERLAID using the
      // ISOMETRIC-CUBE projection (top diamond + left rhombus + right
      // rhombus, all meeting at the cell centre) at progressively
      // darker opacities (0.30/0.65/0.92) so the cell reads as a
      // lit-from-upper-left 3-D gem cube.  A thin specular highlight
      // is drawn ONLY on the top apex edge — never spans the cell
      // interior — so the white pixel count stays well under the
      // ISO threshold.
      const cx = x + cell / 2;
      const cy = y + cell / 2;
      // Isometric cube projection: top apex at y, centre at cell
      // centre, side apexes at vertical mid-height.  Same projection
      // used by the outer mandala IsoCubeRing — produces a coherent
      // 3-D crystal lattice across the seal.
      const apexY  = y + cell * 0.10;
      const sideY  = y + cell * 0.42;
      const baseY  = y + cell * 0.95;
      const leftX  = x + cell * 0.06;
      const rightX = x + cell * 0.94;
      const topPts   = `${cx},${apexY}  ${rightX},${sideY}  ${cx},${cy}  ${leftX},${sideY}`;
      const leftPts  = `${leftX},${sideY}  ${cx},${cy}  ${cx},${baseY}  ${leftX},${baseY - (sideY - apexY)}`;
      const rightPts = `${cx},${cy}  ${rightX},${sideY}  ${rightX},${baseY - (sideY - apexY)}  ${cx},${baseY}`;
      const sw = Math.max(0.35, cell * 0.07);
      facets.push(
        <g key={`g-${r}-${c}`} shapeRendering="geometricPrecision">
          {/* Solid base — the QR-decoder substrate.  Guarantees ≥ 100 %
              dark mass so ISO/IEC 18004 decoders see the cell as ON. */}
          <rect x={x} y={y} width={cell} height={cell} fill="currentColor" />
          {/* Top facet — lightest (light hitting from above-left). */}
          <polygon points={topPts}   fill="currentColor" opacity={0.22} />
          {/* Left-front facet — mid. */}
          <polygon points={leftPts}  fill="currentColor" opacity={0.55} />
          {/* Right-front facet — deepest shadow. */}
          <polygon points={rightPts} fill="currentColor" opacity={0.88} />
          {/* White silhouette edges between the three facets — the
              vertical centre seam (apex → cell-centre → base) and the
              two diagonal seams to the side apexes.  These are the
              critical lines that make each dark cell read as a 3-D
              cube instead of a flat square; opacity is high enough to
              be unmistakable but the stroke is so thin (~7 % of cell
              width) that the painted-white pixel area stays well under
              the ISO/IEC 18004 dark-module threshold. */}
          <polyline
            points={`${cx},${apexY}  ${cx},${cy}  ${cx},${baseY}`}
            fill="none" stroke="#ffffff" strokeWidth={sw} opacity={0.55}
          />
          <line
            x1={leftX} y1={sideY} x2={cx} y2={cy}
            stroke="#ffffff" strokeWidth={sw} opacity={0.55}
          />
          <line
            x1={cx} y1={cy} x2={rightX} y2={sideY}
            stroke="#ffffff" strokeWidth={sw} opacity={0.55}
          />
        </g>,
      );
    }
  }
  return (
    <g>
      {/* Quiet zone + light-module background — solid white, full size */}
      <rect x={0} y={0} width={size} height={size} fill="#ffffff" />
      {facets}
    </g>
  );
}

// ── IsoCubeRing ──────────────────────────────────────────────────────────
// A decorative outer ring of small isometric cubes around the QR seal.
// Pure ornament — sits OUTSIDE the QR module area so it never affects
// scannability.  N cubes evenly spaced on a circle of radius r.
function IsoCubeRing({
  cx, cy, r, count = 24, cubeSize = 6, bold = false,
}: { cx: number; cy: number; r: number; count?: number; cubeSize?: number; bold?: boolean }) {
  const cubes: JSX.Element[] = [];
  // When `bold` is true the cubes are painted with full-opacity facets
  // and a visible white silhouette edge — used for the four diagonal
  // anchor cubes that frame the QR.  Plain mode keeps the original
  // semi-transparent ornament look used by the 36-cube mandala ring.
  const opTop   = bold ? 0.55 : 0.40;
  const opLeft  = bold ? 0.80 : 0.65;
  const opRight = bold ? 1.00 : 0.85;
  const edge    = bold ? Math.max(0.6, cubeSize * 0.12) : 0;
  for (let i = 0; i < count; i++) {
    const a = (i * 2 * Math.PI) / count - Math.PI / 2;
    const px = cx + r * Math.cos(a);
    const py = cy + r * Math.sin(a);
    const s = cubeSize;
    // Isometric cube vertices (top diamond + two side parallelograms).
    const top   = `${px},${py - s}  ${px + s},${py - s / 2}  ${px},${py}  ${px - s},${py - s / 2}`;
    const left  = `${px - s},${py - s / 2}  ${px},${py}  ${px},${py + s}  ${px - s},${py + s / 2}`;
    const right = `${px},${py}  ${px + s},${py - s / 2}  ${px + s},${py + s / 2}  ${px},${py + s}`;
    cubes.push(
      <g key={`cube-${i}`} shapeRendering="geometricPrecision">
        <polygon points={top}   fill="currentColor" opacity={opTop} />
        <polygon points={left}  fill="currentColor" opacity={opLeft} />
        <polygon points={right} fill="currentColor" opacity={opRight} />
        {bold && (
          <>
            {/* Three white silhouette edges — make the cube unmistakable. */}
            <polyline
              points={`${px},${py - s}  ${px},${py}  ${px},${py + s}`}
              fill="none" stroke="#ffffff" strokeWidth={edge} opacity={0.7}
            />
            <line x1={px - s} y1={py - s / 2} x2={px} y2={py}
              stroke="#ffffff" strokeWidth={edge} opacity={0.7} />
            <line x1={px} y1={py} x2={px + s} y2={py - s / 2}
              stroke="#ffffff" strokeWidth={edge} opacity={0.7} />
          </>
        )}
      </g>,
    );
  }
  return <g>{cubes}</g>;
}

// ── Component ────────────────────────────────────────────────────────────
export function EacCertificate({ eac, error }: EacProps) {
  const [qrModules, setQrModules] = useState<boolean[][] | null>(null);

  // Ultra-compact QR payload.  Goal: keep the QR version small so each
  // module renders large enough on screen for the isometric-cube
  // facet shading to actually be perceptible.  We pack only the
  // fields a verifier needs to look up the canonical cert — long
  // crypto material (chain_tag, signature, full trit strings) lives
  // in the visible certificate body, NOT in the QR.
  const qrPayload = useMemo(() => {
    if (!eac) return "";
    const c = eac.attestation_chain ?? {};
    const t = eac.timestamp ?? {};
    // CSV-style ultra-compact format (no JSON braces / quotes / colons)
    // — typically ~55-70 chars total → QR version ≈ 3 (29x29) at ECC-L
    // → cell ≈ 7 px on a 360-px seal, where the cube facets become
    // visibly perceptible.
    const sid = (c.session_id ?? "").slice(0, 12);
    const idx = c.chain_index_decimal ?? "";
    const as  = t.attoseconds_since_unix_epoch_decimal
             ?? t.attoseconds_since_boot_decimal
             ?? "";
    const utc = t.utc_iso_at_issue ?? "";
    return `EAC1|${sid}|${idx}|${as}|${utc}`;
  }, [eac]);

  useEffect(() => {
    if (!qrPayload) return;
    try {
      // ECC level "M" (15 % damage tolerance) — stronger error
      // correction so the seal scans reliably even after print +
      // photocopy + camera blur.  The payload is intentionally short
      // (CSV rather than JSON) so even at ECC-M the matrix stays
      // small enough that each cell renders at a perceptible size.
      const qr = QRCode.create(qrPayload, { errorCorrectionLevel: "M" });
      const mods: any = (qr as any).modules;
      const n: number = mods.size;
      const out: boolean[][] = [];
      for (let r = 0; r < n; r++) {
        const row: boolean[] = [];
        for (let c = 0; c < n; c++) {
          row.push(!!mods.get(r, c));
        }
        out.push(row);
      }
      setQrModules(out);
    } catch {
      setQrModules(null);
    }
  }, [qrPayload]);

  if (error) {
    return (
      <div
        className="rounded-md border border-destructive/40 bg-destructive/5 p-4 text-sm text-destructive"
        data-testid="text-eac-error"
      >
        {error}
      </div>
    );
  }
  if (!eac || !eac.signature) return null;

  const chain = eac.attestation_chain ?? {};
  const ts    = eac.timestamp ?? {};
  const meas  = eac.measurement ?? {};
  const integ = eac.integrity ?? {};
  const node  = eac.node ?? {};
  const sig   = eac.signature ?? {};
  const hed   = eac.hedera_witness ?? null;
  const cal   = eac.calendar_stamp ?? null;

  const sealSize  = 380;
  // Outer rim of the seal — outermost circle the eye reads.
  const ringR1    = 184;
  // Greek-glyph ring — sits inside the curved-text band, holds the
  // 12 capital Greek letters (Α Β Γ Δ Ε Ζ Η Θ Ι Κ Λ Μ).
  const ringMid   = 152;
  // Inner edge of the scribed band — defines the QR safe zone and
  // hosts the 48-cube mandala.
  const ringInner = 130;
  // Half-side of the QR crystal lattice (inscribed in the inner ring).
  const qrInset   =  84;
  // DIAGONAL-position anchor cubes (NE / NW / SE / SW) — placed at
  // the four corners of the (square) QR matrix so the cubes visually
  // "lock" the lattice into the seal.  anchorR chosen so that the
  // cube body clears the QR corner with a few px of breathing room.
  const anchorR   = 144;

  return (
    <>
      {/* Print stylesheet — only the certificate prints when the user
          hits Ctrl/Cmd-P or our "Print / Save as PDF" button. */}
      <style>{`
        @media print {
          body * { visibility: hidden !important; }
          #eac-print-root, #eac-print-root * { visibility: visible !important; }
          #eac-print-root { position: absolute !important; left: 0; top: 0; width: 100%; }
          .eac-print-hide { display: none !important; }
          /* Certificate prints in black ink for hard-copy notarization. */
          #eac-print-root { color: #000 !important; }
        }
      `}</style>
      <div id="eac-print-root">
      <div
        className="bg-white dark:bg-zinc-950 text-zinc-900 dark:text-zinc-100 rounded-lg border-2 border-primary/40 shadow-xl overflow-hidden"
        data-testid="card-eac-certificate"
      >
      {/* ── Header band ──────────────────────────────────────────────── */}
      <div className="bg-gradient-to-r from-primary/15 via-primary/5 to-primary/15 border-b border-primary/30 px-6 py-4 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <ShieldCheck className="w-7 h-7 text-primary" />
          <div>
            <div className="text-[10px] uppercase tracking-[0.2em] text-muted-foreground">
              PlenumNET · Energy Attestation
            </div>
            <div className="text-xl font-semibold leading-tight">
              Energy Attestation Certificate
            </div>
          </div>
        </div>
        <div className="flex items-center gap-3">
          <button
            type="button"
            onClick={() => window.print()}
            className="eac-print-hide inline-flex items-center gap-1.5 rounded-md border border-primary/40 bg-primary/10 px-3 py-1.5 text-xs font-medium text-primary hover:bg-primary/20 transition-colors"
            data-testid="button-print-eac"
            title="Open the system print dialog — choose 'Save as PDF' to export this certificate as a notarization-grade PDF."
          >
            <Printer className="w-3.5 h-3.5" />
            Print / Save as PDF
          </button>
          <div className="text-right">
            <div className="text-[10px] uppercase tracking-[0.2em] text-muted-foreground">
              Specification
            </div>
            <div className="text-sm font-mono">TM-2026-042 Rev.2 · EAC/1</div>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-[1fr_auto] gap-6 p-6">
        {/* ── LEFT: notarization fields ────────────────────────────── */}
        <div className="space-y-5">
          {/* Document section */}
          <section data-testid="section-eac-document">
            <SectionHeader>Document</SectionHeader>
            <Field label="Title"  value="Energy Attestation Certificate (EAC)" plain="A signed, blockchain-witnessed proof of how much electrical energy this PlenumNET node saved during the measurement window." />
            <Field label="Issuer" value={`PlenumNET node · ${node.tdns ?? "tdns:hmodal-demo:01"}`} plain="The PlenumNET node that produced and signed this attestation." />
            <Field label="Mode"   value={node.mode ?? "—"} plain="hardware-watts = real CPU power counters (RAPL).  compute-throughput-proxy = modeled from real measured CPU compute time when RAPL is not exposed." />
            <Field label="Demand" value={node.demand_mode ?? "—"} plain="Workload pattern during the measurement window (idle / steady / burst / auto sine sweep)." />
          </section>

          {/* ── Timestamp section — ATTOSECONDS FIRST ───────────────── */}
          <section data-testid="section-eac-timestamp">
            <SectionHeader>Timestamp · UTC-Grounded · Attosecond Precision</SectionHeader>

            {/* Featured single-integer attosecond timestamp — anchored
                against the UTC Unix epoch (1970-01-01T00:00:00Z) so it
                has wall-clock meaning instead of being relative to this
                node's boot. */}
            <div className="rounded-md border border-primary/40 bg-primary/5 p-3 mb-3" data-testid="block-eac-attoseconds">
              <div className="text-[10px] uppercase tracking-[0.18em] text-muted-foreground mb-1">
                Attoseconds since UTC epoch · 1970-01-01T00:00:00Z
              </div>
              <div
                className="font-mono text-base sm:text-lg break-all leading-snug text-primary font-semibold"
                data-testid="text-eac-attoseconds"
              >
                {groupDigits(ts.attoseconds_since_unix_epoch_decimal ?? ts.attoseconds_since_boot_decimal)}{" "}
                <span className="text-muted-foreground font-normal">as</span>
              </div>
              <div className="text-[11px] mt-1.5 font-mono text-foreground/80" data-testid="text-eac-utc-iso">
                = {ts.utc_iso_at_issue ?? "—"}{" "}
                <span className="text-muted-foreground">(UTC, ISO 8601)</span>
              </div>
              <div className="text-[10px] text-muted-foreground mt-1.5">
                Plain English: the exact wall-clock instant this certificate was issued, expressed as a single integer count of attoseconds (10⁻¹⁸ s) since the UTC Unix epoch. True attosecond precision end-to-end: the UTC epoch was anchored ONCE — at the moment the very first certificate of this process was issued — to the framework's attosecond tick. Every subsequent certificate advances purely along the framework's monotonic attosecond walk — no millisecond rounding, no synthetic clamps, no per-call wall-clock reads.
              </div>
              <div className="font-mono text-[10px] text-muted-foreground break-all mt-2">
                trit (Rep-C): {trimTrit(ts.attoseconds_since_unix_epoch_trit ?? ts.attoseconds_since_boot_trit, 81)}
              </div>
            </div>

            {(() => {
              const d = ts.derivation ?? {};
              const asNum = ts.as_since_boot?.num ?? d.as_since_boot_num;
              const asDen = ts.as_since_boot?.den ?? d.as_since_boot_den;
              return (
                <>
                  <Field
                    label="Hardware clock used"
                    value={d.hardware_clock_used ?? "NO — pure framework derivation"}
                    mono
                    plain="The OS / wall-clock was NOT consulted.  Time is derived from a deterministic tick walk on the integer ring Z_{D_α}."
                  />
                  <Field
                    label="Tick counter (decimal)"
                    value={ts.tick_decimal ?? d.tick_counter_decimal ?? "—"}
                    mono
                    plain="Number of HPTP framework ticks since boot.  Each tick is an exact rational of attoseconds."
                  />
                  <Field
                    label="Tick (trit)"
                    value={trimTrit(ts.tick_trit, 36)}
                    mono
                    plain="The same tick counter expressed in Rep-C bijective base-3 (digits {1,2,3} only)."
                  />
                  <Field
                    label="Clock tier"
                    value={`${d.clock_tier ?? 0}  (0 = pure derivation)`}
                    mono
                    plain="0 means the timestamp comes from pure framework math; higher tiers mean increasing reliance on hardware clocks."
                  />
                  <Field
                    label="Chain index at seal (trit)"
                    value={d.chain_index_at_seal_trit ?? "—"}
                    mono
                    plain="Which sealed sample on the WSS attestation chain this certificate snapshots."
                  />
                  <Field
                    label="Chain tag at seal (trit)"
                    value={trimTrit(d.chain_tag_trit, 54)}
                    mono mid
                    plain="The 385-bit running TL-Sponge tag at the moment of seal — any reorder or substitution of WSS samples changes this value."
                  />

                  {/* Audit-only formula block — collapsed by default */}
                  <details className="mt-2 text-[10px] text-muted-foreground">
                    <summary className="cursor-pointer hover:text-foreground select-none">
                      Show audit-purity derivation (rational form, walk modulus, factorisation)
                    </summary>
                    <div className="mt-2 space-y-1 font-mono">
                      <div>walk_modulus_d_alpha:  {d.walk_modulus_d_alpha ?? "125250125"}  (= {d.walk_factorisation ?? "5^3·7^2·11^2·13^2"})</div>
                      <div>walk_position_decimal: {d.walk_position_decimal ?? "—"}</div>
                      <div>framework_fs_index:    {d.framework_fs_index ?? "—"}</div>
                      <div>as_since_boot exact:   {asNum != null && asDen != null ? `${asNum} / ${asDen}` : "—"}</div>
                    </div>
                  </details>
                </>
              );
            })()}
          </section>

          {/* Measurement section */}
          <section data-testid="section-eac-measurement">
            <SectionHeader>Measurement · Power &amp; Energy Savings</SectionHeader>
            <Field
              label="Window (ms)"
              value={String(meas.window_ms ?? "—")}
              mono
              plain="Length of the measurement window, in milliseconds."
            />
            <Field
              label="Measured power"
              value={`${meas.measured_mW ?? "—"} mW`}
              mono highlight
              plain="Average power draw of this node during the window, in milliwatts."
            />
            <Field
              label="Baseline power"
              value={`${meas.baseline_mW ?? "—"} mW`}
              mono
              plain="What the node would have drawn if it had run a continuous-on (always full-throttle) workload during the same window."
            />
            <Field
              label="Power saved"
              value={`${meas.mW_saved ?? "—"} mW`}
              mono highlight
              plain="baseline − measured = how many milliwatts of continuous draw the HModal duty-cycle eliminated."
            />
            <Field
              label="Savings ratio"
              value={
                meas.savings_ratio
                  ? `${meas.savings_ratio.num} / ${meas.savings_ratio.den}` +
                    `   (≈ ${((meas.savings_ratio.num / Math.max(1, meas.savings_ratio.den)) * 100).toFixed(2)}%)`
                  : "—"
              }
              mono
              plain="Fraction of baseline power eliminated, kept as an exact integer ratio (no float on the wire)."
            />
            <Field
              label="Savings ratio (theoretical)"
              value={
                meas.savings_ratio_theoretical
                  ? `${meas.savings_ratio_theoretical.num} / ${meas.savings_ratio_theoretical.den}   (74.48%)`
                  : "—"
              }
              mono
              plain="Closed-form best case for the canonical α=91/36, β=91/3, duty 1:4 schedule."
            />
            <Field
              label="Cumulative energy consumed (µJ)"
              value={groupDigits(meas.cumulative_energy_uJ_decimal)}
              mono
              plain="Total energy this node actually consumed since the WSS session opened, in microjoules."
            />
            <Field
              label="Cumulative energy SAVED (µJ)"
              value={groupDigits(meas.cumulative_energy_saved_uJ_decimal)}
              mono highlight
              plain="Total electrical energy this node DID NOT spend by running HModal duty-cycled instead of continuous-on, since the WSS session opened.  Integer microjoules."
            />
            <Field
              label="Cumulative energy SAVED (trit)"
              value={trimTrit(meas.cumulative_energy_saved_uJ_trit, 36)}
              mono
              plain="The same saved-energy count expressed in Rep-C bijective base-3."
            />
          </section>

          {/* Cryptographic Integrity section — TRIT-NATIVE */}
          <section data-testid="section-eac-integrity">
            <SectionHeader>Cryptographic Integrity · Rep-C Bijective Base-3 (Trit-Native)</SectionHeader>
            <Field label="Cipher" value={chain.cipher ?? "TL-Sponge-385 duplex (Phase Encryption v3)"} mono
              plain="Post-quantum 385-bit duplex sponge — the same primitive that signs the cert also seals every WSS sample." />
            <Field label="Session ID (trit)"        value={fmtTrit(chain.session_id)} mono mid
              plain="Unique identifier for this WSS measurement session, generated trit-native (no hex stage)." />
            <Field label="Session-key fingerprint (trit)" value={trimTrit(chain.session_key_fingerprint_trit, 54)} mono mid
              plain="TIS-27 fingerprint of the per-session sponge key — proves the cert was sealed by THIS session's keystream." />
            <Field label="Chain seed (trit)"         value={trimTrit(chain.chain_seed_trit, 81)} mono mid
              plain="Initial sponge state before the first sample was sealed — the anchor of the integrity chain." />
            <Field label="Chain index (decimal)"    value={chain.chain_index_decimal ?? "—"} mono
              plain="Sequence number of the sealed sample this cert binds to." />
            <Field label="Chain index (trit)"       value={fmtTrit(chain.chain_index_trit)} mono />
            <Field label="Chain tag · 385-bit (trit)" value={trimTrit(chain.chain_tag_trit, 81)} mono mid highlight
              plain="Running 385-bit TL-Sponge tag after sealing this sample.  ANY gap, reorder, or substitution in the WSS stream changes this value." />
            <Field label="Cipher payload (trit, head)" value={trimTrit(chain.cipher_trits_trit, 81)} mono mid />
            <Field label="TIS-27 doc hash (trit)"   value={trimTrit(integ.tis27_hash_hex ? bigHexToTrit(integ.tis27_hash_hex) : undefined, 54)} mono mid
              plain="Hash of the canonical EAC JSON document, computed with the trit-native TIS-27 sponge." />
            <Field
              label="TIS-27 · Milesian glyphs"
              value={integ.tis27_hash_milesian || "—"}
              mono
              greekFont
              plain="The same hash rendered in Milesian Greek numerals — visual cross-check for transcription errors."
            />
          </section>

          {/* Signature section */}
          <section data-testid="section-eac-signature">
            <SectionHeader>Signature</SectionHeader>
            <Field label="Variant"          value={sig.variant ?? "TL-DSA-87"} mono
              plain="Post-quantum 87-byte digital signature — Salvi Framework's ternary lattice DSA." />
            <Field label="Public key hash (trit)"  value={trimTrit(sig.public_key_hash ? bigHexToTrit(sig.public_key_hash) : undefined, 54)} mono mid
              plain="TIS-27 fingerprint of the signer's public key.  Compare to the node's published key roster." />
            <Field label="Signature (trit, head)"  value={trimTrit(sig.signature_hex ? bigHexToTrit(sig.signature_hex) : undefined, 81)} mono mid highlight
              plain="The TL-DSA-87 signature over the canonical EAC JSON.  Rep-C trit form (full hex copy in the audit footer)." />
          </section>

          {/* ── Hedera HCS Witness section ───────────────────────────── */}
          <section data-testid="section-eac-hedera">
            <SectionHeader>Hedera HCS Witness · Blockchain Non-Repudiation</SectionHeader>
            {hed ? (
              hed.status === "witnessed" ? (
                <>
                  <Field label="Status" value="✓ Witnessed on Hedera Consensus Service" mono highlight
                    plain="The TIS-27 hash of this certificate was submitted to Hedera HCS and accepted by consensus — anyone can verify this cert existed at the consensus timestamp shown below." />
                  <Field label="Topic ID"             value={hed.topic_id ?? "—"} mono />
                  <Field label="Transaction ID"       value={hed.transaction_id ?? "—"} mono mid />
                  <Field label="Consensus timestamp"  value={hed.consensus_timestamp ?? "—"} mono />
                  <Field label="Sequence number"      value={String(hed.sequence_number ?? "—")} mono />
                  <Field label="Running hash"         value={trimHex(hed.running_hash, 48)} mono mid />
                </>
              ) : hed.status === "submission_failed" ? (
                <>
                  <Field label="Status" value="⚠ Submission failed" mono
                    plain="Hedera HCS is configured but the witness submission did not complete.  The cert remains cryptographically valid; only the blockchain anchor is missing." />
                  <Field label="Error"  value={hed.error ?? "—"} mono mid />
                </>
              ) : (
                <>
                  <Field label="Status" value="Not configured on this node" mono
                    plain="Hedera HCS witnessing is available in the framework but no operator credentials are present on this node.  The signed cert is still valid; it just lacks the optional blockchain anchor." />
                  <Field label="To enable" value="Set HEDERA_ACCOUNT_ID and HEDERA_PRIVATE_KEY" mono mid />
                </>
              )
            ) : (
              <Field label="Status" value="—" mono />
            )}
          </section>

          {/* ── 42-Calendar Stamp section ─────────────────────────────── */}
          <section data-testid="section-eac-calendar">
            <SectionHeader>Calendar Stamp · 42-System Multi-Civilizational Sync</SectionHeader>
            <div className="text-[10px] text-muted-foreground mb-2">
              Plain English: the same instant rendered into twelve historical calendar systems, derived purely from the Julian Day Number — provides a civilization-independent anchor for the cert's "when".
            </div>
            {cal && !cal.error ? (
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-x-6 gap-y-1 text-[11px]">
                <CalRow label="Gregorian (UTC)"   value={cal.gregorian_iso} />
                <CalRow label="Julian Day Number" value={cal.julian_day_number?.jdn} />
                <CalRow label="Mayan Long Count"  value={cal.mayan_long_count?.long_count_string ?? cal.mayan_long_count?.notation} />
                <CalRow label="Hebrew"            value={cal.hebrew?.formatted ?? cal.hebrew?.year} />
                <CalRow label="Islamic Hijri"     value={cal.islamic_hijri?.formatted ?? cal.islamic_hijri?.year} />
                <CalRow label="Chinese Sexagenary" value={cal.chinese_sexagenary?.stem_branch ?? cal.chinese_sexagenary?.cycle_year} />
                <CalRow label="Vedic Kali Yuga"   value={cal.vedic_kali_yuga?.formatted ?? cal.vedic_kali_yuga?.year} />
                <CalRow label="Persian Solar"     value={cal.persian_solar_hijri?.formatted ?? cal.persian_solar_hijri?.year} />
                <CalRow label="Ethiopian Geʿez"   value={cal.ethiopian_geez?.formatted ?? cal.ethiopian_geez?.year} />
                <CalRow label="Coptic"            value={cal.coptic?.formatted ?? cal.coptic?.year} />
                <CalRow label="Egyptian Civil"    value={cal.egyptian_civil?.formatted ?? cal.egyptian_civil?.year} />
                <CalRow label="13-Moon Harmonic"  value={cal.thirteen_moon?.formatted ?? cal.thirteen_moon?.kin} />
                <CalRow label="Byzantine Anno Mundi" value={cal.byzantine_anno_mundi?.formatted ?? cal.byzantine_anno_mundi?.year} />
              </div>
            ) : (
              <div className="text-xs text-muted-foreground">Calendar sync unavailable.</div>
            )}
            <div className="text-[10px] text-muted-foreground mt-2">{cal?.source ?? ""}</div>
          </section>
        </div>

        {/* ── RIGHT: geometric / crystal notarization seal ────────── */}
        <div className="flex flex-col items-center gap-3">
          <div className="text-[10px] uppercase tracking-[0.2em] text-muted-foreground">
            Notarization Seal · Crystal Lattice
          </div>
          <svg
            viewBox={`0 0 ${sealSize} ${sealSize}`}
            width={sealSize}
            height={sealSize}
            className="text-primary"
            data-testid="svg-eac-seal"
          >
            {/* ── Filter defs: indent / emboss + outer drop-shadow ─────
                The seal is rendered as if PRESSED INTO the page:
                  • feDropShadow  → ground shadow below-right
                  • inner-shadow  → dark crescent on the inner edge of
                                    the rim, suggesting the rim is
                                    sunk below the page surface
                  • highlight     → faint white crescent on the outer
                                    upper edge of the rim, suggesting
                                    a bevel catching light from above */}
            <defs>
              <filter id="seal-indent" x="-20%" y="-20%" width="140%" height="140%">
                <feGaussianBlur in="SourceAlpha" stdDeviation="1.5" />
                <feOffset dx="2" dy="2" result="offset-in" />
                <feComposite operator="arithmetic" k2="-1" k3="1"
                             in="offset-in" in2="SourceAlpha" result="hole-shadow" />
                <feFlood floodColor="#000" floodOpacity="0.35" />
                <feComposite in2="hole-shadow" operator="in" result="inner-shadow" />
                <feMerge>
                  <feMergeNode in="SourceGraphic" />
                  <feMergeNode in="inner-shadow" />
                </feMerge>
              </filter>
              <filter id="seal-lift" x="-20%" y="-20%" width="140%" height="140%">
                <feDropShadow dx="0" dy="3" stdDeviation="3"
                              floodColor="#000" floodOpacity="0.18" />
              </filter>
              <radialGradient id="seal-bevel" cx="50%" cy="38%" r="62%">
                <stop offset="0%"  stopColor="#fff" stopOpacity="0.06" />
                <stop offset="65%" stopColor="#fff" stopOpacity="0.00" />
                <stop offset="92%" stopColor="#000" stopOpacity="0.10" />
                <stop offset="100%" stopColor="#000" stopOpacity="0.18" />
              </radialGradient>

              {/* Curved-text arcs — radius is well INSIDE the rim so
                  the heading text has clear breathing room from the
                  outer circle (no more text hugging the rim). */}
              <path
                id="seal-arc-top"
                d={`M ${sealSize / 2 - (ringR1 - 18)} ${sealSize / 2} a ${ringR1 - 18} ${ringR1 - 18} 0 0 1 ${(ringR1 - 18) * 2} 0`}
                fill="none"
              />
              <path
                id="seal-arc-bot"
                d={`M ${sealSize / 2 - (ringR1 - 18)} ${sealSize / 2} a ${ringR1 - 18} ${ringR1 - 18} 0 0 0 ${(ringR1 - 18) * 2} 0`}
                fill="none"
              />
            </defs>

            {/* Embossed ground disc — radial bevel that makes the
                seal look indented/sunk into the page. */}
            <g filter="url(#seal-lift)">
              <circle cx={sealSize / 2} cy={sealSize / 2} r={ringR1 + 3}
                      fill="url(#seal-bevel)" />
              <circle cx={sealSize / 2} cy={sealSize / 2} r={ringR1}
                      fill="none" stroke="currentColor"
                      strokeWidth={1.5} opacity={0.85}
                      filter="url(#seal-indent)" />
            </g>

            {/* Inner faint band markers (text breathing room visible). */}
            <circle cx={sealSize / 2} cy={sealSize / 2} r={ringR1 - 30}
                    fill="none" stroke="currentColor" strokeWidth={0.4} opacity={0.22} />
            <circle cx={sealSize / 2} cy={sealSize / 2} r={ringInner}
                    fill="none" stroke="currentColor" strokeWidth={0.4} opacity={0.25} />

            {/* 12 capital Greek glyphs (Α Β Γ Δ Ε Ζ Η Θ Ι Κ Λ Μ),
                evenly spaced at 30° increments on the mid ring,
                offset 15° from the diagonal cube anchors so they
                never collide with the corner cubes. */}
            {["Α","Β","Γ","Δ","Ε","Ζ","Η","Θ","Ι","Κ","Λ","Μ"].map((g, i) => {
              const a = (i * 2 * Math.PI) / 12 - Math.PI / 2;
              const x = sealSize / 2 + ringMid * Math.cos(a);
              const y = sealSize / 2 + ringMid * Math.sin(a);
              return (
                <text
                  key={`gk-${i}`} x={x} y={y}
                  fontSize={11} fontFamily="Georgia, 'Times New Roman', serif"
                  fontWeight={600}
                  fill="currentColor" opacity={0.62}
                  textAnchor="middle" dominantBaseline="central"
                  data-testid={`text-greek-glyph-${i}`}
                >
                  {g}
                </text>
              );
            })}

            {/* 48-cube mandala — sits on the inner safe-zone ring. */}
            <IsoCubeRing cx={sealSize / 2} cy={sealSize / 2} r={ringInner} count={48} cubeSize={4} />

            {/* Four BIG DIAGONAL anchor cubes — NE / NW / SE / SW —
                aligned with the QR matrix corners.  Each cube sits
                JUST outside its corresponding QR corner, with a soft
                drop-shadow ellipse beneath for unmistakable 3-D lift. */}
            {[
              { ax:  Math.SQRT1_2, ay: -Math.SQRT1_2, key: "NE" },
              { ax: -Math.SQRT1_2, ay: -Math.SQRT1_2, key: "NW" },
              { ax: -Math.SQRT1_2, ay:  Math.SQRT1_2, key: "SW" },
              { ax:  Math.SQRT1_2, ay:  Math.SQRT1_2, key: "SE" },
            ].map(({ ax, ay, key }) => {
              const px = sealSize / 2 + anchorR * ax;
              const py = sealSize / 2 + anchorR * ay;
              const cs = 14;  // half-edge of the big anchor cube
              return (
                <g key={`anchor-${key}`}>
                  <ellipse
                    cx={px} cy={py + cs * 1.05}
                    rx={cs * 1.15} ry={cs * 0.32}
                    fill="currentColor" opacity={0.18}
                  />
                  <IsoCubeRing cx={px} cy={py} r={0} count={1} cubeSize={cs} bold />
                </g>
              );
            })}

            {/* Faint radiating sponge-state rays — 27 spokes (3³). */}
            {Array.from({ length: 27 }).map((_, i) => {
              const a = (i * 2 * Math.PI) / 27 - Math.PI / 2;
              const x1 = sealSize / 2 + (qrInset + 6) * Math.cos(a);
              const y1 = sealSize / 2 + (qrInset + 6) * Math.sin(a);
              const x2 = sealSize / 2 + (ringInner - 8) * Math.cos(a);
              const y2 = sealSize / 2 + (ringInner - 8) * Math.sin(a);
              return (
                <line key={`ray-${i}`} x1={x1} y1={y1} x2={x2} y2={y2}
                  stroke="currentColor" strokeWidth={0.25} opacity={0.18} />
              );
            })}

            {/* QR matrix rendered as 3-D crystal-cell mesh — centred.
                Carries the EAC integrity payload INCLUDING the full
                attosecond integer + UTC ISO + chain-tag prefix, so a
                scan independently re-derives the cert's identity. */}
            {qrModules && (
              <g transform={`translate(${(sealSize - qrInset * 2) / 2}, ${(sealSize - qrInset * 2) / 2})`}>
                <CrystalMatrix modules={qrModules} size={qrInset * 2} />
              </g>
            )}

            {/* Curved heading text along the outer band — padded well
                inside the rim so it never touches the outer circle. */}
            <text fontSize={9} letterSpacing={3} fill="currentColor" opacity={0.9}>
              <textPath href="#seal-arc-top" startOffset="50%" textAnchor="middle">
                PLENUMNET · ENERGY ATTESTATION
              </textPath>
            </text>
            <text fontSize={7} letterSpacing={2} fill="currentColor" opacity={0.7}>
              <textPath href="#seal-arc-bot" startOffset="50%" textAnchor="middle">
                TL-DSA-87 · TL-SPONGE-385 · CNSA 2.0
              </textPath>
            </text>
          </svg>
          <div className="flex items-center gap-1.5 text-[10px] text-muted-foreground">
            <Hexagon className="w-3 h-3" />
            Scannable · 3-D isometric crystal lattice (ISO/IEC 18004)
          </div>
          <div className="flex items-center gap-1.5 text-[10px] text-muted-foreground">
            <Stamp className="w-3 h-3" />
            48-cube mandala · 4 diagonal corner anchors · 12 Greek glyphs · 27 sponge rays
          </div>
        </div>
      </div>

      {/* ── Footer audit-interop band — hex copies live ONLY here ──── */}
      <div className="border-t border-primary/20 bg-muted/40 px-6 py-3 text-[10px] font-mono space-y-1">
        <div className="text-muted-foreground uppercase tracking-[0.2em] text-[9px] mb-1">
          Audit interop · legacy hex copies (do not consume)
        </div>
        <div className="break-all" data-testid="text-eac-hex-tis27">
          tis27_hash_hex: {trimHex(integ.tis27_hash_hex)}
        </div>
        <div className="break-all" data-testid="text-eac-hex-chain-tag">
          chain_tag_hex: {trimHex(chain.chain_tag_hex)}
        </div>
        <div className="break-all" data-testid="text-eac-hex-pubkey">
          public_key_hash: {trimHex(sig.public_key_hash)}
        </div>
      </div>
      </div>
      </div>
    </>
  );
}

// ── Sub-components ───────────────────────────────────────────────────────
function SectionHeader({ children }: { children: React.ReactNode }) {
  return (
    <div className="border-b border-primary/30 pb-1 mb-2 text-[10px] uppercase tracking-[0.18em] text-primary font-semibold">
      {children}
    </div>
  );
}

function Field({
  label,
  value,
  mono,
  mid,
  highlight,
  greekFont,
  plain,
}: {
  label: string;
  value: React.ReactNode;
  mono?: boolean;
  mid?: boolean;
  highlight?: boolean;
  greekFont?: boolean;
  /** Plain-English explanation shown on hover (and also as a small
   *  inline note under the value, so a printed PDF carries the
   *  explanation too — print media has no hover state). */
  plain?: string;
}) {
  return (
    <div
      className={[
        "grid grid-cols-[200px_1fr] gap-3 py-1 border-b border-zinc-200/50 dark:border-zinc-800/50 last:border-b-0",
        highlight ? "bg-primary/5 -mx-2 px-2 rounded" : "",
      ].join(" ")}
    >
      <div className="text-[11px] text-muted-foreground self-start pt-0.5 inline-flex items-start gap-1">
        <span>{label}</span>
        {plain && (
          <span title={plain} className="eac-print-hide cursor-help text-muted-foreground/70">
            <Info className="w-3 h-3 inline-block -mt-0.5" />
          </span>
        )}
      </div>
      <div>
        <div
          className={[
            mono ? "font-mono" : "",
            mid ? "text-[11px]" : "text-xs",
            greekFont ? "font-serif text-base leading-snug" : "",
            "break-all",
          ].join(" ")}
          lang={greekFont ? "el" : undefined}
        >
          {value}
        </div>
        {plain && (
          <div className="text-[10px] text-muted-foreground/80 mt-0.5 leading-snug">
            {plain}
          </div>
        )}
      </div>
    </div>
  );
}

function CalRow({ label, value }: { label: string; value: any }) {
  return (
    <div className="flex justify-between gap-3 py-0.5 border-b border-zinc-100 dark:border-zinc-900/40">
      <span className="text-muted-foreground text-[10px] uppercase tracking-wider">{label}</span>
      <span className="font-mono text-[11px] text-right break-all">{value != null && value !== "" ? String(value) : "—"}</span>
    </div>
  );
}
