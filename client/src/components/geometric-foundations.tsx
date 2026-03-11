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

const LIGHT = {
  bg: "hsl(0, 0%, 100%)",
  fg: "hsl(220, 20%, 15%)",
  fgSoft: "hsl(220, 15%, 35%)",
  fgMuted: "hsl(220, 10%, 55%)",
  fgFaint: "hsl(220, 8%, 72%)",
  primary: "hsl(210, 100%, 45%)",
  primarySoft: "hsl(210, 80%, 55%)",
  primaryDim: "hsla(210, 100%, 45%, 0.08)",
  primaryBorder: "hsla(210, 100%, 45%, 0.15)",
  secondary: "hsl(220, 10%, 94%)",
  muted: "hsl(220, 10%, 96%)",
  accent: "hsl(210, 30%, 92%)",
  card: "hsl(0, 0%, 100%)",
  cardBorder: "hsl(220, 13%, 91%)",
  shadow: "0 1px 3px rgba(0,0,0,0.06), 0 1px 2px rgba(0,0,0,0.04)",
  balance: "hsl(210, 100%, 45%)",
  balanceBg: "hsla(210, 100%, 45%, 0.06)",
  esoteric: "hsl(270, 50%, 55%)",
  esotericBg: "hsla(270, 50%, 55%, 0.06)",
  cosmic: "hsl(340, 60%, 55%)",
  cosmicBg: "hsla(340, 60%, 55%, 0.06)",
  green: "hsl(145, 55%, 42%)",
  greenBg: "hsla(145, 55%, 42%, 0.06)",
};

const DARK = {
  bg: "hsl(20, 14%, 4%)",
  fg: "hsl(45, 25%, 91%)",
  fgSoft: "hsl(40, 15%, 70%)",
  fgMuted: "hsl(35, 10%, 50%)",
  fgFaint: "hsl(30, 8%, 35%)",
  primary: "hsl(210, 80%, 55%)",
  primarySoft: "hsl(210, 70%, 65%)",
  primaryDim: "hsla(210, 80%, 55%, 0.1)",
  primaryBorder: "hsla(210, 80%, 55%, 0.18)",
  secondary: "hsl(210, 15%, 25%)",
  muted: "hsl(20, 12%, 10%)",
  accent: "hsl(210, 20%, 18%)",
  card: "hsl(20, 14%, 8%)",
  cardBorder: "hsl(20, 10%, 14%)",
  shadow: "none",
  balance: "hsl(210, 80%, 55%)",
  balanceBg: "hsla(210, 80%, 55%, 0.08)",
  esoteric: "hsl(270, 50%, 65%)",
  esotericBg: "hsla(270, 50%, 65%, 0.08)",
  cosmic: "hsl(340, 55%, 60%)",
  cosmicBg: "hsla(340, 55%, 60%, 0.07)",
  green: "hsl(145, 50%, 50%)",
  greenBg: "hsla(145, 50%, 50%, 0.08)",
};

const FONTS = {
  sans: "'Inter', system-ui, -apple-system, sans-serif",
  mono: "'JetBrains Mono', 'Fira Code', 'SF Mono', monospace",
};

const RADIUS = { lg: 9, md: 6, sm: 3 };

function useDarkMode() {
  const [dark, setDark] = useState(() => {
    if (typeof window === "undefined") return true;
    return document.documentElement.classList.contains("dark") ||
      window.matchMedia("(prefers-color-scheme: dark)").matches;
  });
  useEffect(() => {
    const obs = new MutationObserver(() => {
      setDark(document.documentElement.classList.contains("dark"));
    });
    obs.observe(document.documentElement, { attributes: true, attributeFilter: ["class"] });
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = (e: MediaQueryListEvent) => {
      if (!document.documentElement.classList.contains("dark") &&
          !document.documentElement.classList.contains("light")) {
        setDark(e.matches);
      }
    };
    mq.addEventListener("change", handler);
    return () => { obs.disconnect(); mq.removeEventListener("change", handler); };
  }, []);
  return dark;
}

function useTheme() {
  const dark = useDarkMode();
  return dark ? DARK : LIGHT;
}

function useMediaQuery(query: string) {
  const [matches, setMatches] = useState(() => {
    if (typeof window === "undefined") return false;
    return window.matchMedia(query).matches;
  });
  useEffect(() => {
    const mq = window.matchMedia(query);
    const handler = (e: MediaQueryListEvent) => setMatches(e.matches);
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, [query]);
  return matches;
}

function useInView(threshold = 0.15): [React.RefObject<HTMLDivElement>, boolean] {
  const ref = useRef<HTMLDivElement | null>(null);
  const [vis, setVis] = useState(false);
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const obs = new IntersectionObserver(
      ([e]) => { if (e.isIntersecting) { setVis(true); obs.disconnect(); } },
      { threshold }
    );
    obs.observe(el);
    return () => obs.disconnect();
  }, [threshold]);
  return [ref, vis];
}

function Stat({ value, label, sub, delay = 0 }: { value: number; label: string; sub: string; delay?: number }) {
  const t = useTheme();
  const [ref, vis] = useInView(0.3);
  const [disp, setDisp] = useState(0);
  useEffect(() => {
    if (!vis) return;
    const timer = setTimeout(() => {
      const s = performance.now();
      const dur = 1600;
      const tick = () => {
        const p = Math.min((performance.now() - s) / dur, 1);
        setDisp(Math.floor(value * (1 - Math.pow(1 - p, 3))));
        if (p < 1) requestAnimationFrame(tick);
      };
      tick();
    }, delay);
    return () => clearTimeout(timer);
  }, [vis, value, delay]);

  return (
    <div ref={ref} style={{ textAlign: "center", padding: "26px 16px", background: t.card }} data-testid={`stat-geo-${label.toLowerCase().replace(/\s+/g, '-')}`}>
      <div style={{
        fontSize: 32, fontWeight: 700, fontFamily: FONTS.mono, color: t.primary,
        opacity: vis ? 1 : 0, transform: vis ? "translateY(0)" : "translateY(12px)",
        transition: "opacity 0.6s, transform 0.6s",
      }}>
        {disp.toLocaleString()}
      </div>
      <div style={{ fontSize: 13, fontWeight: 600, color: t.fg, marginTop: 5 }}>{label}</div>
      <div style={{ fontSize: 10, fontFamily: FONTS.mono, color: t.fgMuted, marginTop: 3 }}>{sub}</div>
    </div>
  );
}

function FadeIn({ children, delay = 0, style = {} }: { children: React.ReactNode; delay?: number; style?: React.CSSProperties }) {
  const [ref, vis] = useInView(0.1);
  return (
    <div ref={ref} style={{
      opacity: vis ? 1 : 0,
      transform: vis ? "translateY(0)" : "translateY(24px)",
      transition: `opacity 0.7s ${delay}ms, transform 0.7s ${delay}ms`,
      ...style,
    }}>
      {children}
    </div>
  );
}

