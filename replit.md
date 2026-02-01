# Ternary-Torsion Internet Landing Page

## Overview
A stunning dark-themed landing page for Ternary, a post-quantum internet solutions company. The site showcases deployable components for building quantum-resistant infrastructure.

## Tech Stack
- **Frontend**: React, TypeScript, Tailwind CSS, Framer Motion
- **Backend**: Express.js, Node.js, PostgreSQL
- **Database**: Drizzle ORM with PostgreSQL
- **UI Components**: shadcn/ui (Button, Badge, Card)
- **Routing**: Wouter
- **Styling**: Custom dark tech theme with baby blue accents

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

server/
├── db.ts                  # PostgreSQL database connection
├── storage.ts             # Database storage interface
├── ternary.ts             # Ternary compression utilities
├── routes.ts              # API routes for demo functionality
├── index.ts               # Express server entry point

shared/
├── schema.ts              # Drizzle database schema definitions
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
1. **Header** - Navigation with back-to-home link and Live Demo nav link
2. **Hero** - Product headline, description, CTAs
3. **Features** - Grid of key capabilities (compression, efficiency, compatibility)
4. **Architecture** - Layered diagram showing PostgreSQL integration
5. **Installation** - Quick start code blocks with copy functionality
6. **Live Demo** - Interactive compression demo with real PostgreSQL backend
7. **Performance** - Benchmark charts and metrics
8. **Pricing** - Tier cards (Community, Enterprise, Cloud)
9. **Use Cases** - Target applications and industries
10. **CTA Section** - Get Started call-to-action
11. **Footer** - Links and copyright

## Database Schema
- **demo_sessions** - Tracks demo session metadata
- **binary_storage** - Stores original binary data with size metrics
- **ternary_storage** - Stores compressed ternary data with compression ratios
- **compression_benchmarks** - Records compression performance metrics

## API Endpoints
- `POST /api/demo/run` - Run compression demo with dataset (sensor/events/logs)
- `GET /api/demo/stats` - Get aggregated compression statistics
- `GET /api/demo/session/:sessionId` - Get details for a specific demo session

## Live Demo Features
- Three sample datasets: Sensor Readings, User Events, Log Entries
- Data preview table showing first 5 rows
- Run Compression Demo button with processing animation
- Results stored in PostgreSQL database
- Results showing binary vs ternary size comparison (56-62% savings)
- Animated progress bars and "At Scale" projections

## Navigation
- Landing page components section has clickable TernaryDB card linking to /ternarydb
- Footer Product section links to /ternarydb using wouter Link for client-side navigation

## Running the App
```bash
npm run dev
```
The app runs on port 5000.
