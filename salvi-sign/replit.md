# SalviSign - E-Signature Platform

## Overview
SalviSign is a clean, sleek e-signature platform inspired by DocuSign. It provides document envelope management, field placement, signature collection, and audit trail tracking.

## Tech Stack
- **Frontend**: React + TypeScript + Vite + Tailwind CSS + shadcn/ui + wouter routing
- **Backend**: Express.js + TypeScript
- **Database**: PostgreSQL with Drizzle ORM
- **State Management**: TanStack React Query

## Project Structure
```
client/src/
  pages/
    dashboard.tsx      - Main dashboard with envelope listing
    envelope-new.tsx   - Create new envelope with recipients
    envelope-editor.tsx - Visual field placement editor
    envelope-detail.tsx - Envelope detail view with audit trail
    sign.tsx           - Signing view for recipients (standalone, no sidebar)
  components/
    app-sidebar.tsx    - Sidebar navigation
    theme-provider.tsx - Dark/light mode provider
    theme-toggle.tsx   - Theme toggle button
    status-badge.tsx   - Status badge component
    ui/                - shadcn/ui components

server/
  index.ts    - Express server entry
  routes.ts   - API routes
  storage.ts  - Database storage layer
  db.ts       - Database connection
  seed.ts     - Seed data

shared/
  schema.ts   - Drizzle schema + Zod validation + TypeScript types
```

## Key Features
- Dashboard with envelope stats and listing
- Create envelopes with multiple recipients (signer/viewer/witness roles)
- Visual field placement editor (signature, date, text, checkbox, initials)
- Signing flow with typed signature (4 fonts) or drawn signature
- Audit trail with timeline view
- Dark/light mode
- Sign page is standalone (no sidebar)

## Database Schema
- `users` - User accounts
- `envelopes` - Document envelopes (draft/sent/signing/completed)
- `recipients` - Envelope recipients with roles and status
- `fields` - Placed fields (signature, date, text, checkbox, initials)
- `audit_logs` - Activity tracking

## API Routes
- `GET/POST /api/envelopes` - List/create envelopes
- `GET/PATCH/DELETE /api/envelopes/:id` - Get/update/delete envelope
- `GET /api/envelopes/:id/recipients` - List recipients
- `GET /api/recipients/:id` - Get single recipient
- `GET/PUT /api/envelopes/:id/fields` - Get/replace fields
- `POST /api/envelopes/:id/sign` - Sign envelope
- `GET /api/envelopes/:id/audit` - Audit logs

## User Preferences
- Default dark mode
- Blue primary theme (hue 203)

## Running
- `npm run dev` starts both frontend and backend on port 5000
- `npm run db:push` pushes schema to database
