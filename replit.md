# PlenumNET Framework Marketing Website

## Overview
A professional light-themed marketing website for PlenumNET (formerly Salvi), a post-quantum internet solutions company. The site showcases deployable components for building quantum-resistant infrastructure, including PlenumDB product page with live compression demo and comprehensive whitepaper management.

## Tech Stack
- **Frontend**: React, TypeScript, Tailwind CSS, Framer Motion
- **Backend**: Express.js, Node.js, PostgreSQL
- **Database**: Drizzle ORM with PostgreSQL
- **UI Components**: shadcn/ui (Button, Badge, Card)
- **Routing**: Wouter
- **Styling**: Light theme with white background and blue accents
- **Authentication**: Replit Auth (GitHub, Google, Apple, X, email/password)

## Project Structure
```
client/
├── src/
│   ├── pages/
│   │   ├── landing.tsx        # Main landing page with all sections
│   │   ├── ternarydb.tsx      # PlenumDB product page
│   │   ├── whitepaper.tsx     # Whitepaper viewer with TOC
│   │   └── github-manager.tsx # Admin-only GitHub file manager
│   ├── components/ui/         # shadcn/ui components
│   ├── index.css              # Theme CSS variables (light theme)
│   └── App.tsx                # App router

server/
├── salvi-core/             # PlenumNET Framework Core API
│   ├── index.ts            # Module exports
│   ├── ternary-types.ts    # Trit representations (A, B, C)
│   ├── ternary-operations.ts # GF(3) operations
│   ├── femtosecond-timing.ts # Femtosecond timestamps
│   └── phase-encryption.ts # Phase-split encryption
├── db.ts                   # PostgreSQL database connection
├── storage.ts              # Database storage interface
├── ternary.ts              # Ternary compression utilities
├── routes.ts               # API routes
├── index.ts                # Express server entry point

shared/
├── schema.ts               # Drizzle database schema
```

