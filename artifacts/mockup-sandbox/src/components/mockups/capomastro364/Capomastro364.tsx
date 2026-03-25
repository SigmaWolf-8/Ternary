import { useState } from "react";

const layers = [
  { id: "circle", label: "Circle (364°)", default: true },
  { id: "square", label: "Square (√182)", default: true },
  { id: "triangle", label: "Triangle + 144,000", default: true },
  { id: "pentagon", label: "Pentagon", default: true },
  { id: "hexagon", label: "Hexagon", default: true },
  { id: "heptagon", label: "Heptagon", default: true },
  { id: "octagon", label: "Octagon", default: true },
  { id: "nonagon", label: "Nonagon", default: true },
  { id: "decagon", label: "Decagon", default: true },
  { id: "arc182", label: "182° Red Arc", default: true },
  { id: "arc650", label: "650° Green Arc", default: true },
  { id: "labels", label: "Degree Labels", default: true },
];

export default function Capomastro364() {
  const [visible, setVisible] = useState<Record<string, boolean>>(
    Object.fromEntries(layers.map((l) => [l.id, l.default]))
  );

  const toggle = (id: string) =>
    setVisible((v) => ({ ...v, [id]: !v[id] }));

  const resetAll = () =>
    setVisible(Object.fromEntries(layers.map((l) => [l.id, true])));

  const show = (id: string) => visible[id] !== false;

  return (
    <div style={{ display: "flex", width: "100%", height: "100vh", background: "#0a0a0a", fontFamily: "system-ui", color: "#fff" }}>
      <div style={{ width: 240, padding: 16, background: "rgba(0,0,0,0.95)", borderRight: "1px solid #333", overflowY: "auto", flexShrink: 0 }}>
        <h3 style={{ margin: "0 0 12px", fontSize: 15, color: "#00ffff" }}>Capomastro 364° Layers</h3>
        {layers.map((l) => (
          <label key={l.id} style={{ display: "flex", alignItems: "center", gap: 8, margin: "6px 0", cursor: "pointer", fontSize: 13 }}>
            <input type="checkbox" checked={show(l.id)} onChange={() => toggle(l.id)} />
            {l.label}
          </label>
        ))}
        <hr style={{ borderColor: "#444", margin: "12px 0" }} />
        <button onClick={resetAll} style={{ padding: "6px 14px", background: "#222", color: "#fff", border: "1px solid #555", borderRadius: 6, cursor: "pointer", fontSize: 13 }}>
          Reset All
        </button>
      </div>

      <div style={{ flex: 1, display: "flex", alignItems: "center", justifyContent: "center" }}>
        <svg viewBox="0 0 1200 1200" xmlns="http://www.w3.org/2000/svg" style={{ width: "90%", height: "90%", maxWidth: 800, maxHeight: 800 }}>
          {show("circle") && (
            <g>
              <circle cx="600" cy="600" r="400" fill="none" stroke="#00ffff" strokeWidth="14" />
              <text x="600" y="70" textAnchor="middle" fontSize="32" fill="#00ffff" fontFamily="monospace">364°</text>
            </g>
          )}

          {show("square") && (
            <g>
              <rect x="200" y="200" width="800" height="800" fill="none" stroke="#ffff00" strokeWidth="14" />
              <text x="600" y="1080" textAnchor="middle" fontSize="22" fill="#ffff00">√182 = √(14×13)</text>
            </g>
          )}

          {show("triangle") && (
            <g>
              <polygon points="600,220 280,780 920,780" fill="none" stroke="#ff00ff" strokeWidth="14" />
              <text x="600" y="580" textAnchor="middle" fontSize="52" fill="#ff00ff" fontWeight="bold">144,000</text>
            </g>
          )}

          {show("pentagon") && (
            <g>
              <polygon points="600,240 820,360 820,640 600,760 380,640" fill="none" stroke="#00ff00" strokeWidth="12" />
            </g>
          )}

          {show("hexagon") && (
            <g>
              <polygon points="600,250 780,340 820,520 780,700 600,790 420,700" fill="none" stroke="#00aaff" strokeWidth="12" />
            </g>
          )}

          {show("heptagon") && (
            <g>
              <polygon points="600,235 760,310 820,460 780,620 680,760 520,760 420,620" fill="none" stroke="#ffaa00" strokeWidth="11" />
            </g>
          )}

          {show("octagon") && (
            <g>
              <polygon points="600,230 730,300 800,430 800,570 730,700 600,770 470,700 400,570" fill="none" stroke="#aa00ff" strokeWidth="11" />
            </g>
          )}

          {show("nonagon") && (
            <g>
              <polygon points="600,225 720,280 800,380 820,500 780,620 700,730 500,730 420,620 380,500" fill="none" stroke="#00ffaa" strokeWidth="10" />
            </g>
          )}

          {show("decagon") && (
            <g>
              <polygon points="600,220 710,270 790,360 820,480 800,600 730,700 600,770 470,700 400,600 380,480" fill="none" stroke="#ff8800" strokeWidth="10" />
            </g>
          )}

          {show("arc182") && (
            <g>
              <path d="M600,200 A400,400 0 0,0 200,600" fill="none" stroke="#ff0000" strokeWidth="18" strokeLinecap="round" />
              <text x="320" y="300" fontSize="24" fill="#ff0000">182°</text>
            </g>
          )}

          {show("arc650") && (
            <g>
              <path d="M600,200 A400,400 0 0,1 1000,600" fill="none" stroke="#00ff00" strokeWidth="18" strokeLinecap="round" />
              <text x="860" y="300" fontSize="24" fill="#00ff00">650° (286°)</text>
            </g>
          )}

          {show("labels") && (
            <g>
              <text x="600" y="140" textAnchor="middle" fontSize="18" fill="#ffffff">0°</text>
              <text x="1040" y="610" fontSize="18" fill="#ffffff">91°</text>
              <text x="600" y="1050" textAnchor="middle" fontSize="18" fill="#ffffff">182°</text>
              <text x="160" y="610" fontSize="18" fill="#ffffff">273°</text>

              <text x="600" y="160" textAnchor="middle" fontSize="13" fill="#666">T₇ = 13 = 1 radian</text>
              <text x="1040" y="630" fontSize="13" fill="#666">π = 14</text>
              <text x="600" y="1070" textAnchor="middle" fontSize="13" fill="#666">half circle = 13 × 14</text>

              <line x1="600" y1="195" x2="600" y2="205" stroke="#666" strokeWidth="2" />
              <line x1="995" y1="600" x2="1005" y2="600" stroke="#666" strokeWidth="2" />
              <line x1="600" y1="995" x2="600" y2="1005" stroke="#666" strokeWidth="2" />
              <line x1="195" y1="600" x2="205" y2="600" stroke="#666" strokeWidth="2" />
            </g>
          )}
        </svg>
      </div>
    </div>
  );
}
