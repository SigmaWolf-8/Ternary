# Ternary-Torsion Internet Landing Page

## Overview
A stunning dark-themed landing page for Ternary, a post-quantum internet solutions company. The site showcases deployable components for building quantum-resistant infrastructure.

## Tech Stack
- **Frontend**: React, TypeScript, Tailwind CSS, Framer Motion
- **UI Components**: shadcn/ui (Button, Badge, Card)
- **Routing**: Wouter
- **Styling**: Custom dark tech theme with teal/cyan accents

## Project Structure
```
client/
├── src/
│   ├── pages/
│   │   └── landing.tsx    # Main landing page with all sections
│   ├── components/ui/     # shadcn/ui components
│   ├── index.css          # Theme CSS variables
│   └── App.tsx            # App router
├── index.html             # Entry HTML with SEO meta tags
```

## Theme Colors
- Background: Deep navy blue (#0a192f equivalent)
- Accent/Primary: Teal (#00d4aa equivalent)
- Text: Light blue-gray for primary, muted for secondary

## Landing Page Sections
1. **Header** - Fixed nav with logo, links, GitHub & Contact buttons, mobile menu
2. **Hero** - Badge, headline, description, CTAs, animated stats
3. **Approach** - Timeline showing phased implementation (Phase 1-4)
4. **Components** - Grid of market-ready components (libternary, Timing API, FPGA, etc.)
5. **Comparison** - Current Internet vs Ternary Architecture table
6. **Target Markets** - Financial, Research, Industrial IoT
7. **CTA Section** - Get Started call-to-action
8. **Footer** - Links, social icons, copyright

## Running the App
```bash
npm run dev
```
The app runs on port 5000.
