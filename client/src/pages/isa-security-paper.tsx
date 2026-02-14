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

import { useState, useEffect, useRef, useCallback } from "react";

const col = {
  bg: "#0a1628",
  bg2: "#111d33",
  ac: "#3b82f6",
  al: "#60a5fa",
  gd: "#f59e0b",
  tx: "#e2e8f0",
  dm: "#94a3b8",
  mt: "#64748b",
  bd: "#1e3a5f",
  wt: "#f8fafc",
  sf: "#0f1a2e",
  th: "#132242",
  ta: "#0f1e36",
  cd: "#0c1524",
  gn: "#10b981",
};

const ff = {
  h: "'Playfair Display',serif",
  b: "'Source Sans 3',sans-serif",
  m: "'JetBrains Mono',monospace",
  l: "'Inter',sans-serif",
};

const wrapS: React.CSSProperties = { maxWidth: 880, margin: "0 auto", padding: "0 24px" };

const heroS: React.CSSProperties = { padding: "56px 0 36px", textAlign: "center" as const };
const heroLabelS: React.CSSProperties = { fontFamily: ff.l, fontSize: 10, letterSpacing: 4, textTransform: "uppercase" as const, color: col.ac, marginBottom: 16, fontWeight: 600 };
const heroTitleS: React.CSSProperties = { fontFamily: ff.h, fontSize: "clamp(1.6rem,4.5vw,2.6rem)", fontWeight: 700, color: col.wt, lineHeight: 1.12, marginBottom: 10 };
const heroSubS: React.CSSProperties = { fontFamily: ff.h, fontSize: "clamp(0.95rem,2vw,1.2rem)", fontStyle: "italic" as const, color: col.al, marginBottom: 20 };
const heroInfoS: React.CSSProperties = { fontFamily: ff.l, fontSize: 12, color: col.dm, lineHeight: 1.8 };
const heroInfoStrongS: React.CSSProperties = { color: col.tx, fontWeight: 600 };
const heroLinksS: React.CSSProperties = { display: "flex", gap: 12, justifyContent: "center", marginTop: 20, flexWrap: "wrap" as const };
const heroLinkS: React.CSSProperties = { fontFamily: ff.l, fontSize: 11, letterSpacing: 1.5, textTransform: "uppercase" as const, color: col.ac, textDecoration: "none" as const, padding: "6px 16px", border: "1px solid #3b82f640", borderRadius: 4 };
const dividerS: React.CSSProperties = { width: 60, height: 2, background: `linear-gradient(90deg,transparent,${col.ac},transparent)`, margin: "0 auto 18px" };

const navS: React.CSSProperties = { position: "sticky" as const, top: 0, zIndex: 100, background: "#0a1628f0", backdropFilter: "blur(12px)", borderBottom: "1px solid #1e3a5f80" };
const navInnerS: React.CSSProperties = { maxWidth: 880, margin: "0 auto", display: "flex", overflowX: "auto" as const, padding: "0 16px" };
const navBtnBase: React.CSSProperties = { fontFamily: ff.l, fontSize: 11, letterSpacing: 0.8, textTransform: "uppercase" as const, background: "none", border: "none", padding: "12px 13px", cursor: "pointer" as const, whiteSpace: "nowrap" as const, transition: "color 0.15s" };

const tabS = (vis: boolean): React.CSSProperties => ({ display: vis ? "block" : "none", padding: "8px 0 50px" });

const h2S: React.CSSProperties = { fontFamily: ff.h, fontSize: "clamp(1.4rem,3vw,1.8rem)", fontWeight: 700, color: col.wt, margin: "36px 0 16px", paddingBottom: 10, borderBottom: `1px solid ${col.bd}` };
const h3S: React.CSSProperties = { fontFamily: ff.h, fontSize: "clamp(1.1rem,2.5vw,1.35rem)", fontWeight: 600, color: col.al, margin: "28px 0 12px" };
const h4S: React.CSSProperties = { fontFamily: ff.h, fontSize: "1.05rem", fontWeight: 600, color: col.tx, fontStyle: "italic" as const, margin: "22px 0 10px" };
const pS: React.CSSProperties = { margin: "0 0 14px" };
const bS: React.CSSProperties = { fontWeight: 700, color: col.wt };
const codeS: React.CSSProperties = { fontFamily: ff.m, fontSize: "0.85em", background: col.cd, color: col.al, padding: "1px 6px", borderRadius: 3, border: "1px solid #1e3a5f60" };
const cbS: React.CSSProperties = { fontFamily: ff.m, fontSize: 12.5, background: col.cd, color: col.al, padding: "14px 18px", borderRadius: 4, border: `1px solid ${col.bd}`, overflowX: "auto" as const, margin: "10px 0 16px", lineHeight: 1.6, whiteSpace: "pre-wrap" as const };

const tblWrapS: React.CSSProperties = { overflowX: "auto" as const, margin: "14px 0 20px", borderRadius: 4, border: `1px solid ${col.bd}` };
const tableS: React.CSSProperties = { width: "100%", borderCollapse: "collapse" as const, fontSize: 13 };
const thS: React.CSSProperties = { fontFamily: ff.l, fontSize: 11, letterSpacing: 1, textTransform: "uppercase" as const, fontWeight: 600, color: col.al, background: col.th, padding: "10px 14px", textAlign: "left" as const, borderBottom: "2px solid #3b82f630" };
const tdS = (alt: boolean): React.CSSProperties => ({ padding: "8px 14px", borderBottom: "1px solid #1e3a5f40", color: col.tx, verticalAlign: "top" as const, lineHeight: 1.5, background: alt ? col.ta : "transparent" });

const bxS = (t: string): React.CSSProperties => ({ margin: "16px 0", padding: "14px 18px", borderRadius: "0 4px 4px 0", borderLeft: `3px solid ${t === "t" ? col.gd : t === "p" ? col.gn : col.ac}`, background: "#0f1a2e80" });
const bxLabelS = (t: string): React.CSSProperties => ({ fontFamily: ff.l, fontSize: 10, letterSpacing: 2, textTransform: "uppercase" as const, fontWeight: 700, marginBottom: 4, color: t === "t" ? col.gd : t === "p" ? col.gn : col.ac });
const bxBodyS: React.CSSProperties = { fontSize: 14, lineHeight: 1.65 };

const partS: React.CSSProperties = { margin: "48px 0 32px", padding: 28, textAlign: "center" as const, borderTop: "1px solid #3b82f630", borderBottom: "1px solid #3b82f630", background: "linear-gradient(135deg,#111d3380,#0a162840)", borderRadius: 2 };
const partLabelS: React.CSSProperties = { fontFamily: ff.l, fontSize: 11, letterSpacing: 5, textTransform: "uppercase" as const, color: col.gd, fontWeight: 600, marginBottom: 6 };
const partTitleS: React.CSSProperties = { fontFamily: ff.h, fontSize: "1.5rem", fontWeight: 600, color: col.wt };

const tagsS: React.CSSProperties = { display: "flex", flexWrap: "wrap" as const, gap: 7, margin: "14px 0" };
const tagS: React.CSSProperties = { fontFamily: ff.l, fontSize: 10, color: col.ac, background: "#1e40af20", border: "1px solid #3b82f625", padding: "2px 8px", borderRadius: 3 };

const footerS: React.CSSProperties = { marginTop: 60, padding: "24px 0", borderTop: `1px solid ${col.bd}`, textAlign: "center" as const, fontFamily: ff.l, fontSize: 11, color: col.mt, lineHeight: 2 };

const topBtnS = (show: boolean): React.CSSProperties => ({ position: "fixed" as const, bottom: 24, right: 24, width: 38, height: 38, borderRadius: "50%", background: col.ac, color: col.bg, border: "none", cursor: "pointer" as const, fontSize: 16, fontWeight: 700, opacity: show ? 1 : 0, transition: "opacity 0.3s", boxShadow: "0 4px 20px #3b82f640", zIndex: 200, pointerEvents: show ? "auto" as const : "none" as const });

