/**
 * Copyright (c) 2025–2026 Capomastro Holdings Ltd. (Canada)
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

import { useState } from "react";

const PHI = 1.6180339887;

interface MoonData {
  num: number | string;
  name: string;
  start: string;
  end: string;
  arc: string;
  days?: number;
  isIntercalary?: boolean;
}

const originalMoons: MoonData[] = [
  { num: 1, name: "Magnetic Moon", start: "Apr 1", end: "Apr 28", arc: "pre" },
  { num: 2, name: "Lunar Moon", start: "Apr 29", end: "May 26", arc: "pre" },
  { num: 3, name: "Electric Moon", start: "May 27", end: "Jun 23", arc: "pre" },
  { num: 4, name: "Self-Existing Moon", start: "Jun 24", end: "Jul 21", arc: "pre" },
  { num: 5, name: "Overtone Moon", start: "Jul 22", end: "Aug 18", arc: "pre" },
  { num: 6, name: "Rhythmic Moon", start: "Aug 19", end: "Sep 15", arc: "pre" },
  { num: 7, name: "Resonant Moon", start: "Sep 16", end: "Oct 13", arc: "pre" },
  { num: 8, name: "Galactic Moon", start: "Oct 14", end: "Nov 10", arc: "pre" },
  { num: "DOT", name: "Day Out of Time", start: "Nov 11", end: "Nov 11", arc: "phi", days: 1 },
  { num: 9, name: "Solar Moon", start: "Nov 12", end: "Dec 9", arc: "post" },
  { num: 10, name: "Planetary Moon", start: "Dec 10", end: "Jan 6", arc: "post" },
  { num: 11, name: "Spectral Moon", start: "Jan 7", end: "Feb 3", arc: "post" },
  { num: 12, name: "Crystal Moon", start: "Feb 4", end: "Mar 3", arc: "post" },
  { num: 13, name: "Cosmic Moon", start: "Mar 4", end: "Mar 31", arc: "post" },
];

const alternateMoons: MoonData[] = [
  { num: 1, name: "Magnetic Moon", start: "Apr 1", end: "Apr 28", arc: "pre", days: 28 },
  { num: 2, name: "Lunar Moon", start: "May 1", end: "May 28", arc: "pre", days: 28 },
  { num: 3, name: "Electric Moon", start: "Jun 1", end: "Jun 28", arc: "pre", days: 28 },
  { num: 4, name: "Self-Existing Moon", start: "Jul 1", end: "Jul 28", arc: "pre", days: 28 },
  { num: 5, name: "Overtone Moon", start: "Aug 1", end: "Aug 28", arc: "pre", days: 28 },
  { num: 6, name: "Rhythmic Moon", start: "Sep 1", end: "Sep 28", arc: "pre", days: 28 },
  { num: 7, name: "Resonant Moon", start: "Oct 1", end: "Oct 28", arc: "pre", days: 28 },
  { num: 8, name: "Galactic Moon", start: "Nov 1", end: "Nov 28", arc: "pre", days: 28 },
  { num: "DOT", name: "Day Out of Time", start: "\u2014", end: "\u2014", arc: "phi", days: 1 },
  { num: 13, name: "Cosmic Moon", start: "Intercalary", end: "28 Days", arc: "intercalary", days: 28, isIntercalary: true },
  { num: 9, name: "Solar Moon", start: "Dec 1", end: "Dec 28", arc: "post", days: 28 },
  { num: 10, name: "Planetary Moon", start: "Jan 1", end: "Jan 28", arc: "post", days: 28 },
  { num: 11, name: "Spectral Moon", start: "Feb 1", end: "Feb 28", arc: "post", days: 28 },
  { num: 12, name: "Crystal Moon", start: "Mar 1", end: "Mar 28", arc: "post", days: 28 },
];

const goldenAngle = 137.508;

function SpiralDot({ index, total, radius }: { index: number; total: number; radius: number }) {
  const angle = index * goldenAngle * (Math.PI / 180);
  const r = radius * Math.sqrt(index / total);
  const x = 50 + r * Math.cos(angle);
  const y = 50 + r * Math.sin(angle);
  return <circle cx={x} cy={y} r={1.2} fill="#c9a84c" opacity={0.12 + (index / total) * 0.35} />;
}

function GoldenSpiral() {
  return (
    <svg viewBox="0 0 100 100" style={{ position: "absolute", top: 0, left: 0, width: "100%", height: "100%", opacity: 0.06, pointerEvents: "none" }}>
      {Array.from({ length: 200 }, (_, i) => (
        <SpiralDot key={i} index={i} total={200} radius={48} />
      ))}
    </svg>
  );
}

function MoonRow({ moon, side, index, isHovered, onHover, onLeave }: {
  moon: MoonData;
  side: string;
  index: number;
  isHovered: boolean;
  onHover: () => void;
  onLeave: () => void;
}) {
  const isDot = moon.arc === "phi";
  const isIntercalary = moon.isIntercalary;

  const arcColors: Record<string, { main: string; dim: string }> = {
    pre: { main: "#c9a84c", dim: "rgba(201,168,76,0.5)" },
    post: { main: "#7090b8", dim: "rgba(112,144,184,0.5)" },
    phi: { main: "#c9a84c", dim: "#c9a84c" },
    intercalary: { main: "#d4845a", dim: "rgba(212,132,90,0.5)" },
  };
  const colors = arcColors[moon.arc];

  if (isDot) {
    return (
      <div style={{
        display: "flex", alignItems: "center", gap: 10, padding: "10px 12px",
        background: "rgba(201,168,76,0.06)",
        borderTop: "1px solid rgba(201,168,76,0.12)",
        borderBottom: "1px solid rgba(201,168,76,0.12)",
        animation: `fadeIn 0.5s ease-out ${0.04 * index}s both`
      }} data-testid={`moon-row-dot-${side}`}>
        <div style={{
          width: 8, height: 8, borderRadius: "50%", background: "#c9a84c",
          animation: "dotPulse 3s ease-in-out infinite", flexShrink: 0
        }} />
        <div style={{ flex: 1, minWidth: 0 }}>
          <div style={{ fontSize: 14, fontWeight: 600, color: "#c9a84c", letterSpacing: 0.5 }}>
            Day Out of Time
          </div>
          <div style={{ fontFamily: "'JetBrains Mono', monospace", fontSize: 9, color: "#8a7a5c", marginTop: 1 }}>
            {side === "original" ? "Nov 11 \u00b7 Day 225 \u00b7 \u230a364/\u03c6\u230b" : "Day 225 \u00b7 \u03c6-fracture \u00b7 1 day"}
          </div>
        </div>
        <div style={{
          fontFamily: "'JetBrains Mono', monospace", fontSize: 9, color: "#c9a84c",
          letterSpacing: 1, padding: "2px 8px", border: "1px solid rgba(201,168,76,0.25)"
        }}>
          \u03c6 SPLIT
        </div>
      </div>
    );
  }

  if (isIntercalary) {
    return (
      <div style={{
        display: "flex", alignItems: "center", gap: 10, padding: "12px 12px",
        background: "rgba(212,132,90,0.05)",
        borderTop: "1px solid rgba(212,132,90,0.12)",
        borderBottom: "1px solid rgba(212,132,90,0.12)",
        animation: `fadeIn 0.5s ease-out ${0.04 * index}s both`
      }} data-testid="moon-row-intercalary">
        <div style={{
          fontFamily: "'JetBrains Mono', monospace", fontSize: 11,
          color: "rgba(212,132,90,0.6)", width: 24, textAlign: "center", flexShrink: 0
        }}>13</div>
        <div style={{
          width: 6, height: 6, borderRadius: "50%", background: "#d4845a", opacity: 0.7, flexShrink: 0
        }} />
        <div style={{ flex: 1, minWidth: 0 }}>
          <div style={{ fontSize: 15, fontWeight: 600, color: "#d4845a", letterSpacing: 0.5 }}>
            Cosmic Moon <span style={{ fontSize: 11, fontWeight: 400, fontStyle: "italic", color: "#a06840" }}>{"\u2014"} Intercalary</span>
          </div>
          <div style={{ fontFamily: "'JetBrains Mono', monospace", fontSize: 9, color: "#7a5a3c", marginTop: 1 }}>
            28 epagomenal days {"\u00b7"} collected from 29th{"\u2013"}31st of months
          </div>
        </div>
        <div style={{
          fontFamily: "'JetBrains Mono', monospace", fontSize: 11, color: "#d4845a", flexShrink: 0
        }}>28d</div>
      </div>
    );
  }

  return (
    <div
      onMouseEnter={onHover} onMouseLeave={onLeave}
      style={{
        display: "flex", alignItems: "center", gap: 10, padding: "8px 12px",
        borderBottom: "1px solid rgba(201,168,76,0.04)",
        transition: "background 0.3s",
        background: isHovered ? "rgba(201,168,76,0.03)" : "transparent",
        animation: `fadeIn 0.4s ease-out ${0.04 * index}s both`,
        cursor: "default"
      }}
      data-testid={`moon-row-${side}-${moon.num}`}
    >
      <div style={{
        fontFamily: "'JetBrains Mono', monospace", fontSize: 11,
        color: colors.dim, width: 24, textAlign: "center", flexShrink: 0
      }}>
        {String(moon.num).padStart(2, "0")}
      </div>
      <div style={{
        width: 6, height: 6, borderRadius: "50%", background: colors.main,
        opacity: isHovered ? 1 : 0.35, transition: "opacity 0.3s", flexShrink: 0
      }} />
      <div style={{ flex: 1, minWidth: 0 }}>
        <div style={{
          fontSize: 14, fontWeight: isHovered ? 600 : 400,
          color: isHovered ? colors.main : "#e8e4dc",
          transition: "all 0.3s", letterSpacing: 0.3
        }}>
          {moon.name}
        </div>
      </div>
      <div style={{
        fontFamily: "'JetBrains Mono', monospace", fontSize: 12,
        color: "#a09888", display: "flex", gap: 5, alignItems: "center", flexShrink: 0
      }}>
        <span>{moon.start}</span>
        <span style={{ color: "#3a3428", fontSize: 9 }}>{"\u2192"}</span>
        <span>{moon.end}</span>
      </div>
    </div>
  );
}

export default function ThirteenMoonPage() {
  const [hoveredMoon, setHoveredMoon] = useState<string | null>(null);
  const [activeView, setActiveView] = useState("comparison");

  return (
    <div style={{
      minHeight: "100vh", background: "#0a0b0f", color: "#e8e4dc",
      fontFamily: "'Cormorant Garamond', Georgia, serif",
      position: "relative", overflow: "hidden"
    }} data-testid="page-thirteen-moon">
      <style>{`
        @import url('https://fonts.googleapis.com/css2?family=Cormorant+Garamond:ital,wght@0,300;0,400;0,500;0,600;0,700;1,400;1,500&family=JetBrains+Mono:wght@300;400;500&display=swap');
        @keyframes fadeIn {
          from { opacity: 0; transform: translateY(8px); }
          to { opacity: 1; transform: translateY(0); }
        }
        @keyframes glowPulse {
          0%, 100% { box-shadow: 0 0 15px rgba(201,168,76,0.04); }
          50% { box-shadow: 0 0 30px rgba(201,168,76,0.1); }
        }
        @keyframes dotPulse {
          0%, 100% { transform: scale(1); opacity: 0.7; }
          50% { transform: scale(1.5); opacity: 1; }
        }
        .moon-tab-btn {
          background: none; border: 1px solid rgba(201,168,76,0.2); color: #8a8070;
          padding: 7px 18px; font-family: 'JetBrains Mono', monospace; font-size: 10px;
          letter-spacing: 1.5px; text-transform: uppercase; cursor: pointer; transition: all 0.3s;
          border-radius: 0;
        }
        .moon-tab-btn:hover { border-color: rgba(201,168,76,0.5); color: #c9a84c; }
        .moon-tab-btn.active { background: rgba(201,168,76,0.1); border-color: #c9a84c; color: #c9a84c; }
      `}</style>

      <GoldenSpiral />

      <div style={{ maxWidth: 1150, margin: "0 auto", padding: "36px 20px", position: "relative" }}>

        <header style={{ textAlign: "center", marginBottom: 40, animation: "fadeIn 0.7s ease-out" }}>
          <div style={{
            fontFamily: "'JetBrains Mono', monospace", fontSize: 9, letterSpacing: 4,
            color: "#4a4438", textTransform: "uppercase", marginBottom: 14
          }}>
            Capomastro Holdings Ltd. {"\u00b7"} Applied Physics Division
          </div>
          <h1 style={{
            fontSize: "clamp(26px, 4.5vw, 44px)", fontWeight: 300, letterSpacing: 2,
            color: "#e8e4dc", lineHeight: 1.2, marginBottom: 6
          }} data-testid="text-thirteen-moon-title">
            The 13-Moon <span style={{ color: "#c9a84c", fontWeight: 500 }}>Harmonic</span> Calendar
          </h1>
          <div style={{ fontSize: 15, fontStyle: "italic", color: "#8a8070", marginBottom: 20 }}>
            Salvi Epoch {"\u00b7"} April 1, 2025 {"\u00b7"} Year 1 {"\u2014"} 8/5 Fibonacci Split at{" "}
            <span style={{ color: "#c9a84c", fontWeight: 600, fontStyle: "italic" }}>{"\u03c6"}</span>
          </div>
          <div style={{
            width: 50, height: 1, margin: "0 auto 20px",
            background: "linear-gradient(to right, transparent, #c9a84c, transparent)"
          }} />
          <div style={{ display: "flex", gap: 6, justifyContent: "center", flexWrap: "wrap" }}>
            {[
              { key: "comparison", label: "Side-by-Side" },
              { key: "structure", label: "Day Count" },
              { key: "info", label: "Architecture" }
            ].map(tab => (
              <button key={tab.key} className={`moon-tab-btn ${activeView === tab.key ? "active" : ""}`}
                onClick={() => setActiveView(tab.key)}
                data-testid={`button-tab-${tab.key}`}>
                {tab.label}
              </button>
            ))}
          </div>
        </header>

        <div style={{
          display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(140px, 1fr))",
          gap: 12, marginBottom: 36, animation: "fadeIn 0.8s ease-out 0.15s both"
        }}>
          {[
            { label: "Total Days", value: "365", sub: `13 ${"\u00d7"} 28 + 1 DOT` },
            { label: `Pre-${"\u03c6"} Arc`, value: "8 Moons", sub: "224 days" },
            { label: "DOT", value: "Day 225", sub: `${"\u230a"}364/${"\u03c6"}${"\u230b"}` },
            { label: "Intercalary", value: "28 Days", sub: "Cosmic Moon (13th)" },
            { label: `Post-${"\u03c6"} Arc`, value: "5 Moons", sub: "140 days" },
          ].map((stat, i) => (
            <div key={i} style={{
              background: "rgba(201,168,76,0.03)", border: "1px solid rgba(201,168,76,0.08)",
              padding: "14px 10px", textAlign: "center",
              animation: `glowPulse ${3 + i * 0.4}s ease-in-out infinite`
            }} data-testid={`stat-${i}`}>
              <div style={{ fontFamily: "'JetBrains Mono', monospace", fontSize: 9, letterSpacing: 2, color: "#4a4438", textTransform: "uppercase", marginBottom: 5 }}>
                {stat.label}
              </div>
              <div style={{ fontSize: 22, fontWeight: 600, color: "#c9a84c", lineHeight: 1 }}>
                {stat.value}
              </div>
              <div style={{ fontFamily: "'JetBrains Mono', monospace", fontSize: 9, color: "#5a5244", marginTop: 3 }}>
                {stat.sub}
              </div>
            </div>
          ))}
        </div>

        {activeView === "comparison" && (
          <div style={{ animation: "fadeIn 0.5s ease-out" }} data-testid="view-comparison">
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 20 }}>

              <div>
                <div style={{
                  fontFamily: "'JetBrains Mono', monospace", fontSize: 10, letterSpacing: 2,
                  color: "#5a5244", textTransform: "uppercase", padding: "10px 12px",
                  borderBottom: "1px solid rgba(201,168,76,0.15)", marginBottom: 2
                }}>
                  Original {"\u2014"} Continuous Count
                  <div style={{ fontSize: 8, color: "#3a3428", marginTop: 3, letterSpacing: 1 }}>
                    13 {"\u00d7"} 28 + 1 DOT = 365 {"\u00b7"} Apr 1 {"\u2192"} Mar 31
                  </div>
                </div>
                {originalMoons.map((moon, i) => (
                  <MoonRow key={`orig-${i}`} moon={moon} side="original" index={i}
                    isHovered={hoveredMoon === `orig-${moon.num}`}
                    onHover={() => setHoveredMoon(`orig-${moon.num}`)}
                    onLeave={() => setHoveredMoon(null)} />
                ))}
              </div>

              <div>
                <div style={{
                  fontFamily: "'JetBrains Mono', monospace", fontSize: 10, letterSpacing: 2,
                  color: "#5a5244", textTransform: "uppercase", padding: "10px 12px",
                  borderBottom: "1px solid rgba(201,168,76,0.15)", marginBottom: 2
                }}>
                  Alternate {"\u2014"} 1st-Aligned + Intercalary
                  <div style={{ fontSize: 8, color: "#3a3428", marginTop: 3, letterSpacing: 1 }}>
                    8 {"\u00d7"} 28 + DOT + Cosmic(28) + 4 {"\u00d7"} 28 = 365 {"\u00b7"} 8/5 {"\u03c6"}-split
                  </div>
                </div>
                {alternateMoons.map((moon, i) => (
                  <MoonRow key={`alt-${i}`} moon={moon} side="alternate" index={i}
                    isHovered={hoveredMoon === `alt-${moon.num}`}
                    onHover={() => setHoveredMoon(`alt-${moon.num}`)}
                    onLeave={() => setHoveredMoon(null)} />
                ))}
              </div>
            </div>

            <div style={{
              marginTop: 20, padding: "16px", background: "rgba(212,132,90,0.04)",
              border: "1px solid rgba(212,132,90,0.1)", animation: "fadeIn 0.6s ease-out 0.3s both"
            }} data-testid="section-epagomenal">
              <div style={{
                fontFamily: "'JetBrains Mono', monospace", fontSize: 10, letterSpacing: 1.5,
                color: "#d4845a", textTransform: "uppercase", marginBottom: 10
              }}>
                {"\u25c6"} Cosmic Moon {"\u2014"} Epagomenal / Intercalary Composition
              </div>
              <div style={{
                display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(120px, 1fr))",
                gap: 6, marginBottom: 12
              }}>
                {[
                  { month: "Apr", days: `29${"\u2013"}30`, count: 2 },
                  { month: "May", days: `29${"\u2013"}31`, count: 3 },
                  { month: "Jun", days: `29${"\u2013"}30`, count: 2 },
                  { month: "Jul", days: `29${"\u2013"}31`, count: 3 },
                  { month: "Aug", days: `29${"\u2013"}31`, count: 3 },
                  { month: "Sep", days: `29${"\u2013"}30`, count: 2 },
                  { month: "Oct", days: `29${"\u2013"}31`, count: 3 },
                  { month: "Nov", days: `29${"\u2013"}30`, count: 2 },
                  { month: "Dec", days: `29${"\u2013"}31`, count: 3 },
                  { month: "Jan", days: `29${"\u2013"}31`, count: 3 },
                  { month: "Mar", days: `29${"\u2013"}31`, count: 3 },
                ].map((g, i) => (
                  <div key={i} style={{
                    fontFamily: "'JetBrains Mono', monospace", fontSize: 10, color: "#8a6840",
                    padding: "4px 8px", background: "rgba(212,132,90,0.04)",
                    border: "1px solid rgba(212,132,90,0.08)", display: "flex", justifyContent: "space-between"
                  }}>
                    <span>{g.month} {g.days}</span>
                    <span style={{ color: "#6a5030" }}>{g.count}d</span>
                  </div>
                ))}
              </div>
              <div style={{
                fontFamily: "'JetBrains Mono', monospace", fontSize: 10, color: "#6a5a3c",
                lineHeight: 1.8, borderTop: "1px solid rgba(212,132,90,0.08)", paddingTop: 10
              }}>
                <span style={{ color: "#d4845a" }}>TOTAL:</span> 2+3+2+3+3+2+3+2+3+3+3 ={" "}
                <span style={{ color: "#d4845a", fontWeight: 600 }}>29 days</span> {"\u2192"}{" "}
                28 (Cosmic Moon) + 1 (DOT) ={" "}
                <span style={{ color: "#c9a84c", fontWeight: 600 }}>365</span>
                <br />
                <span style={{ color: "#d4845a" }}>FEB:</span> Exactly 28 days {"\u2014"} zero epagomenal remainder. In leap years, Feb 29 = Hunab Ku day.
                <br />
                <span style={{ color: "#d4845a" }}>PLACEMENT:</span> Intercalary month sits at the {"\u03c6"}-fracture {"\u2014"} mirroring Egyptian epagomenal days,
                Roman Mercedonius, and Celtic intercalary periods near the winter solstice boundary.
              </div>
            </div>
          </div>
        )}

        {activeView === "structure" && (
          <div style={{ animation: "fadeIn 0.5s ease-out" }} data-testid="view-structure">
            <div style={{ marginBottom: 20, textAlign: "center", fontSize: 14, color: "#8a8070", fontStyle: "italic" }}>
              Day-by-day accumulation {"\u2014"} how the two systems map across 365 days
            </div>

            <div style={{ position: "relative", marginBottom: 60 }}>
              <div style={{ height: 44, display: "flex", border: "1px solid rgba(201,168,76,0.1)" }}>
                <div style={{
                  width: `${(224/365)*100}%`,
                  background: "linear-gradient(135deg, rgba(201,168,76,0.12), rgba(201,168,76,0.05))",
                  display: "flex", alignItems: "center", justifyContent: "center",
                  fontFamily: "'JetBrains Mono', monospace", fontSize: 9, color: "#c9a84c", letterSpacing: 1
                }}>8 MOONS {"\u00b7"} 224d</div>
                <div style={{ width: `${(1/365)*100}%`, minWidth: 3, background: "#c9a84c", position: "relative" }}>
                  <div style={{
                    position: "absolute", top: -22, left: "50%", transform: "translateX(-50%)",
                    fontFamily: "'JetBrains Mono', monospace", fontSize: 8, color: "#c9a84c", whiteSpace: "nowrap"
                  }}>DOT</div>
                </div>
                <div style={{
                  width: `${(28/365)*100}%`,
                  background: "linear-gradient(135deg, rgba(212,132,90,0.15), rgba(212,132,90,0.06))",
                  borderLeft: "1px solid rgba(212,132,90,0.3)", borderRight: "1px solid rgba(212,132,90,0.3)",
                  display: "flex", alignItems: "center", justifyContent: "center",
                  fontFamily: "'JetBrains Mono', monospace", fontSize: 8, color: "#d4845a", letterSpacing: 1
                }}>COSMIC {"\u00b7"} 28d</div>
                <div style={{
                  flex: 1,
                  background: "linear-gradient(135deg, rgba(112,144,184,0.1), rgba(112,144,184,0.03))",
                  display: "flex", alignItems: "center", justifyContent: "center",
                  fontFamily: "'JetBrains Mono', monospace", fontSize: 9, color: "#7090b8", letterSpacing: 1
                }}>4 MOONS {"\u00b7"} 112d (+ Cosmic = 5)</div>
              </div>
              {[
                { label: "Day 1", left: 0 },
                { label: "224", left: (223/365)*100 },
                { label: "225", left: (224.5/365)*100 },
                { label: "253", left: (253/365)*100 },
                { label: "365", left: 97 },
              ].map((m, i) => (
                <div key={i} style={{
                  position: "absolute", left: `${m.left}%`, top: 52,
                  fontFamily: "'JetBrains Mono', monospace", fontSize: 8, color: "#4a4438"
                }}>
                  <div style={{ width: 1, height: 5, background: "rgba(201,168,76,0.15)", marginBottom: 2 }} />
                  {m.label}
                </div>
              ))}
            </div>

            <div style={{ marginTop: 30 }}>
              <div style={{
                display: "grid", gridTemplateColumns: "100px 1fr 1fr",
                fontFamily: "'JetBrains Mono', monospace", fontSize: 9, letterSpacing: 1.5,
                color: "#4a4438", textTransform: "uppercase", padding: "8px 12px",
                borderBottom: "1px solid rgba(201,168,76,0.15)"
              }}>
                <div>Salvi Days</div>
                <div>Original (Continuous)</div>
                <div>Alternate (1st-Aligned)</div>
              </div>
              {[
                { range: `1${"\u2013"}224`, orig: `Moons 1${"\u2013"}8 (Apr 1 ${"\u2192"} Nov 10)`, alt: `Moons 1${"\u2013"}8 (Apr 1${"\u2013"}28 ${"\u2192"} Nov 1${"\u2013"}28)`, hl: "" },
                { range: "225", orig: `DOT ${"\u2014"} Nov 11`, alt: `DOT ${"\u2014"} ${"\u03c6"}-fracture`, hl: "phi" },
                { range: `226${"\u2013"}253`, orig: "Moon 9 partial + Moon 10 start", alt: "Cosmic Moon (Intercalary, 28d)", hl: "intercalary" },
                { range: `254${"\u2013"}365`, orig: `Moons 10${"\u2013"}13 ${"\u2192"} Mar 31`, alt: `Moons 9${"\u2013"}12 (Dec 1${"\u2013"}28 ${"\u2192"} Mar 1${"\u2013"}28)`, hl: "" },
              ].map((row, i) => (
                <div key={i} style={{
                  display: "grid", gridTemplateColumns: "100px 1fr 1fr",
                  padding: "10px 12px", borderBottom: "1px solid rgba(201,168,76,0.04)",
                  background: row.hl === "phi" ? "rgba(201,168,76,0.04)"
                    : row.hl === "intercalary" ? "rgba(212,132,90,0.04)" : "transparent",
                  animation: `fadeIn 0.4s ease-out ${i * 0.08}s both`
                }} data-testid={`structure-row-${i}`}>
                  <div style={{
                    fontFamily: "'JetBrains Mono', monospace", fontSize: 12, fontWeight: 600,
                    color: row.hl === "phi" ? "#c9a84c" : row.hl === "intercalary" ? "#d4845a" : "#6a6050"
                  }}>{row.range}</div>
                  <div style={{ fontFamily: "'JetBrains Mono', monospace", fontSize: 11, color: "#8a8070" }}>
                    {row.orig}
                  </div>
                  <div style={{ fontFamily: "'JetBrains Mono', monospace", fontSize: 11, color: "#8a8070" }}>
                    {row.alt}
                  </div>
                </div>
              ))}
            </div>

            <div style={{
              marginTop: 30, padding: "16px", background: "rgba(201,168,76,0.02)",
              border: "1px solid rgba(201,168,76,0.08)", animation: "fadeIn 0.5s ease-out 0.2s both"
            }}>
              <div style={{
                fontFamily: "'JetBrains Mono', monospace", fontSize: 10, letterSpacing: 1.5,
                color: "#5a5244", textTransform: "uppercase", marginBottom: 10
              }}>
                Accounting Proof
              </div>
              <div style={{ fontFamily: "'JetBrains Mono', monospace", fontSize: 11, color: "#6a6050", lineHeight: 2 }}>
                <div><span style={{ color: "#c9a84c" }}>ORIGINAL:</span> 8{"\u00d7"}28 + 1(DOT) + 5{"\u00d7"}28 = 224 + 1 + 140 = <span style={{ color: "#c9a84c", fontWeight: 600 }}>365</span></div>
                <div><span style={{ color: "#c9a84c" }}>ALTERNATE:</span> 8{"\u00d7"}28 + 1(DOT) + 28(Cosmic) + 4{"\u00d7"}28 = 224 + 1 + 28 + 112 = <span style={{ color: "#c9a84c", fontWeight: 600 }}>365</span></div>
                <div><span style={{ color: "#c9a84c" }}>{"\u03c6"} CHECK:</span> 364/{"\u03c6"} = 364/1.6180339... = <span style={{ color: "#c9a84c", fontWeight: 600 }}>224.96</span> {"\u2248"} Day 225 {"\u2714"}</div>
                <div><span style={{ color: "#c9a84c" }}>FIBONACCI:</span> Pre:Post = 8:5 = 1.600 {"\u2248"} {"\u03c6"} = 1.618 (error: 1.1%) {"\u2714"}</div>
              </div>
            </div>
          </div>
        )}

        {activeView === "info" && (
          <div style={{ animation: "fadeIn 0.5s ease-out" }} data-testid="view-architecture">
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 20, marginBottom: 20 }}>
              <div style={{
                padding: "20px", background: "rgba(201,168,76,0.02)",
                border: "1px solid rgba(201,168,76,0.08)"
              }}>
                <div style={{
                  fontFamily: "'JetBrains Mono', monospace", fontSize: 10, letterSpacing: 2,
                  color: "#c9a84c", textTransform: "uppercase", marginBottom: 14
                }}>
                  Original Model
                </div>
                <div style={{ fontSize: 14, color: "#a09888", lineHeight: 1.8 }}>
                  13 continuous 28-day moons running from April 1 to March 31.
                  The Day Out of Time falls naturally on November 11 (Day 225 = {"\u230a"}364/{"\u03c6"}{"\u230b"}).
                  Every moon starts the day after the previous one ends {"\u2014"} no gaps, no overlaps.
                </div>
                <div style={{
                  fontFamily: "'JetBrains Mono', monospace", fontSize: 10, color: "#5a5244",
                  marginTop: 14, padding: "8px", background: "rgba(201,168,76,0.03)",
                  border: "1px solid rgba(201,168,76,0.06)"
                }}>
                  Strengths: Simplicity, continuous counting, clean day-number addressing.
                  <br />Every day has exactly one address: Moon.Day (e.g. 7.14).
                </div>
              </div>

              <div style={{
                padding: "20px", background: "rgba(212,132,90,0.02)",
                border: "1px solid rgba(212,132,90,0.08)"
              }}>
                <div style={{
                  fontFamily: "'JetBrains Mono', monospace", fontSize: 10, letterSpacing: 2,
                  color: "#d4845a", textTransform: "uppercase", marginBottom: 14
                }}>
                  Alternate Model
                </div>
                <div style={{ fontSize: 14, color: "#a09888", lineHeight: 1.8 }}>
                  12 moons anchored to the 1st{"\u2013"}28th of each Gregorian month, plus
                  a 13th intercalary Cosmic Moon (28 days) inserted at the {"\u03c6"}-fracture point.
                  The remaining 29th{"\u2013"}31st days are collected as epagomenal days forming the Cosmic Moon.
                </div>
                <div style={{
                  fontFamily: "'JetBrains Mono', monospace", fontSize: 10, color: "#5a5244",
                  marginTop: 14, padding: "8px", background: "rgba(212,132,90,0.03)",
                  border: "1px solid rgba(212,132,90,0.06)"
                }}>
                  Strengths: Gregorian alignment (moons start on 1st), familiar structure.
                  <br />Mirrors Egyptian/Roman intercalary traditions at the {"\u03c6"}-fracture.
                </div>
              </div>
            </div>

            <div style={{
              padding: "20px", background: "rgba(201,168,76,0.02)",
              border: "1px solid rgba(201,168,76,0.08)", marginBottom: 20
            }}>
              <div style={{
                fontFamily: "'JetBrains Mono', monospace", fontSize: 10, letterSpacing: 2,
                color: "#c9a84c", textTransform: "uppercase", marginBottom: 14
              }}>
                Mathematical Foundation
              </div>
              <div style={{
                display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(250px, 1fr))", gap: 16
              }}>
                {[
                  {
                    title: "Golden Ratio Split",
                    formula: `364/${"\u03c6"} = 224.96 ${"\u2248"} Day 225`,
                    desc: "The Day Out of Time falls at the precise golden ratio division of the 364-day cycle."
                  },
                  {
                    title: "Fibonacci Moon Ratio",
                    formula: "8:5 = 1.600",
                    desc: `Pre-${"\u03c6"} and post-${"\u03c6"} arcs use consecutive Fibonacci numbers (8, 5) whose ratio converges to ${"\u03c6"}.`
                  },
                  {
                    title: "Perfect Divisibility",
                    formula: `13 ${"\u00d7"} 28 = 364 = 365 ${"\u2212"} 1`,
                    desc: "364 is uniquely divisible by 7, 4, 13, and 28. The single remainder day becomes the DOT."
                  },
                  {
                    title: `Harmonic Convergence (${"\u03c6"})`,
                    formula: `${"\u03c6"} = (1 + ${"\u221a"}5) / 2`,
                    desc: "The irrational constant governing phyllotaxis, Fibonacci spirals, and optimal sphere packing."
                  },
                ].map((item, i) => (
                  <div key={i} style={{
                    padding: "12px", background: "rgba(201,168,76,0.03)",
                    border: "1px solid rgba(201,168,76,0.06)"
                  }} data-testid={`math-card-${i}`}>
                    <div style={{ fontFamily: "'JetBrains Mono', monospace", fontSize: 10, color: "#c9a84c", letterSpacing: 1, marginBottom: 6 }}>
                      {item.title}
                    </div>
                    <div style={{ fontFamily: "'JetBrains Mono', monospace", fontSize: 16, color: "#e8e4dc", fontWeight: 500, marginBottom: 6 }}>
                      {item.formula}
                    </div>
                    <div style={{ fontSize: 12, color: "#6a6050", lineHeight: 1.6 }}>
                      {item.desc}
                    </div>
                  </div>
                ))}
              </div>
            </div>

            <div style={{
              padding: "20px", background: "rgba(201,168,76,0.02)",
              border: "1px solid rgba(201,168,76,0.08)"
            }}>
              <div style={{
                fontFamily: "'JetBrains Mono', monospace", fontSize: 10, letterSpacing: 2,
                color: "#c9a84c", textTransform: "uppercase", marginBottom: 14
              }}>
                Historical Precedent
              </div>
              <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(200px, 1fr))", gap: 12 }}>
                {[
                  { culture: "Egyptian", detail: "12 months of 30 days + 5 epagomenal days dedicated to Osiris, Horus, Set, Isis, Nephthys." },
                  { culture: "Roman", detail: "Mercedonius: 27-day intercalary month inserted after February 23 to realign the calendar." },
                  { culture: "Celtic", detail: "Intercalary periods near solstices. The Coligny calendar bronze tablet shows a 5-year lunisolar cycle." },
                  { culture: "Mayan", detail: "Wayeb': 5 'nameless days' at the end of the Haab' 365-day cycle. Considered dangerous/liminal." },
                  { culture: "Abri Blanchard", detail: "~28,000 BCE bone artifact with 69 sequential marks tracking 2.4 lunar months. Earliest known calendar." },
                  { culture: "Salvi Epoch", detail: "April 1, 2025: Day Zero. The Cosmic Moon intercalary mirrors all these traditions at the golden ratio point." },
                ].map((item, i) => (
                  <div key={i} style={{
                    padding: "10px", background: "rgba(201,168,76,0.02)",
                    border: "1px solid rgba(201,168,76,0.05)"
                  }}>
                    <div style={{ fontFamily: "'JetBrains Mono', monospace", fontSize: 10, color: "#c9a84c", letterSpacing: 1, marginBottom: 4 }}>
                      {item.culture}
                    </div>
                    <div style={{ fontSize: 12, color: "#6a6050", lineHeight: 1.6 }}>
                      {item.detail}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}

        <footer style={{
          textAlign: "center", marginTop: 50, paddingTop: 20,
          borderTop: "1px solid rgba(201,168,76,0.06)",
          animation: "fadeIn 0.8s ease-out 0.4s both"
        }}>
          <div style={{ fontFamily: "'JetBrains Mono', monospace", fontSize: 9, letterSpacing: 3, color: "#3a3428", textTransform: "uppercase" }}>
            Capomastro Holdings Ltd. {"\u00b7"} Temporal Unification Architecture
          </div>
          <div style={{ fontFamily: "'JetBrains Mono', monospace", fontSize: 8, color: "#2a2420", marginTop: 6 }}>
            {"\u03c6"} = 1.6180339887... {"\u00b7"} 364/{"\u03c6"} = 224.96 {"\u00b7"} 8/5 = 1.600
          </div>
        </footer>
      </div>
    </div>
  );
}