function MetatronsCube({ size = 440 }: { size?: number }) {
  const t = useTheme();
  const dark = useDarkMode();
  const [phase, setPhase] = useState(0);
  const [hov, setHov] = useState<number | null>(null);
  const raf = useRef<number>(0);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const isVisible = useRef(false);

  useEffect(() => {
    const prefersReducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (prefersReducedMotion) return;

    const el = containerRef.current;
    if (el) {
      const obs = new IntersectionObserver(([entry]) => { isVisible.current = entry.isIntersecting; }, { threshold: 0.05 });
      obs.observe(el);
      var cleanup = () => obs.disconnect();
    }

    let on = true;
    const tick = () => {
      if (!on) return;
      if (isVisible.current) {
        setPhase(p => (p + 0.004) % (Math.PI * 2));
      }
      raf.current = requestAnimationFrame(tick);
    };
    raf.current = requestAnimationFrame(tick);
    return () => { on = false; cancelAnimationFrame(raf.current); cleanup?.(); };
  }, []);

  const cx = size / 2, cy = size / 2;
  const rI = size * 0.135, rO = size * 0.30, rD = size * 0.42;
  const nR: Record<string, number> = { central: 9, inner: 5.5, outer: 4.5, depth: 6 };

  const central = { x: cx, y: cy, idx: 0, ring: "central", name: "Central" };
  const inner = Array.from({ length: 6 }, (_, i) => {
    const a = (i * Math.PI * 2) / 6 + phase * 0.25;
    return { x: cx + rI * Math.cos(a), y: cy + rI * Math.sin(a), idx: i + 1, ring: "inner", name: `Inner ${i + 1}` };
  });
  const outer = Array.from({ length: 5 }, (_, i) => {
    const a = (i * Math.PI * 2) / 5 + Math.PI / 10 - phase * 0.15;
    return { x: cx + rO * Math.cos(a), y: cy + rO * Math.sin(a), idx: i + 7, ring: "outer", name: `Outer ${i + 7}` };
  });
  const depth = { x: cx, y: cy - rD, idx: 12, ring: "depth", name: "Depth" };
  const all = [central, ...inner, ...outer, depth];

  type Node = typeof central;
  const edges: { a: Node; b: Node; t: string }[] = [];
  inner.forEach(n => edges.push({ a: central, b: n, t: "f" }));
  for (let i = 0; i < 6; i++) {
    edges.push({ a: inner[i], b: inner[(i + 1) % 6], t: "i" });
    edges.push({ a: inner[i], b: inner[(i + 2) % 6], t: "is" });
    edges.push({ a: inner[i], b: inner[(i + 3) % 6], t: "io" });
  }
  for (let i = 0; i < 5; i++) {
    edges.push({ a: outer[i], b: outer[(i + 1) % 5], t: "o" });
    edges.push({ a: outer[i], b: outer[(i + 2) % 5], t: "os" });
  }
  for (let i = 0; i < 5; i++) {
    edges.push({ a: inner[i], b: outer[i], t: "c" });
    edges.push({ a: inner[i + 1], b: outer[i], t: "c" });
  }
  inner.forEach(n => edges.push({ a: depth, b: n, t: "d" }));

  const edgeAlpha = dark ? 1 : 0.7;
  const ec: Record<string, string> = {
    f: `hsla(210, 80%, 55%, ${0.22 * edgeAlpha})`,
    i: `hsla(210, 80%, 55%, ${0.12 * edgeAlpha})`,
    is: `hsla(210, 70%, 55%, ${0.06 * edgeAlpha})`,
    io: `hsla(210, 60%, 55%, ${0.03 * edgeAlpha})`,
    o: `hsla(270, 50%, 60%, ${0.10 * edgeAlpha})`,
    os: `hsla(270, 50%, 60%, ${0.05 * edgeAlpha})`,
    c: `hsla(340, 55%, 55%, ${0.07 * edgeAlpha})`,
    d: `hsla(210, 60%, 60%, ${0.06 * edgeAlpha})`,
  };
  const nc: Record<string, string> = {
    central: t.primary,
    inner: t.primarySoft,
    outer: t.esoteric,
    depth: `hsla(210, 60%, ${dark ? 65 : 50}%, 0.85)`,
  };

  const glowColor = dark ? "hsla(210, 80%, 55%, 0.12)" : "hsla(210, 100%, 45%, 0.06)";
  const glowEdge = dark ? "hsla(210, 80%, 55%, 0.03)" : "hsla(210, 100%, 45%, 0.015)";
  const orbitStroke = dark ? "hsla(210, 50%, 50%, 0.06)" : "hsla(210, 60%, 50%, 0.08)";
  const labelColor = dark ? t.fgFaint : t.fgMuted;

  return (
    <div ref={containerRef}>
    <svg viewBox={`0 0 ${size} ${size}`} role="img" aria-label="Metatron's Cube — 13-dimensional ternary network visualization with 3 concentric rings of nodes connected by edges" style={{ width: "100%", maxWidth: size, display: "block" }} data-testid="svg-metatrons-cube">
      <defs>
        <radialGradient id="mcG" cx="50%" cy="50%" r="50%">
          <stop offset="0%" stopColor={glowColor} />
          <stop offset="60%" stopColor={glowEdge} />
          <stop offset="100%" stopColor="transparent" />
        </radialGradient>
        <filter id="ng"><feGaussianBlur stdDeviation="3" result="b" /><feMerge><feMergeNode in="b" /><feMergeNode in="SourceGraphic" /></feMerge></filter>
        <filter id="sg"><feGaussianBlur stdDeviation="1.5" result="b" /><feMerge><feMergeNode in="b" /><feMergeNode in="SourceGraphic" /></feMerge></filter>
      </defs>

      <circle cx={cx} cy={cy} r={rD * 0.85} fill="url(#mcG)" />
      {[rI + 14, rO + 14, rD].map((r, i) => (
        <circle key={i} cx={cx} cy={cy} r={r} fill="none" stroke={orbitStroke}
          strokeWidth="0.6" strokeDasharray={i === 2 ? "3 5" : "none"} />
      ))}

      {edges.map((e, i) => (
        <line key={i} x1={e.a.x} y1={e.a.y} x2={e.b.x} y2={e.b.y}
          stroke={ec[e.t]} strokeWidth={e.t === "f" ? 0.9 : 0.5} />
      ))}

      {all.map((n, i) => {
        const r = nR[n.ring];
        const isH = hov === n.idx;
        return (
          <g key={i} onMouseEnter={() => setHov(n.idx)} onMouseLeave={() => setHov(null)}
            style={{ cursor: "pointer" }}
            filter={n.ring === "central" ? "url(#ng)" : isH ? "url(#sg)" : undefined}>
            {n.ring === "central" && (
              <circle cx={n.x} cy={n.y} r={r + 5} fill="none" stroke={t.primary} strokeWidth="0.6" opacity="0.4">
                <animate attributeName="r" values={`${r + 4};${r + 7};${r + 4}`} dur="4s" repeatCount="indefinite" />
              </circle>
            )}
            <circle cx={n.x} cy={n.y} r={isH ? r + 2 : r} fill={nc[n.ring]} />
            {isH && (
              <text x={n.x} y={n.y + r + 14} textAnchor="middle" fill={t.fgSoft}
                fontSize="9" fontFamily={FONTS.mono}>
                Axis {n.idx} · Rep C {n.idx + 1}
              </text>
            )}
          </g>
        );
      })}

      <text x={cx} y={cy - rD - 10} textAnchor="middle" fill={labelColor} fontSize="8.5" fontFamily={FONTS.mono} letterSpacing="1">DEPTH · SHELL SELECTOR</text>
      <text x={cx + rO + 20} y={cy - 8} textAnchor="start" fill={labelColor} fontSize="8" fontFamily={FONTS.mono}>Outer (5)</text>
      <text x={cx + rI + 18} y={cy + 5} textAnchor="start" fill={labelColor} fontSize="8" fontFamily={FONTS.mono}>Inner (6)</text>
      <text x={cx} y={size - 12} textAnchor="middle" fill={t.fgFaint} fontSize="8" fontFamily={FONTS.mono} letterSpacing="0.5">
        13 circles · 3 shells · 1,594,323 vertices
      </text>
    </svg>
    </div>
  );
}

