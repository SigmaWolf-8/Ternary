# SalviSign - Ternary-Native E-Signature Platform

A clean, sleek, highly customizable e-signature platform built on top of the Salvi/Ternary Framework.

## Overview

SalviSign provides enterprise-grade document signing with:
- Document envelope management with multiple recipients
- Visual field placement editor (signature, date, text, checkbox, initials)
- Typed signature (4 fonts) or drawn signature capture
- Audit trail with timeline view
- Swiss Banker black-and-gold aesthetic
- Dark/light mode with site-wide zoom control

## Tech Stack

- **Frontend**: React + TypeScript + Vite + Tailwind CSS + shadcn/ui
- **Backend**: Express.js + TypeScript (migrating to Fastify)
- **Database**: PostgreSQL with Drizzle ORM
- **State Management**: TanStack React Query

## Key Integrations

- All crypto/signing/timing via existing Ternary endpoints
- PlenumDB storage via dedicated endpoint
- Ternary constants & accents from shared/
- HPTP timestamps for audit immutability

## Development

```bash
cd salvi-sign
npm install
npm run dev          # concurrently client + server
```

## Database Schema

- `users` - User accounts
- `envelopes` - Document envelopes (draft/sent/signing/completed)
- `recipients` - Envelope recipients with roles and status
- `fields` - Placed fields (signature, date, text, checkbox, initials)
- `audit_logs` - Activity tracking

## Status

Phase 0 complete. Foundation laid for Ternary integration.
