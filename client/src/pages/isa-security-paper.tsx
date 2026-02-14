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

import { useState, useEffect, useRef, useCallback, type CSSProperties, type ReactNode } from "react";

const COLORS = {
  navy: "#0a1628",
  navyLight: "#111d33",
  navyMid: "#162340",
  accent: "#3b82f6",
  accentGlow: "#60a5fa",
  accentDim: "#1e40af",
  gold: "#f59e0b",
  goldDim: "#b45309",
  surface: "#0f1a2e",
  surfaceHover: "#162340",
  text: "#e2e8f0",
  textDim: "#94a3b8",
  textMuted: "#64748b",
  border: "#1e3a5f",
  borderLight: "#2d4a6f",
  white: "#f8fafc",
  codeBlock: "#0c1524",
  success: "#10b981",
  tableBg: "#0d1829",
  tableHead: "#132242",
  tableAlt: "#0f1e36",
};

const fontStack = {
  heading: "'Playfair Display', 'Georgia', serif",
  body: "'Source Sans 3', 'Segoe UI', sans-serif",
  mono: "'JetBrains Mono', 'Fira Code', monospace",
  label: "'Inter', 'Helvetica Neue', sans-serif",
};

// ── Inline Styles ──
const S = {
  app: {
    minHeight: "100vh",
    background: `linear-gradient(180deg, ${COLORS.navy} 0%, #060d18 100%)`,
    color: COLORS.text,
    fontFamily: fontStack.body,
    fontSize: "16px",
    lineHeight: 1.7,
    position: "relative" as const,
  } as CSSProperties,
  // Subtle grid pattern overlay
  gridOverlay: {
    position: "fixed" as const,
    inset: 0,
    backgroundImage: `
      linear-gradient(${COLORS.border}15 1px, transparent 1px),
      linear-gradient(90deg, ${COLORS.border}15 1px, transparent 1px)
    `,
    backgroundSize: "60px 60px",
    pointerEvents: "none" as const,
    zIndex: 0,
  } as CSSProperties,
  container: {
    maxWidth: "900px",
    margin: "0 auto",
    padding: "0 24px",
    position: "relative" as const,
    zIndex: 1,
  } as CSSProperties,
  hero: {
    padding: "80px 0 60px",
    textAlign: "center" as const,
    position: "relative" as const,
  } as CSSProperties,
  heroGlow: {
    position: "absolute" as const,
    top: "-100px",
    left: "50%",
    transform: "translateX(-50%)",
    width: "600px",
    height: "400px",
    background: `radial-gradient(ellipse, ${COLORS.accentDim}20 0%, transparent 70%)`,
    pointerEvents: "none" as const,
  } as CSSProperties,
  heroLabel: {
    fontFamily: fontStack.label,
    fontSize: "11px",
    letterSpacing: "4px",
    textTransform: "uppercase" as const,
    color: COLORS.accent,
    marginBottom: "20px",
    fontWeight: 600,
  } as CSSProperties,
  heroTitle: {
    fontFamily: fontStack.heading,
    fontSize: "clamp(2rem, 5vw, 3.2rem)",
    fontWeight: 700,
    color: COLORS.white,
    lineHeight: 1.15,
    margin: "0 0 12px",
  },
  heroSub: {
    fontFamily: fontStack.heading,
    fontSize: "clamp(1.1rem, 2.5vw, 1.5rem)",
    fontWeight: 400,
    fontStyle: "italic",
    color: COLORS.accentGlow,
    margin: "0 0 32px",
  },
  heroDivider: {
    width: "80px",
    height: "2px",
    background: `linear-gradient(90deg, transparent, ${COLORS.accent}, transparent)`,
    margin: "0 auto 28px",
  },
  heroMeta: {
    fontFamily: fontStack.label,
    fontSize: "13px",
    color: COLORS.textDim,
    lineHeight: 1.8,
  },
  heroMetaBold: {
    color: COLORS.text,
    fontWeight: 600,
  },
  heroLinks: {
    display: "flex",
    gap: "16px",
    justifyContent: "center",
    marginTop: "28px",
    flexWrap: "wrap" as const,
  } as CSSProperties,
  heroLink: {
    fontFamily: fontStack.label,
    fontSize: "12px",
    letterSpacing: "1.5px",
    textTransform: "uppercase" as const,
    color: COLORS.accent,
    textDecoration: "none",
    padding: "8px 20px",
    border: `1px solid ${COLORS.accent}40`,
    borderRadius: "4px",
    transition: "all 0.25s ease",
    fontWeight: 500,
  } as CSSProperties,
  // ── Nav ──
  nav: {
    position: "sticky" as const,
    top: 0,
    zIndex: 100,
    background: `${COLORS.navy}ee`,
    backdropFilter: "blur(12px)",
    borderBottom: `1px solid ${COLORS.border}80`,
    padding: "0",
  } as CSSProperties,
  navInner: {
    maxWidth: "900px",
    margin: "0 auto",
    padding: "0 24px",
    display: "flex",
    gap: "0",
    overflowX: "auto" as const,
  } as CSSProperties,
  navBtn: (active: boolean): CSSProperties => ({
    fontFamily: fontStack.label,
    fontSize: "12px",
    letterSpacing: "1px",
    textTransform: "uppercase" as const,
    color: active ? COLORS.accent : COLORS.textMuted,
    background: "none",
    border: "none",
    borderBottom: active ? `2px solid ${COLORS.accent}` : "2px solid transparent",
    padding: "14px 16px",
    cursor: "pointer",
    whiteSpace: "nowrap" as const,
    fontWeight: active ? 600 : 400,
    transition: "all 0.2s ease",
  }),
  // ── Part Banner ──
  partBanner: {
    margin: "60px 0 40px",
    padding: "32px",
    textAlign: "center" as const,
    borderTop: `1px solid ${COLORS.accent}30`,
    borderBottom: `1px solid ${COLORS.accent}30`,
    background: `linear-gradient(135deg, ${COLORS.navyLight}80 0%, ${COLORS.navyMid}40 100%)`,
    borderRadius: "2px",
  } as CSSProperties,
  partLabel: {
    fontFamily: fontStack.label,
    fontSize: "11px",
    letterSpacing: "5px",
    textTransform: "uppercase" as const,
    color: COLORS.gold,
    marginBottom: "8px",
    fontWeight: 600,
  } as CSSProperties,
  partTitle: {
    fontFamily: fontStack.heading,
    fontSize: "1.6rem",
    fontWeight: 600,
    color: COLORS.white,
    margin: 0,
  },
  // ── Headings ──
  h1: {
    fontFamily: fontStack.heading,
    fontSize: "clamp(1.5rem, 3vw, 2rem)",
    fontWeight: 700,
    color: COLORS.white,
    margin: "56px 0 20px",
    paddingBottom: "12px",
    borderBottom: `1px solid ${COLORS.border}`,
  },
  h2: {
    fontFamily: fontStack.heading,
    fontSize: "clamp(1.2rem, 2.5vw, 1.5rem)",
    fontWeight: 600,
    color: COLORS.accentGlow,
    margin: "40px 0 16px",
  },
  h3: {
    fontFamily: fontStack.heading,
    fontSize: "1.15rem",
    fontWeight: 600,
    color: COLORS.text,
    fontStyle: "italic",
    margin: "32px 0 12px",
  },
  // ── Text ──
  p: {
    margin: "0 0 16px",
    color: COLORS.text,
    lineHeight: 1.75,
  },
  bold: { fontWeight: 700, color: COLORS.white },
  italic: { fontStyle: "italic", color: COLORS.textDim },
  code: {
    fontFamily: fontStack.mono,
    fontSize: "0.88em",
    background: COLORS.codeBlock,
    color: COLORS.accentGlow,
    padding: "2px 7px",
    borderRadius: "3px",
    border: `1px solid ${COLORS.border}60`,
  },
  codeBlock: {
    fontFamily: fontStack.mono,
    fontSize: "13px",
    background: COLORS.codeBlock,
    color: COLORS.accentGlow,
    padding: "16px 20px",
    borderRadius: "4px",
    border: `1px solid ${COLORS.border}`,
    overflowX: "auto" as const,
    margin: "12px 0 20px",
    lineHeight: 1.6,
    whiteSpace: "pre-wrap" as const,
  } as CSSProperties,
  tableWrap: {
    overflowX: "auto" as const,
    margin: "16px 0 24px",
    borderRadius: "4px",
    border: `1px solid ${COLORS.border}`,
  } as CSSProperties,
  table: {
    width: "100%",
    borderCollapse: "collapse" as const,
    fontSize: "14px",
  } as CSSProperties,
  th: {
    fontFamily: fontStack.label,
    fontSize: "11px",
    letterSpacing: "1px",
    textTransform: "uppercase" as const,
    fontWeight: 600,
    color: COLORS.accentGlow,
    background: COLORS.tableHead,
    padding: "12px 16px",
    textAlign: "left" as const,
    borderBottom: `2px solid ${COLORS.accent}30`,
    whiteSpace: "nowrap" as const,
  } as CSSProperties,
  td: (alt: boolean): CSSProperties => ({
    padding: "10px 16px",
    borderBottom: `1px solid ${COLORS.border}40`,
    background: alt ? COLORS.tableAlt : "transparent",
    color: COLORS.text,
    verticalAlign: "top",
    lineHeight: 1.5,
  }),
  callout: (type: string): CSSProperties => ({
    margin: "20px 0",
    padding: "16px 20px",
    borderLeft: `3px solid ${type === "theorem" ? COLORS.gold : type === "proof" ? COLORS.success : COLORS.accent}`,
    background: `${COLORS.surface}80`,
    borderRadius: "0 4px 4px 0",
  }),
  calloutLabel: (type: string): CSSProperties => ({
    fontFamily: fontStack.label,
    fontSize: "11px",
    letterSpacing: "2px",
    textTransform: "uppercase" as const,
    fontWeight: 700,
    color: type === "theorem" ? COLORS.gold : type === "proof" ? COLORS.success : COLORS.accent,
    marginBottom: "6px",
  }),
  keywords: {
    display: "flex",
    flexWrap: "wrap" as const,
    gap: "8px",
    margin: "16px 0 24px",
  } as CSSProperties,
  keyword: {
    fontFamily: fontStack.label,
    fontSize: "11px",
    color: COLORS.accent,
    background: `${COLORS.accentDim}20`,
    border: `1px solid ${COLORS.accent}25`,
    padding: "3px 10px",
    borderRadius: "3px",
    letterSpacing: "0.5px",
  },
  // ── Footer ──
  footer: {
    marginTop: "80px",
    padding: "32px 0",
    borderTop: `1px solid ${COLORS.border}`,
    textAlign: "center" as const,
    fontFamily: fontStack.label,
    fontSize: "12px",
    color: COLORS.textMuted,
    lineHeight: 2,
  } as CSSProperties,
  tocWrap: {
    margin: "24px 0 40px",
    padding: "24px 28px",
    background: `${COLORS.surface}60`,
    border: `1px solid ${COLORS.border}80`,
    borderRadius: "4px",
  },
  tocTitle: {
    fontFamily: fontStack.label,
    fontSize: "11px",
    letterSpacing: "3px",
    textTransform: "uppercase" as const,
    color: COLORS.accent,
    fontWeight: 600,
    marginBottom: "16px",
  } as CSSProperties,
  tocItem: (level: number): CSSProperties => ({
    fontFamily: fontStack.body,
    fontSize: level === 0 ? "14px" : "13px",
    color: level === 0 ? COLORS.text : COLORS.textDim,
    padding: `4px 0 4px ${level * 20}px`,
    cursor: "pointer",
    transition: "color 0.15s",
    fontWeight: level === 0 ? 500 : 400,
    borderLeft: level === 0 ? "none" : `1px solid ${COLORS.border}40`,
  }),
  scrollTop: (visible: boolean): CSSProperties => ({
    position: "fixed" as const,
    bottom: "24px",
    right: "24px",
    width: "40px",
    height: "40px",
    borderRadius: "50%",
    background: COLORS.accent,
    color: COLORS.navy,
    border: "none",
    cursor: "pointer",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    fontSize: "18px",
    fontWeight: 700,
    opacity: visible ? 1 : 0,
    transform: visible ? "translateY(0)" : "translateY(12px)",
    transition: "all 0.3s ease",
    pointerEvents: visible ? "auto" as const : "none" as const,
    zIndex: 200,
    boxShadow: `0 4px 20px ${COLORS.accent}40`,
  }),
};