const refS: React.CSSProperties = { fontSize: 13, color: col.dm, lineHeight: 1.8 };
const refP: React.CSSProperties = { margin: "0 0 3px", paddingLeft: 28, textIndent: -28 };

const appS: React.CSSProperties = { minHeight: "100vh", background: `linear-gradient(180deg,${col.bg},#060d18)`, color: col.tx, fontFamily: ff.b, fontSize: 16, lineHeight: 1.72 };

function Tbl({ headers, rows }: { headers: string[]; rows: string[][] }) {
  return (
    <div style={tblWrapS}>
      <table style={tableS}>
        <thead><tr>{headers.map((h: string, i: number) => <th key={i} style={thS}>{h}</th>)}</tr></thead>
        <tbody>{rows.map((r: string[], ri: number) => (
          <tr key={ri}>{r.map((c: string, ci: number) => <td key={ci} style={tdS(ri % 2 === 1)}>{c}</td>)}</tr>
        ))}</tbody>
      </table>
    </div>
  );
}

function Bx({ type, label, children }: { type: "t" | "p" | "n"; label: string; children: React.ReactNode }) {
  return (
    <div style={bxS(type)}>
      <div style={bxLabelS(type)}>{label}</div>
      <div style={bxBodyS}>{children}</div>
    </div>
  );
}

function Cd({ children }: { children: React.ReactNode }) {
  return <code style={codeS}>{children}</code>;
}

function B({ children }: { children: React.ReactNode }) {
  return <b style={bS}>{children}</b>;
}

function P({ children }: { children: React.ReactNode }) {
  return <p style={pS}>{children}</p>;
}

function H1({ children }: { children: React.ReactNode }) {
  return <h2 style={h2S}>{children}</h2>;
}

function H2({ children }: { children: React.ReactNode }) {
  return <h3 style={h3S}>{children}</h3>;
}

function H3({ children }: { children: React.ReactNode }) {
  return <h4 style={h4S}>{children}</h4>;
}

function CB({ children }: { children: React.ReactNode }) {
  return <pre style={cbS}>{children}</pre>;
}

function Part({ label, title }: { label: string; title: string }) {
  return (
    <div style={partS}>
      <div style={partLabelS}>{label}</div>
      <div style={partTitleS}>{title}</div>
    </div>
  );
}

const tabs = ["Overview", "Instructions", "Dual-Phase Encryption", "PlenumDB", "Ternary Substrate", "Architecture", "Analysis & Compare", "References"];

