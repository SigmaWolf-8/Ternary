import { useState } from "react";

const STEP = 13;
const SLOTS = 27;

function slotFromOffset(off: number): [number, number, number] {
  const p = Math.floor(off / 9) + 1;
  const r = (Math.floor(off / 3) % 3) + 1;
  const i = (off % 3) + 1;
  return [p, r, i];
}

const planeNames = ["Data", "Control", "Mgmt"];
const planeColors = ["#3b82f6", "#f59e0b", "#ef4444"];

function thresholdTrit(k: number, n: number): number {
  const v = Math.floor((3 * k) / n);
  return v >= 2 ? 2 : v;
}

export default function CoprimeProjection() {
  const [mode, setMode] = useState<"threshold" | "coprime" | "hybrid">("threshold");
  const [highlightScore, setHighlightScore] = useState<number | null>(null);

  const bijection: { score: number; offset: number; slot: [number, number, number]; plane: string }[] = [];
  for (let s = 0; s < SLOTS; s++) {
    const off = (s * STEP) % SLOTS;
    const slot = slotFromOffset(off);
    bijection.push({ score: s, offset: off, slot, plane: planeNames[slot[0] - 1] });
  }

  const thresholdMapping: { k: number; trit: number; label: string }[] = [];
  for (let k = 0; k <= 9; k++) {
    const t = thresholdTrit(k, 9);
    thresholdMapping.push({ k, trit: t, label: planeNames[t] });
  }

  const hybridMapping: { k: number; plane: number; role: number; inst: number }[] = [];
  for (let k = 0; k <= 9; k++) {
    const plane = thresholdTrit(k, 9);
    const sub = (k * 4) % 9;
    const role = Math.floor(sub / 3);
    const inst = sub % 3;
    hybridMapping.push({ k, plane, role, inst });
  }

  const cubeSlots: { offset: number; slot: [number, number, number]; highlighted: boolean }[] = [];
  for (let off = 0; off < 27; off++) {
    const slot = slotFromOffset(off);
    let highlighted = false;
    if (highlightScore !== null) {
      if (mode === "coprime") {
        highlighted = off === (highlightScore * STEP) % SLOTS;
      } else {
        highlighted = off === highlightScore;
      }
    }
    cubeSlots.push({ offset: off, slot, highlighted });
  }

  return (
    <div style={{ width: "100%", minHeight: "100vh", background: "#0a0a0a", color: "#fff", fontFamily: "monospace", padding: 20, boxSizing: "border-box" }}>
      <h2 style={{ color: "#00ffff", margin: "0 0 4px", fontSize: 18 }}>Coprime Projection Analysis</h2>
      <p style={{ color: "#888", margin: "0 0 16px", fontSize: 12 }}>27 slots | Step 13 | gcd(13,27)=1 | 13+14=27 | 13+14+1=28</p>

      <div style={{ display: "flex", gap: 8, marginBottom: 16 }}>
        {(["threshold", "coprime", "hybrid"] as const).map((m) => (
          <button key={m} onClick={() => setMode(m)} style={{
            padding: "6px 14px", fontSize: 12, borderRadius: 6, cursor: "pointer",
            background: mode === m ? "#00ffff" : "#222", color: mode === m ? "#000" : "#fff",
            border: mode === m ? "1px solid #00ffff" : "1px solid #444", fontFamily: "monospace",
          }}>
            {m === "threshold" ? "Current (Threshold)" : m === "coprime" ? "Full Coprime (×13 mod 27)" : "Hybrid (Threshold + ×4 mod 9)"}
          </button>
        ))}
      </div>

      <div style={{ display: "flex", gap: 24, flexWrap: "wrap" }}>
        <div style={{ flex: "1 1 320px" }}>
          <h3 style={{ color: "#ffff00", fontSize: 14, margin: "0 0 8px" }}>
            {mode === "threshold" ? "Threshold: floor(3k/9)" : mode === "coprime" ? "Bijection: (score × 13) mod 27" : "Hybrid: Plane=threshold, Role×Inst=×4 mod 9"}
          </h3>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(220px, 1fr))", gap: 4 }}>
            {mode === "threshold" && thresholdMapping.map((m) => (
              <div key={m.k} onMouseEnter={() => setHighlightScore(m.k)} onMouseLeave={() => setHighlightScore(null)} style={{
                padding: "4px 8px", background: "#111", borderRadius: 4, fontSize: 12,
                borderLeft: `3px solid ${planeColors[m.trit]}`, cursor: "pointer",
              }}>
                k={m.k} → GF3={m.trit} → RepC={m.trit + 1} <span style={{ color: planeColors[m.trit] }}>({m.label})</span>
              </div>
            ))}
            {mode === "coprime" && bijection.map((b) => (
              <div key={b.score} onMouseEnter={() => setHighlightScore(b.score)} onMouseLeave={() => setHighlightScore(null)} style={{
                padding: "4px 8px", background: "#111", borderRadius: 4, fontSize: 11,
                borderLeft: `3px solid ${planeColors[b.slot[0] - 1]}`, cursor: "pointer",
              }}>
                s={b.score.toString().padStart(2, " ")} → off={b.offset.toString().padStart(2, " ")} → [{b.slot.join(",")}] <span style={{ color: planeColors[b.slot[0] - 1] }}>({b.plane})</span>
              </div>
            ))}
            {mode === "hybrid" && hybridMapping.map((m) => (
              <div key={m.k} onMouseEnter={() => setHighlightScore(m.k)} onMouseLeave={() => setHighlightScore(null)} style={{
                padding: "4px 8px", background: "#111", borderRadius: 4, fontSize: 12,
                borderLeft: `3px solid ${planeColors[m.plane]}`, cursor: "pointer",
              }}>
                k={m.k} → [{m.plane + 1},{m.role + 1},{m.inst + 1}] <span style={{ color: planeColors[m.plane] }}>({planeNames[m.plane]})</span>
                <span style={{ color: "#666" }}> sub={(m.k * 4) % 9}</span>
              </div>
            ))}
          </div>
        </div>

        <div style={{ flex: "0 0 280px" }}>
          <h3 style={{ color: "#ffff00", fontSize: 14, margin: "0 0 8px" }}>3³ Cube (27 slots)</h3>
          <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
            {[1, 2, 3].map((plane) => (
              <div key={plane}>
                <div style={{ fontSize: 11, color: planeColors[plane - 1], marginBottom: 4 }}>
                  Plane {plane} ({planeNames[plane - 1]})
                </div>
                <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: 3 }}>
                  {[1, 2, 3].map((role) =>
                    [1, 2, 3].map((inst) => {
                      const off = (plane - 1) * 9 + (role - 1) * 3 + (inst - 1);
                      const isGateway = plane === 2 && role === 2 && inst === 2;
                      let isHighlighted = false;
                      if (highlightScore !== null && mode === "coprime") {
                        isHighlighted = off === (highlightScore * STEP) % SLOTS;
                      }
                      return (
                        <div key={`${plane}${role}${inst}`} style={{
                          width: 28, height: 28, borderRadius: 4, display: "flex", alignItems: "center", justifyContent: "center",
                          fontSize: 9, fontWeight: isGateway ? "bold" : "normal",
                          background: isHighlighted ? planeColors[plane - 1] : isGateway ? "#333" : "#1a1a1a",
                          color: isHighlighted ? "#000" : isGateway ? "#00ffff" : "#666",
                          border: isGateway ? "1px solid #00ffff" : "1px solid #333",
                        }}>
                          {off}
                        </div>
                      );
                    })
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      <div style={{ marginTop: 20, padding: 12, background: "#111", borderRadius: 8, fontSize: 12, lineHeight: 1.6 }}>
        <div style={{ color: "#00ffff", fontWeight: "bold", marginBottom: 6 }}>Constants (all derived)</div>
        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(200px, 1fr))", gap: 4 }}>
          <span><span style={{ color: "#ffff00" }}>27</span> = 3³ = SLOTS_PER_NODE</span>
          <span><span style={{ color: "#ffff00" }}>28</span> = Z₂₈ = CYCLIC_ORDER</span>
          <span><span style={{ color: "#ffff00" }}>13</span> = T₇ = 111₃ = 1 radian</span>
          <span><span style={{ color: "#ffff00" }}>14</span> = π_ternary</span>
          <span><span style={{ color: "#ff00ff" }}>13 + 14 = 27</span> (slots)</span>
          <span><span style={{ color: "#ff00ff" }}>13 + 14 + 1 = 28</span> (cyclic)</span>
          <span><span style={{ color: "#00ff00" }}>gcd(13, 27) = 1</span> → bijection</span>
          <span><span style={{ color: "#00ff00" }}>13 mod 9 = 4</span> → sub-step</span>
        </div>
      </div>
    </div>
  );
}