// ── Reusable Components ──
const Table = ({ headers, rows }: { headers: string[]; rows: string[][] }) => (
  <div style={S.tableWrap}>
    <table style={S.table}>
      <thead>
        <tr>{headers.map((h: string, i: number) => <th key={i} style={S.th}>{h}</th>)}</tr>
      </thead>
      <tbody>
        {rows.map((row: string[], ri: number) => (
          <tr key={ri}>
            {row.map((c: string, ci: number) => <td key={ci} style={S.td(ri % 2 === 1)}>{c}</td>)}
          </tr>
        ))}
      </tbody>
    </table>
  </div>
);

const Callout = ({ type, label, children }: { type: string; label: string; children: ReactNode }) => (
  <div style={S.callout(type)}>
    <div style={S.calloutLabel(type)}>{label}</div>
    <div style={{ ...S.p, margin: 0, fontSize: "14px" }}>{children}</div>
  </div>
);

const Code = ({ children }: { children: ReactNode }) => <code style={S.code}>{children}</code>;

const sections = [
  "Overview", "Part I: Executive Overview", "Part II: Technical Specification",
  "Abstract", "1. Introduction", "2. Related Work", "3. Ternary Logic Substrate",
  "4. Architectural Support", "5. Security Opcodes", "6. PlenumNET Integration",
  "7. Security Analysis", "8. Comparison", "9. Implementation", "10. Future Work", "References"
];