function MagicSquare() {
  const t = useTheme();
  const [hovR, setHovR] = useState<number | null>(null);
  const v = [[111, 14, 208], [208, 111, 14], [14, 208, 111]];
  const tr = [[0, -1, 1], [1, 0, -1], [-1, 1, 0]];
  const lb = [["identity", "shift-2", "shift-1"], ["shift-1", "identity", "shift-2"], ["shift-2", "shift-1", "identity"]];
  const colorMap: Record<number, { bg: string; border: string; text: string }> = {
    111: { bg: t.balanceBg, border: t.primaryBorder, text: t.balance },
    14:  { bg: t.esotericBg, border: `hsla(270, 50%, 55%, 0.15)`, text: t.esoteric },
    208: { bg: t.cosmicBg, border: `hsla(340, 55%, 55%, 0.12)`, text: t.cosmic },
  };

  return (
    <div style={{ display: "inline-block" }} data-testid="magic-square">
      <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: 3, fontFamily: FONTS.mono }}>
        {v.flatMap((row, r) =>
          row.map((val, c) => {
            const cm = colorMap[val];
            return (
              <div key={`${r}-${c}`}
                onMouseEnter={() => setHovR(r)} onMouseLeave={() => setHovR(null)}
                style={{
                  width: 80, height: 66, display: "flex", flexDirection: "column",
                  alignItems: "center", justifyContent: "center",
                  background: hovR === r ? t.primaryDim : cm.bg,
                  border: `1px solid ${cm.border}`, borderRadius: RADIUS.md,
                  cursor: "default", transition: "background 0.2s",
                }}>
                <span style={{ fontSize: 18, fontWeight: 600, color: cm.text }}>{val}</span>
                <span style={{ fontSize: 9, color: t.fgMuted, marginTop: 3 }}>
                  mod 3 = {val % 3} → {tr[r][c] === 0 ? "0" : tr[r][c] > 0 ? "+1" : "−1"}
                </span>
                <span style={{ fontSize: 7.5, color: t.fgFaint, marginTop: 1 }}>{lb[r][c]}</span>
              </div>
            );
          })
        )}
      </div>
      <div style={{ textAlign: "center", fontSize: 10, fontFamily: FONTS.mono, color: t.fgMuted, marginTop: 8, letterSpacing: 0.5 }}>
        Σ row = Σ col = Σ diag = <span style={{ color: t.primary, fontWeight: 600 }}>333</span>
      </div>
    </div>
  );
}

function useInterCubeTopology() {
  const [data, setData] = useState<{ vertices: number; dimensions: number; neighborsPerCube: number } | null>(null);
  useEffect(() => {
    fetch("/api/salvi/inter-cube/topology")
      .then(r => r.ok ? r.json() : null)
      .then(d => d && setData(d))
      .catch(() => {});
  }, []);
  return data;
}

function ServiceCard({ icon, tag, name, stat, statLabel, desc, delay = 0 }: {
  icon: string; tag: string; name: string; stat: string; statLabel: string; desc: string; delay?: number;
}) {
  const t = useTheme();
  const [ref, vis] = useInView(0.1);
  const [h, setH] = useState(false);
  return (
    <div ref={ref}
      onMouseEnter={() => setH(true)} onMouseLeave={() => setH(false)}
      data-testid={`card-service-${tag.toLowerCase()}`}
      style={{
        background: t.card, border: `1px solid ${h ? t.primaryBorder : t.cardBorder}`,
        borderRadius: RADIUS.lg, padding: "28px 24px",
        boxShadow: h ? t.shadow : "none",
        opacity: vis ? 1 : 0, transform: vis ? "translateY(0)" : "translateY(16px)",
        transition: `opacity 0.5s ${delay}ms, transform 0.5s ${delay}ms, border-color 0.3s, box-shadow 0.3s`,
      }}>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 14 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
          <span style={{ fontSize: 20, opacity: 0.75 }}>{icon}</span>
          <span style={{
            fontSize: 8.5, fontFamily: FONTS.mono, letterSpacing: 1.5, color: t.primary,
            textTransform: "uppercase" as const, background: t.primaryDim,
            padding: "3px 8px", borderRadius: RADIUS.sm, fontWeight: 600,
          }}>{tag}</span>
        </div>
        <div style={{ textAlign: "right" }}>
          <div style={{ fontSize: 18, fontWeight: 700, fontFamily: FONTS.mono, color: t.primary, lineHeight: 1 }}>{stat}</div>
          <div style={{ fontSize: 8.5, fontFamily: FONTS.mono, color: t.fgMuted, marginTop: 2 }}>{statLabel}</div>
        </div>
      </div>
      <h4 style={{ fontSize: 16, fontWeight: 600, color: t.fg, margin: "0 0 8px" }}>{name}</h4>
      <p style={{ fontSize: 13, lineHeight: 1.7, color: t.fgSoft, margin: 0 }}>{desc}</p>
    </div>
  );
}

