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
│   │   ├── landing.tsx    # Main landing page with all sections
│   │   └── ternarydb.tsx  # TernaryDB product page
│   ├── components/ui/     # shadcn/ui components
│   ├── index.css          # Theme CSS variables
│   └── App.tsx            # App router with / and /ternarydb routes
├── index.html             # Entry HTML with SEO meta tags
```

## Theme Colors
- Background: Deep navy blue (#0a192f equivalent)
- Accent/Primary: Baby blue (#7cc5e6 equivalent)
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

## TernaryDB Product Page (/ternarydb)
1. **Header** - Navigation with back-to-home link
2. **Hero** - Product headline, description, CTAs
3. **Features** - Grid of key capabilities (compression, efficiency, compatibility)
4. **Architecture** - Layered diagram showing PostgreSQL integration
5. **Performance** - Benchmark charts and metrics
6. **Pricing** - Tier cards (Starter, Pro, Enterprise)
7. **Use Cases** - Target applications and industries
8. **CTA Section** - Get Started call-to-action
9. **Footer** - Links and copyright

## Navigation
- Landing page components section has clickable TernaryDB card linking to /ternarydb
- Footer Product section links to /ternarydb using wouter Link for client-side navigation

## Running the App
```bash
npm run dev
```
The app runs on port 5000.
