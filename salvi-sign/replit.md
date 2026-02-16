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
    envelope-detail.tsx - Envelope detail view with audit trail + sign links
    sign.tsx            - Signing view for recipients (standalone, no sidebar)
    settings.tsx        - User settings (timezone, profile, date format)
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
    pdfForms.ts         - PDF form baking with pdf-lib
    pdfCrypto.ts        - AES-256-GCM encryption/decryption for PDF at rest
    zk.ts               - ZK proof generation + verification (server-side)

shared/
  schema.ts             - Drizzle schema + Zod validation + TypeScript types

zk-wasm/                - Rust ZK circuit compiled to WASM
  Cargo.toml            - Rust dependencies (sha2, wasm-bindgen, serde)
  src/lib.rs            - Groth16-style ZK proof generation + verification

github-push/            - Scripts for pushing to GitHub (SigmaWolf-8/Ternary)
```

## Key Features
- Dashboard with envelope stats and listing
- Create envelopes with multiple recipients (signer/viewer/witness roles)
- Visual field placement editor (signature, date, text, checkbox, initials)
- Graphical field placeholders with icons and labels
- Signing flow with typed signature (4 fonts) or drawn signature
- HPTP certification when all signers complete (femtosecond timestamps + ML-DSA)
- Audit trail with timeline view + HPTP timestamps
- Multi-tenant isolation with RLS
- PlenumDB hybrid save (immutable truth + Postgres metadata)
- Phase 3 ZK proof system (Rust WASM, Groth16-style, client-side verification)
- ZK-verified secure sharing page (/share/:id)
- IP geo audit logging for enterprise compliance
- "Good Vibrations" (Great Vibes) script font for SalviSign branding
- Dark/light mode with Swiss Banker black-and-gold palette
- Site-wide zoom control (70-130%)
- Sign page and Share page are standalone (no sidebar)
- PDF encryption at rest (AES-256-GCM, auto-decrypt on retrieval, backward-compat)
- Inline recipient CRUD from editor sidebar (add/edit/remove with email validation)
- Generate Seal auto-repositions signatures above seal (left-justified, stacked)
- Femtosecond-precision seal timestamps (performance.now() high-resolution timing)
- Email notifications via Resend API when envelopes are sent (Swiss Banker themed HTML template)

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
- `GET/PATCH/DELETE /api/envelopes/:id` - Get/update/delete envelope (PATCH validates recipients before send)
- `GET/POST /api/envelopes/:id/recipients` - List/create recipients
- `GET /api/recipients/:id` - Get single recipient
- `PATCH /api/recipients/:id` - Update recipient (name, email, role)
- `DELETE /api/recipients/:id` - Delete recipient
- `GET/PUT /api/envelopes/:id/fields` - Get/replace fields
- `POST /api/envelopes/:id/sign` - Sign envelope (auto-certifies when all signers complete)
- `GET /api/envelopes/:id/audit` - Audit logs
- `GET /api/envelopes/:id/bake` - Download baked/signed PDF
- `POST /api/envelopes/:id/share-proof` - Generate ZK share proof (requires completed status)
- `GET /api/envelopes/:id/share` - Get share data (envelope summary + zkData)
- `POST /api/envelopes/:id/verify-proof` - Server-side ZK proof verification
- `POST /api/tenants` - Create tenant

## Environment Secrets
- `DATABASE_URL` - PostgreSQL connection string
- `PLENUM_API_KEY` - PlenumNET API key for secure document operations
- `SESSION_SECRET` - Session encryption key
- `Resend_API_KEY` - Resend API key for email notifications
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
- Phase 3: DONE (Rust WASM ZK proof system, client-side ZK verifier, server-side proof gen, share page with ZK gate, IP geo audit logging, send validation, Good Vibrations font, GitHub push)
- Phase 3.1: DONE (Dashboard click navigates to editor, multi-PDF upload with pdf-lib stitching, editor "Add Pages" for appending PDFs, drag-and-drop PDF reordering)
- Phase 3.2: DONE (9 signature fonts with custom TTF embedding in baked PDFs via @pdf-lib/fontkit, orphaned field claiming on sign page, cursive fonts: Great Vibes, Dancing Script, Pacifico, Sacramento, Alex Brush)

## Running
- `npm run dev` starts both frontend and backend on port 5000
- `npm run db:push` pushes schema to database
