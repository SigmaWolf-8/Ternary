# Installer Assets

Build-time generated assets for PlenumNET MSI installers.

## Required Files

- `banner.bmp` — 493×58 pixels, WiX installer banner
- `dialog.bmp` — 493×312 pixels, WiX dialog background
- `license.rtf` — License text in RTF format

## Banner Specification

- Left-aligned Capomastro "P" mark at 48×48 (padded)
- Right-aligned product name in Century Gothic Bold 18pt
- Text color: #F0EDE8 (text-heading)
- Background: #1D1915 (bg-muted) with "P" lattice watermark at 5% opacity

## Dialog Background Specification

- Base color: #0F0C0A (bg-page)
- "P" lattice watermark at 3% opacity, centered
- Footer bar: full width, 24px height, #1D1915 (bg-muted) background
- Footer text: "Capomastro Holdings Ltd. — Applied Physics Division"
- Footer font: Century Gothic Regular 9pt, right-aligned
- Footer text color: #998F82 (text-label)

These BMP files are generated at build time by `plenum-pack` from the SVG sources
in `assets/icons/svg/` using the brand palette defined in the specification.