function RoutingDemo() {
  const t = useTheme();
  const dark = useDarkMode();
  const [ref, vis] = useInView(0.1);
  const [result, setResult] = useState<{
    nextHop: number[]; dimensionFixed: number; totalDistance: number;
    availablePaths: number; shortestPathCount: number; isDetour: boolean;
  } | null>(null);
  const [source, setSource] = useState([1,1,1,1,1,1,1,1,1,1,1,1,1]);
  const [dest, setDest] = useState([3,2,1,1,2,3,1,2,3,1,2,3,1]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [registered, setRegistered] = useState(false);

  const ensureRegistered = useCallback(async () => {
    if (registered) return true;
    try {
      const resp = await fetch("/api/salvi/inter-cube/crs/register", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ endpoint: "demo:51820", publicKey: "demo-key", desiredAddress: source }),
      });
      if (resp.ok) { setRegistered(true); return true; }
    } catch {}
    return false;
  }, [registered, source]);

  const computeRoute = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const ok = await ensureRegistered();
      if (!ok) { setError("Failed to register cube — service unavailable"); setLoading(false); return; }
      const resp = await fetch("/api/salvi/inter-cube/glb/forward", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ destination: dest, flowId: Math.floor(Math.random() * 10000) }),
      });
      if (resp.ok) { setResult(await resp.json()); }
      else { setError("Forwarding computation failed"); }
    } catch { setError("Network error — try again"); }
    setLoading(false);
  }, [dest, ensureRegistered]);

  const randomDest = useCallback(() => {
    const d = Array.from({ length: 13 }, () => Math.floor(Math.random() * 3) + 1);
    setDest(d);
    setResult(null);
  }, []);

  return (
    <div ref={ref} style={{
      opacity: vis ? 1 : 0, transform: vis ? "translateY(0)" : "translateY(20px)",
      transition: "opacity 0.7s 200ms, transform 0.7s 200ms",
    }}>
      <div style={{
        background: t.card, border: `1px solid ${t.cardBorder}`, borderRadius: RADIUS.lg,
        padding: "22px 24px", boxShadow: t.shadow,
      }} data-testid="routing-demo">
        <div style={{ display: "flex", alignItems: "center", gap: 10, marginBottom: 18 }}>
          <span style={{ fontSize: 16 }}>⬡</span>
          <span style={{
            fontSize: 9, fontFamily: FONTS.mono, letterSpacing: 2, color: t.primary,
            textTransform: "uppercase" as const, fontWeight: 600,
          }}>Live Geometric Routing</span>
        </div>

        <div style={{ display: "grid", gridTemplateColumns: "1fr auto 1fr", gap: 12, alignItems: "center", marginBottom: 18 }}>
          <div>
            <div style={{ fontSize: 9, fontFamily: FONTS.mono, color: t.fgMuted, letterSpacing: 1, marginBottom: 6 }}>SOURCE</div>
            <div style={{
              fontFamily: FONTS.mono, fontSize: 11, color: t.fgSoft, padding: "8px 12px",
              background: t.muted, border: `1px solid ${t.cardBorder}`, borderRadius: RADIUS.md,
              wordBreak: "break-all" as const,
            }}>{source.join(",")}</div>
          </div>
          <div style={{ fontSize: 20, color: t.fgFaint, marginTop: 14 }}>→</div>
          <div>
            <div style={{ fontSize: 9, fontFamily: FONTS.mono, color: t.fgMuted, letterSpacing: 1, marginBottom: 6 }}>DESTINATION</div>
            <div style={{
              fontFamily: FONTS.mono, fontSize: 11, color: t.esoteric, padding: "8px 12px",
              background: t.muted, border: `1px solid ${t.cardBorder}`, borderRadius: RADIUS.md,
              wordBreak: "break-all" as const,
            }}>{dest.join(",")}</div>
          </div>
        </div>

        <div style={{ display: "flex", gap: 8, marginBottom: 18 }}>
          <button onClick={computeRoute} disabled={loading} data-testid="button-compute-route"
            style={{
              flex: 1, padding: "10px 20px", fontSize: 12, fontWeight: 600, fontFamily: FONTS.mono,
              background: t.primary, color: "#fff", border: "none", borderRadius: RADIUS.md,
              cursor: loading ? "wait" : "pointer", opacity: loading ? 0.7 : 1,
              transition: "opacity 0.2s", letterSpacing: 0.5,
            }}>{loading ? "Computing…" : "Compute Next Hop"}</button>
          <button onClick={randomDest} data-testid="button-random-dest"
            style={{
              padding: "10px 16px", fontSize: 12, fontWeight: 600, fontFamily: FONTS.mono,
              background: "transparent", color: t.primary, border: `1px solid ${t.primaryBorder}`,
              borderRadius: RADIUS.md, cursor: "pointer", letterSpacing: 0.5,
            }}>Randomize</button>
        </div>

        {result && (
          <div style={{
            padding: "16px 18px", background: dark ? "hsla(145, 50%, 50%, 0.06)" : "hsla(145, 55%, 42%, 0.04)",
            border: `1px solid ${dark ? "hsla(145, 50%, 50%, 0.12)" : "hsla(145, 55%, 42%, 0.1)"}`,
            borderRadius: RADIUS.md, fontFamily: FONTS.mono, fontSize: 11,
          }} data-testid="routing-result">
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 10 }}>
              <div>
                <span style={{ color: t.fgMuted, fontSize: 9, letterSpacing: 1 }}>NEXT HOP</span>
                <div style={{ color: t.green, fontWeight: 600, marginTop: 3, fontSize: 12 }}>{result.nextHop.join(",")}</div>
              </div>
              <div>
                <span style={{ color: t.fgMuted, fontSize: 9, letterSpacing: 1 }}>DIMENSION FIXED</span>
                <div style={{ color: t.primary, fontWeight: 600, marginTop: 3, fontSize: 12 }}>Axis {result.dimensionFixed}</div>
              </div>
              <div>
                <span style={{ color: t.fgMuted, fontSize: 9, letterSpacing: 1 }}>HAMMING DISTANCE</span>
                <div style={{ color: t.fg, fontWeight: 600, marginTop: 3, fontSize: 12 }}>{result.totalDistance} hops</div>
              </div>
              <div>
                <span style={{ color: t.fgMuted, fontSize: 9, letterSpacing: 1 }}>SHORTEST PATHS</span>
                <div style={{ color: t.esoteric, fontWeight: 600, marginTop: 3, fontSize: 12 }}>{result.shortestPathCount.toLocaleString()}</div>
              </div>
            </div>
            {result.isDetour && (
              <div style={{ marginTop: 10, fontSize: 10, color: t.cosmic, fontWeight: 600 }}>
                DETOUR — all direct paths blocked by dead neighbors
              </div>
            )}
          </div>
        )}

        {error && (
          <div style={{
            padding: "12px 16px", marginTop: result ? 0 : 0,
            background: dark ? "hsla(0, 50%, 50%, 0.06)" : "hsla(0, 55%, 50%, 0.04)",
            border: `1px solid ${dark ? "hsla(0, 50%, 50%, 0.15)" : "hsla(0, 55%, 50%, 0.1)"}`,
            borderRadius: RADIUS.md, fontFamily: FONTS.mono, fontSize: 11, color: t.cosmic,
          }} data-testid="routing-error">{error}</div>
        )}
      </div>
    </div>
  );
}

function SubCard({ icon, tag, title, axis, desc, delay = 0 }: { icon: string; tag: string; title: string; axis: string; desc: string; delay?: number }) {
  const t = useTheme();
  const [ref, vis] = useInView(0.1);
  const [h, setH] = useState(false);
  return (
    <div ref={ref}
      onMouseEnter={() => setH(true)} onMouseLeave={() => setH(false)}
      data-testid={`card-subsystem-${tag.toLowerCase()}`}
      style={{
        background: t.card, border: `1px solid ${h ? t.primaryBorder : t.cardBorder}`,
        borderRadius: RADIUS.lg, padding: "26px 22px",
        boxShadow: h ? t.shadow : "none",
        opacity: vis ? 1 : 0, transform: vis ? "translateY(0)" : "translateY(16px)",
        transition: `opacity 0.5s ${delay}ms, transform 0.5s ${delay}ms, border-color 0.3s, box-shadow 0.3s`,
      }}>
      <div style={{ display: "flex", alignItems: "center", gap: 10, marginBottom: 12 }}>
        <span style={{ fontSize: 18, opacity: 0.7 }}>{icon}</span>
        <span style={{
          fontSize: 8.5, fontFamily: FONTS.mono, letterSpacing: 1.5, color: t.primary,
          textTransform: "uppercase" as const, background: t.primaryDim,
          padding: "3px 8px", borderRadius: RADIUS.sm,
        }}>{tag}</span>
      </div>
      <h4 style={{ fontSize: 16, fontWeight: 600, color: t.fg, margin: "0 0 5px" }}>{title}</h4>
      <div style={{ fontSize: 10.5, fontFamily: FONTS.mono, color: t.fgMuted, marginBottom: 10 }}>{axis}</div>
      <p style={{ fontSize: 13, lineHeight: 1.7, color: t.fgSoft, margin: 0 }}>{desc}</p>
    </div>
  );
}

function RepCard({ name, subtitle, digits, desc, color, bg, border, highlight }: {
  name: string; subtitle: string; digits: string; desc: string;
  color: string; bg: string; border: string; highlight?: boolean;
}) {
  const t = useTheme();
  const [ref, vis] = useInView(0.15);
  const [h, setH] = useState(false);
  return (
    <div ref={ref}
      onMouseEnter={() => setH(true)} onMouseLeave={() => setH(false)}
      data-testid={`card-rep-${name.toLowerCase().replace(/\s/g, '-')}`}
      style={{
        background: h ? t.primaryDim : bg,
        border: `1px solid ${h ? color : border}`,
        borderRadius: RADIUS.lg, padding: "30px 26px", position: "relative" as const, overflow: "hidden" as const,
        boxShadow: h ? t.shadow : "none",
        opacity: vis ? 1 : 0, transform: vis ? "translateY(0)" : "translateY(20px)",
        transition: "opacity 0.6s, transform 0.6s, border-color 0.3s, background 0.3s, box-shadow 0.3s",
      }}>
      {highlight && (
        <div style={{
          position: "absolute" as const, top: 14, right: 14,
          fontSize: 8, fontFamily: FONTS.mono, letterSpacing: 1.5, fontWeight: 600,
          color: t.primary, background: t.primaryDim, padding: "3px 10px", borderRadius: RADIUS.sm,
        }}>DEFENSE GRADE</div>
      )}
      <div style={{ fontSize: 22, fontWeight: 700, color, marginBottom: 2 }}>{name}</div>
      <div style={{ fontSize: 10, fontFamily: FONTS.mono, color: t.fgMuted, marginBottom: 14 }}>{subtitle}</div>
      <div style={{ fontSize: 22, fontFamily: FONTS.mono, color, letterSpacing: 3, marginBottom: 16, fontWeight: 500 }}>{digits}</div>
      <p style={{ fontSize: 13, lineHeight: 1.7, color: t.fgSoft, margin: 0 }}>{desc}</p>
    </div>
  );
}

