# PlenumNET Brand Specification

## Fonts

### Header / Logo Font
- **Font**: Orbitron
- **Weight**: 800 (ExtraBold)
- **Letter Spacing**: 0.12em
- **Text Transform**: UPPERCASE
- **Color**: #E4DFD5 (warm off-white)
- **Text Shadow**: `0 1px 0 rgba(0,0,0,0.1), 0 2px 0 rgba(0,0,0,0.07), 0 3px 8px rgba(0,0,0,0.08), 0 0 20px rgba(56,189,248,0.15)`
- **Source**: `https://fonts.googleapis.com/css2?family=Orbitron:wght@800&display=swap`

### Body / Section Font
- **Font**: Inter
- **Fallback**: system-ui, -apple-system, sans-serif
- **Weights Used**: 400 (regular body), 600 (labels/subheads), 700 (section headings)

### Monospace / Data Font
- **Font**: JetBrains Mono
- **Fallback**: Fira Code, SF Mono, monospace
- **Weights Used**: 400, 600, 700
- **Usage**: Stat counters, addresses, code blocks, terminal, section labels

---

## Colors (Dark Mode — Primary Theme)

### Backgrounds
| Token         | Value                          | Hex Approx   | Usage                        |
|---------------|--------------------------------|--------------|------------------------------|
| `bg`          | `hsl(20, 14%, 4%)`            | `#0F0C0A`    | Page background              |
| `card`        | `hsl(20, 14%, 8%)`            | `#181411`    | Card/panel backgrounds       |
| `muted`       | `hsl(20, 12%, 10%)`           | `#1D1915`    | Muted surfaces               |
| `secondary`   | `hsl(210, 15%, 25%)`          | `#363D47`    | Secondary elements           |
| `accent`      | `hsl(210, 20%, 18%)`          | `#273041`    | Accent backgrounds           |

### Text
| Token         | Value                          | Hex Approx   | Usage                        |
|---------------|--------------------------------|--------------|------------------------------|
| `fg`          | `hsl(45, 25%, 91%)`           | `#ECE8DF`    | Primary text (headings)      |
| `fgSoft`      | `hsl(40, 15%, 70%)`           | `#BCB3A3`    | Body text                    |
| `fgMuted`     | `hsl(35, 10%, 50%)`           | `#8A8173`    | Labels, captions             |
| `fgFaint`     | `hsl(30, 8%, 35%)`            | `#5E5851`    | Faint/decorative text        |

### Primary (Blue)
| Token           | Value                            | Hex Approx   | Usage                      |
|-----------------|----------------------------------|--------------|----------------------------|
| `primary`       | `hsl(210, 80%, 55%)`            | `#4A9EF5`    | Accent text, stat values   |
| `primarySoft`   | `hsl(210, 70%, 65%)`            | `#6DB3F7`    | Soft accent                |
| `primaryDim`    | `hsla(210, 80%, 55%, 0.1)`      | —            | Tinted backgrounds         |
| `primaryBorder` | `hsla(210, 80%, 55%, 0.18)`     | —            | Accent borders             |

### Borders
| Token         | Value                          | Hex Approx   | Usage                        |
|---------------|--------------------------------|--------------|------------------------------|
| `cardBorder`  | `hsl(20, 10%, 14%)`           | `#272220`    | Card borders                 |

### Semantic
| Token         | Value                          | Hex Approx   | Usage                        |
|---------------|--------------------------------|--------------|------------------------------|
| `balance`     | `hsl(210, 80%, 55%)`          | `#4A9EF5`    | Balance/math highlight       |
| `esoteric`    | `hsl(270, 50%, 65%)`          | `#A87FD4`    | Esoteric/special highlight   |
| `cosmic`      | `hsl(340, 55%, 60%)`          | `#D46B8A`    | Cosmic/warning highlight     |
| `green`       | `hsl(145, 50%, 50%)`          | `#40BF6E`    | Success/positive             |

### Navigation-Specific
| Element              | Value      | Usage                           |
|----------------------|------------|----------------------------------|
| Nav text             | `#E4DFD5`  | Header menu items                |
| Nav hover/active     | `#38BDF8`  | Sky blue accent on hover/open    |

---

## Colors (Light Mode)

### Backgrounds
| Token         | Value                          | Usage                        |
|---------------|--------------------------------|------------------------------|
| `bg`          | `hsl(0, 0%, 100%)`            | Page background              |
| `card`        | `hsl(0, 0%, 100%)`            | Cards                        |
| `muted`       | `hsl(220, 10%, 96%)`          | Muted surfaces               |

### Text
| Token         | Value                          | Usage                        |
|---------------|--------------------------------|------------------------------|
| `fg`          | `hsl(220, 20%, 15%)`          | Primary text                 |
| `fgSoft`      | `hsl(220, 15%, 35%)`          | Body text                    |
| `fgMuted`     | `hsl(220, 10%, 55%)`          | Labels                       |

### Primary (Blue)
| Token         | Value                          | Usage                        |
|---------------|--------------------------------|------------------------------|
| `primary`     | `hsl(210, 100%, 45%)`         | Accent                       |

---

## Inter-Cube Infrastructure Section (Screenshot)

### Section Label ("INTER-CUBE INFRASTRUCTURE")
- Font: JetBrains Mono
- Size: ~9px
- Letter Spacing: 2px
- Color: `primary` — `hsl(210, 80%, 55%)` / ~#4A9EF5
- Text Transform: UPPERCASE

### Section Heading ("Four Services. Pure Geometry.")
- Font: Inter
- Size: 28px
- Weight: 700
- Color: `fg` — `hsl(45, 25%, 91%)` / ~#ECE8DF
- "Zero Routing Tables." in `primary` — `hsl(210, 80%, 55%)` / ~#4A9EF5

### Body Text
- Font: Inter
- Size: 15px
- Line Height: 1.75
- Color: `fgSoft` — `hsl(40, 15%, 70%)` / ~#BCB3A3

### Stat Counters (2,541,865,828,329 etc.)
- Font: JetBrains Mono
- Size: 32px
- Weight: 700
- Color: `primary` — `hsl(210, 80%, 55%)` / ~#4A9EF5

### Stat Labels ("Address Space", "Neighbors" etc.)
- Font: Inter
- Size: 13px
- Weight: 600
- Color: `fg` — `hsl(45, 25%, 91%)` / ~#ECE8DF

### Stat Sub-labels ("3²⁶ Rep C vertices" etc.)
- Font: JetBrains Mono
- Size: 10px
- Color: `fgMuted` — `hsl(35, 10%, 50%)` / ~#8A8173

### Stat Card Background
- Background: `card` — `hsl(20, 14%, 8%)` / ~#181411
- Border: `cardBorder` — `hsl(20, 10%, 14%)` / ~#272220
- Hover Border: `primaryBorder` — `hsla(210, 80%, 55%, 0.18)`
- Border Radius: 9px

---

## Logo

- **Wordmark Font**: Orbitron, 800 weight, UPPERCASE, 0.12em letter spacing
- **Icon**: Geometric monogram — a "P" formed from overlapping angular strokes with a vertical axis and horizontal crossbar, creating a distinctive mark that references ternary branching
- **Logo File**: See `plenumnet-logo.svg`
