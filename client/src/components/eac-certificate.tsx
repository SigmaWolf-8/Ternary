import QRCode from "qrcode";
import { useEffect, useMemo, useState } from "react";
import { ShieldCheck, Stamp, Hexagon } from "lucide-react";

// ────────────────────────────────────────────────────────────────────────────
// Energy Attestation Certificate (EAC) — professional notarization layout
// styled after the SignHere notarization PDF reference (TM-2026-042 Rev.2).
//
// All cryptographic identifiers (chain_tag, session_id, chain_seed,
// cipher_trits, tis27_hash) are surfaced TRIT-NATIVE in Rep-C bijective
// base-3 (digit set {1,2,3}).  Hex copies are rendered only inside the
// "Audit interop" footer block.  No hex appears in any primary field.
//
// The SCANNABLE seal QR (right) is the standard ISO/IEC 18004 matrix —
// generated from the canonical EAC payload — but every module is rendered
// as a triangular trit glyph (▲ / ▼ / ◆) so the seal reads as a
// post-quantum geometric mesh while still being scannable by any
// conformant QR reader.  Around it, three concentric Forge polygons
// (3-gon / 7-gon / 11-gon → Coprime Triple {7,11,13}) encode the
// chain_index, tick_counter, and savings_ratio respectively.
// ────────────────────────────────────────────────────────────────────────────

interface EacProps {
  eac: any;
  error?: string | null;
}