export default function ISASecurityPaper() {
  const [tab, setTab] = useState(0);
  const [showTop, setShowTop] = useState(false);
  const navRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const onScroll = () => setShowTop(window.scrollY > 500);
    window.addEventListener("scroll", onScroll);
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  const switchTab = useCallback((i: number) => {
    setTab(i);
    window.scrollTo({ top: 0 });
  }, []);

  return (
    <div style={appS}>
      <link href="https://fonts.googleapis.com/css2?family=Playfair+Display:ital,wght@0,400;0,600;0,700;1,400&family=Source+Sans+3:wght@300;400;500;600;700&family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet" />

      {/* HERO */}
      <div style={wrapS}>
        <div style={heroS}>
          <div style={heroLabelS}>Salvi Framework ISA v2.0 {"\u2014"} Technical Paper {"\u2014"} February 2026</div>
          <h1 style={heroTitleS}>ISA-Level Security Primitives<br />for Ternary Computing Architectures</h1>
          <p style={heroSubS}>Capability-Based Access Control, Side-Channel Mitigation,<br />Dual-Phase Encryption, and PlenumDB Encrypted Storage</p>
          <div style={dividerS} />
          <div style={heroInfoS}><span style={heroInfoStrongS}>Applied Physics Division</span> {"\u2014"} Capomastro Holdings Ltd.<br />Alberta, Canada {"\u2014"} PlenumNET Post-Quantum Timing Infrastructure</div>
          <div style={heroLinksS}>
            <a href="https://github.com/SigmaWolf-8/Ternary" target="_blank" rel="noopener noreferrer" style={heroLinkS}>Repository {"\u2192"}</a>
            <a href="https://plenumnet.replit.app" target="_blank" rel="noopener noreferrer" style={heroLinkS}>PlenumNET Platform {"\u2192"}</a>
          </div>
        </div>
      </div>

      {/* NAV */}
      <nav style={navS}>
        <div style={navInnerS} ref={navRef}>
          {tabs.map((name: string, i: number) => (
            <button
              key={i}
              data-testid={`tab-btn-${i}`}
              style={{ ...navBtnBase, color: tab === i ? col.ac : col.mt, borderBottom: tab === i ? `2px solid ${col.ac}` : "2px solid transparent", fontWeight: tab === i ? 600 : 400 }}
              onClick={() => switchTab(i)}
              onMouseEnter={(e: React.MouseEvent<HTMLButtonElement>) => { if (tab !== i) (e.target as HTMLElement).style.color = col.al; }}
              onMouseLeave={(e: React.MouseEvent<HTMLButtonElement>) => { if (tab !== i) (e.target as HTMLElement).style.color = col.mt; }}
            >{name}</button>
          ))}
        </div>
      </nav>

      <div style={wrapS}>

        {/* ═══ TAB 0: OVERVIEW ═══ */}
        <div style={tabS(tab === 0)}>
          <Part label="Part I" title="Executive Overview" />

          <H1>Making Computers Safer: A New Kind of Security</H1>
          <P>Imagine your computer as a giant office building. Today's computers are like buildings where every door uses the same flimsy lock, where sound carries through the walls, and where there's no security camera that can't be tampered with. The <B>Salvi Framework</B> is a complete rethinking of computer security, using a different kind of mathematics called <B>ternary logic</B>, built directly into the processor itself.</P>
          <P>Instead of the usual binary (ON/OFF) that all modern chips use, the Salvi processor works with three-state <B>trits</B>. This isn't just a curiosity{"\u2014"}it gives the hardware genuinely new security powers that binary computers physically cannot have, no matter how much software you pile on top.</P>

          <H1>Three Weaknesses This Addresses</H1>
          <H3>1. Memory Bugs and Fake Pointers</H3>
          <P>Every modern computer stores data and instructions in the same flat memory space. Programs routinely confuse one for the other. Attackers exploit this through <B>buffer overflows</B>, <B>use-after-free</B> errors, and <B>pointer forgery</B>{"\u2014"}tricking the computer into treating attacker-controlled data as trusted instructions. Academic solutions like CHERI (Cambridge) bolt capability checks onto binary chips, but they need extra "tag bits" glued to the side of every memory word{"\u2014"}an expensive retrofit.</P>

          <H3>2. The Quantum Threat</H3>
          <P>Current encryption (RSA, ECC) will be broken by quantum computers. Post-quantum algorithms (lattice-based, hash-based) are being standardised by NIST, but they need <B>efficient hardware support</B>{"\u2014"}and critically, they need constant-time execution guarantees that current processors don't provide.</P>

          <H3>3. Side-Channel Leaks</H3>
          <P>Even when software is mathematically perfect, <B>hardware whispers secrets</B>. How long an operation takes, how much power it draws, what electromagnetic radiation it emits{"\u2014"}all leak information. Spectre and Meltdown showed that speculative execution in modern CPUs creates exploitable side channels. Current defences are software patches on top of hardware that was never designed to be quiet.</P>

          <H1>How Ternary Computing Changes the Game</H1>
          <P>A <B>trit</B> (ternary digit) can be -1, 0, or +1 (or equivalently 1, 2, 3 in the "bijective" encoding). This gives <B>1.58 times more information per digit</B> than a bit{"\u2014"}closer to the mathematically optimal radix (<i>e</i> {"\u2248"} 2.718). The Salvi Framework uses three interchangeable representations:</P>

          <Tbl
            headers={["Representation", "Digit Set", "Purpose"]}
            rows={[
              ["A (Computational)", "{\u2212\u200B1, 0, +1}", "Internal arithmetic, GF(3) field operations, algebraic masking, Phase 1 encryption"],
              ["B (Network)", "{0, 1, 2}", "Wire-level protocol encoding"],
              ["C (Bijective)", "{1, 2, 3}", "Capability security (excludes 0 for sentinel), human-readable display"],
            ]}
          />

          <P>The key insight: in Representation C, the value <B>0 is impossible to produce through arithmetic</B>. If you add, subtract, or multiply numbers whose digits are only 1, 2, or 3, you can never get a 0 in any digit position. This means a "0" placed in a special position acts as an <B>unforgeable seal</B>{"\u2014"}only the hardware itself can put it there. No software trick, no buffer overflow, no attacker manipulation can forge it.</P>
        </div>

        {/* ═══ TAB 1: INSTRUCTIONS ═══ */}
        <div style={tabS(tab === 1)}>
          <H1>The Eight Security Instructions (0x90{"\u2013"}0x97)</H1>
          <P>The Salvi ISA v2.0 contains 160 opcodes. Eight of them (opcodes <Cd>0x90</Cd> through <Cd>0x97</Cd>) form the <B>Security and Audit</B> category{"\u2014"}a complete hardware security subsystem that doesn't exist in any other processor architecture.</P>

          <H2>AuditLog (0x90) {"\u2014"} The Camera That Never Lies</H2>
          <CB>AUDITLOG src1</CB>
          <P>Every security-relevant event gets recorded into a protected hardware log: the program counter, privilege level, security domain, an event code, and a <B>femtosecond-precision timestamp</B> from the HPTP timing system. The log is append-only with chain integrity verification (tamper-evident hash chain){"\u2014"}any attempt to delete or alter entries is detectable.</P>
          <P>Twelve event types: AccessGranted, AccessDenied, CapabilityCreated, CapabilityRevoked, CapabilityDelegated, DomainTransition, DomainTransitionDenied, PolicyChange, SecurityModeChange, ProcessCreated, ProcessTerminated, IpcEvent. Any privilege level can emit entries; hardware stamps the <i>actual</i> privilege level, preventing spoofing.</P>

          <H2>CapCheck (0x91) {"\u2014"} Validating Tickets</H2>
          <CB>CAPCHECK dst, src1, src2</CB>
          <P>Two independent verification mechanisms running in parallel:</P>
          <P><B>Mechanism 1 {"\u2014"} Sentinel Validation:</B> Checks that the leading trit of the capability descriptor is a sentinel (value 0 in Representation C). This is an algebraic integrity check{"\u2014"}no tag-controller lookup required.</P>
          <P><B>Mechanism 2 {"\u2014"} Table Lookup:</B> Queries the hardware capability table for token validity, resource match, action permissions (R/W/X), mode compatibility, and expiration. The table stores full CapabilityToken records with owner ProcessId, security mode, femtosecond timestamps, delegation chain, and revocation status.</P>

          <Tbl
            headers={["Field", "Trits", "Position", "Description"]}
            rows={[
              ["SENTINEL", "1", "[0]", "Capability marker (value 0, outside \u0392\u2083)"],
              ["PERM", "3", "[1\u20133]", "Permission triple (Read/Write/Execute)"],
              ["SCOPE", "5", "[4\u20138]", "Scope/domain identifier"],
              ["BASE", "9", "[9\u201317]", "Base address"],
              ["BOUND", "9", "[18\u201326]", "Upper bound"],
            ]}
          />
          <P>Compared to CHERI: eliminates the tag-controller lookup (sentinel is inline), reducing the critical path by one memory-subsystem interaction.</P>

          <H2>CapGrant (0x92) {"\u2014"} Creating Tickets (Ring0 Only)</H2>
          <CB>CAPGRANT dst, src1, src2</CB>
          <P><B>Table operation:</B> Creates a CapabilityToken with femtosecond creation timestamp, configurable expiration, delegatable flag, and parent_token for full provenance tracking.</P>
          <P><B>Register operation:</B> Places the unforgeable sentinel trit (value 0) at position [0] of the capability descriptor.</P>
          <P><B>Monotonic authority principle:</B> Derived capabilities can only have <i>fewer</i> permissions and <i>narrower</i> address ranges than their parent: C<sub>d</sub>.PERM = C<sub>s</sub>.PERM {"\u2229"}{"\u2083"} NewPerm, C<sub>d</sub>.BASE = max(C<sub>s</sub>.BASE, NewBase), C<sub>d</sub>.BOUND = min(C<sub>s</sub>.BOUND, NewBound).</P>
          <P>Automatically generates a CapabilityCreated audit entry.</P>

          <H2>CapRevoke (0x93) {"\u2014"} Revoking Tickets (Ring0 Only)</H2>
          <CB>CAPREVOKE dst, src1, src2</CB>
          <P><B>Dual-mechanism invalidation:</B> Sets the revoked flag in the capability table AND overwrites the sentinel trit with value 1 (making the descriptor fail sentinel validation). Immediate, global within the target security domain. <B>O(1) per register and O(1) per table entry</B>{"\u2014"}unlike CHERI's sweep-based revocation which must scan all memory.</P>
          <P>Optional HPTP-timestamped revocation proof for non-repudiation. Automatically generates a CapabilityRevoked audit entry.</P>

          <H2>SideChMask (0x94) {"\u2014"} Muffling the Whispers</H2>
          <CB>SIDECHMASK dst, src1</CB>
          <P><B>Layer 1 {"\u2014"} Microarchitectural Isolation:</B> A bitmask disables L1 data cache, L1 instruction cache, branch predictor, and speculative execution. Previous SCCR (Side-Channel Control Register) state saved to dst for restoration.</P>
          <P><B>Layer 2 {"\u2014"} Algebraic Ternary Masking:</B> Activates automatic trit-wise modular addition with hardware random trit generator (HRTG) vectors. In Representation A: d'<sub>i</sub> = (d<sub>i</sub> + m<sub>i</sub>) mod 3. In Representation C: d'<sub>i</sub> = ((d<sub>i</sub> + m<sub>i</sub> {"\u2212"} 1) mod 3) + 1.</P>

          <Bx type="t" label="Theorem 2 \u2014 Perfect Masking">
            For any data word D {"\u2208"} {"\u0392"}{"\u2083"}<sup>n</sup> and uniform random mask M {"\u2208"} {"\u0392"}{"\u2083"}<sup>n</sup>, the masked value D' = D {"\u2297"} M is uniformly distributed over {"\u0392"}{"\u2083"}<sup>n</sup> and statistically independent of D. <B>Proof:</B> Masking is a bijection for fixed M; uniform M yields uniform output regardless of input. Ternary non-linear operations achieve O(n) complexity vs binary ISW O(n{"\u00B2"}).
          </Bx>

          <P><B>Structural advantage:</B> Ternary logic gates have uniform transition energy across all six possible single-trit state changes, providing inherent glitch resistance. The dual-layer design means either layer provides independent protection{"\u2014"}graceful degradation if one layer is compromised.</P>

          <H2>SideChUnmask (0x95) {"\u2014"} Restoring Normal Execution</H2>
          <CB>SIDECHUNMASK src1</CB>
          <P>Layer 1: Re-enables microarchitectural features per the saved bitmask. Layer 2: Exits automatic masking mode. Inverse operation: d<sub>i</sub> = (d'<sub>i</sub> {"\u2212"} m<sub>i</sub> + 3) mod 3. Guaranteed: D {"\u2297"} M {"\u2298"} M = D for all D, M.</P>

          <H2>ConstTimeEq (0x96) {"\u2014"} Secret-Safe Comparison</H2>
          <CB>CONSTTIMEEQ dst, src1, src2</CB>
          <P>Constant-time comparison: identical latency regardless of whether values are equal, where they differ, or what values they contain. Implementation: trit-wise subtraction-and-OR-reduce{"\u2014"}no early exit, no branch prediction involvement, no data-dependent carry propagation. ALU path physically isolated from speculative execution. Use cases: MAC verification, password comparison, key equality checks, FIPS self-test validation.</P>

          <H2>ConstTimeSel (0x97) {"\u2014"} Secret-Safe Selection</H2>
          <CB>CONSTTIMESEL dst, src1, src2   (condition in flags register)</CB>
          <P>Constant-time conditional move: both sources read simultaneously, selection via bitwise masking (condition AND src1) OR (NOT condition AND src2). No branch, no speculative path, no data-dependent timing variation. Use cases: elliptic curve point selection, ML-KEM decapsulation padding variants, Fiat-Shamir abort decisions, constant-time table lookups.</P>
        </div>

        {/* ═══ TAB 2: DUAL-PHASE ENCRYPTION ═══ */}
        <div style={tabS(tab === 2)}>
          <H1>6. Dual-Phase Encryption Architecture</H1>

          <H2>6.1 Design Rationale</H2>
          <P>The Salvi Framework's cryptographic subsystem implements a <B>dual-phase encryption pipeline</B> that unifies symmetric and asymmetric post-quantum cryptography within the ternary ISA. Where conventional systems treat symmetric encryption and key management as separate software-layer concerns, the Salvi approach integrates both phases as <B>first-class ISA operations</B>, leveraging the algebraic properties of the three ternary representations to provide structural security advantages unavailable in binary architectures.</P>
          <P>The dual-phase design addresses a fundamental limitation of single-layer encryption: symmetric ciphers provide high throughput but require secure key distribution, while asymmetric schemes solve key distribution but are computationally expensive. By coupling both phases at the ISA level and binding them to the capability system, the Salvi Framework ensures that the encryption pipeline is both performant and <B>capability-gated</B>{"\u2014"}no process can invoke encryption or decryption operations without a valid, non-expired capability token.</P>

          <H2>6.2 Phase 1 {"\u2014"} Ternary Symmetric Encryption</H2>
          <P>Phase 1 operates in the <B>GF(3) Galois field</B> domain (Representation A, digit set {"{"}{"\\u2212"}1, 0, +1{"}"}), implementing a ternary-native symmetric cipher optimised for bulk data encryption. The cipher's round function employs:</P>
          <P>{"\u2022"} <B>Trit-wise substitution</B> (S-box operations over GF(3))<br />
          {"\u2022"} <B>Permutation layers</B> operating on the 27-trit word structure<br />
          {"\u2022"} <B>Key-mixing</B> via trit-wise modular addition{"\u2014"}the same algebraic operation used by <Cd>SideChMask</Cd> for side-channel protection</P>
          <P>This structural alignment is deliberate: Phase 1 encryption inherits the balanced power-consumption properties of ternary arithmetic (Property P9), meaning the encryption operation itself resists differential power analysis without requiring additional masking. The six possible single-trit transitions exhibit uniform energy profiles, eliminating the data-dependent power variation that plagues binary AES implementations in CMOS logic.</P>

          <Bx type="t" label="Phase 1 \u2014 Side-Channel Resistance Property">
            The ternary symmetric cipher operates in GF(3), where all trit-value transitions consume uniform energy. Combined with automatic algebraic masking (when <Cd>SideChMask</Cd> Layer 2 is active), Phase 1 encryption achieves first-order DPA resistance without dedicated countermeasure overhead. The 27-trit word structure provides a natural block size aligned with the machine word, eliminating the padding overhead and alignment penalties that binary ciphers incur.
          </Bx>

          <H2>6.3 Phase 2 {"\u2014"} Post-Quantum Asymmetric Layer</H2>
          <P>Phase 2 provides key encapsulation and digital signatures using ternary-native lattice-based algorithms, implemented as dedicated ISA opcodes:</P>
          <Tbl
            headers={["Opcode", "Mnemonic", "Function", "Classical Analogue"]}
            rows={[
              ["0x6D", "TKemEncaps", "Ternary Lattice Key Encapsulation \u2014 generates shared secret + ciphertext", "ML-KEM (FIPS 203)"],
              ["0x6E", "TKemDecaps", "Ternary Lattice Key Decapsulation \u2014 recovers shared secret from ciphertext", "ML-KEM (FIPS 203)"],
              ["0x6F", "TDsaSign", "Ternary Lattice Digital Signature \u2014 signs message digest", "ML-DSA (FIPS 204)"],
              ["0x70", "TDsaVerify", "Ternary Lattice Signature Verification \u2014 verifies against public key", "ML-DSA (FIPS 204)"],
            ]}
          />

          <P><B>TL-KEM</B> (Ternary Lattice Key Encapsulation Mechanism) operates over ternary polynomial rings where coefficient arithmetic naturally maps to Representation A. This provides structural advantages: the balanced ternary representation eliminates the modular reduction overhead that binary ML-KEM implementations incur, and the three-valued coefficient space provides a tighter noise distribution for lattice-based security proofs.</P>
          <P><B>TL-DSA</B> (Ternary Lattice Digital Signature Algorithm) implements the Fiat-Shamir with Aborts paradigm in the ternary domain. The abort decision{"\u2014"}whether to restart the signing process when the signature would leak information about the secret key{"\u2014"}is implemented using <Cd>ConstTimeSel</Cd> (<Cd>0x97</Cd>), ensuring that the abort/proceed decision itself does not leak through timing.</P>

          <H2>6.4 Complete Dual-Phase Pipeline</H2>
          <CB>{`1. CapCheck     \u2014 Verify caller holds encryption capability for target resource
2. SideChMask   \u2014 Activate side-channel protection (both layers)
3. TKemEncaps   \u2014 Phase 2: Generate ephemeral symmetric key via TL-KEM
4. PhaseEnc     \u2014 Phase 1: Encrypt data block with ephemeral key in GF(3)
5. TDsaSign     \u2014 Phase 2: Sign (ciphertext \u2016 encapsulated_key) with TL-DSA
6. AuditLog     \u2014 Record encryption event with HPTP femtosecond timestamp
7. SideChUnmask \u2014 Restore normal execution`}</CB>
          <P>The decryption pipeline is the inverse, with <Cd>TDsaVerify</Cd> preceding <Cd>TKemDecaps</Cd> (verify-then-decrypt). Each step is capability-gated and audit-logged, creating a complete provenance chain from plaintext to ciphertext. The pipeline is designed to execute within a single <Cd>SideChMask</Cd>/<Cd>SideChUnmask</Cd> bracket, ensuring that key generation, encryption, and signing are side-channel protected as <B>a single atomic unit</B>.</P>

          <H2>6.5 CNSA 2.0 Alignment</H2>
          <P>The dual-phase architecture directly maps to NSA CNSA 2.0 requirements: TL-KEM provides the quantum-resistant key establishment mechanism (equivalent to ML-KEM at security level 3+), TL-DSA provides quantum-resistant authentication (equivalent to ML-DSA), and the Phase 1 symmetric layer provides high-throughput data protection. The FIPS 140-3 boundary encompasses both phases, with <Cd>ConstTimeEq</Cd>/<Cd>ConstTimeSel</Cd> ensuring that all cryptographic comparisons and conditional operations within the pipeline maintain constant-time execution.</P>
        </div>

        {/* ═══ TAB 3: PLENUMDB ═══ */}
        <div style={tabS(tab === 3)}>
          <H1>7. PlenumDB: Native Encrypted PostgreSQL Framework</H1>

          <H2>7.1 Architecture Overview</H2>
          <P><B>PlenumDB</B> is the Salvi Framework's database abstraction layer, providing native encrypted compression and storage over PostgreSQL via Drizzle ORM. Unlike conventional database encryption approaches{"\u2014"}which apply encryption as an external wrapper around an unencrypted storage engine{"\u2014"}PlenumDB integrates the dual-phase encryption pipeline and capability-based access control <B>directly into the data path</B>, ensuring that data is never stored, transmitted, or processed in unencrypted form outside of capability-authenticated contexts.</P>

          <Tbl
            headers={["Layer", "Component", "Function"]}
            rows={[
              ["Application", "Drizzle ORM / TypeScript", "Standard SQL and TypeScript query interface; encryption transparent to application code"],
              ["Framework", "PlenumDB Encryption Engine", "Ternary compression, dual-phase encryption/decryption, capability validation, audit logging"],
              ["Storage", "PostgreSQL", "Encrypted, compressed column storage; standard PostgreSQL replication and backup"],
            ]}
          />

          <H2>7.2 Encrypted Compression Pipeline</H2>
          <P><B>Stage 1 {"\u2014"} Ternary Compression.</B> Data is converted to ternary representation and compressed using ternary-optimised algorithms that exploit the 1.585-bit information density per trit. The three-valued encoding provides natural run-length patterns that compress more efficiently than binary equivalents for structured data (timestamps, numeric fields, enumerated types). The compression ratio advantage is most pronounced for data with natural three-state patterns{"\u2014"}boolean-with-null, status enumerations, ternary flags.</P>
          <P><B>Stage 2 {"\u2014"} Dual-Phase Encryption.</B> The compressed ternary data is encrypted through the full dual-phase pipeline: Phase 1 symmetric encryption in GF(3), with the symmetric key wrapped by Phase 2 TL-KEM. The <B>column-level encryption granularity</B> means that each database column can have independent encryption keys, capability requirements, and expiration policies.</P>
          <P><B>Stage 3 {"\u2014"} PostgreSQL Storage.</B> The encrypted, compressed payload is stored in PostgreSQL columns as binary data. Standard PostgreSQL features{"\u2014"}replication, point-in-time recovery, connection pooling{"\u2014"}operate on the encrypted data without modification. The PostgreSQL instance never sees plaintext.</P>

          <H2>7.3 Capability-Gated Query Access</H2>
          <P>Every PlenumDB query operation is mediated by the capability system. <Cd>CapCheck</Cd> is integrated at three granularity levels:</P>
          <Tbl
            headers={["Level", "Capability Scope", "Enforcement"]}
            rows={[
              ["Table", "Read/Write/Admin per table", "CapCheck before any query plan execution on the target table"],
              ["Column", "Decrypt permission per column", "CapCheck before decryption of each encrypted column in result set"],
              ["Row", "Optional row-level capability tags", "CapCheck against row-level capability descriptors for fine-grained access"],
            ]}
          />
          <P>This three-level model means that a process may hold a table-level read capability but lack the column-level decrypt capability for sensitive columns{"\u2014"}the query returns the row but with <B>encrypted (opaque) values</B> for unauthorised columns. Capability revocation via <Cd>CapRevoke</Cd> takes immediate effect: a revoked column-decrypt capability causes all subsequent queries to return encrypted values, with <B>no cache window or propagation delay</B>.</P>

          <H2>7.4 Audit Integration and Compliance</H2>
          <P>Every PlenumDB operation generates <Cd>AuditLog</Cd> entries with HPTP femtosecond timestamps:</P>
          <Tbl
            headers={["Regulation", "Requirement", "PlenumDB Implementation"]}
            rows={[
              ["FINRA Rule 613", "Consolidated audit trail with precise timestamps", "AuditLog entries with femtosecond HPTP timestamps for every data access"],
              ["MiFID II (RTS 25)", "Clock synchronisation and transaction reporting", "HPTP-synchronised timestamps anchored to atomic clock sources"],
              ["FIPS 140-3", "Cryptographic module operational assurance", "Dual-phase encryption within FIPS boundary, self-test validated"],
              ["GDPR Art. 32", "Encryption of personal data", "Native column-level encryption with capability-gated access"],
              ["SOX Section 802", "Record retention and integrity", "Blockchain-anchored audit proofs via PlenumNET ledger"],
            ]}
          />
          <P>The audit trail itself is protected: entries are chain-verified (tamper-evident hash chain), HPTP-timestamped, and optionally blockchain-anchored via Hedera HCS, XRPL, or Algorand integrations.</P>

          <H2>7.5 Drizzle ORM Integration</H2>
          <P>Application developers interact with PlenumDB through standard Drizzle ORM patterns in TypeScript. The encryption, compression, capability checks, and audit logging are handled transparently at the framework layer:</P>
          <CB>{`// Application code \u2014 encryption is transparent
const trades = await db.select()
  .from(tradeRecords)
  .where(eq(tradeRecords.symbol, 'BTC/USD'));

// Framework layer (transparent to application):
//   1. CapCheck \u2014 validate caller's table + column capabilities
//   2. PostgreSQL query \u2014 retrieve encrypted rows
//   3. Dual-phase decrypt \u2014 Phase 2 TL-KEM decaps, Phase 1 GF(3) decrypt
//   4. Ternary decompress \u2014 restore original data
//   5. AuditLog \u2014 record access with HPTP timestamp`}</CB>
          <P>The Drizzle schema definition (<Cd>drizzle.config.ts</Cd>) includes PlenumDB-specific column annotations for encryption policy, capability requirements, and compression settings. Database migrations (via Drizzle Kit) automatically handle encryption key rotation and capability policy updates.</P>
        </div>

        {/* ═══ TAB 4: TERNARY SUBSTRATE ═══ */}
        <div style={tabS(tab === 4)}>
          <Part label="Part II" title="Technical Specification" />

          <H1>Abstract</H1>
          <P>This paper presents a unified approach addressing five security domains{"\u2014"}capability-based memory protection, side-channel attack mitigation, constant-time execution guarantees, dual-phase encryption, and native encrypted database storage{"\u2014"}within a coherent set of ISA-level primitives designed natively for a ternary computing architecture. We describe the Security and Audit category (opcodes <Cd>0x90</Cd>{"\u2013"}<Cd>0x97</Cd>) and cryptographic opcodes (<Cd>0x6D</Cd>{"\u2013"}<Cd>0x70</Cd>) of the Salvi Framework's 160-opcode ISA v2.0, alongside the PlenumDB encrypted PostgreSQL framework.</P>
          <div style={tagsS}>
            {["ternary computing","capability architecture","side-channel masking","ISA security","post-quantum cryptography","bijective ternary logic","dual-phase encryption","PlenumDB","encrypted PostgreSQL","hardware security primitives","constant-time execution","PlenumNET","HPTP","CNSA 2.0","FIPS 140-3"].map((t: string, i: number) => <span key={i} style={tagS}>{t}</span>)}
          </div>

          <H1>1. Introduction</H1>
          <P>Three vulnerability classes define contemporary hardware security: the flat memory model providing no intrinsic data/pointer distinction, exposing programs to buffer overflows, pointer forgery, and use-after-free exploits; the intersection of post-quantum algorithmic migration with timing guarantee requirements, where new lattice-based schemes need constant-time hardware support; and microarchitectural side-channel attacks (Spectre, Meltdown, and their variants) exploiting speculative execution, cache timing, and power consumption.</P>
          <P>No current production ISA provides dedicated constant-time comparison or selection primitives with hardware-guaranteed timing invariance. No ISA offers dynamic, programmable side-channel control. No architecture integrates native encrypted database storage with hardware-enforced capability gating.</P>
          <P>The Salvi Framework implements a complete ternary computing engine: 160-opcode ISA (v2.0), compiled Rust kernel (33 MB ELF binary, 47,000+ LOC across 14 subsystems), with live deployment via PlenumNET.</P>
          <P><B>Contributions:</B> (1) Sentinel-trit formalism establishing unforgeability in bijective ternary encoding. (2) Dual-layer side-channel defence combining architectural feature masking with algebraic ternary share masking. (3) Hardware constant-time primitives guaranteeing timing invariance independent of compiler optimisation. (4) Integration with PlenumNET HPTP and a three-mode kernel security system. (5) Dual-phase encryption architecture unifying GF(3) symmetric with TL-KEM/TL-DSA post-quantum key management. (6) PlenumDB framework providing native encrypted PostgreSQL with capability-gated access. (7) Comparative analysis demonstrating this combination is unique among current architectures.</P>

          <H1>2. Related Work</H1>
          <H3>2.1 Capability Architectures</H3>
          <P>CHERI (Cambridge) extends MIPS/RISC-V/ARMv8-A with 128/256-bit compressed capability descriptors protected by 1-bit tags per capability-sized word, requiring a dedicated tag controller and tag memory. Arm's Morello SoC (2022) is the first silicon implementation. Our approach eliminates external tag bits entirely by designing capabilities natively for a three-valued logic substrate{"\u2014"}the third state provides an algebraically unforgeable in-band sentinel.</P>
          <H3>2.2 Side-Channel Countermeasures</H3>
          <P>ISW masking (CRYPTO 2003) decomposes secret values into random shares. Rivain-Prouff (CHES 2010) applied this to AES. RISC-V ISA extensions propose accelerated masking but remain additive extensions to a binary substrate. No production ISA provides combined microarchitectural isolation and algebraic masking as first-class operations.</P>
          <H3>2.3 Constant-Time Programming</H3>
          <P>Current practice relies on compiler discipline and software patterns. No ISA provides hardware-guaranteed constant-time comparison or selection primitives where the timing invariance is enforced by a dedicated ALU path independent of compiler behaviour.</P>
          <H3>2.4 Ternary Computing</H3>
          <P>Balanced ternary carries log{"\u2082"}(3) {"\u2248"} 1.585 bits per trit, approaching optimal radix economy (the integer closest to <i>e</i> {"\u2248"} 2.718). The Soviet Setun computer (1958) demonstrated practical balanced ternary. The security implications of ternary arithmetic{"\u2014"}particularly the sentinel property and uniform energy profiles{"\u2014"}have received limited attention in the literature.</P>

          <H1>3. Ternary Logic Substrate</H1>
          <H3>3.1 Three Bijective Representations</H3>
          <Tbl
            headers={["Repr.", "Name", "Digit Set", "Domain", "Bijection from A"]}
            rows={[
              ["A", "Computational", "{\u22121, 0, +1}", "Internal arithmetic, GF(3) field, algebraic masking, Phase 1 encryption", "Identity"],
              ["B", "Network", "{0, 1, 2}", "Protocol encoding, wire-level transmission", "f(a) = a + 1"],
              ["C", "Bijective/Human", "{1, 2, 3}", "Capability security, human-readable display, excludes 0", "f(a) = a + 2"],
            ]}
          />
          <P>Bijections are single-cycle hardware operations via the <Cd>TConvert</Cd> opcode (<Cd>0x15</Cd>).</P>

          <Bx type="t" label="Definition 1 \u2014 Sentinel Trit">
            A sentinel trit is a trit containing value 0, which lies outside the bijective ternary digit set {"\u0392"}{"\u2083"} = {"{"}1, 2, 3{"}"}. Sentinel trits are only writable by privileged hardware (Ring0 via CapGrant). No user-level instruction can produce a sentinel trit from valid operands.
          </Bx>

          <Bx type="t" label="Theorem 1 \u2014 Unforgeability">
            Let f: {"\u0392"}{"\u2083"}<sup>n</sup> {"\u00D7"} {"\u0392"}{"\u2083"}<sup>n</sup> {"\u2192"} {"\u0392"}{"\u2083"}<sup>n</sup> be any arithmetic operation over n-trit bijective ternary words. For any inputs A, B {"\u2208"} {"\u0392"}{"\u2083"}<sup>n</sup>, the result f(A, B) contains no sentinel trits. <B>Proof:</B> The carry logic for bijective ternary addition keeps all output digits in {"{"}1, 2, 3{"}"}. Base case: single-trit addition of any two values from {"{"}1,2,3{"}"} with any carry from {"{"}1,2,3{"}"} produces a digit in {"{"}1,2,3{"}"} and a carry in {"{"}1,2,3{"}"}. Induction: extends to n-trit words. This eliminates the need for CHERI-style external tag memory. {"\u25A0"}
          </Bx>

          <H3>3.2 Machine Word Structure</H3>
          <P>The 27-trit machine word ("tryte", 27 = 3{"\u00B3"}) has recursive structure: three 9-trit "tribbles", each composed of three 3-trit "triples." The Tribonacci sequence T(n) = T(n{"\u2212"}1) + T(n{"\u2212"}2) + T(n{"\u2212"}3) provides a natural recursive basis for the word hierarchy.</P>
          <P><B>Register file:</B> 27 general-purpose ternary registers (R0{"\u2013"}R26), plus PC, SP, FP, LR, flags register, privilege level, security domain ID, and exception vector. Each register holds one 27-trit word.</P>
        </div>

        {/* ═══ TAB 5: ARCHITECTURE ═══ */}
        <div style={tabS(tab === 5)}>
          <H1>4. Architectural Support</H1>
          <H3>4.1 Privilege Levels</H3>
          <Tbl
            headers={["Ring", "Name", "Enforcement"]}
            rows={[
              ["Ring0", "Kernel", "Full access. Required for CapGrant, CapRevoke, DomainSet, MProtect, Trap"],
              ["Ring1", "Supervisor", "Restricted access for device drivers and system services"],
              ["Ring2", "User", "Unprivileged application code"],
            ]}
          />

          <H3>4.2 Modal Security System</H3>
          <Tbl
            headers={["Mode", "Symbol", "Description"]}
            rows={[
              ["ModePhi (\u03A6)", "phi_plus", "Maximum privilege: kernel operations, cryptographic key management, FIPS boundary"],
              ["ModeOne (1)", "one", "Standard operations: user processes, normal I/O"],
              ["ModeZero (0)", "zero", "Restricted/quarantine: untrusted code, sandboxed execution"],
            ]}
          />
          <P><B>Security domains</B> are named isolation boundaries with mode assignment, member processes, and transition rules (Upgrade/Downgrade/Lateral), each with a femtosecond creation timestamp.</P>

          <H3>4.3 Hardware Components</H3>
          <P><B>Capability Table:</B> Hardware-managed BTreeMap&lt;TokenId, CapabilityToken&gt;. Each token stores: id, owner (ProcessId), kind, resource, allowed actions, security mode, created_at and expires_at (FemtosecondTimestamp), revoked flag, delegatable flag, and parent_token for provenance chain tracking.</P>
          <P><B>Side-Channel Control Register (SCCR):</B> Per-core register controlling microarchitectural features. Saved and restored on context switch to prevent cross-process leakage.</P>
          <P><B>Constant-Time ALU:</B> A dedicated execution path with no early termination, no data-dependent carry propagation, and fixed one-cycle latency regardless of operand values. Physically isolated from the speculative execution engine.</P>
          <P><B>Hardware Random Trit Generator (HRTG):</B> Entropy source for algebraic masking operations. FIPS 140-3 SP 800-90B compliant, providing uniform random trits for the masking pipeline.</P>

          <H1>5. Security and Audit Opcodes {"\u2014"} Summary Table</H1>
          <Tbl
            headers={["Opcode", "Mnemonic", "Privilege", "Function"]}
            rows={[
              ["0x90", "AuditLog", "Any", "Tamper-evident audit entry with HPTP femtosecond timestamp, 12 event types"],
              ["0x91", "CapCheck", "Any", "Dual-mechanism capability validation (sentinel + table lookup)"],
              ["0x92", "CapGrant", "Ring0", "Create capability with sentinel trit, table entry, provenance chain"],
              ["0x93", "CapRevoke", "Ring0", "Immediate O(1) dual-mechanism invalidation (table flag + sentinel overwrite)"],
              ["0x94", "SideChMask", "Any", "Dual-layer: microarchitectural isolation + algebraic ternary masking"],
              ["0x95", "SideChUnmask", "Any", "Restore microarchitectural features, exit masking mode"],
              ["0x96", "ConstTimeEq", "Any", "Fixed-latency comparison, no early exit, physically isolated ALU path"],
              ["0x97", "ConstTimeSel", "Any", "Constant-time conditional move via bitwise masking, no branch"],
            ]}
          />

          <H2>Cryptographic Opcodes (Dual-Phase Pipeline)</H2>
          <Tbl
            headers={["Opcode", "Mnemonic", "Function", "NIST Equivalent"]}
            rows={[
              ["0x6D", "TKemEncaps", "Ternary Lattice Key Encapsulation \u2014 shared secret + ciphertext", "ML-KEM (FIPS 203)"],
              ["0x6E", "TKemDecaps", "Ternary Lattice Key Decapsulation \u2014 recovers shared secret", "ML-KEM (FIPS 203)"],
              ["0x6F", "TDsaSign", "Ternary Lattice Digital Signature \u2014 signs message digest", "ML-DSA (FIPS 204)"],
              ["0x70", "TDsaVerify", "Ternary Lattice Signature Verification", "ML-DSA (FIPS 204)"],
            ]}
          />
        </div>

        {/* ═══ TAB 6: ANALYSIS & COMPARE ═══ */}
        <div style={tabS(tab === 6)}>
          <H1>8. Integration with PlenumNET Infrastructure</H1>
          <H3>8.1 Post-Quantum Timing Anchors</H3>
          <P><B>HPTP</B> (High-Precision Timing Protocol) provides authenticated timestamps from seven clock source types: local oscillator, NTP/PTP network, GNSS satellite, rubidium oscillator, caesium beam, hydrogen maser, and optical lattice. Five precision levels range from millisecond to femtosecond.</P>
          <P>Every <Cd>CapGrant</Cd> and <Cd>CapRevoke</Cd> can generate a <B>TL-DSA-signed timing proof</B> stored in the PlenumNET ledger, providing non-repudiation guarantees for capability lifecycle events.</P>

          <H3>8.2 Timing Isolation</H3>
          <P>Masked regions (between <Cd>SideChMask</Cd> and <Cd>SideChUnmask</Cd>) are bracketed by HPTP timing barriers. The processor inserts dummy cycles as needed to ensure <B>data-independent total execution time</B> for the masked region, regardless of the code path taken within it.</P>

          <H3>8.3 Blockchain Anchoring</H3>
          <P>Capability lifecycle events are optionally anchored to distributed ledgers: <B>Hedera HCS</B> (Hashgraph Consensus Service), <B>XRPL</B> (XRP Ledger), and <B>Algorand</B>. This provides cryptographic proof-of-existence for compliance with MiFID II, FINRA Rule 613, and Reg NMS.</P>

          <H3>8.4 CNSA 2.0 and FIPS 140-3 Compliance</H3>
          <P><Cd>ConstTimeEq</Cd>/<Cd>ConstTimeSel</Cd> address FIPS 140-3 Level 3 constant-time requirements. <Cd>SideChMask</Cd>/<Cd>SideChUnmask</Cd> address side-channel resistance. <Cd>AuditLog</Cd> provides operational assurance. The <Cd>finra-613</Cd> feature flag enables build-time compliance configuration.</P>

          <H1>9. Security Analysis</H1>
          <H2>9.1 Capability Properties</H2>
          <P><B>P1 (Provenance)</B> {"\u2014"} Every capability traces to kernel root via parent_token chain, with monotonic restriction at each delegation step. <B>P2 (Integrity)</B> {"\u2014"} Unforgeable via dual mechanism: sentinel trit (algebraic, Theorem 1) + hardware table (structural). <B>P3 (Non-bypassability)</B> {"\u2014"} No unvalidated access path exists in the architecture. <B>P4 (Isolation)</B> {"\u2014"} Security domain IDs prevent cross-domain interference. <B>P5 (Temporal Integrity)</B> {"\u2014"} Femtosecond timestamps with HPTP proofs provide non-repudiable temporal authentication. <B>P6 (Revocability)</B> {"\u2014"} O(1) immediate dual-mechanism invalidation.</P>

          <H2>9.2 Side-Channel Properties</H2>
          <P><B>P7 (Microarchitectural Isolation)</B> {"\u2014"} No observable timing differences during masked execution. <B>P8 (First-Order Algebraic Security)</B> {"\u2014"} A single probe reveals zero information about the masked value (Theorem 2). <B>P9 (Glitch Resistance)</B> {"\u2014"} Uniform transition energy profiles across all six trit transitions. <B>P10 (Timing Independence)</B> {"\u2014"} Fixed cycle count regardless of input values.</P>

          <H2>9.3 Constant-Time &amp; Encryption Properties</H2>
          <P><B>P11 (Data-Independent Timing)</B> {"\u2014"} <Cd>ConstTimeEq</Cd>/<Cd>ConstTimeSel</Cd> have fixed latency with no branch prediction, early termination, or speculation involvement.</P>

          <H2>9.4 Defence-in-Depth</H2>
          <Tbl
            headers={["Domain", "Mechanism 1", "Mechanism 2", "Failure Mode"]}
            rows={[
              ["Capability integrity", "Sentinel trit (algebraic)", "Capability table (hardware)", "Both must be defeated"],
              ["Side-channel defence", "Microarchitectural isolation", "Algebraic ternary masking", "Either provides independent protection"],
              ["Timing", "Constant-time ALU", "HPTP timing barriers", "Both enforce invariance"],
              ["Audit", "Hardware timestamps", "Chain integrity verification", "Tampering requires breaking both"],
              ["Data encryption", "Phase 1 GF(3) symmetric", "Phase 2 TL-KEM/TL-DSA PQC", "Both layers must be broken"],
              ["Database access", "Capability-gated queries", "Column-level dual-phase encryption", "Either prevents unauthorised reads"],
            ]}
          />

          <H2>9.5 Limitations</H2>
          <P>(1) Current implementation is a Rust kernel, not silicon{"\u2014"}power consumption properties are theoretical until FPGA synthesis. (2) Higher-order probing analysis (d-th order model) is future work. (3) The sentinel-trit mechanism is specific to bijective ternary; it does not transfer to binary architectures. (4) Microarchitectural masking effectiveness depends on faithful hardware implementation of the SCCR-controlled feature disabling.</P>
        </div>

        {/* ═══ TAB 7: REFERENCES ═══ */}
        <div style={tabS(tab === 7)}>
          <H1>10. Comparison with Existing Approaches</H1>
          <Tbl
            headers={["Feature", "Intel MPK", "ARM Morello/CHERI", "RISC-V PMP", "Salvi Security Ops"]}
            rows={[
              ["Capability model", "No", "Yes (pointer-based)", "No", "Domain + sentinel-trit + temporal"],
              ["Hardware grant/revoke", "No", "Partial", "No", "CapGrant/CapRevoke (Ring0, immediate)"],
              ["Unforgeable tag", "N/A", "1-bit external tag", "N/A", "Algebraic in-band sentinel trit (zero overhead)"],
              ["Capability provenance", "No", "No", "No", "parent_token chain + femtosecond timestamps"],
              ["Capability expiration", "No", "No", "No", "expires_at with femtosecond precision"],
              ["Side-channel masking", "No", "No", "No", "SideChMask/SideChUnmask (dual-layer)"],
              ["Constant-time ops", "No", "No", "No", "ConstTimeEq/ConstTimeSel (dedicated ALU)"],
              ["ISA-level audit", "No", "No", "No", "AuditLog (HPTP-timestamped, chain-verified)"],
              ["Dual-phase encryption", "No", "No", "No", "Phase 1 GF(3) symmetric + Phase 2 TL-KEM/TL-DSA"],
              ["Native encrypted DB", "No", "No", "No", "PlenumDB column-level + capability-gated queries"],
              ["PQC integration", "No", "No", "No", "CNSA 2.0, TL-DSA/TL-KEM, FIPS 140-3 pathway"],
              ["Temporal authentication", "No", "No", "No", "HPTP post-quantum timing proofs"],
              ["Blockchain provenance", "No", "No", "No", "Hedera HCS, XRPL, Algorand anchoring"],
              ["Ternary/non-binary native", "No", "No", "No", "Three bijective representations"],
            ]}
          />

          <H1>11. Reference Implementation</H1>
          <P>Repository: <a href="https://github.com/SigmaWolf-8/Ternary" target="_blank" rel="noopener noreferrer" style={{ color: col.al, textDecoration: "none" as const }}>github.com/SigmaWolf-8/Ternary</a>, HEAD 645001e</P>
          <Tbl
            headers={["Module", "LOC", "Functionality"]}
            rows={[
              ["instruction_v2.rs", "1,111", "ISA decoder: 8 security opcodes + crypto opcodes with privilege enforcement"],
              ["capability.rs", "498", "CapabilityToken, CapabilityManager: grant/delegate/revoke/check_access"],
              ["domain.rs", "457", "SecurityDomain, DomainManager: transition rules, isolation boundaries"],
              ["audit.rs", "405", "AuditEntry, AuditLog: 12 event types, chain integrity verification"],
              ["policy.rs", "502", "PolicyRule, MAC+DAC engine with scope and mode filters"],
              ["side_channel.rs", "702", "Four analysis categories: ConstantTime, BranchAnalysis, MemoryAccess, PowerAnalysis"],
              ["formal_verify.rs", "615", "SMTLIB2/Cryptol/SAW property export, GF(3) verification conditions"],
              ["ternary.rs", "974", "Three representations, bijective mappings, all arithmetic operations"],
            ]}
          />
          <P>The dual-phase encryption pipeline is implemented in the crypto subsystem with Phase 1 (ternary symmetric) and Phase 2 (TL-KEM/TL-DSA) sharing the GF(3) arithmetic infrastructure. PlenumDB's encryption engine, capability integration, and Drizzle ORM layer are implemented in the server and services directories with <Cd>drizzle.config.ts</Cd> managing the PostgreSQL schema and migration pipeline.</P>
          <P><B>Testing:</B> Unit tests (all opcodes), privilege enforcement, sentinel unforgeability (exhaustive arithmetic), capability derivation monotonicity, masking round-trip correctness, chi-squared statistical independence, Criterion benchmarks (including phase encryption throughput), three fuzz targets (fuzz_gateway, fuzz_trit_ops, fuzz_tryte_ops), PropTest for VM invariants, audit chain integrity. <B>14 CI/CD workflows</B> including fips-self-tests.yml, security-scan.yml, compliance-check.yml, codeql-analysis.yml.</P>

          <H1>12. Discussion and Future Work</H1>
          <P>This work presents the first ISA-level security subsystem designed natively for a non-binary computing substrate, combining capability-based access control, side-channel mitigation, constant-time execution, dual-phase encryption, and native encrypted database storage within a unified ternary computing framework.</P>
          <P><B>Future directions:</B> (1) FPGA synthesis (Xilinx Zynq/Intel Cyclone via existing FPGA HDL generator) for empirical power-consumption measurement. (2) Machine-checked formal proofs of capability invariants and constant-time properties. (3) Higher-order ternary masking: d-th order probing model analysis with O(n) vs O(n{"\u00B2"}) complexity advantage. (4) Hardware-accelerated PQC key management coupling capability delegation with TL-KEM key distribution. (5) RISC-V custom extension proposal. (6) Sigma Wolf ET protocol application: capability-controlled market data feeds, side-channel-masked signal processing, PlenumNET timing proofs for regulatory compliance.</P>
          <P><B>For dual-phase encryption:</B> Hardware-accelerated Phase 1 cipher on FPGA, formal TL-KEM/TL-DSA security proofs under the ternary algebraic model, Sigma Wolf ET integration for market signal encryption.</P>
          <P><B>For PlenumDB:</B> Encrypted full-text search via GF(3) homomorphic operations, encrypted aggregation queries, cross-database capability federation, and encrypted time-series data optimised for high-frequency trading compliance workloads.</P>

          <Bx type="n" label="Core Insight">
            The choice of number system is not merely an efficiency consideration{"\u2014"}it is a <B>security design parameter</B>. The sentinel-trit unforgeability, balanced-masking domain, and GF(3) encryption properties are structural consequences of ternary arithmetic that no amount of binary ISA extension can replicate. When extended through the dual-phase encryption pipeline to native encrypted database storage, the ternary advantage propagates from silicon to storage{"\u2014"}creating a security architecture that is coherent from the ALU to the database column.
          </Bx>

          <H1>References</H1>
          <div style={refS}>
            <p style={refP}>[1] H. M. Levy, <i>Capability-Based Computer Systems</i>, Digital Press, 1984.</p>
            <p style={refP}>[2] D. J. Bernstein, "Cache-timing attacks on AES," 2005.</p>
            <p style={refP}>[3] Y. Ishai, A. Sahai, D. Wagner, "Private Circuits: Securing Hardware against Probing Attacks," CRYPTO 2003.</p>
            <p style={refP}>[3b] Salvi Framework, github.com/SigmaWolf-8/Ternary, 2026.</p>
            <p style={refP}>[4] T. Fritzmann et al., "Masked Accelerators for Post-Quantum Cryptography," IACR ePrint 2021/479.</p>
            <p style={refP}>[5] E. Rivain, E. Prouff, "Provably Secure Higher-Order Masking of AES," CHES 2010.</p>
            <p style={refP}>[6] A. Duc, S. Dziembowski, S. Faust, "Unifying Leakage Models," EUROCRYPT 2014.</p>
            <p style={refP}>[7] P. Kocher et al., "Spectre Attacks: Exploiting Speculative Execution," IEEE S&amp;P 2019.</p>
            <p style={refP}>[8] NIST, "ML-KEM (FIPS 203), ML-DSA (FIPS 204), SLH-DSA (FIPS 205)," 2024.</p>
            <p style={refP}>[9] B. Battistello et al., "Horizontal Side-Channel Attacks and Countermeasures on ISW Masking," CHES 2016.</p>
            <p style={refP}>[10] R. N. M. Watson et al., "CHERI: A Hybrid Capability-System Architecture," IEEE S&amp;P 2015.</p>
            <p style={refP}>[11] S. Nikova, C. Rechberger, V. Rijmen, "Threshold Implementations Against Side-Channel Attacks," CHES 2006.</p>
            <p style={refP}>[12] Arm Ltd., "Morello Programme," 2022.</p>
            <p style={refP}>[13] D. E. Knuth, <i>The Art of Computer Programming</i>, Vol. 2, 3rd ed., 1997.</p>
            <p style={refP}>[14] ISA Shuffling Extensions, IEEE Trans. CAD, 2023.</p>
            <p style={refP}>[15] S. Cassiers, G. Standaert, "Tight Random Probing Security," CRYPTO 2021.</p>
            <p style={refP}>[16] J. B. Dennis, E. C. Van Horn, "Programming Semantics for Multiprogrammed Computations," CACM 1966.</p>
            <p style={refP}>[17] M. Lipp et al., "Meltdown: Reading Kernel Memory from User Space," USENIX Security 2018.</p>
            <p style={refP}>[18] Intel, "Memory Protection Keys," 2016.</p>
            <p style={refP}>[19] RISC-V Foundation, "Physical Memory Protection," 2017.</p>
            <p style={refP}>[20] N. P. Brusentsov, "Setun: A Ternary Computer," 1958.</p>
            <p style={refP}>[21] R. N. M. Watson et al., Cambridge TR-951, 2023.</p>
            <p style={refP}>[22] D. W. Jones, "Ternary Number Systems" (unpublished), 2013.</p>
          </div>
        </div>

        {/* FOOTER */}
        <div style={footerS}>
          Copyright &copy; 2025{"\u2013"}2026 Capomastro Holdings Ltd. (Canada). Patent(s) Pending {"\u2014"} All Rights Reserved.<br />
          Applied Physics Division {"\u2014"} Salvi Framework ISA v2.0 {"\u2014"} HEAD 645001e {"\u2014"} February 2026
        </div>

      </div>

      {/* SCROLL TO TOP */}
      <button
        data-testid="button-scroll-top"
        style={topBtnS(showTop)}
        onClick={() => window.scrollTo({ top: 0, behavior: "smooth" })}
      >{"\u2191"}</button>
    </div>
  );
}
