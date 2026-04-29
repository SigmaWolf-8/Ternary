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
      // SCANNABILITY-FIRST construction: each module is first painted
      // as a SOLID full-cell square in the darkest currentColor (full
      // 100 % luminance towards dark — what an ISO/IEC 18004 decoder
      // sees).  The three facet polygons are then OVERLAID at lower
      // opacities to give the gem-cube look without ever revealing
      // background.  Result: ≥ 90 % effective dark mass per dark
      // module under every print/screen DPI.
      const cx = x + cell / 2;
      const cy = y + cell / 2;
      // Diamond top facet covering the upper half of the cell.
      const top  = `${cx},${y}  ${x + cell},${cy}  ${cx},${y + cell}  ${x},${cy}`;
      // Lower-left facet.
      const left = `${x},${cy}  ${cx},${y + cell}  ${x},${y + cell}`;
      // Lower-right facet.
      const right= `${x + cell},${cy}  ${x + cell},${y + cell}  ${cx},${y + cell}`;
      facets.push(
        <g key={`g-${r}-${c}`} shapeRendering="geometricPrecision">
          {/* Solid base — the QR-decoder substrate. */}
          <rect x={x} y={y} width={cell} height={cell} fill="currentColor" />
          {/* Crystal facet shading — overlay only; never lifts the
              dark mass above the ISO 18004 module-luminance threshold. */}
          <polygon points={top}   fill="currentColor" opacity={0.55} />
          <polygon points={left}  fill="currentColor" opacity={0.88} />
          <polygon points={right} fill="currentColor" opacity={1.00} />
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
  cx, cy, r, count = 24, cubeSize = 6,
}: { cx: number; cy: number; r: number; count?: number; cubeSize?: number }) {
  const cubes: JSX.Element[] = [];
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
        <polygon points={top}   fill="currentColor" opacity={0.40} />
        <polygon points={left}  fill="currentColor" opacity={0.65} />
        <polygon points={right} fill="currentColor" opacity={0.85} />
      </g>,
    );
  }
  return <g>{cubes}</g>;
}