// ── Helpers ──────────────────────────────────────────────────────────────
function fmtTrit(s: string | undefined, group = 9): string {
  // Bijective base-3 of the integer 0 is the empty string by definition
  // (Spec v3.3.33 §3.2 — Rep-C has no zero glyph).  Render that as "0"
  // for human display so the field never appears empty.
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

// QR matrix renderer with a strict, decoder-safe contrast contract:
//   - light modules → SOLID white (no decoration, no opacity tricks)
//   - dark  modules → SOLID currentColor square that occupies the FULL
//     cell (no shape tricks; required for ISO/IEC 18004 readability)
//   - on top of every dark cell we OVERLAY a small triangular trit glyph
//     in white at low opacity, purely as a visual "post-quantum mesh"
//     accent that does not break the dark→light contrast a scanner sees.
//   - a 4-module-wide quiet zone of solid white is reserved around the
//     entire matrix (ISO/IEC 18004 §6.3.8 minimum quiet zone).
//   - the entire QR area is OPAQUE white so the decorative geometric
//     polygons drawn underneath the seal do not bleed into the matrix.
function TritGlyphMatrix({
  modules,
  size,
  fg = "currentColor",
}: {
  modules: boolean[][];
  size: number;
  fg?: string;
}) {
  const n = modules.length;
  const QUIET = 4;                       // ISO/IEC 18004 minimum quiet zone (modules)
  const total = n + QUIET * 2;
  const cell = size / total;
  const offset = QUIET * cell;
  const cells: JSX.Element[] = [];
  // Dark squares — the actual scannable matrix.
  for (let r = 0; r < n; r++) {
    for (let c = 0; c < n; c++) {
      if (!modules[r][c]) continue;
      const x = offset + c * cell;
      const y = offset + r * cell;
      cells.push(
        <rect
          key={`d-${r}-${c}`}
          x={x}
          y={y}
          width={cell}
          height={cell}
          fill={fg}
          shapeRendering="crispEdges"
        />,
      );
    }
  }
  // Decorative trit-glyph overlay — small white triangle inside each
  // dark module.  Stays well within the dark square (≤ 35% of the cell)
  // so the cell remains "mostly dark" to a QR decoder.
  const decor: JSX.Element[] = [];
  for (let r = 0; r < n; r++) {
    for (let c = 0; c < n; c++) {
      if (!modules[r][c]) continue;
      const x = offset + c * cell;
      const y = offset + r * cell;
      const cx = x + cell / 2;
      const pointsUp = (r + c) % 2 === 0;
      const inset = cell * 0.32;
      const tri = pointsUp
        ? `${cx},${y + cell - inset - cell * 0.12} ${x + inset},${y + cell - cell * 0.18} ${x + cell - inset},${y + cell - cell * 0.18}`
        : `${cx},${y + inset + cell * 0.12} ${x + inset},${y + cell * 0.18} ${x + cell - inset},${y + cell * 0.18}`;
      decor.push(
        <polygon
          key={`o-${r}-${c}`}
          points={tri}
          fill="#ffffff"
          opacity={0.30}
          stroke="none"
        />,
      );
    }
  }
  return (
    <g>
      {/* Quiet zone + light-module background — solid white, full size */}
      <rect x={0} y={0} width={size} height={size} fill="#ffffff" />
      {cells}
      {decor}
    </g>
  );
}

// ── Component ────────────────────────────────────────────────────────────
export function EacCertificate({ eac, error }: EacProps) {
  const [qrModules, setQrModules] = useState<boolean[][] | null>(null);

  // Canonical payload encoded in the QR — short summary so the QR matrix
  // stays scannable on screen.  Field paths follow the actual server
  // schema (TM-2026-042 Rev.2 §4.3): tick_decimal lives at the timestamp
  // root, not under derivation.  Full canonical JSON is in the
  // <details> disclosure below the seal.
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
    });
  }, [eac]);

  useEffect(() => {
    if (!qrPayload) return;
    try {
      // QRCode.create is synchronous in the qrcode package — returns a
      // QRCode object whose `.modules` is a BitMatrix-like with `.size`
      // and `.get(row, col)` (1 = dark module, 0 = light module).
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

  const sealSize  = 256;          // px — overall seal region
  const ringR1    = 124;          // outermost decorative ring
  const ringR2    = 110;          // savings polygon (11-gon)
  const ringR3    =  98;          // tick polygon (7-gon)
  const ringR4    =  86;          // chain-index polygon (3-gon)
  const qrInset   =  74;          // QR matrix size

  return (
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
        <div className="text-right">
          <div className="text-[10px] uppercase tracking-[0.2em] text-muted-foreground">
            Specification
          </div>
          <div className="text-sm font-mono">TM-2026-042 Rev.2 · EAC/1</div>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-[1fr_auto] gap-6 p-6">
        {/* ── LEFT: notarization fields ────────────────────────────── */}
        <div className="space-y-5">
          {/* Document section */}
          <section data-testid="section-eac-document">
            <SectionHeader>Document</SectionHeader>
            <Field label="Title" value="Energy Attestation Certificate (EAC)" />
            <Field label="Issuer" value={`PlenumNET node · ${node.tdns ?? "tdns:hmodal-demo:01"}`} />
            <Field label="Mode"   value={node.mode ?? "—"} />
            <Field label="Demand" value={node.demand_mode ?? "—"} />
          </section>

          {/* Timestamp section — pure first-principles.  Server schema:
              top-level fields live on `timestamp`; the rich derivation
              breakdown lives on `timestamp.derivation`. */}
          <section data-testid="section-eac-timestamp">
            <SectionHeader>Timestamp · Pure First-Principles Tick Walk</SectionHeader>
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
                  />
                  <Field label="Walk modulus D_α" value={`${d.walk_modulus_d_alpha ?? "125250125"}  (= ${d.walk_factorisation ?? "5^3·7^2·11^2·13^2"})`} mono />
                  <Field label="Tick counter (decimal)" value={ts.tick_decimal ?? d.tick_counter_decimal ?? "—"} mono />
                  <Field label="Tick (trit)"            value={trimTrit(ts.tick_trit, 36)} mono />
                  <Field label="Walk position"          value={d.walk_position_decimal ?? "—"} mono />
                  <Field label="Framework fs index"     value={d.framework_fs_index ?? "—"} mono />
                  <Field
                    label="as since boot (rational)"
                    value={asNum != null && asDen != null ? `${asNum} / ${asDen}` : "—"}
                    mono
                  />
                  <Field label="Clock tier"            value={`${d.clock_tier ?? 0}  (0 = pure derivation)`} mono />
                  <Field
                    label="Chain index at seal (trit)"
                    value={d.chain_index_at_seal_trit ?? "—"}
                    mono
                  />
                  <Field
                    label="Chain tag at seal (trit)"
                    value={trimTrit(d.chain_tag_trit, 54)}
                    mono
                    mid
                  />
                </>
              );
            })()}
          </section>

          {/* Measurement section */}
          <section data-testid="section-eac-measurement">
            <SectionHeader>Measurement · Power &amp; Energy Savings</SectionHeader>
            <Field label="Window (ms)"      value={String(meas.window_ms ?? "—")} mono />
            <Field label="Measured power"   value={`${meas.measured_mW ?? "—"} mW`} mono highlight />
            <Field label="Baseline power"   value={`${meas.baseline_mW ?? "—"} mW`} mono />
            <Field label="Power saved"      value={`${meas.mW_saved ?? "—"} mW`} mono highlight />
            <Field
              label="Savings ratio"
              value={
                meas.savings_ratio
                  ? `${meas.savings_ratio.num} / ${meas.savings_ratio.den}` +
                    `   (≈ ${((meas.savings_ratio.num / Math.max(1, meas.savings_ratio.den)) * 100).toFixed(2)}%)`
                  : "—"
              }
              mono
            />
            <Field
              label="Savings ratio (theoretical)"
              value={
                meas.savings_ratio_theoretical
                  ? `${meas.savings_ratio_theoretical.num} / ${meas.savings_ratio_theoretical.den}   (74.48%)`
                  : "—"
              }
              mono
            />
            <Field label="Cumulative energy (µJ)" value={meas.cumulative_energy_uJ_decimal ?? "—"} mono />
            <Field label="Cumulative energy (trit)" value={trimTrit(meas.cumulative_energy_uJ_trit, 36)} mono />
          </section>

          {/* Cryptographic Integrity section — TRIT-NATIVE */}
          <section data-testid="section-eac-integrity">
            <SectionHeader>Cryptographic Integrity · Rep-C Bijective Base-3 (Trit-Native)</SectionHeader>
            <Field label="Cipher" value={chain.cipher ?? "TL-Sponge-385 duplex (Phase Encryption v3)"} mono />
            <Field label="Session ID (trit)"        value={fmtTrit(chain.session_id)} mono mid />
            <Field label="Session-key fingerprint (trit)" value={trimTrit(chain.session_key_fingerprint_trit, 54)} mono mid />
            <Field label="Chain seed (trit)"         value={trimTrit(chain.chain_seed_trit, 81)} mono mid />
            <Field label="Chain index (decimal)"    value={chain.chain_index_decimal ?? "—"} mono />
            <Field label="Chain index (trit)"       value={fmtTrit(chain.chain_index_trit)} mono />
            <Field label="Chain tag · 385-bit (trit)" value={trimTrit(chain.chain_tag_trit, 81)} mono mid highlight />
            <Field label="Cipher payload (trit, head)" value={trimTrit(chain.cipher_trits_trit, 81)} mono mid />
            <Field label="TIS-27 doc hash (trit)"   value={trimTrit(integ.tis27_hash_hex ? bigHexToTrit(integ.tis27_hash_hex) : undefined, 54)} mono mid />
            <Field
              label="TIS-27 · Milesian glyphs"
              value={integ.tis27_hash_milesian || "—"}
              mono
              greekFont
            />
          </section>

          {/* Signature section */}
          <section data-testid="section-eac-signature">
            <SectionHeader>Signature</SectionHeader>
            <Field label="Variant"          value={sig.variant ?? "TL-DSA-87"} mono />
            <Field label="Public key hash (trit)"  value={trimTrit(sig.public_key_hash ? bigHexToTrit(sig.public_key_hash) : undefined, 54)} mono mid />
            <Field label="Signature (trit, head)"  value={trimTrit(sig.signature_hex ? bigHexToTrit(sig.signature_hex) : undefined, 81)} mono mid highlight />
          </section>
        </div>

        {/* ── RIGHT: geometric notarization seal ───────────────────── */}
        <div className="flex flex-col items-center gap-3">
          <div className="text-[10px] uppercase tracking-[0.2em] text-muted-foreground">
            Notarization Seal
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
            <circle cx={sealSize / 2} cy={sealSize / 2} r={ringR1 - 6} fill="none" stroke="currentColor" strokeWidth={0.6} opacity={0.4} />

            {/* 11-gon — savings ratio (Coprime Triple, 11) */}
            <polygon
              points={polygonPoints(sealSize / 2, sealSize / 2, ringR2, 11)}
              fill="none"
              stroke="currentColor"
              strokeWidth={0.8}
              opacity={0.55}
            />
            {/* 7-gon — tick counter (Coprime Triple, 7) */}
            <polygon
              points={polygonPoints(sealSize / 2, sealSize / 2, ringR3, 7)}
              fill="none"
              stroke="currentColor"
              strokeWidth={0.8}
              opacity={0.55}
            />
            {/* 13-gon — implicit Coprime Triple anchor */}
            <polygon
              points={polygonPoints(sealSize / 2, sealSize / 2, (ringR2 + ringR3) / 2, 13)}
              fill="none"
              stroke="currentColor"
              strokeWidth={0.4}
              opacity={0.30}
            />
            {/* 3-gon — chain index (ternary anchor) */}
            <polygon
              points={polygonPoints(sealSize / 2, sealSize / 2, ringR4, 3)}
              fill="none"
              stroke="currentColor"
              strokeWidth={1.0}
              opacity={0.6}
            />

            {/* QR matrix rendered as triangular trit-glyph mesh */}
            {qrModules && (
              <g transform={`translate(${(sealSize - qrInset * 2) / 2}, ${(sealSize - qrInset * 2) / 2})`}>
                <TritGlyphMatrix
                  modules={qrModules}
                  size={qrInset * 2}
                  fg="currentColor"
                />
              </g>
            )}

            {/* Center stamp accent */}
            <circle
              cx={sealSize / 2}
              cy={sealSize / 2}
              r={4}
              fill="currentColor"
              opacity={0}
            />

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
            Scannable · ISO/IEC 18004 trit-glyph mesh
          </div>
          <div className="flex items-center gap-1.5 text-[10px] text-muted-foreground">
            <Stamp className="w-3 h-3" />
            Coprime Triple {"{7,11,13}"} · Ternary 3-gon anchor
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
}: {
  label: string;
  value: React.ReactNode;
  mono?: boolean;
  mid?: boolean;
  highlight?: boolean;
  greekFont?: boolean;
}) {
  return (
    <div
      className={[
        "grid grid-cols-[180px_1fr] gap-3 py-1 border-b border-zinc-200/50 dark:border-zinc-800/50 last:border-b-0",
        highlight ? "bg-primary/5 -mx-2 px-2 rounded" : "",
      ].join(" ")}
    >
      <div className="text-[11px] text-muted-foreground self-start pt-0.5">{label}</div>
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
    </div>
  );
}

// Convert a hex string to Rep-C bijective base-3 (digits {1,2,3}) on the
// client.  Used to render fields the server happens to only emit in hex
// (e.g. signature_hex, public_key_hash) so the certificate stays
// trit-native everywhere user-facing.
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
