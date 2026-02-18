# TERNARY_EPHEMERIS_INTEGRATION_GUIDE.md
# Single source of truth for both projects
# Copy-paste this entire file into BOTH Replit projects:
#   1. Astrology frontend (https://a4e186d0-...replit.dev/)
#   2. PlenumNET backend (https://PlenumNET.replit.app)
# Last updated: February 18, 2026

## PROJECT ROLES – VERY IMPORTANT

### PlenumNET (backend / API / Rust)
YOU ARE THE COMPUTATIONAL ENGINE.
Responsible for:
- All ternary mathematics (BalancedFloat, trig functions, Kepler solver)
- Ephemeris calculations from first principles
- Exposing REST/GraphQL endpoints
- Returning JSON with floating-point ternary values (no snapping)

Do NOT write:
- JavaScript, HTML, CSS, React, frontend fetch logic, UI rendering

### Astrology App (frontend / JavaScript)
YOU ARE THE USER INTERFACE.
Responsible for:
- Calling the PlenumNET API
- Displaying standard ecliptic longitudes (0–360°) as primary truth
- Showing continuous ternary longitude (0–364°) from API
- Visualizing resonance score (0–1) and optional spiral
- Handling user input (date/time → JD, lat/lon)

Do NOT write:
- Rust code
- BalancedFloat or any ternary arithmetic
- Ephemeris calculations

## ARCHITECTURE OVERVIEW

```
Astrology App (JS/Replit)  ↔  HTTP/JSON  ↔  PlenumNET API (Rust/Axum)  ↔  ternary-math Core
```

- All heavy math lives in `ternary-math/src` (GitHub repo)
- Frontend only calls API and renders results

## API CONTRACT (both projects must follow)

Base URL (for dev): https://plenumnet.replit.app  (or your custom domain later)

### Endpoint 1 – Convert degrees/radians
POST /api/v1/ephemeris/convert
```json
{
  "type": "std_deg",
  "value": 123.45,
  "return_resonance": true
}
```
Response:
```json
{
  "std_deg": 123.45,
  "ternary_deg": 124.812,
  "ternary_rad": 9.601,
  "resonance": 0.78,
  "nearest_z28": 10
}
```

### Endpoint 2 – Planetary ephemeris
POST /api/v1/ephemeris/position
```json
{
  "planet": "earth",
  "jd": 2460740.0,
  "include_resonance": true,
  "observer": {
    "lat": 53.54,
    "lon": -113.49,
    "alt": 0
  }
}
```
Response:
```json
{
  "planet": "earth",
  "jd": 2460740.0,
  "ecliptic_longitude": 165.234,
  "ecliptic_latitude": 0.0,
  "distance_au": 1.001,
  "ternary_longitude": 167.069,
  "ternary_latitude": 0.0,
  "ternary_rad": 12.851,
  "resonance": 0.556,
  "nearest_z28": 13
}
```

### Endpoint 3 – Batch ephemeris (all planets at once)
POST /api/v1/ephemeris/batch
```json
{
  "planets": ["sun", "moon", "mars", "venus", "jupiter", "saturn"],
  "jd": 2460740.0,
  "include_resonance": true
}
```
Response:
```json
{
  "jd": 2460740.0,
  "planets": {
    "sun": { "ecliptic_longitude": 345.17, "ternary_longitude": 349.01, "resonance": 0.69, ... },
    "moon": { "ecliptic_longitude": 59.64, "ternary_longitude": 60.30, "resonance": 0.28, ... },
    ...
  }
}
```

### Endpoint 4 – API info
GET /api/v1/ephemeris/info
Returns metadata, supported planet list, system constants.

## KEY CODE SNIPPETS

### Rust – Core constants (ternary-math/src/constants.rs)
```rust
pub const FULL_CIRCLE_DEG: f64 = 364.0;
pub const RADIAN_DEG: f64 = 13.0;
pub const TWO_PI_TERNARY: f64 = 28.0;
pub const TAU_TRIBONACCI: f64 = 1.839286755214161;

#[inline]
pub fn std_deg_to_ternary_deg(std_deg: f64) -> f64 {
    std_deg * (FULL_CIRCLE_DEG / 360.0)
}

#[inline]
pub fn ternary_deg_to_resonance(tern_deg: f64) -> f64 {
    let mod13 = tern_deg % 13.0;
    let dist = mod13.min(13.0 - mod13);
    1.0 - (dist / 6.5)
}
```

### JavaScript – Frontend API call example (Astrology App)
```js
const API_BASE = process.env.PLENUMNET_API_URL || 'https://plenumnet.replit.app';

async function fetchTernaryEphemeris(planet, jd, lat = 53.54, lon = -113.49) {
  try {
    const response = await fetch(`${API_BASE}/api/v1/ephemeris/position`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        planet,
        jd,
        include_resonance: true,
        observer: { lat, lon, alt: 0 }
      })
    });

    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return await response.json();
  } catch (err) {
    console.error('API call failed:', err);
    return null;
  }
}

async function fetchAllPlanets(jd, lat = 53.54, lon = -113.49) {
  const response = await fetch(`${API_BASE}/api/v1/ephemeris/batch`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      planets: ['sun', 'moon', 'mercury', 'venus', 'mars', 'jupiter', 'saturn', 'uranus', 'neptune', 'pluto'],
      jd,
      include_resonance: true,
      observer: { lat, lon, alt: 0 }
    })
  });

  if (!response.ok) throw new Error(`HTTP ${response.status}`);
  return await response.json();
}
```

## QUICK START CHECKLIST

**In PlenumNET (backend):**
1. Ternary ephemeris API is live at /api/v1/ephemeris/*
2. Supports all planets: sun, moon, mercury, venus, earth, mars, jupiter, saturn, uranus, neptune, pluto
3. Returns continuous ternary coordinates (never snapped)
4. CORS allows *.replit.dev and *.replit.app origins

**In Astrology App (frontend):**
1. Add the `fetchTernaryEphemeris` / `fetchAllPlanets` functions above
2. Call them when user submits birth data
3. Display `data.ternary_longitude` and resonance % in UI
4. Optional: draw 364° wheel + 28 spokes (CSS/SVG radial lines)

## CLOSING NOTES

- Do **not** duplicate math logic between projects.
- All ternary precision work belongs in PlenumNET → API → JSON.
- Frontend only consumes results and renders.
- Use Edmonton coords (53.54, -113.49) for local testing.

If Replit AI asks which project it's in, tell it to read this file first.

Happy building!
Chill (@capo_mastro)