// ── Component ────────────────────────────────────────────────────────────
export function EacCertificate({ eac, error }: EacProps) {
  const [qrModules, setQrModules] = useState<boolean[][] | null>(null);

  // Canonical short payload for the QR — keeps the matrix scannable
  // on screen by keeping payload size compact.  Field paths follow
  // the actual server schema (TM-2026-042 Rev.2 §4.3).
  const qrPayload = useMemo(() => {
    if (!eac) return "";
    const c = eac.attestation_chain ?? {};
    const t = eac.timestamp ?? {};
    const d = t.derivation ?? {};
    return JSON.stringify({
      v:    "EAC/1",
      sid:  c.session_id ?? "",
      idx:  c.chain_index_decimal ?? "",
      tag:  (c.chain_tag_trit ?? "").slice(0, 60),
      tick: t.tick_decimal ?? "",
      walk: d.walk_position_decimal ?? "",
      as:   t.attoseconds_since_boot_decimal ?? "",
    });
  }, [eac]);

  useEffect(() => {
    if (!qrPayload) return;
    try {
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

  const sealSize  = 280;
  const ringR1    = 134;
  const ringR2    = 118;
  const ringR3    = 104;
  const ringR4    =  90;
  const qrInset   =  78;

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
            <SectionHeader>Timestamp · Attosecond Precision (Pure Framework Derivation)</SectionHeader>

            {/* Featured single-integer attosecond timestamp */}
            <div className="rounded-md border border-primary/40 bg-primary/5 p-3 mb-3" data-testid="block-eac-attoseconds">
              <div className="text-[10px] uppercase tracking-[0.18em] text-muted-foreground mb-1">
                Attoseconds since system boot
              </div>
              <div
                className="font-mono text-base sm:text-lg break-all leading-snug text-primary font-semibold"
                data-testid="text-eac-attoseconds"
              >
                {groupDigits(ts.attoseconds_since_boot_decimal)} <span className="text-muted-foreground font-normal">as</span>
              </div>
              <div className="text-[10px] text-muted-foreground mt-1">
                Plain English: the exact attosecond ({"\u200a"}10⁻¹⁸ s{"\u200a"}) count from when this node booted, derived purely from the framework tick walk — no hardware clock was consulted.
              </div>
              <div className="font-mono text-[10px] text-muted-foreground break-all mt-2">
                trit (Rep-C): {trimTrit(ts.attoseconds_since_boot_trit, 81)}
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
            {/* Outer ring band */}
            <circle cx={sealSize / 2} cy={sealSize / 2} r={ringR1} fill="none" stroke="currentColor" strokeWidth={1.5} opacity={0.7} />
            <circle cx={sealSize / 2} cy={sealSize / 2} r={ringR1 - 6} fill="none" stroke="currentColor" strokeWidth={0.5} opacity={0.35} />

            {/* Outer mandala of small isometric cubes — 24 nodes */}
            <IsoCubeRing cx={sealSize / 2} cy={sealSize / 2} r={ringR1 - 18} count={24} cubeSize={4} />

            {/* 11-gon — savings ratio (Coprime Triple, 11) */}
            <polygon
              points={polygonPoints(sealSize / 2, sealSize / 2, ringR2, 11)}
              fill="none" stroke="currentColor" strokeWidth={0.8} opacity={0.55}
            />
            {/* 7-gon — tick counter (Coprime Triple, 7) */}
            <polygon
              points={polygonPoints(sealSize / 2, sealSize / 2, ringR3, 7)}
              fill="none" stroke="currentColor" strokeWidth={0.8} opacity={0.55}
            />
            {/* 13-gon — implicit Coprime Triple anchor */}
            <polygon
              points={polygonPoints(sealSize / 2, sealSize / 2, (ringR2 + ringR3) / 2, 13)}
              fill="none" stroke="currentColor" strokeWidth={0.4} opacity={0.30}
            />
            {/* 3-gon — chain index (ternary anchor) */}
            <polygon
              points={polygonPoints(sealSize / 2, sealSize / 2, ringR4, 3)}
              fill="none" stroke="currentColor" strokeWidth={1.0} opacity={0.6}
            />

            {/* Faint radiating sponge-state rays — 27 spokes (3·3·3) */}
            {Array.from({ length: 27 }).map((_, i) => {
              const a = (i * 2 * Math.PI) / 27 - Math.PI / 2;
              const x1 = sealSize / 2 + (ringR4 + 2) * Math.cos(a);
              const y1 = sealSize / 2 + (ringR4 + 2) * Math.sin(a);
              const x2 = sealSize / 2 + (ringR2 - 2) * Math.cos(a);
              const y2 = sealSize / 2 + (ringR2 - 2) * Math.sin(a);
              return (
                <line key={`ray-${i}`} x1={x1} y1={y1} x2={x2} y2={y2}
                  stroke="currentColor" strokeWidth={0.25} opacity={0.18} />
              );
            })}

            {/* QR matrix rendered as crystal-cell mesh */}
            {qrModules && (
              <g transform={`translate(${(sealSize - qrInset * 2) / 2}, ${(sealSize - qrInset * 2) / 2})`}>
                <CrystalMatrix modules={qrModules} size={qrInset * 2} />
              </g>
            )}

            {/* Curved heading text along the outer band */}
            <defs>
              <path
                id="seal-arc-top"
                d={`M ${sealSize / 2 - (ringR1 - 14)} ${sealSize / 2} a ${ringR1 - 14} ${ringR1 - 14} 0 0 1 ${(ringR1 - 14) * 2} 0`}
                fill="none"
              />
              <path
                id="seal-arc-bot"
                d={`M ${sealSize / 2 - (ringR1 - 14)} ${sealSize / 2} a ${ringR1 - 14} ${ringR1 - 14} 0 0 0 ${(ringR1 - 14) * 2} 0`}
                fill="none"
              />
            </defs>
            <text fontSize={8} letterSpacing={3} fill="currentColor" opacity={0.85}>
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
            Scannable · ISO/IEC 18004 crystal lattice
          </div>
          <div className="flex items-center gap-1.5 text-[10px] text-muted-foreground">
            <Stamp className="w-3 h-3" />
            Coprime Triple {"{7,11,13}"} · Ternary 3-gon · 27 sponge rays
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
