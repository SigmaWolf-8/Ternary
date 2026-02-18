# Sign Here - E-Signature Platform

## Overview
Sign Here is a clean, sleek e-signature platform inspired by DocuSign, built as a module within the Ternary monorepo. It provides document envelope management, field placement, signature collection, audit trail tracking, and integrates with PlenumNET for secure document storage and witnessing.

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
    dashboard.tsx       - File Cabinet (main envelope listing)
    envelope-new.tsx    - Create new envelope with recipients
    envelope-editor.tsx - Visual field placement editor
    envelope-detail.tsx - Envelope detail view with audit trail + sign links
    sign.tsx            - Signing view for recipients (standalone, no sidebar)
    certificate.tsx     - Certificate of Completion view (gold seal, HPTP, PDF viewer)
    templates.tsx       - Template Gallery (search, filter, preview, fork, use)
    wbs-tagging.tsx     - Multi-tag assignment page (assign multiple WBS tags per envelope)
    settings.tsx        - User settings (timezone, profile, date format)
  components/
    app-sidebar.tsx     - Sidebar navigation
    zoom-control.tsx    - Site-wide zoom control (70-130%)
    onboarding-tour.tsx - Guided onboarding tour with opt-out (localStorage persistence)
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
    pdfCrypto.ts        - CNSA 2.0 compliant encryption for PDF documents (HKDF-SHA512 + AES-256-GCM) with legacy backward-compat
    fieldCrypto.ts      - CNSA 2.0 field-level encryption for all database tables (HKDF-SHA512 + AES-256-GCM)
    fileConvert.ts      - LibreOffice-based DOCX/XLSX/CSV to PDF conversion
    zk.ts               - ZK proof generation + verification (server-side)
    wsCollab.ts         - WebSocket collaboration server (room registry, presence, field sync)
    aiFields.ts         - AI field detection (regex patterns, document type recognition)

client/src/lib/
  useCollab.ts          - WebSocket collaboration hook (presence, cursors, field ops)
  offlineCache.ts       - IndexedDB offline caching (envelopes, fields, recipients, pending ops)

shared/
  schema.ts             - Drizzle schema + Zod validation + TypeScript types

zk-wasm/                - Rust ZK circuit compiled to WASM
  Cargo.toml            - Rust dependencies (sha2, wasm-bindgen, serde)
  src/lib.rs            - Groth16-style ZK proof generation + verification