## Theme Colors
- Background: White (#ffffff)
- Primary: Blue (HSL 210 100% 45%)
- Text: Dark gray for primary, muted for secondary

## PlenumNET Core API

### Ternary Operations (`/api/salvi/ternary/`)
Based on the whitepaper's Unified Ternary Logic System:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/docs` | GET | API documentation |
| `/convert` | POST | Convert between representations A/B/C |
| `/add` | POST | Ternary addition in GF(3) |
| `/multiply` | POST | Ternary multiplication in GF(3) |
| `/rotate` | POST | Bijective ternary rotation |
| `/not` | POST | Ternary negation |
| `/xor` | POST | Ternary XOR |
| `/batch` | POST | Batch ternary operations |
| `/density/:tritCount` | GET | Information density calculator |

### Femtosecond Timing (`/api/salvi/timing/`)
| Endpoint | Method | Description |
|----------|--------|-------------|
| `/timestamp` | GET | Get femtosecond-precision timestamp |
| `/metrics` | GET | Get timing metrics |
| `/batch/:count` | GET | Generate batch of timestamps |

### Phase Encryption (`/api/salvi/phase/`)
| Endpoint | Method | Description |
|----------|--------|-------------|
| `/config/:mode` | GET | Get phase configuration |
| `/split` | POST | Split data into phase-encrypted components |
| `/recombine` | POST | Recombine phase-split data |
| `/recommend` | GET | Get recommended encryption mode |

### Ternary Representations (from Whitepaper)
- **Representation A** (Computational): {-1, 0, +1}
- **Representation B** (Network): {0, 1, 2}
- **Representation C** (Human): {1, 2, 3}

### Bijections
- A→B: `f(a) = a + 1`
- A→C: `f(a) = a + 2`
- B→C: `f(b) = b + 1`

## Demo API Endpoints
- `POST /api/demo/run` - Run compression demo
- `POST /api/demo/upload` - Upload file for compression testing
- `GET /api/demo/stats` - Get aggregated statistics
- `GET /api/demo/history` - Get compression history
- `GET /api/demo/data/:sessionId` - Get paginated data with metrics

## Whitepaper API
- `GET /api/whitepapers` - List all whitepapers
- `GET /api/whitepapers/active` - Get active whitepaper
- `POST /api/whitepapers` - Create new whitepaper

## GitHub Manager API (Admin Only)
| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/github/token` | POST | Save GitHub Personal Access Token |
| `/api/github/status` | GET | Check if token is configured |
| `/api/github/repos/:owner/:repo/contents` | GET | Get repository contents (use `?path=` for subdirs) |
| `/api/github/file/:owner/:repo` | GET | Get file content (use `?path=` for file path) |
| `/api/github/file/:owner/:repo` | PUT | Create/update file (body: path, content, message, sha) |
| `/api/github/file/:owner/:repo` | DELETE | Delete file (body: path, sha, message) |
| `/api/user/admin-status` | GET | Check if current user is admin |

## Kong Konnect API (API Gateway Integration)
**Security Note**: All mutation endpoints (POST) and config access require admin authentication.

| Endpoint | Method | Auth | Description |
|----------|--------|------|-------------|
| `/api/kong/status` | GET | Public | Check Kong Konnect connection status and user info |
| `/api/kong/organization` | GET | Public | Get Kong organization details |
| `/api/kong/control-planes` | GET | Public | List all Kong control planes |
| `/api/kong/control-planes/:cpId/services` | GET | Public | Get services for a control plane |
| `/api/kong/control-planes/:cpId/services` | POST | Admin | Create a service (body: name, url, enabled, tags) |
| `/api/kong/control-planes/:cpId/routes` | GET | Public | Get routes for a control plane |
| `/api/kong/control-planes/:cpId/services/:serviceId/routes` | POST | Admin | Create a route (body: name, paths, methods) |
| `/api/kong/control-planes/:cpId/plugins` | GET | Public | Get plugins for a control plane |
| `/api/kong/control-planes/:cpId/services/:serviceId/plugins` | POST | Admin | Add plugin to service (body: name, config) |
| `/api/kong/control-planes/:cpId/sync-plenumnet` | POST | Admin | Sync all PlenumNET services to Kong |
| `/api/kong/control-planes/:cpId/generate-deployment` | POST | Admin | Generate TLS certs and deployment package |
| `/api/kong/control-planes/:cpId/deploy-to-cloud` | POST | Admin | Push Dockerfile to GitHub for Render/Railway deploy (body: platform, owner, repo) |
| `/api/kong/config` | GET | Admin | Get local Kong configuration file (contains API keys) |
| `/api/kong/save-to-github` | POST | Admin | Save Kong config to GitHub (body: owner, repo, path) |

### Cloud Deployment
The deploy-to-cloud endpoint:
1. Generates TLS certificates (10-year validity)
2. Uploads certificate to Kong Konnect
3. Pushes Dockerfile and entrypoint.sh to GitHub (private key NOT committed)
4. Returns TLS private key for user to set as `KONG_TLS_KEY` env var in cloud platform
5. Returns one-click deploy URLs for Render (free tier) and Railway ($5 credit)

### Kong Configuration Files
The `kong/` directory contains:
- `kong.yaml` - Declarative Kong configuration (decK format)
- `README.md` - Documentation for AI agents

### For AI Agents
To access PlenumNET APIs through Kong Gateway:
1. Get the Kong configuration from `/api/kong/config` or GitHub repo
2. Use API keys defined in kong.yaml (`plenumnet-ai-agent-key`)
3. Access endpoints through Kong proxy with rate limiting and security

## Database Schema
- **users** - Authenticated users (email, name, profile image, isAdmin, githubToken)
- **sessions** - User sessions for authentication
- **demo_sessions** - Demo session metadata
- **binary_storage** - Original binary data
- **ternary_storage** - Compressed ternary data
- **compression_benchmarks** - Performance metrics
- **compression_history** - Historical compression records
- **whitepapers** - Whitepaper content (83 sections)

## Pages
1. **Landing** (`/`) - Hero, approach, components, comparison, markets, CTA
2. **PlenumDB** (`/ternarydb`) - Product page with live compression demo
3. **Whitepaper** (`/whitepaper`) - Full whitepaper viewer with Table of Contents
4. **GitHub Manager** (`/github`) - Admin-only page to manage GitHub repository files
5. **Kong Konnect Integration** (`/kong-konnect`) - API gateway integration page with Kong Konnect

### GitHub Manager Features
   - **Sorting**: Sort files by name, size, or type (ascending/descending)
   - **Filtering**: Search files by name, filter by type (code/docs/config/other)
   - **Metrics Dashboard**: Shows file counts, folder counts, total size, and type breakdown
   - **View Modes**: Toggle between list view and grid view

## Running the App
```bash
npm run dev
```
The app runs on port 5000.
