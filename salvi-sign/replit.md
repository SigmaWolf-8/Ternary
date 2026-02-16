# SalviSign - E-Signature Platform

## Overview
SalviSign is a clean, sleek e-signature platform inspired by DocuSign, built as a module within the Ternary monorepo. It provides document envelope management, field placement, signature collection, audit trail tracking, and integrates with PlenumNET for secure document storage and witnessing.

## Tech Stack
- **Frontend**: React + TypeScript + Vite + Tailwind CSS + shadcn/ui + wouter routing
- **Backend**: Express.js + TypeScript (Fastify migration planned for standalone deployment)
- **Database**: PostgreSQL with Drizzle ORM + Row-Level Security (RLS)
- **State Management**: TanStack React Query
- **External Services**: PlenumNET API (document storage, witnessing, HPTP timing, ML-DSA crypto)
- **Design**: Swiss Banker black-and-gold aesthetic, Inter font, site-wide zoom control

## Project Structure
```
client/src/
  pages/
    dashboard.tsx       - Main dashboard with envelope listing
    envelope-new.tsx    - Create new envelope with recipients
    envelope-editor.tsx - Visual field placement editor
    envelope-detail.tsx - Envelope detail view with audit trail
    sign.tsx            - Signing view for recipients (standalone, no sidebar)
  components/
    app-sidebar.tsx     - Sidebar navigation
    zoom-control.tsx    - Site-wide zoom control (70-130%)
    status-badge.tsx    - Status badge component (gold palette)
    theme-provider.tsx  - Dark/light mode provider
    theme-toggle.tsx    - Theme toggle button
    ui/                 - shadcn/ui components

server/
  index.ts              - Express server entry
  routes.ts             - API routes (with tenant middleware + Plenum integration)
  storage.ts            - Database storage layer
  db.ts                 - Database connection
  seed.ts               - Seed data
  middleware/
    tenant.ts           - Multi-tenant middleware (x-tenant-id header)
  services/
    plenum.ts           - PlenumNET API wrappers (phase/split, witness, timing, ml-dsa)
    saveCopy.ts         - Hybrid save logic (PlenumDB → witness → Postgres + HPTP audit)

shared/
  schema.ts             - Drizzle schema + Zod validation + TypeScript types

github-push/            - Scripts for pushing to GitHub (SigmaWolf-8/Ternary)
```

## Key Features
- Dashboard with envelope stats and listing
- Create envelopes with multiple recipients (signer/viewer/witness roles)
- Visual field placement editor (signature, date, text, checkbox, initials)
- Signing flow with typed signature (4 fonts) or drawn signature
- Audit trail with timeline view + HPTP timestamps
- Multi-tenant isolation with RLS
- PlenumDB hybrid save (immutable truth + Postgres metadata)
- Dark/light mode with Swiss Banker black-and-gold palette
- Site-wide zoom control (70-130%)
- Sign page is standalone (no sidebar)

## Database Schema
- `tenants` - Multi-tenant accounts
- `users` - User accounts (with tenantId, role)
- `envelopes` - Document envelopes (draft/sent/signing/completed) with tenantId, plenumDocId, zkProof
- `recipients` - Envelope recipients with roles and status
- `fields` - Placed fields (signature, date, text, checkbox, initials)
- `audit_logs` - Activity tracking with tenantId, hpTpTimestamp, metadata (JSONB)

## API Routes
- `GET /api/health` - Health check (includes Plenum connectivity status)
- `GET/POST /api/envelopes` - List/create envelopes (supports hybrid PlenumDB save with x-tenant-id)
- `GET/PATCH/DELETE /api/envelopes/:id` - Get/update/delete envelope
- `GET /api/envelopes/:id/recipients` - List recipients
- `GET /api/recipients/:id` - Get single recipient
- `GET/PUT /api/envelopes/:id/fields` - Get/replace fields
- `POST /api/envelopes/:id/sign` - Sign envelope
- `GET /api/envelopes/:id/audit` - Audit logs
- `POST /api/tenants` - Create tenant

## Environment Secrets
- `DATABASE_URL` - PostgreSQL connection string
- `PLENUM_API_KEY` - PlenumNET API key for secure document operations
- `SESSION_SECRET` - Session encryption key
- `GitHUB_API_Token` - GitHub API token for pushing to SigmaWolf-8/Ternary

## GitHub Integration
- Repo: https://github.com/SigmaWolf-8/Ternary
- Module path: `salvi-sign/`
- CI: `.github/workflows/ci-salvi-sign.yml`

## User Preferences
- Default dark mode
- Gold/amber primary theme (hue 40, Swiss Banker aesthetic)
- Inter font with 13px base size
- Uppercase tracking labels

## Phase Status
- Phase 0: DONE (repo setup, file structure, GitHub push)
- Phase 1: DONE (multi-tenant backend, Plenum wrappers, RLS, hybrid save, health check)
- Phase 2: DONE (PDF upload/viewer, multi-page rendering, field placement editor with drag/resize/snap, signing flow with PDF rendering, PDF baking with pdf-lib, download buttons, API response optimization)
- Phase 2.5: DONE (graphical field placeholders with icons/labels, HPTP certification on all-signers-complete, witness signing, ML-DSA signatures, certified PDF download, audit trail with HPTP timestamps)
- Phase 3: PENDING (ZK + Security)

## Running
- `npm run dev` starts both frontend and backend on port 5000
- `npm run db:push` pushes schema to database