github-push/            - Scripts for pushing to GitHub (SigmaWolf-8/Ternary)
```

## Key Features
- Dashboard with envelope stats, search bar, status filters, and clickable stat cards
- Create envelopes with multiple recipients (signer/viewer/witness roles)
- Visual field placement editor (signature, date, text, checkbox, initials)
- Graphical field placeholders with icons and labels
- Signing flow with typed signature (8 fonts) or drawn signature
- iOS mobile-optimized signing view: responsive PDF (full-width), full-screen touch signature pad, 44px touch targets, haptic feedback, gold confetti completion, floating Next Field FAB, sticky progress header with conic-gradient ring, dark zinc-950 theme with amber/gold accents, undo strokes, ink color picker, safe-area padding, overscroll-behavior-none, 100dvh viewport
- HPTP certification when all signers complete (femtosecond timestamps + ML-DSA)
- Audit trail with timeline view + HPTP timestamps
- Multi-tenant isolation with RLS
- CNSA 2.0 encryption pipeline (HKDF-SHA512 key derivation + AES-256-GCM + PlenumNET dual-phase split)
- Field-level encryption at rest for ALL database tables (8 tables, all PII fields encrypted with fenc: prefix)
- ML-DSA post-quantum signatures at document upload AND each signing event
- PlenumDB hybrid save (immutable truth + Postgres metadata)
- File format support: PDF, DOCX, XLSX, CSV (auto-converted to PDF via LibreOffice)
- Phase 3 ZK proof system (Rust WASM, Groth16-style, client-side verification)
- ZK-verified secure sharing page (/share/:id)
- IP geolocation audit logging on signing + ZK proof events (ipapi.co HTTPS with caching)
- "Good Vibrations" (Great Vibes) script font for Sign Here branding
- Dark/light mode with Swiss Banker black-and-gold palette
- Site-wide zoom control (70-130%)
- Sign page and Share page are standalone (no sidebar)
- PDF encryption at rest (AES-256-GCM, auto-decrypt on retrieval, backward-compat)
- Inline recipient CRUD from editor sidebar (add/edit/remove with email validation)
- Generate Seal places signatures INLINE with seal (side by side) with document footer line above both
- Femtosecond-precision seal timestamps (performance.now() high-resolution timing)
- Email notifications via Resend API when envelopes are sent (Swiss Banker themed HTML template)
- About page with certifications, compliance equivalences, PDF Stapler and Document Converter feature descriptions
- Admin page with tenant/user CRUD, role management, platform statistics, SaaS architecture documentation, and SAdmin-exclusive SaaS panel
- 5-tier role model: SAdmin (platform creator), Admin (tenant owner), Manager (envelope oversight), Signer (document signing), Viewer (read-only compliance)
- SAdmin role: isPlatformCreator flag, protected from deletion/role change, exclusive SaaS settings tab with customer management, feature toggles, and pricing tiers
- Version display in app header (v1.0.0 | PlenumNET v2.1)
- Template Gallery with search, category filter, preview modal, fork, use, and delete actions
- 6 built-in public templates (NDA, Employment Offer, Service Agreement, Lease, Consent Form, Invoice)
- Save envelope fields as reusable template from envelope detail page
- WBS Tags system: 13 configurable Work Breakdown Structure tags per tenant for envelope categorization and filtering
- Multi-tag WBS assignment: envelopes can have multiple WBS tags (many-to-many junction table)
- WBS Tagging page for bulk multi-tag assignment across all envelopes
- WBS tag toggles in envelope editor sidebar (multi-select, replaces single dropdown)
- Dashboard WBS tag filter row with colored tag buttons and untagged filter
- Drag-and-drop WBS tag reordering via @dnd-kit
- Separate Templates menu section in sidebar (Templates + WBS Tags)
- Logo click navigates to Dashboard and replays video animation twice

## Database Schema
- `tenants` - Multi-tenant accounts
- `users` - User accounts (with tenantId, role)
- `envelopes` - Document envelopes (draft/sent/signing/completed) with tenantId, plenumDocId, zkProof
- `recipients` - Envelope recipients with roles and status
- `fields` - Placed fields (signature, date, text, checkbox, initials)
- `audit_logs` - Activity tracking with tenantId, hpTpTimestamp, metadata (JSONB)
- `templates` - Reusable document templates (name, description, category, tags, fieldDefs jsonb, isPublic, tenantId, forkedFromId)
- `wbs_tags` - Work Breakdown Structure tags (name, color, sortOrder, tenantId, max 13 per tenant)
- `envelope_wbs_tags` - Many-to-many junction table for envelope-to-WBS-tag assignments

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
- `POST /api/envelopes/:id/ai-detect` - AI field detection (regex pattern matching + doc type)
- `WS /ws/collab` - WebSocket collaboration (presence, cursors, field sync)
- `POST /api/tenants` - Create tenant
- `GET /api/admin/stats` - Platform statistics (tenants, users, envelopes)
- `GET /api/admin/tenants` - List all tenants
- `GET /api/admin/users` - List all users (password excluded)
- `POST /api/admin/users` - Create user with role/tenant assignment
- `PATCH /api/admin/users/:id` - Update user role/email/tenant
- `DELETE /api/admin/users/:id` - Delete user
- `GET /api/templates` - List templates (public + tenant-scoped)
- `POST /api/templates` - Create template
- `GET /api/templates/:id` - Get single template
- `POST /api/templates/:id/fork` - Fork a template (creates copy with forkedFromId)
- `DELETE /api/templates/:id` - Delete template
- `POST /api/envelopes/:id/save-as-template` - Save envelope fields as reusable template
- `GET /api/envelope-wbs-tags` - List all envelope-WBS-tag associations
- `GET /api/envelopes/:id/wbs-tags` - Get WBS tags assigned to an envelope
- `PUT /api/envelopes/:id/wbs-tags` - Set WBS tags for an envelope (replaces all)
- `GET /api/wbs-tags` - List WBS tags (tenant-scoped)
- `POST /api/wbs-tags` - Create WBS tag (max 13 per tenant)
- `PATCH /api/wbs-tags/:id` - Update WBS tag (name, color, sortOrder)
- `DELETE /api/wbs-tags/:id` - Delete WBS tag
- `GET /api/saas/settings` - SaaS platform settings (SAdmin-only)
- `PATCH /api/saas/settings` - Update SaaS platform settings (SAdmin-only)

## Environment Secrets
- `DATABASE_URL` - PostgreSQL connection string
- `PLENUM_API_KEY` - PlenumNET API key for secure document operations
- `SESSION_SECRET` - Session encryption key
- `Resend_API_KEY` - Resend API key for email notifications
- `GitHUB_API_Token` - GitHub API token for pushing to SigmaWolf-8/Ternary

## GitHub Integration
- Repo: https://github.com/SigmaWolf-8/Ternary
- Module path: `sign-here/`
- CI: `.github/workflows/ci-sign-here.yml`

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
- Phase 4: DONE (WebSocket real-time collaboration with room registry/presence/field sync, AI field detection with regex patterns and document type recognition, remote cursor rendering on PDF pages, offline-first IndexedDB caching with pending ops queue, conditional field logic with dependsOnFieldId/dependsOnValue)

## Running
- `npm run dev` starts both frontend and backend on port 5000
- `npm run db:push` pushes schema to database