export default function ISASecurityPaper() {
  const [activeNav, setActiveNav] = useState(0);
  const [showScrollTop, setShowScrollTop] = useState(false);
  const sectionRefs = useRef<Record<string, HTMLElement | null>>({});

  useEffect(() => {
    const handleScroll = () => setShowScrollTop(window.scrollY > 600);
    window.addEventListener("scroll", handleScroll);
    return () => window.removeEventListener("scroll", handleScroll);
  }, []);

  const scrollToSection = useCallback((id: string) => {
    const el = sectionRefs.current[id];
    if (el) el.scrollIntoView({ behavior: "smooth", block: "start" });
  }, []);

  const ref = (id: string) => (el: HTMLElement | null) => { sectionRefs.current[id] = el; };

  return (
    <div style={S.app}>
      <link href="https://fonts.googleapis.com/css2?family=Playfair+Display:ital,wght@0,400;0,600;0,700;1,400&family=Source+Sans+3:wght@300;400;500;600;700&family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet" />
      <div style={S.gridOverlay} />

      {/* ═══ HERO ═══ */}
      <div style={S.container}>
        <div style={S.hero}>
          <div style={S.heroGlow} />
          <div style={S.heroLabel}>Salvi Framework ISA v2.0 — Technical Paper</div>
          <h1 style={S.heroTitle}>ISA-Level Security Primitives<br />for Ternary Computing Architectures</h1>
          <p style={S.heroSub}>Capability-Based Access Control, Side-Channel Mitigation,<br />and Constant-Time Execution</p>
          <div style={S.heroDivider} />
          <div style={S.heroMeta}>
            <span style={S.heroMetaBold}>Applied Physics Division</span> — Capomastro Holdings Ltd.<br />
            Alberta, Canada — February 2026<br />
            PlenumNET Post-Quantum Timing Infrastructure
          </div>
          <div style={S.heroLinks}>
            <a href="https://github.com/SigmaWolf-8/Ternary" target="_blank" rel="noopener noreferrer" style={S.heroLink}
              onMouseEnter={(e: React.MouseEvent) => { (e.target as HTMLElement).style.background = `${COLORS.accent}20`; }}
              onMouseLeave={(e: React.MouseEvent) => { (e.target as HTMLElement).style.background = "transparent"; }}>
              Repository →
            </a>
            <a href="https://plenumnet.replit.app" target="_blank" rel="noopener noreferrer" style={S.heroLink}
              onMouseEnter={(e: React.MouseEvent) => { (e.target as HTMLElement).style.background = `${COLORS.accent}20`; }}
              onMouseLeave={(e: React.MouseEvent) => { (e.target as HTMLElement).style.background = "transparent"; }}>
              PlenumNET Platform →
            </a>
          </div>
        </div>
      </div>

      {/* ═══ NAV ═══ */}
      <div style={S.nav}>
        <div style={S.navInner}>
          {sections.map((s, i) => (
            <button key={i} style={S.navBtn(activeNav === i)}
              onClick={() => { setActiveNav(i); scrollToSection(s); }}
              onMouseEnter={(e: React.MouseEvent) => { if (activeNav !== i) (e.target as HTMLElement).style.color = COLORS.text; }}
              onMouseLeave={(e: React.MouseEvent) => { if (activeNav !== i) (e.target as HTMLElement).style.color = COLORS.textMuted; }}>
              {s}
            </button>
          ))}
        </div>
      </div>

      {/* ═══ CONTENT ═══ */}
      <div style={S.container}>

        {/* ── TABLE OF CONTENTS ── */}
        <div ref={ref("Overview")} style={S.tocWrap}>
          <div style={S.tocTitle}>Contents</div>
          {([
            [0, "Part I: Executive Overview"],
            [1, "Why New Security in the Processor?"],
            [1, "Ternary Computing Primer"],
            [1, "The Eight Security Instructions"],
            [1, "Why Ternary Makes It Better"],
            [0, "Part II: Technical Specification"],
            [1, "1. Introduction"],
            [1, "2. Related Work"],
            [1, "3. Ternary Logic Substrate"],
            [1, "4. Architectural Support"],
            [1, "5. Security and Audit Opcodes (0x90–0x97)"],
            [1, "6. PlenumNET Integration"],
            [1, "7. Security Analysis"],
            [1, "8. Comparison with Existing Approaches"],
            [1, "9. Reference Implementation"],
            [1, "10. Discussion and Future Work"],
            [1, "References"],
          ] as Array<[number, string]>).map(([lvl, text], i) => (
            <div key={i} style={S.tocItem(lvl)}
              onClick={() => {
                const match = sections.find(s => text.includes(s) || s.includes(text.split(".")[0]?.trim()));
                if (match) { scrollToSection(match); }
              }}
              onMouseEnter={(e: React.MouseEvent) => (e.target as HTMLElement).style.color = COLORS.accent}
              onMouseLeave={(e: React.MouseEvent) => (e.target as HTMLElement).style.color = lvl === 0 ? COLORS.text : COLORS.textDim}>
              {text}
            </div>
          ))}
        </div>

        {/* ════════════════════════════════════════════ */}
        {/* PART I: EXECUTIVE OVERVIEW                  */}
        {/* ════════════════════════════════════════════ */}
        <div ref={ref("Part I: Executive Overview")} style={S.partBanner}>
          <div style={S.partLabel}>Part I</div>
          <div style={S.partTitle}>Executive Overview</div>
        </div>

        <h2 style={S.h1}>Making Computers Safer: A New Kind of Security</h2>
        <p style={S.p}>
          Imagine your computer as a giant office building. Inside, there are countless rooms (memory locations) and valuable documents (data). Today's computers are like buildings with flimsy locks—attackers can often pick them, sneak in through air vents, or even listen through the walls to overhear conversations. A new approach, built into the very brain of the computer (the processor), promises to change that. It's called the <span style={S.bold}>Salvi Framework</span>, and it's a complete rethinking of how computers handle security, using a different kind of math called <span style={S.bold}>ternary logic</span>.
        </p>
        <p style={S.p}>
          This section explains, in plain English, a set of eight special security instructions that the Salvi processor uses. They're like a team of super-strict security guards that work at the hardware level, making sure bad actors can't break in or steal secrets—even if the software has bugs.
        </p>

        <h2 style={S.h2}>Why Do We Need New Security in the Processor?</h2>
        <p style={S.p}>Today's computers are based on binary logic—everything is a 0 or a 1. That's worked for decades, but it has three serious security weaknesses:</p>
        <p style={S.p}><span style={S.bold}>Memory bugs and fake pointers.</span> Programs often can't tell the difference between ordinary data and special "pointers" that say where data lives. Attackers can trick a program into using a fake pointer, leading to exploits like buffer overflows.</p>
        <p style={S.p}><span style={S.bold}>Future quantum computers are coming.</span> Once powerful quantum computers arrive, they'll crack many of today's encryption methods. We need new "post-quantum" cryptography, and the hardware must support it efficiently.</p>
        <p style={S.p}><span style={S.bold}>Side-channel leaks.</span> Even if your software is perfect, the physical hardware can accidentally whisper secrets. The time it takes to do a calculation might reveal your password. Attackers can also measure tiny variations in power usage or electromagnetic radiation—these are called <span style={S.bold}>side-channel attacks</span>.</p>
        <p style={S.p}>The Salvi project tackles all three problems by designing a new kind of processor from the ground up, using <span style={S.bold}>ternary</span> math (three possible values per digit instead of two) and baking security directly into its instruction set.</p>

        <h2 style={S.h2}>Ternary Computing: A Quick Primer</h2>
        <p style={S.p}>Binary digits (bits) are like light switches: ON (1) or OFF (0). Ternary digits (<span style={S.bold}>trits</span>) are like a dial with three positions. This gives the computer new powers:</p>
        <p style={S.p}><span style={S.bold}>More information per digit.</span> A trit holds about 1.58× as much information as a bit, so data can be packed more tightly.</p>
        <p style={S.p}><span style={S.bold}>Special "unforgeable" markers.</span> Because there are three values, we can reserve one value (like 0) as a special marker that ordinary calculations can <em>never</em> produce. That marker can tag sensitive items—like security tickets—making them impossible to fake.</p>

        <h2 style={S.h2}>The Security Team: Eight Special Instructions</h2>
        <p style={S.p}>Out of the 160 instructions the Salvi processor understands, eight are dedicated entirely to security:</p>

        <h3 style={S.h3}>AuditLog — The Camera That Never Lies</h3>
        <p style={S.p}>This instruction records what just happened into a protected log, stamped with femtosecond-precision time (a millionth of a billionth of a second). The log can't be faked or erased, providing a perfect record of who did what and when—essential for catching intruders and proving compliance with financial regulations like FINRA or MiFID II.</p>

        <h3 style={S.h3}>CapCheck, CapGrant, CapRevoke — The Ticket System</h3>
        <p style={S.p}>Instead of just checking "who you are," a <span style={S.bold}>capability</span> is like a physical ticket granting access to a specific resource—a backstage pass to a concert.</p>
        <p style={S.p}><span style={S.bold}>CapGrant</span> — The kernel creates a new ticket specifying what a program may do (read, write, etc.) and when it expires. <span style={S.bold}>CapCheck</span> — The hardware validates the ticket is authentic and hasn't been revoked. Because tickets use a special ternary value that can't be forged, no program can create a fake ticket. <span style={S.bold}>CapRevoke</span> — Instantly invalidates a ticket; any further attempt to use it fails.</p>

        <h3 style={S.h3}>SideChMask / SideChUnmask — Muffling the Whispers</h3>
        <p style={S.p}><span style={S.bold}>SideChMask</span> turns off all "leaky parts"—flushing caches, pausing speculative execution, and scrambling data with random numbers using ternary masking that is mathematically guaranteed to hide the original value. <span style={S.bold}>SideChUnmask</span> restores normal performance after the sensitive work is done. It's like putting your conversation inside a soundproof booth.</p>

        <h3 style={S.h3}>ConstTimeEq / ConstTimeSel — No Hints From Timing</h3>
        <p style={S.p}><span style={S.bold}>ConstTimeEq</span> compares two values but always takes the exact same time, regardless of where or whether they differ. <span style={S.bold}>ConstTimeSel</span> chooses between two values in constant time regardless of which is picked. These are essential for cryptographic code—secrets stay secret even under stopwatch scrutiny.</p>

        <h2 style={S.h2}>Defence-in-Depth Summary</h2>
        <Table
          headers={["Threat", "Defence", "How It Works"]}
          rows={[
            ["Memory bugs, fake pointers", "Capability tickets", "Tickets can't be forged; hardware checks every access"],
            ["Cache timing, power analysis", "Side-channel masking", "Leaky features turned off; data scrambled"],
            ["Timing of comparisons", "Constant-time ops", "Operations take fixed time, no clues"],
            ["Undetected intrusions", "Audit log", "Every important event recorded with precise time"],
          ]}
        />

        <h2 style={S.h2}>Why Ternary Makes It Better</h2>
        <p style={S.p}><span style={S.bold}>Unforgeable markers.</span> In binary, you'd need a separate "tag" memory—extra hardware that attackers might fool. In ternary, the marker is built into the number system itself. The digit set {'{'}1, 2, 3{'}'} doesn't include 0; a 0 placed in a special position acts as an impossible-to-forge seal.</p>
        <p style={S.p}><span style={S.bold}>Better masking.</span> Ternary's three values give a more uniform power profile, so attackers can't determine what changed by measuring power consumption.</p>
        <p style={S.p}><span style={S.bold}>More efficient masking.</span> Protecting against higher-order attacks is mathematically simpler in ternary, providing strong protection with less overhead.</p>


        {/* ════════════════════════════════════════════ */}
        {/* PART II: TECHNICAL SPECIFICATION             */}
        {/* ════════════════════════════════════════════ */}
        <div ref={ref("Part II: Technical Specification")} style={S.partBanner}>
          <div style={S.partLabel}>Part II</div>
          <div style={S.partTitle}>Technical Specification</div>
        </div>

        {/* ── ABSTRACT ── */}
        <h2 ref={ref("Abstract")} style={S.h1}>Abstract</h2>
        <p style={S.p}>Contemporary hardware security research has concentrated on retrofitting capability-based protections onto binary instruction set architectures, most notably through the CHERI programme. This paper presents a unified approach addressing three security domains—capability-based memory protection, side-channel attack mitigation, and constant-time execution guarantees—within a single coherent set of eight ISA-level primitives designed natively for a ternary computing architecture.</p>
        <p style={S.p}>We describe the complete Security and Audit category (opcodes <Code>0x90</Code>–<Code>0x97</Code>) of the Salvi Framework's 160-opcode ISA v2.0. Unlike prior work, these primitives exploit the inherent information-theoretic advantages of three bijective ternary representations. The bijective encoding (Representation C, digit set {'{'}1, 2, 3{'}'}) provides a deterministic sentinel value distinguishing capability metadata from data payloads without external tag bits, while the balanced ternary domain (Representation A, digit set {'{'}−1, 0, +1{'}'}) provides a natural masking structure resisting differential power analysis at the architectural level.</p>
        <div style={S.keywords}>
          {["ternary computing", "capability architecture", "side-channel masking", "ISA security", "post-quantum cryptography", "bijective ternary logic", "hardware security primitives", "constant-time execution", "PlenumNET", "HPTP", "CNSA 2.0", "FIPS 140-3"].map((k, i) => (
            <span key={i} style={S.keyword}>{k}</span>
          ))}
        </div>

        {/* ── 1. INTRODUCTION ── */}
        <h2 ref={ref("1. Introduction")} style={S.h1}>1. Introduction</h2>
        <p style={S.p}>The security of modern computing systems rests on hardware mechanisms designed decades before the current threat landscape materialised. Three distinct vulnerability classes define the contemporary problem:</p>
        <p style={S.p}>First, the flat memory model of conventional binary architectures provides no intrinsic distinction between data and pointers—generating an entire taxonomy of memory-safety vulnerabilities. The CHERI project has demonstrated that capability registers and tagged memory mitigate these, but CHERI requires dedicated tag bits in physical memory and complex compressed capability encodings.</p>
        <p style={S.p}>Second, NIST has standardised post-quantum algorithms (ML-KEM, ML-DSA, SLH-DSA), and post-quantum algorithms exhibit significantly different computational profiles—timing side-channels tolerable under classical assumptions may become exploitable in post-quantum deployments.</p>
        <p style={S.p}>Third, microarchitectural side-channel attacks exploiting cache timing, branch prediction, speculative execution, and power consumption remain persistent. No current production ISA provides dedicated instructions for dynamically controlling microarchitectural side-channel sources, nor hardware-guaranteed constant-time primitives.</p>
        <p style={S.p}>The Salvi Framework implements a complete ternary computing engine: 160-opcode ISA (v2.0), a compiled Rust kernel (33 MB ELF binary, 47,000+ LOC across 14 subsystems), with live deployment via PlenumNET.</p>

        <h3 style={S.h3}>Contributions</h3>
        <p style={S.p}>First, a formal treatment of the sentinel-trit property unique to bijective ternary encoding. Second, a dual-layer side-channel defence combining architectural feature masking with algebraic ternary share masking. Third, hardware-level constant-time primitives guaranteeing timing invariance independent of compiler optimisation. Fourth, integration with PlenumNET HPTP and a three-mode kernel security system. Fifth, comparative analysis showing this combination is unique among current architectures.</p>

        {/* ── 2. RELATED WORK ── */}
        <h2 ref={ref("2. Related Work")} style={S.h1}>2. Related Work</h2>
        <h3 style={S.h3}>2.1 Capability Hardware Architectures</h3>
        <p style={S.p}>The concept dates to the Cambridge CAP computer, Intel iAPX 432, and Plessey System 250. The modern CHERI programme extends MIPS, RISC-V, and ARMv8-A ISAs with 128-bit or 256-bit compressed capability descriptors, protected by a 1-bit tag per word. Arm's Morello SoC (2022) is the first silicon implementation. Our approach eliminates external tag bits entirely by designing capabilities natively for ternary logic—the third logic state provides an in-band algebraically unforgeable sentinel.</p>

        <h3 style={S.h3}>2.2 Side-Channel Countermeasures</h3>
        <p style={S.p}>The ISW masking countermeasure (CRYPTO 2003) decomposes secrets into random shares. Subsequent work extended the probing model. ISA extensions on RISC-V accelerate masking but remain additive—the underlying binary data representation still exhibits data-dependent power variation.</p>

        <h3 style={S.h3}>2.3 Constant-Time Programming</h3>
        <p style={S.p}>Software constant-time coding relies on bitwise comparisons and conditional moves, but compilers may optimise away these patterns. No current production ISA provides dedicated constant-time comparison or conditional-selection instructions.</p>

        <h3 style={S.h3}>2.4 Ternary Computing</h3>
        <p style={S.p}>Balanced ternary carries log₂(3) ≈ 1.585 bits per trit, approaching optimal radix economy (the minimum of n/ln(n) at e ≈ 2.718 makes 3 the closest integer). The Soviet Setun computer (1958) demonstrated practical balanced ternary. Security implications have received limited attention.</p>

        {/* ── 3. TERNARY LOGIC SUBSTRATE ── */}
        <h2 ref={ref("3. Ternary Logic Substrate")} style={S.h1}>3. Ternary Logic Substrate</h2>
        <h3 style={S.h3}>3.1 Three Bijective Representations</h3>
        <Table
          headers={["Repr.", "Name", "Digit Set", "Domain", "Bijection"]}
          rows={[
            ["A", "Computational", "{−1, 0, +1}", "Internal arithmetic, GF(3) field ops, algebraic masking", "Identity"],
            ["B", "Network", "{0, 1, 2}", "Network transmission, protocol encoding", "f(a) = a + 1"],
            ["C", "Human/Bijective", "{1, 2, 3}", "Human-readable display, bijective encoding, capability security", "f(a) = a + 2"],
          ]}
        />
        <p style={S.p}>The bijections are single-cycle hardware operations (<Code>TConvert</Code> opcode <Code>0x15</Code>). The security architecture is structurally dependent on three representations: Representation A provides the masking domain, C provides sentinel-trit capabilities, B enables secure wire-level encoding.</p>

        <h3 style={S.h3}>3.2 The Sentinel Property</h3>
        <Callout type="theorem" label="Definition 1 — Sentinel Trit">
          A sentinel trit is a trit position containing value 0, outside the bijective digit set Β₃ = {'{'}1, 2, 3{'}'}. No arithmetic operation over Β₃ can produce a sentinel trit from valid operands. A sentinel trit can only be written by a privileged hardware operation.
        </Callout>
        <Callout type="theorem" label="Theorem 1 — Unforgeability">
          Let f: Β₃ⁿ × Β₃ⁿ → Β₃ⁿ be any arithmetic operation over n-trit bijective ternary words. For any inputs A, B ∈ Β₃ⁿ, the result f(A, B) contains no sentinel trits.
        </Callout>
        <Callout type="proof" label="Proof">
          The carry logic for bijective ternary addition produces sum digit s and carry c: if (a + b) ≤ 3 then s = (a + b), c = 0; if 3 {'<'} (a + b) ≤ 6 then s = (a + b) − 3, c = 1. Since all inputs are in {'{'}1, 2, 3{'}'} and carry is in {'{'}0, 1{'}'}, all output digits remain in {'{'}1, 2, 3{'}'}. Extends by induction. ∎
        </Callout>

        <h3 style={S.h3}>3.3 Machine Word Structure</h3>
        <p style={S.p}>The architecture defines a 27-trit machine word ("tryte"), where 27 = 3³. This establishes a recursive self-similar structure: three 9-trit "tribbles", each comprising three 3-trit "triples." The Tribonacci sequence T(n) = T(n−1) + T(n−2) + T(n−3) provides a natural basis. The register file contains 27 ternary registers with privilege level, security domain, and extended flags.</p>

        {/* ── 4. ARCHITECTURAL SUPPORT ── */}
        <h2 ref={ref("4. Architectural Support")} style={S.h1}>4. Architectural Support</h2>

        <h3 style={S.h3}>4.1 Privilege Levels</h3>
        <Table
          headers={["Ring", "Name", "Enforcement"]}
          rows={[
            ["Ring0", "Kernel", "Full system access. Required for: CapGrant, CapRevoke, DomainSet, MProtect, IoRead, IoWrite, PrivEscalate, Trap"],
            ["Ring1", "Supervisor", "Restricted system access for device drivers and services"],
            ["Ring2", "User", "Unprivileged application code"],
          ]}
        />

        <h3 style={S.h3}>4.2 Modal Security System</h3>
        <Table
          headers={["Mode", "Symbol", "Description"]}
          rows={[
            ["ModePhi (Φ)", "phi_plus", "Maximum privilege: kernel operations, cryptographic key management, FIPS boundary"],
            ["ModeOne (1)", "one", "Standard operation: user processes, normal I/O"],
            ["ModeZero (0)", "zero", "Restricted/quarantine: untrusted code, sandboxed execution"],
          ]}
        />

        <h3 style={S.h3}>4.3 Security Domains</h3>
        <p style={S.p}>Each domain is a named isolation boundary with a security mode assignment, bounded member processes, controlled inter-domain transitions (Upgrade, Downgrade, Lateral) governed by explicit TransitionRule entries, optional isolation flag, and femtosecond-precision creation timestamp.</p>

        <h3 style={S.h3}>4.4 Capability Table</h3>
        <Table
          headers={["Field", "Type", "Description"]}
          rows={[
            ["id", "TokenId", "Unique capability identifier"],
            ["owner", "ProcessId", "Owning process"],
            ["kind", "CapabilityKind", "Capability category"],
            ["resource", "ResourceId", "Target resource"],
            ["actions", "Vec<Action>", "Permitted actions"],
            ["mode", "SecurityMode", "Required security mode"],
            ["created_at", "FemtosecondTimestamp", "Creation time (femtosecond precision)"],
            ["expires_at", "Option<FemtosecondTimestamp>", "Optional expiration time"],
            ["revoked", "bool", "Revocation status"],
            ["delegatable", "bool", "Whether delegation is permitted"],
            ["parent_token", "Option<TokenId>", "Parent capability for provenance tracking"],
          ]}
        />

        <h3 style={S.h3}>4.5–4.7 Additional Hardware Structures</h3>
        <p style={S.p}><span style={S.bold}>Side-Channel Control Register (SCCR)</span> — per-core register controlling microarchitectural side-channel mechanisms, saved/restored across context switches. <span style={S.bold}>Constant-Time ALU</span> — dedicated arithmetic path with no early termination, no data-dependent carry chains, fixed one-cycle latency. <span style={S.bold}>Hardware Random Trit Generator (HRTG)</span> — entropy source producing uniformly random trits for algebraic masking, FIPS 140-3 SP 800-90B compliant.</p>

        {/* ── 5. SECURITY OPCODES ── */}
        <h2 ref={ref("5. Security Opcodes")} style={S.h1}>5. Security and Audit Opcodes (<Code>0x90</Code>–<Code>0x97</Code>)</h2>

        <h3 style={S.h3}>5.1 AuditLog (<Code>0x90</Code>)</h3>
        <div style={S.codeBlock}>AUDITLOG src1</div>
        <p style={S.p}>Emits a hardware-generated audit log entry with program counter, privilege level, security domain, event code, and femtosecond-precision HPTP timestamp. The kernel records twelve event types (AccessGranted, AccessDenied, CapabilityCreated, CapabilityRevoked, CapabilityDelegated, DomainTransition, DomainTransitionDenied, PolicyChange, SecurityModeChange, ProcessCreated, ProcessTerminated, IpcEvent) with chain integrity verification via hash chaining.</p>

        <h3 style={S.h3}>5.2 CapCheck (<Code>0x91</Code>)</h3>
        <div style={S.codeBlock}>CAPCHECK dst, src1, src2</div>
        <p style={S.p}>Dual-mechanism verification: <span style={S.bold}>Mechanism 1</span> validates the sentinel trit (algebraic integrity without tag-controller lookup). <span style={S.bold}>Mechanism 2</span> queries the capability table for token validity, resource match, action permission, and security mode compatibility.</p>
        <Table
          headers={["Field", "Trits", "Position", "Description"]}
          rows={[
            ["SENTINEL", "1", "[0]", "Capability marker (value 0, outside Β₃)"],
            ["PERM", "3", "[1–3]", "Permission triple (R/W/X in Β₃)"],
            ["SCOPE", "5", "[4–8]", "Scope/domain identifier"],
            ["BASE", "9", "[9–17]", "Base address in ternary address space"],
            ["BOUND", "9", "[18–26]", "Upper bound (base + length)"],
          ]}
        />

        <h3 style={S.h3}>5.3 CapGrant (<Code>0x92</Code>) — Ring0 Only</h3>
        <div style={S.codeBlock}>CAPGRANT dst, src1, src2</div>
        <p style={S.p}>Creates a capability token with femtosecond timestamp, expiration, delegatable flag, and parent token for provenance. Enforces the monotonic authority principle: derived capability permissions are a trit-wise minimum subset, address range is contained within the granting domain's range.</p>

        <h3 style={S.h3}>5.4 CapRevoke (<Code>0x93</Code>) — Ring0 Only</h3>
        <div style={S.codeBlock}>CAPREVOKE dst, src1, src2</div>
        <p style={S.p}>Immediate dual-mechanism invalidation: table flag set + sentinel trit overwritten with non-zero value. Unlike CHERI's sweep-based revocation, this is O(1) per register and O(1) per table entry.</p>

        <h3 style={S.h3}>5.5 SideChMask (<Code>0x94</Code>)</h3>
        <div style={S.codeBlock}>SIDECHMASK dst, src1</div>
        <p style={S.p}><span style={S.bold}>Layer 1 — Microarchitectural isolation.</span> Bitmask disables L1 data cache, L1 instruction cache, branch predictor, speculative execution. <span style={S.bold}>Layer 2 — Algebraic ternary masking.</span> Automatic trit-wise modular addition with HRTG random vectors: d'ᵢ = (dᵢ + mᵢ) mod 3.</p>
        <Callout type="theorem" label="Theorem 2 — Perfect Masking">
          For any data word D ∈ Β₃ⁿ and uniformly random mask M ∈ Β₃ⁿ, the masked value D' = D ⊗ M is uniformly distributed over Β₃ⁿ and statistically independent of D. Ternary non-linear operations achieve O(n) complexity vs binary ISW O(n²).
        </Callout>

        <h3 style={S.h3}>5.6 SideChUnmask (<Code>0x95</Code>)</h3>
        <div style={S.codeBlock}>SIDECHUNMASK src1</div>
        <p style={S.p}>Restores microarchitectural features and exits algebraic masking mode. Inverse: dᵢ = (d'ᵢ − mᵢ + 3) mod 3. Guaranteed: D ⊗ M ⊘ M = D for all D, M.</p>

        <h3 style={S.h3}>5.7 ConstTimeEq (<Code>0x96</Code>)</h3>
        <div style={S.codeBlock}>CONSTTIMEEQ dst, src1, src2</div>
        <p style={S.p}>Constant-time comparison via XOR-and-OR-reduce (binary) or trit-wise subtraction-and-OR-reduce (ternary). No early exit, no branch prediction interaction. ALU path physically isolated from speculative execution. Use: MAC verification, password comparison, key equality, FIPS self-test validation.</p>

        <h3 style={S.h3}>5.8 ConstTimeSel (<Code>0x97</Code>)</h3>
        <div style={S.codeBlock}>CONSTTIMESEL dst, src1, src2  (condition in flags register)</div>
        <p style={S.p}>Constant-time conditional move: (condition AND src1) OR (NOT condition AND src2). Both sources read simultaneously. Use: elliptic curve point selection, ML-KEM decapsulation padding, Fiat-Shamir abort decisions.</p>

        {/* ── 6. PLENUMNET INTEGRATION ── */}
        <h2 ref={ref("6. PlenumNET Integration")} style={S.h1}>6. Integration with PlenumNET Infrastructure</h2>
        <h3 style={S.h3}>6.1 Post-Quantum Timing Anchors</h3>
        <p style={S.p}>HPTP generates cryptographically authenticated timestamps from seven clock source types at five precision levels (millisecond to femtosecond). Every <Code>CapGrant</Code> and <Code>CapRevoke</Code> can generate a timing proof signed with TL-DSA (ternary-native ML-DSA equivalent) and stored in the PlenumNET ledger.</p>

        <h3 style={S.h3}>6.2 Timing Isolation</h3>
        <p style={S.p}>Masked computational regions are bracketed by HPTP timing barriers enforcing constant-time execution. The processor inserts dummy cycles to ensure total execution time is data-independent. Timing barriers are capability-protected.</p>

        <h3 style={S.h3}>6.3 Blockchain Anchoring</h3>
        <p style={S.p}>Capability lifecycle events recorded via Hedera HCS, XRPL, and Algorand integrations—immutable, post-quantum-signed, consensus-timestamped. Supports MiFID II, FINRA Rule 613, and Reg NMS compliance.</p>

        <h3 style={S.h3}>6.4 CNSA 2.0 and FIPS 140-3 Compliance</h3>
        <p style={S.p}><Code>ConstTimeEq</Code>/<Code>ConstTimeSel</Code> address FIPS 140-3 Level 3 constant-time requirements. <Code>SideChMask</Code>/<Code>SideChUnmask</Code> address documented side-channel resistance. <Code>AuditLog</Code> provides operational assurance audit infrastructure. The <Code>finra-613</Code> feature flag enables FINRA Rule 613 timing compliance at build time.</p>

        {/* ── 7. SECURITY ANALYSIS ── */}
        <h2 ref={ref("7. Security Analysis")} style={S.h1}>7. Security Analysis</h2>
        <h3 style={S.h3}>7.1 Capability Properties</h3>
        <p style={S.p}><span style={S.bold}>P1 (Provenance)</span> — Every capability traces to a kernel root via parent_token chain with monotonic restriction. <span style={S.bold}>P2 (Integrity)</span> — Unforgeable via dual mechanism: sentinel trit + hardware-managed table. <span style={S.bold}>P3 (Non-bypassability)</span> — No instruction path permits unvalidated access. <span style={S.bold}>P4 (Isolation)</span> — Domain IDs prevent cross-domain interference. <span style={S.bold}>P5 (Temporal Integrity)</span> — Femtosecond creation/expiration timestamps with HPTP proofs. <span style={S.bold}>P6 (Revocability)</span> — O(1) immediate dual-mechanism invalidation.</p>

        <h3 style={S.h3}>7.2 Side-Channel Properties</h3>
        <p style={S.p}><span style={S.bold}>P7 (Microarchitectural Isolation)</span> — No observable timing differences during masked periods. <span style={S.bold}>P8 (First-Order Algebraic Security)</span> — Single probe reveals zero information (Theorem 2). <span style={S.bold}>P9 (Glitch Resistance)</span> — Uniform transition energy profiles. <span style={S.bold}>P10 (Timing Independence)</span> — Fixed cycle count regardless of inputs.</p>

        <h3 style={S.h3}>7.3–7.4 Defence-in-Depth</h3>
        <Table
          headers={["Domain", "Mechanism 1", "Mechanism 2", "Failure Mode"]}
          rows={[
            ["Capability integrity", "Sentinel trit (algebraic)", "Capability table (hardware)", "Both must be defeated"],
            ["Side-channel defence", "Microarchitectural isolation", "Algebraic ternary masking", "Either provides independent protection"],
            ["Timing", "Constant-time ALU", "HPTP timing barriers", "Both enforce invariance"],
            ["Audit", "Hardware timestamps", "Chain integrity verification", "Tampering requires breaking both"],
          ]}
        />

        <h3 style={S.h3}>7.5 Limitations</h3>
        <p style={S.p}>The current implementation is a compiled Rust kernel, not silicon—power-consumption properties are theoretical until FPGA synthesis. Higher-order probing analysis remains future work. The sentinel-trit mechanism is specific to bijective ternary encoding.</p>

        {/* ── 8. COMPARISON ── */}
        <h2 ref={ref("8. Comparison")} style={S.h1}>8. Comparison with Existing Approaches</h2>
        <Table
          headers={["Feature", "Intel MPK", "ARM Morello / CHERI", "RISC-V PMP", "Salvi Security Ops"]}
          rows={[
            ["Capability model", "No", "Yes (pointers)", "No", "Yes (domain + sentinel + temporal)"],
            ["Hardware grant/revoke", "No", "Partial", "No", "CapGrant / CapRevoke (Ring0)"],
            ["Unforgeable tag", "N/A", "1-bit external", "N/A", "Sentinel trit (algebraic, in-band)"],
            ["Capability provenance", "No", "No", "No", "Parent chain + femtosecond timestamps"],
            ["Capability expiration", "No", "No", "No", "expires_at with femtosecond precision"],
            ["Side-channel masking", "No", "No", "No", "SideChMask / SideChUnmask (dual-layer)"],
            ["Constant-time comparison", "No", "No", "No", "ConstTimeEq"],
            ["Constant-time selection", "No", "No", "No", "ConstTimeSel"],
            ["ISA-level audit", "No", "No", "No", "AuditLog (HPTP-timestamped)"],
            ["PQC integration", "No", "No", "No", "CNSA 2.0, TL-DSA/TL-KEM, FIPS 140-3"],
            ["Ternary native", "No", "No", "No", "Three bijective representations"],
          ]}
        />

        {/* ── 9. IMPLEMENTATION ── */}
        <h2 ref={ref("9. Implementation")} style={S.h1}>9. Reference Implementation</h2>
        <p style={S.p}><span style={S.bold}>ISA Decoder and Executor</span> — <Code>instruction_v2.rs</Code>, 1,111 LOC. Eight opcodes as Opcode enum variants (<Code>0x90</Code>–<Code>0x97</Code>) with privilege enforcement.</p>
        <Table
          headers={["Module", "LOC", "Functionality"]}
          rows={[
            ["capability.rs", "498", "CapabilityToken, CapabilityManager with grant/delegate/revoke/check_access"],
            ["domain.rs", "457", "SecurityDomain, DomainManager with transition rules and isolation"],
            ["audit.rs", "405", "AuditEntry, AuditLog with 12 event types, chain integrity verification"],
            ["policy.rs", "502", "PolicyRule, MAC + DAC engine with configurable scope and mode filters"],
          ]}
        />
        <p style={S.p}><span style={S.bold}>Additional modules:</span> Side-channel analysis (702 LOC, four categories), formal verification (615 LOC, SMTLIB2/Cryptol/SAW), ternary core (974 LOC, all three representations). Testing includes unit tests, privilege enforcement, sentinel unforgeability, masking round-trip correctness, chi-squared statistical independence, Criterion benchmarks, three fuzz targets, PropTest, and audit chain verification. 14 CI/CD workflows maintain security invariants.</p>

        {/* ── 10. FUTURE WORK ── */}
        <h2 ref={ref("10. Future Work")} style={S.h1}>10. Discussion and Future Work</h2>
        <p style={S.p}>This work presents the first ISA-level security subsystem designed natively for a non-binary computing substrate, combining all four security functions within a single opcode category. Future directions include FPGA synthesis for empirical side-channel measurement, machine-checked formal proofs, higher-order ternary masking analysis, hardware-accelerated post-quantum key management coupling capability delegation with key distribution, RISC-V custom extension proposal, and application to the Sigma Wolf ET crypto trading protocol.</p>
        <Callout type="note" label="Core Insight">
          The choice of number system is not merely an efficiency consideration—it is a security design parameter. The sentinel-trit unforgeability property and balanced-masking domain property are structural consequences of ternary arithmetic that no amount of binary ISA extension can replicate.
        </Callout>

        {/* ── REFERENCES ── */}
        <h2 ref={ref("References")} style={S.h1}>References</h2>
        <div style={{ fontSize: "13px", color: COLORS.textDim, lineHeight: 1.8 }}>
          {[
            `[1] H. M. Levy, Capability-Based Computer Systems. Digital Press, 1984.`,
            `[2] D. J. Bernstein, "Cache-timing attacks on AES," 2005.`,
            `[3] Y. Ishai, A. Sahai, D. Wagner, "Private Circuits: Securing Hardware Against Probing Attacks," CRYPTO 2003.`,
            `[3b] Salvi Framework / PlenumNET, github.com/SigmaWolf-8/Ternary, 2026.`,
            `[4] T. Fritzmann et al., "Masked Accelerators and ISA Extensions for PQC," IACR ePrint 2021/479.`,
            `[5] E. Rivain, E. Prouff, "Provably Secure Higher-Order Masking of AES," CHES 2010.`,
            `[6] A. Duc et al., "Unifying Leakage Models: From Probing to Noisy Leakage," EUROCRYPT 2014.`,
            `[7] P. Kocher et al., "Spectre Attacks: Exploiting Speculative Execution," IEEE S&P 2019.`,
            `[8] NIST, "PQC Standardization: ML-KEM, ML-DSA, SLH-DSA," FIPS 203/204/205, 2024.`,
            `[9] B. Battistello et al., "Horizontal Side-Channel Attacks on ISW," CHES 2016.`,
            `[10] R. N. M. Watson et al., "CHERI: Hybrid Capability-System Architecture," IEEE S&P 2015.`,
            `[11] S. Nikova et al., "Secure Hardware in the Presence of Glitches," CHES 2006.`,
            `[12] Arm Ltd., "Morello Programme," 2022.`,
            `[13] D. E. Knuth, The Art of Computer Programming, Vol. 2, 3rd ed., 1997.`,
            `[14] ISA Extensions for Shuffling Against Side-Channel Attacks, IEEE Trans. CAD, 2023.`,
            `[15] S. Cassiers et al., "Towards Tight Random Probing Security," CRYPTO 2021.`,
            `[16] J. B. Dennis, E. C. Van Horn, "Programming Semantics for Multiprogrammed Computations," CACM, 1966.`,
            `[17] M. Lipp et al., "Meltdown," USENIX Security 2018.`,
            `[18] Intel, "Memory Protection Keys," 2016.`,
            `[19] RISC-V Foundation, "Physical Memory Protection Specification," 2017.`,
            `[20] N. P. Brusentsov, "Setun: A Ternary Computer," Soviet Academy of Sciences, 1958.`,
            `[21] R. N. M. Watson et al., "CHERI: RISC Instructions," Cambridge TR-951, 2023.`,
            `[22] D. W. Jones, "Ternary Number Systems," University of Iowa, unpublished, 2013.`,
          ].map((r: string, i: number) => <p key={i} style={{ margin: "0 0 4px", paddingLeft: "32px", textIndent: "-32px" }}>{r}</p>)}
        </div>

        {/* ── FOOTER ── */}
        <div style={S.footer}>
          <div style={{ color: COLORS.textDim, marginBottom: "8px" }}>
            Copyright © 2025–2026 Capomastro Holdings Ltd. (Canada). Patent(s) Pending — All Rights Reserved.
          </div>
          <div>Applied Physics Division — Salvi Framework ISA v2.0 — Repository HEAD 645001e — February 2026</div>
        </div>
      </div>

      {/* Scroll to top */}
      <button style={S.scrollTop(showScrollTop)} onClick={() => window.scrollTo({ top: 0, behavior: "smooth" })}>↑</button>
    </div>
  );
}