function AlgebraicDetails() {
  const t = useTheme();
  const [expanded, setExpanded] = useState(false);

  return (
    <div>
      <button
        onClick={() => setExpanded(!expanded)}
        data-testid="button-toggle-math"
        style={{
          display: "flex", alignItems: "center", gap: 8,
          padding: "10px 16px", fontSize: 13, fontWeight: 600,
          fontFamily: FONTS.mono, color: t.primary,
          background: t.primaryDim, border: `1px solid ${t.primaryBorder}`,
          borderRadius: RADIUS.md, cursor: "pointer",
          width: "100%", textAlign: "left" as const,
        }}
      >
        <span style={{ transform: expanded ? "rotate(90deg)" : "rotate(0deg)", transition: "transform 0.2s", fontSize: 10 }}>&#9654;</span>
        {expanded ? "Hide the math" : "See the math"}
      </button>

      {expanded && (
        <div style={{ marginTop: 12 }}>
          <div style={{
            fontFamily: FONTS.mono, fontSize: 15, background: t.primaryDim,
            border: `1px solid ${t.primaryBorder}`, borderRadius: RADIUS.lg,
            padding: "20px 26px", lineHeight: 2.2,
          }}>
            <div style={{ fontSize: 10, color: t.fgMuted, letterSpacing: 1, marginBottom: 6 }}>S&#x2083; &#x2245; Aff(1, &#x1D53D;&#x2083;)</div>
            <div style={{ color: t.fgSoft }}>
              <span style={{ color: t.primary, fontWeight: 600 }}>&pi;</span>(x) = (<span style={{ color: t.esoteric }}>a</span> &middot; x + <span style={{ color: t.cosmic }}>b</span>)
              <span style={{ color: t.fgMuted }}> mod 3</span>
            </div>
            <div style={{ fontSize: 10.5, color: t.fgMuted, marginTop: 4 }}>
              a &isin; {"{1, 2}"}  &middot;  b &isin; {"{0, 1, 2}"}  &rarr;  6 permutations, zero tables
            </div>
          </div>

          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 8, marginTop: 14 }}>
            {[
              { op: "Compose", f: "(a\u2081a\u2082, a\u2081b\u2082+b\u2081) mod 3" },
              { op: "Inverse", f: "(a, 3\u2212a\u00B7b) mod 3" },
              { op: "Sign", f: "a=1 \u2192 even \u00B7 a=2 \u2192 odd" },
              { op: "Order", f: "1 (id) \u00B7 2 (swap) \u00B7 3 (cycle)" },
            ].map((x, i) => (
              <div key={i} style={{
                padding: "10px 14px", background: t.muted,
                border: `1px solid ${t.cardBorder}`, borderRadius: RADIUS.md,
              }}>
                <div style={{ fontSize: 9.5, color: t.fgMuted, fontFamily: FONTS.mono, marginBottom: 3 }}>{x.op}</div>
                <div style={{ fontSize: 11.5, color: t.fgSoft, fontFamily: FONTS.mono }}>{x.f}</div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

function SectionLabel({ text }: { text: string }) {
  const t = useTheme();
  return (
    <div style={{
      fontSize: 11, fontFamily: FONTS.mono, letterSpacing: 3, fontWeight: 500,
      color: t.primary, textTransform: "uppercase" as const, marginBottom: 16,
    }}>{text}</div>
  );
}

export default function GeometricFoundations() {
  const t = useTheme();
  const dark = useDarkMode();

  const topology = useInterCubeTopology();
  const isMobile = useMediaQuery("(max-width: 768px)");
  const dividerColor = dark ? "hsla(210, 30%, 50%, 0.1)" : "hsla(220, 13%, 85%, 0.6)";
  const gridOpacity = dark ? 0.02 : 0.03;
  const gridColor = dark ? "hsla(210, 30%, 50%, 0.3)" : "hsla(220, 20%, 70%, 0.25)";

  return (
    <div id="geometric-foundations" style={{
      background: t.bg, color: t.fgSoft, fontFamily: FONTS.sans,
      position: "relative", overflow: "hidden",
    }} data-testid="section-geometric-foundations">
      <div style={{
        position: "absolute", inset: 0, pointerEvents: "none", opacity: gridOpacity, zIndex: 0,
        backgroundImage: `linear-gradient(${gridColor} 1px, transparent 1px), linear-gradient(90deg, ${gridColor} 1px, transparent 1px)`,
        backgroundSize: "80px 80px",
      }} />

      <section style={{ maxWidth: 1140, margin: "0 auto", padding: "80px 28px 60px", position: "relative", zIndex: 1 }}>
        <div style={{ width: 60, height: 1, background: dividerColor, marginBottom: 56 }} />
        <div style={{ display: "grid", gridTemplateColumns: "minmax(0,1fr) minmax(0,1fr)", gap: 60, alignItems: "center" }}>
          <FadeIn>
            <SectionLabel text="Geometric Foundations" />
            <h2 style={{
              fontSize: 42, fontWeight: 700, lineHeight: 1.12, margin: "0 0 24px", color: t.fg,
            }}>
              Architecture Derived{" "}
              <span style={{ color: t.primary }}>From Geometry</span>
            </h2>
            <p style={{ fontSize: 16, lineHeight: 1.75, color: t.fgSoft, maxWidth: 460, margin: "0 0 16px" }}>
              Most platforms are assembled from separate parts — a networking layer, a security system,
              a timing protocol — bolted together and hoped to be compatible. PlenumNET is different.
              Every subsystem is derived from one{" "}
              <strong style={{ color: t.primary }}>13-dimensional geometric structure</strong>.
            </p>
            <p style={{ fontSize: 15, lineHeight: 1.75, color: t.fgSoft, maxWidth: 460, margin: 0 }}>
              The network topology, the encryption engine, the address system, and the timing protocol
              are all expressions of the same object. They don't just interoperate — they're
              mathematically guaranteed to be consistent.
            </p>
          </FadeIn>
          <FadeIn delay={200}>
            <MetatronsCube size={440} />
          </FadeIn>
        </div>
      </section>

      <section style={{ maxWidth: 1140, margin: "0 auto", padding: "40px 28px 80px", position: "relative", zIndex: 1 }}>
        <FadeIn>
          <div style={{ textAlign: "center", marginBottom: 48 }}>
            <h3 style={{ fontSize: 28, fontWeight: 700, lineHeight: 1.2, margin: "0 0 12px", color: t.fg }}>
              One Object. Five Subsystems.
            </h3>
            <p style={{ fontSize: 15, lineHeight: 1.75, color: t.fgSoft, maxWidth: 620, margin: "0 auto" }}>
              The 13D ternary cube has 3¹³ = 1,594,323 vertices. Its symmetry group determines
              how packets route, how the sponge diffuses, how addresses validate, and how timing
              tokens encode.
            </p>
          </div>
        </FadeIn>

        <div style={{
          display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: 1,
          background: t.cardBorder, borderRadius: RADIUS.lg, overflow: "hidden", marginBottom: 48,
        }}>
          <Stat value={13} label="Named Axes" sub="Metatron's circles" delay={0} />
          <Stat value={1594323} label="Vertices" sub="3¹³ per cube · scales infinitely" delay={100} />
          <Stat value={715} label="Tesseract Families" sub="C(13,4) 4D sub-cubes" delay={200} />
          <Stat value={28} label="Angular Positions" sub="Z₂₈ ternary circle" delay={300} />
        </div>

        <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: 14 }}>
          <SubCard delay={0} icon="◎" tag="Topology" title="Torsion Network" axis="All 13 axes → network graph"
            desc="Vertices ARE network nodes. Hamming distance IS hop count. Geodesic routing follows cube adjacency — no separate topology layer." />
          <SubCard delay={80} icon="⬡" tag="Cryptography" title="Keyed Sponge" axis="Inner ring (axes 1–6) → 3⁶ = 729 trits"
            desc="The sponge state is a 6D sub-cube. Permutations from S₃ ≀ S₆ — every element is a Hamming-preserving bijection. No precomputed tables." />
          <SubCard delay={160} icon="◇" tag="Security" title="Address Sentinel" axis="Rep C {1,2,3} → zero is impossible"
            desc="Bijective ternary makes zero structurally impossible. A zero in any address digit is mathematical proof of forgery — constant-time, opaque errors." />
          <SubCard delay={240} icon="△" tag="HPTP" title="Timing Tokens" axis="Z₂₈ · 13° = 364° = Saturnian year"
            desc="28 angular positions encode calendar conversions, timestamp tokens, and regulatory compliance. All timing math inherits from one cyclic group." />
          <SubCard delay={320} icon="☐" tag="Isolation" title="Shell Security Domains" axis="Depth axis (axis 12) → 3 shells"
            desc="Inner, Void, Outer — three shells partition vertices into security domains. Cross-shell transitions require explicit authorization." />
          <SubCard delay={400} icon="≡" tag="Diffusion" title="Round Constants" axis="Magic Square [111, 14, 208] mod 3"
            desc="Sponge constants derived at compile time from the Saturnian circulant matrix. Auditable derivation, magic sum 333." />
        </div>
      </section>

      <section style={{ maxWidth: 1140, margin: "0 auto", padding: "80px 28px", borderTop: `1px solid ${dividerColor}`, position: "relative", zIndex: 1 }} data-testid="section-squaring-the-circle">
        <FadeIn>
          <div style={{ textAlign: "center", marginBottom: 48 }}>
            <SectionLabel text="Foundational Axiom" />
            <h3 style={{ fontSize: 28, fontWeight: 700, lineHeight: 1.2, margin: "0 0 12px", color: t.fg }}>
              Squaring the Circle.{" "}
              <span style={{ color: t.primary }}>Exactly.</span>
            </h3>
            <p style={{ fontSize: 15, lineHeight: 1.75, color: t.fgSoft, maxWidth: 640, margin: "0 auto" }}>
              Classical mathematics proved squaring the circle impossible under Euclidean
              compass-and-straightedge axioms. PlenumNET derives the circle from different first
              principles — the base-3 repunit identity 111111₃ = 364 — producing exact integer
              geometry where π = 14 and every constant closes in whole numbers.
            </p>
          </div>
        </FadeIn>

        <FadeIn delay={100}>
          <div style={{
            display: "grid", gridTemplateColumns: isMobile ? "repeat(2, 1fr)" : "repeat(4, 1fr)", gap: 1,
            background: t.cardBorder, borderRadius: RADIUS.lg, overflow: "hidden", marginBottom: 32,
          }}>
            <Stat value={364} label="Ternary Degrees" sub="111111₃ = full circle" delay={0} />
            <Stat value={14} label="π (Exact)" sub="integer — not 3.14159…" delay={100} />
            <Stat value={13} label="1 Radian" sub="= 111₃ = T₇ degrees" delay={200} />
            <Stat value={28} label="2π Radians" sub="= 27 trits + 1 confidence" delay={300} />
          </div>
        </FadeIn>

        <FadeIn delay={200}>
          <div style={{
            padding: "28px 32px", background: t.card, border: `1px solid ${t.cardBorder}`,
            borderRadius: RADIUS.lg, boxShadow: t.shadow, marginBottom: 32,
          }} data-testid="card-squaring-derivation">
            <div style={{ fontSize: 9, fontFamily: FONTS.mono, letterSpacing: 1.5, color: t.primary, marginBottom: 16, fontWeight: 600 }}>DERIVATION CHAIN</div>
            <div style={{
              fontFamily: FONTS.mono, fontSize: 13, lineHeight: 2.4, color: t.fgSoft,
              padding: "18px 22px", background: t.primaryDim,
              border: `1px solid ${t.primaryBorder}`, borderRadius: RADIUS.md,
            }}>
              <div><span style={{ color: t.fgMuted, fontSize: 10, display: "inline-block", width: 90 }}>ontology</span> 3³ = <span style={{ color: t.fg, fontWeight: 600 }}>27</span> <span style={{ color: t.fgMuted }}> core trits</span></div>
              <div><span style={{ color: t.fgMuted, fontSize: 10, display: "inline-block", width: 90 }}>square</span> 3² = <span style={{ color: t.fg, fontWeight: 600 }}>9</span> <span style={{ color: t.fgMuted }}> confidence levels</span></div>
              <div><span style={{ color: t.fgMuted, fontSize: 10, display: "inline-block", width: 90 }}>dimensions</span> 27 + 1 = <span style={{ color: t.fg, fontWeight: 600 }}>28</span> <span style={{ color: t.fgMuted }}> effective dimensions</span></div>
              <div><span style={{ color: t.fgMuted, fontSize: 10, display: "inline-block", width: 90 }}>closure</span> 13 × 28 = <span style={{ color: t.primary, fontWeight: 600 }}>364</span> <span style={{ color: t.fgMuted }}> circle closed</span></div>
              <div><span style={{ color: t.fgMuted, fontSize: 10, display: "inline-block", width: 90 }}>repunit</span> 111111₃ = <span style={{ color: t.primary, fontWeight: 600 }}>364</span> <span style={{ color: t.fgMuted }}> identity confirmed</span></div>
              <div><span style={{ color: t.fgMuted, fontSize: 10, display: "inline-block", width: 90 }}>radian</span> 364 / 28 = <span style={{ color: t.fg, fontWeight: 600 }}>13°</span> <span style={{ color: t.fgMuted }}> exact</span></div>
              <div><span style={{ color: t.fgMuted, fontSize: 10, display: "inline-block", width: 90 }}>π</span> 28 / 2 = <span style={{ color: t.primary, fontWeight: 700 }}>14</span> <span style={{ color: t.fgMuted }}> exact integer</span></div>
            </div>
          </div>
        </FadeIn>

        <div style={{ display: "grid", gridTemplateColumns: isMobile ? "1fr" : "1fr 1fr", gap: 16 }}>
          <FadeIn delay={300}>
            <div style={{
              padding: "22px 24px", background: t.card, border: `1px solid ${t.cardBorder}`,
              borderRadius: RADIUS.lg, boxShadow: t.shadow, height: "100%",
            }} data-testid="card-classical-impossibility">
              <div style={{ fontSize: 9, fontFamily: FONTS.mono, letterSpacing: 1.5, color: t.esoteric, marginBottom: 12, fontWeight: 600 }}>CLASSICAL PROBLEM</div>
              <p style={{ fontSize: 14, lineHeight: 1.75, color: t.fgSoft, margin: "0 0 10px" }}>
                In Euclidean geometry, π is transcendental — it cannot be expressed as the root of any
                polynomial with rational coefficients. Under those axioms, squaring the circle is provably
                impossible with compass and straightedge.
              </p>
              <p style={{ fontSize: 13, lineHeight: 1.75, color: t.fgMuted, margin: 0 }}>
                That constraint is specific to the Euclidean derivation of the circle from continuous ratios.
                A different derivation from different first principles produces different constants.
              </p>
            </div>
          </FadeIn>
          <FadeIn delay={380}>
            <div style={{
              padding: "22px 24px", background: t.card, border: `1px solid ${t.cardBorder}`,
              borderRadius: RADIUS.lg, boxShadow: t.shadow, height: "100%",
            }} data-testid="card-ternary-resolution">
              <div style={{ fontSize: 9, fontFamily: FONTS.mono, letterSpacing: 1.5, color: t.primary, marginBottom: 12, fontWeight: 600 }}>TERNARY RESOLUTION — THE CLOSED GEOMETRIC LOOP</div>
              <p style={{ fontSize: 14, lineHeight: 1.75, color: t.fgSoft, margin: "0 0 10px" }}>
                PlenumNET discretizes the circle as exactly 364 degrees.
                27 ontological trits + 1 Collision Resolution Digit = 28 effective dimensions.
                13 × 28 = 364 — a perfect closure where 13 (routing dimension, Tribonacci T₇,
                ternary radian) completes the loop. This is no coincidence: the geometry returns to itself.
              </p>
              <p style={{ fontSize: 13, lineHeight: 1.75, color: t.fgSoft, margin: "0 0 8px" }}>
                Fixed 27-hop routing diameter, ontological clustering preserved,
                per-instance uniqueness without displacement, zero waste in 14-byte wire format.
              </p>
              <p style={{ fontSize: 13, lineHeight: 1.75, color: t.fgMuted, margin: 0, fontStyle: "italic" }}>
                The universe of discourse closes — elegantly, inevitably.
              </p>
            </div>
          </FadeIn>
        </div>
      </section>

      <section style={{ maxWidth: 1140, margin: "0 auto", padding: "80px 28px", borderTop: `1px solid ${dividerColor}`, position: "relative", zIndex: 1 }} data-testid="section-inter-cube">
        <FadeIn>
          <div style={{ textAlign: "center", marginBottom: 48 }}>
            <SectionLabel text="Inter-Cube Infrastructure" />
            <h3 style={{ fontSize: 28, fontWeight: 700, lineHeight: 1.2, margin: "0 0 12px", color: t.fg }}>
              Four Services. Pure Geometry.{" "}
              <span style={{ color: t.primary }}>Zero Routing Tables.</span>
            </h3>
            <p style={{ fontSize: 15, lineHeight: 1.75, color: t.fgSoft, maxWidth: 640, margin: "0 auto 8px" }}>
              When the network grows beyond a single cube, these four services handle connections
              between cubes — and they do it without routing tables. Because routing is pure geometry,
              the network scales infinitely: stack another 13 trits and the address space jumps from
              1.6 million to 2.5 trillion nodes with no architectural change. This works today.
            </p>
            <p style={{ fontSize: 14, lineHeight: 1.75, color: t.fgMuted, maxWidth: 640, margin: "0 auto" }}>
              Greedy geodesic forwarding across the 13D ternary cube.
              Hamming distance IS hop count. Adjacency IS the routing table. Four services orchestrate
              the control plane — the geometry does the rest.
            </p>
          </div>
        </FadeIn>

        {topology && (
          <FadeIn delay={100}>
            <div style={{
              display: "grid", gridTemplateColumns: isMobile ? "repeat(2, 1fr)" : "repeat(5, 1fr)", gap: 1,
              background: t.cardBorder, borderRadius: RADIUS.lg, overflow: "hidden", marginBottom: 32,
            }}>
              <Stat value={2541865828329} label="Address Space" sub="3²⁶ Rep C vertices · scales infinitely" delay={0} />
              <Stat value={topology.neighborsPerCube} label="Neighbors" sub="per cube (2 × 13)" delay={100} />
              <Stat value={20726199} label="Encrypted Tunnels" sub="unique PQ tunnels per populated cube" delay={200} />
              <Stat value={4} label="Services" sub="GLB · CON · CRS · FTS" delay={300} />
              <Stat value={0} label="Routing Tables" sub="pure geometric forwarding" delay={400} />
            </div>
          </FadeIn>
        )}

        <div style={{ display: "grid", gridTemplateColumns: isMobile ? "1fr" : "repeat(2, 1fr)", gap: 16, marginBottom: 16 }}>
          <ServiceCard delay={0} icon="◎" tag="GLB" name="Geometric Load Balancer"
            stat="O(d)" statLabel="forwarding"
            desc="Greedy geodesic forwarding with flow affinity. FNV-1a hashes the flow ID to select a consistent dimension, ensuring packets in the same flow traverse identical paths. Dead neighbors trigger detour computation." />
          <ServiceCard delay={80} icon="⬡" tag="CON" name="Cube Overlay Network"
            stat="26" statLabel="tunnel peers"
            desc="20.7M unique post-quantum encrypted tunnels per populated cube — each key derived from the geometric positions of the two endpoints via TLSponge-385. Full tunnel state machine: Init → Handshake → Active → Rekeying." />
          <ServiceCard delay={160} icon="◇" tag="CRS" name="Cube Registration Service"
            stat="3¹³" statLabel="address space"
            desc="Bitmap allocator over 1,594,323 Rep C addresses with flatIndex/fromFlatIndex bijection. Sequential scan with nextHint for deterministic allocation. Heartbeat-based endpoint updates." />
          <ServiceCard delay={240} icon="△" tag="FTS" name="Fault Tolerance Service"
            stat="4" statLabel="health states"
            desc="Four-state health machine: Up → Suspect → Down → Recovering. Configurable miss thresholds and grace periods. Dead-set publication feeds GLB for real-time path avoidance." />
        </div>

        <div style={{ display: "grid", gridTemplateColumns: isMobile ? "1fr" : "1fr 1fr", gap: 16 }}>
          <FadeIn delay={300}>
            <div style={{
              padding: "22px 24px", background: t.card, border: `1px solid ${t.cardBorder}`,
              borderRadius: RADIUS.lg, boxShadow: t.shadow,
            }} data-testid="card-routing-principle">
              <div style={{ fontSize: 9, fontFamily: FONTS.mono, letterSpacing: 1.5, color: t.primary, marginBottom: 12, fontWeight: 600 }}>ROUTING PRINCIPLE</div>
              <div style={{
                fontFamily: FONTS.mono, fontSize: 13, lineHeight: 2.2, color: t.fgSoft,
                padding: "14px 18px", background: t.primaryDim,
                border: `1px solid ${t.primaryBorder}`, borderRadius: RADIUS.md,
              }}>
                <div><span style={{ color: t.fgMuted, fontSize: 10 }}>distance</span>(<span style={{ color: t.primary }}>src</span>, <span style={{ color: t.esoteric }}>dst</span>) = <span style={{ color: t.fg, fontWeight: 600 }}>Hamming(src, dst)</span></div>
                <div><span style={{ color: t.fgMuted, fontSize: 10 }}>next_hop</span> = fix <span style={{ color: t.primary }}>one trit</span> where src[i] ≠ dst[i]</div>
                <div><span style={{ color: t.fgMuted, fontSize: 10 }}>paths</span> = <span style={{ color: t.esoteric, fontWeight: 600 }}>d!</span> <span style={{ color: t.fgMuted }}>(d = Hamming distance)</span></div>
              </div>
              <div style={{ fontSize: 11, color: t.fgMuted, marginTop: 10, fontFamily: FONTS.mono }}>
                Every path is shortest. Flow affinity selects deterministically.
              </div>
            </div>
          </FadeIn>

          <RoutingDemo />
        </div>
      </section>

      <section style={{ maxWidth: 1140, margin: "0 auto", padding: "80px 28px", borderTop: `1px solid ${dividerColor}`, position: "relative", zIndex: 1 }}>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 80 }}>
          <FadeIn>
            <SectionLabel text="Algebraic Core" />
            <h3 style={{ fontSize: 28, fontWeight: 700, lineHeight: 1.2, margin: "0 0 12px", color: t.fg }}>
              No Lookup Tables. Pure Arithmetic.
            </h3>
            <p style={{ fontSize: 15, lineHeight: 1.75, color: t.fgSoft, maxWidth: 440, margin: "0 0 16px" }}>
              Every value in the system can be transformed using just two operations: multiply and add,
              then wrap around at 3. Think of adjusting a recipe — you can scale ingredients and shift
              amounts, but the proportions always stay in the ternary world.
            </p>
            <p style={{ fontSize: 14, lineHeight: 1.75, color: t.fgMuted, maxWidth: 440, margin: "0 0 24px" }}>
              This means no lookup tables, no data-dependent memory access,
              and no timing side-channels — all six possible permutations are computed in constant time.
            </p>

            <AlgebraicDetails />
          </FadeIn>

          <FadeIn delay={150}>
            <SectionLabel text="Deterministic Constants" />
            <h3 style={{ fontSize: 28, fontWeight: 700, lineHeight: 1.2, margin: "0 0 12px", color: t.fg }}>
              Constants From Geometry, Not Choice.
            </h3>
            <p style={{ fontSize: 15, lineHeight: 1.75, color: t.fgSoft, maxWidth: 440, margin: "0 0 24px" }}>
              The circulant matrix [111, 14, 208] has row/column/diagonal sum 333.
              Reduced mod 3, it produces deterministic sponge round constants — no arbitrary choices, fully auditable.
            </p>

            <div style={{ display: "flex", justifyContent: "center", margin: "0 0 24px" }}>
              <MagicSquare />
            </div>

            <div style={{
              padding: "18px 22px", background: t.muted,
              border: `1px solid ${t.cardBorder}`, borderRadius: RADIUS.lg,
              fontFamily: FONTS.mono, fontSize: 12, lineHeight: 2.2,
            }}>
              <div style={{ fontSize: 9.5, color: t.fgMuted, letterSpacing: 1, marginBottom: 4 }}>COMPILE-TIME DERIVATION</div>
              <div><span style={{ color: t.balance, fontWeight: 600 }}>111</span><span style={{ color: t.fgMuted }}> mod 3 = 0 →</span><span style={{ color: t.fgSoft }}> 0 </span><span style={{ color: t.fgFaint }}>(identity)</span></div>
              <div><span style={{ color: t.esoteric, fontWeight: 600 }}>&nbsp;14</span><span style={{ color: t.fgMuted }}> mod 3 = 2 →</span><span style={{ color: t.fgSoft }}> −1</span><span style={{ color: t.fgFaint }}> (shift-2)</span></div>
              <div><span style={{ color: t.cosmic, fontWeight: 600 }}>208</span><span style={{ color: t.fgMuted }}> mod 3 = 1 →</span><span style={{ color: t.fgSoft }}> +1</span><span style={{ color: t.fgFaint }}> (shift-1)</span></div>
              <div style={{ marginTop: 8, fontSize: 10, color: t.fgMuted }}>Period 9 = 3² · tiles 81× into 729-trit sponge state</div>
            </div>
          </FadeIn>
        </div>
      </section>

      <section style={{ maxWidth: 1140, margin: "0 auto", padding: "80px 28px", borderTop: `1px solid ${dividerColor}`, position: "relative", zIndex: 1 }}>
        <FadeIn>
          <div style={{ textAlign: "center", marginBottom: 48 }}>
            <h3 style={{ fontSize: 28, fontWeight: 700, lineHeight: 1.2, margin: "0 0 12px", color: t.fg }}>
              Three Representations. One Sentinel.
            </h3>
            <p style={{ fontSize: 15, lineHeight: 1.75, color: t.fgSoft, maxWidth: 600, margin: "0 auto" }}>
              The architecture mentions "three bijective representations" — here's why they matter,
              and why Rep C makes forgery detection a mathematical certainty.
            </p>
          </div>
        </FadeIn>

        <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: 16 }}>
          <RepCard name="Rep A" subtitle="Balanced Ternary · Computational" digits="{−1, 0, +1}"
            color={t.green} bg={t.greenBg} border={`hsla(145, 50%, 45%, 0.12)`}
            desc="Native arithmetic. Rust arrays, VM registers, sponge permutations. Zero is the balanced midpoint — valid here." />
          <RepCard name="Rep B" subtitle="Standard Ternary · Wire Encoding" digits="{0, 1, 2}"
            color={t.esoteric} bg={t.esotericBg} border={`hsla(270, 50%, 55%, 0.12)`}
            desc="Unsigned encoding for flat index computation and network wire format. Maps directly to base-3 digits." />
          <RepCard name="Rep C" subtitle="Bijective Ternary · Trust Boundary" digits="{1, 2, 3}"
            color={t.primary} bg={t.primaryDim} border={t.primaryBorder} highlight
            desc="Zero is structurally impossible. Used at every trust boundary — VM operands, torsion routing, address validation. Zero anywhere = proof of forgery." />
        </div>

        <FadeIn delay={200}>
          <div style={{
            marginTop: 24, padding: "24px 28px", background: t.card,
            border: `1px solid ${t.cardBorder}`, borderRadius: RADIUS.lg,
            boxShadow: t.shadow,
            display: "grid", gridTemplateColumns: "auto 1fr", gap: 20, alignItems: "center",
          }} data-testid="card-sentinel-property">
            <div style={{
              width: 52, height: 52, borderRadius: "50%",
              background: t.primaryDim, border: `1px solid ${t.primaryBorder}`,
              display: "flex", alignItems: "center", justifyContent: "center",
              fontSize: 24, fontFamily: FONTS.mono, color: t.primary, fontWeight: 700,
            }}>0</div>
            <div>
              <div style={{ fontSize: 15, fontWeight: 600, color: t.fg, marginBottom: 4 }}>The Sentinel Property</div>
              <div style={{ fontSize: 13, lineHeight: 1.75, color: t.fgSoft }}>
                In Rep C, zero is not a valid digit, axis number, or address component.
                Its appearance anywhere is{" "}
                <strong style={{ color: t.primary }}>structurally impossible from valid computation</strong>.
                Validation is constant-time (bitwise, no branching on secrets) with opaque error reporting.
              </div>
            </div>
          </div>
        </FadeIn>
      </section>

      <section style={{
        maxWidth: 800, margin: "0 auto", padding: "80px 28px 100px",
        borderTop: `1px solid ${dividerColor}`, textAlign: "center", position: "relative", zIndex: 1,
      }}>
        <FadeIn>
          <div style={{ fontSize: 10, fontFamily: FONTS.mono, letterSpacing: 4, color: t.fgFaint, marginBottom: 20 }}>
            GEOMETRIA PRIMUS
          </div>
          <h3 style={{ fontSize: 30, fontWeight: 700, lineHeight: 1.25, margin: "0 0 16px", color: t.fg }}>
            The Geometry Isn't Decoration.{" "}
            <span style={{ color: t.primary }}>It's Why the Architecture Works.</span>
          </h3>
          <p style={{ fontSize: 15, lineHeight: 1.75, color: t.fgSoft, maxWidth: 540, margin: "0 auto" }}>
            When your network topology, cryptographic diffusion, address validation,
            and timing infrastructure all emerge from one 13-dimensional structure,
            they don't just interoperate — they're mathematically guaranteed to be consistent.
          </p>
          <div style={{ display: "flex", gap: 12, justifyContent: "center", marginTop: 32, flexWrap: "wrap" as const }}>
            <a href="/docs" data-testid="button-read-whitepaper" style={{
              padding: "12px 32px", fontSize: 14, fontWeight: 600,
              background: t.primary, color: "#fff", border: "none", borderRadius: RADIUS.md,
              cursor: "pointer", fontFamily: FONTS.sans, letterSpacing: 0.3,
              textDecoration: "none", display: "inline-block",
            }}>Read the Whitepaper</a>
            <a href="https://github.com/SigmaWolf-8/Ternary" target="_blank" rel="noopener noreferrer" data-testid="button-view-source" style={{
              padding: "12px 32px", fontSize: 14, fontWeight: 600,
              background: "transparent", color: t.primary,
              border: `1px solid ${t.primaryBorder}`, borderRadius: RADIUS.md,
              cursor: "pointer", fontFamily: FONTS.sans, letterSpacing: 0.3,
              textDecoration: "none", display: "inline-block",
            }}>View Source</a>
          </div>
          <div style={{ marginTop: 48, fontSize: 11, fontFamily: FONTS.mono, color: t.fgFaint, letterSpacing: 1.5 }}>
            TEMPORIS ARCHITECTURA ABSOLUTA
          </div>
        </FadeIn>
      </section>
    </div>
  );
}
