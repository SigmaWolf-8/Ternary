/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * Ternary Ephemeris Engine
 * Continuous ternary angular system (364° circle, 13° ternary radian, Z₂₈ lattice)
 * with real planetary position computation from Keplerian orbital elements.
 */

export const FULL_CIRCLE_DEG = 364.0;
export const STD_CIRCLE_DEG = 360.0;
export const TERNARY_RADIAN_DEG = 13.0;
export const Z28_COUNT = 28;
export const TAU_TRIBONACCI = 1.839286755214161;
export const DEG_TO_RAD = Math.PI / 180.0;
export const J2000_EPOCH = 2451545.0;

export function stdDegToTernaryDeg(stdDeg: number): number {
  return stdDeg * (FULL_CIRCLE_DEG / STD_CIRCLE_DEG);
}

export function ternaryDegToStdDeg(ternDeg: number): number {
  return ternDeg * (STD_CIRCLE_DEG / FULL_CIRCLE_DEG);
}

export function stdDegToTernaryRad(stdDeg: number): number {
  return stdDeg * (FULL_CIRCLE_DEG / STD_CIRCLE_DEG) / TERNARY_RADIAN_DEG;
}

export function ternaryDegToResonance(ternDeg: number): number {
  const mod13 = ((ternDeg % TERNARY_RADIAN_DEG) + TERNARY_RADIAN_DEG) % TERNARY_RADIAN_DEG;
  const dist = Math.min(mod13, TERNARY_RADIAN_DEG - mod13);
  return 1.0 - dist / 6.5;
}

export function nearestZ28(ternDeg: number): number {
  const normalized = ((ternDeg % FULL_CIRCLE_DEG) + FULL_CIRCLE_DEG) % FULL_CIRCLE_DEG;
  return Math.round(normalized / TERNARY_RADIAN_DEG) % Z28_COUNT;
}

function normalizeAngle(deg: number): number {
  return ((deg % 360) + 360) % 360;
}

function solveKepler(M: number, e: number, tolerance = 1e-12, maxIter = 50): number {
  let E = M;
  for (let i = 0; i < maxIter; i++) {
    const dE = (E - e * Math.sin(E) - M) / (1 - e * Math.cos(E));
    E -= dE;
    if (Math.abs(dE) < tolerance) break;
  }
  return E;
}

function trueAnomalyFromE(E: number, e: number): number {
  return 2 * Math.atan2(
    Math.sqrt(1 + e) * Math.sin(E / 2),
    Math.sqrt(1 - e) * Math.cos(E / 2)
  );
}

interface OrbitalElements {
  a: number;
  e: number;
  I: number;
  L: number;
  longPeri: number;
  longNode: number;
  aDot: number;
  eDot: number;
  IDot: number;
  LDot: number;
  longPeriDot: number;
  longNodeDot: number;
}

const PLANET_ELEMENTS: Record<string, OrbitalElements> = {
  mercury: {
    a: 0.38709927, e: 0.20563593, I: 7.00497902, L: 252.25032350,
    longPeri: 77.45779628, longNode: 48.33076593,
    aDot: 0.00000037, eDot: 0.00001906, IDot: -0.00594749, LDot: 149472.67411175,
    longPeriDot: 0.16047689, longNodeDot: -0.12534081
  },
  venus: {
    a: 0.72333566, e: 0.00677672, I: 3.39467605, L: 181.97909950,
    longPeri: 131.60246718, longNode: 76.67984255,
    aDot: 0.00000390, eDot: -0.00004107, IDot: -0.00078890, LDot: 58517.81538729,
    longPeriDot: 0.00268329, longNodeDot: -0.27769418
  },
  earth: {
    a: 1.00000261, e: 0.01671123, I: -0.00001531, L: 100.46457166,
    longPeri: 102.93768193, longNode: 0.0,
    aDot: 0.00000562, eDot: -0.00004392, IDot: -0.01294668, LDot: 35999.37244981,
    longPeriDot: 0.32327364, longNodeDot: 0.0
  },
  mars: {
    a: 1.52371034, e: 0.09339410, I: 1.84969142, L: -4.55343205,
    longPeri: -23.94362959, longNode: 49.55953891,
    aDot: 0.00001847, eDot: 0.00007882, IDot: -0.00813131, LDot: 19140.30268499,
    longPeriDot: 0.44441088, longNodeDot: -0.29257343
  },
  jupiter: {
    a: 5.20288700, e: 0.04838624, I: 1.30439695, L: 34.39644051,
    longPeri: 14.72847983, longNode: 100.47390909,
    aDot: -0.00011607, eDot: -0.00013253, IDot: -0.00183714, LDot: 3034.74612775,
    longPeriDot: 0.21252668, longNodeDot: 0.20469106
  },
  saturn: {
    a: 9.53667594, e: 0.05386179, I: 2.48599187, L: 49.95424423,
    longPeri: 92.59887831, longNode: 113.66242448,
    aDot: -0.00125060, eDot: -0.00050991, IDot: 0.00193609, LDot: 1222.49362201,
    longPeriDot: -0.41897216, longNodeDot: -0.28867794
  },
  uranus: {
    a: 19.18916464, e: 0.04725744, I: 0.77263783, L: 313.23810451,
    longPeri: 170.95427630, longNode: 74.01692503,
    aDot: -0.00196176, eDot: -0.00004397, IDot: -0.00242939, LDot: 428.48202785,
    longPeriDot: 0.40805281, longNodeDot: 0.04240589
  },
  neptune: {
    a: 30.06992276, e: 0.00859048, I: 1.77004347, L: -55.12002969,
    longPeri: 44.96476227, longNode: 131.78422574,
    aDot: 0.00026291, eDot: 0.00005105, IDot: 0.00035372, LDot: 218.45945325,
    longPeriDot: -0.32241464, longNodeDot: -0.00508664
  },
  pluto: {
    a: 39.48211675, e: 0.24882730, I: 17.14001206, L: 238.92903833,
    longPeri: 224.06891629, longNode: 110.30393684,
    aDot: -0.00031596, eDot: 0.00005170, IDot: 0.00004818, LDot: 145.20780515,
    longPeriDot: -0.04062942, longNodeDot: -0.01183482
  }
};

const MOON_ELEMENTS = {
  L0: 218.3165, LDot: 13.17639648,
  M0: 134.9634, MDot: 13.06499295,
  F0: 93.2721, FDot: 13.22935024,
  D0: 297.8502, DDot: 12.19074912,
  Om0: 125.0445, OmDot: -0.05295377
};

const SUN_MEAN = {
  L0: 280.46646, LDot: 36000.76983 / 36525.0,
  M0: 357.52911, MDot: 35999.05029 / 36525.0
};

export interface EphemerisResult {
  ecliptic_longitude: number;
  ecliptic_latitude: number;
  distance_au: number;
  ternary_longitude: number;
  ternary_latitude: number;
  ternary_rad: number;
  resonance: number;
  nearest_z28: number;
}

function computePlanetPosition(planet: string, jd: number): EphemerisResult {
  const T = (jd - J2000_EPOCH) / 36525.0;

  if (planet === "sun") {
    const L = normalizeAngle(SUN_MEAN.L0 + SUN_MEAN.LDot * (jd - J2000_EPOCH));
    const M = normalizeAngle(SUN_MEAN.M0 + SUN_MEAN.MDot * (jd - J2000_EPOCH));
    const Mrad = M * DEG_TO_RAD;
    const C = (1.9146 - 0.004817 * T - 0.000014 * T * T) * Math.sin(Mrad)
            + (0.019993 - 0.000101 * T) * Math.sin(2 * Mrad)
            + 0.00029 * Math.sin(3 * Mrad);
    const sunLon = normalizeAngle(L + C);
    const e = 0.016708634 - 0.000042037 * T;
    const v = (M + C) * DEG_TO_RAD;
    const R = 1.000001018 * (1 - e * e) / (1 + e * Math.cos(v));
    const ternLon = stdDegToTernaryDeg(sunLon);
    return {
      ecliptic_longitude: sunLon, ecliptic_latitude: 0, distance_au: R,
      ternary_longitude: ternLon, ternary_latitude: 0,
      ternary_rad: ternLon / TERNARY_RADIAN_DEG,
      resonance: ternaryDegToResonance(ternLon), nearest_z28: nearestZ28(ternLon)
    };
  }

  if (planet === "moon") {
    const d = jd - J2000_EPOCH;
    const L = normalizeAngle(MOON_ELEMENTS.L0 + MOON_ELEMENTS.LDot * d);
    const M = normalizeAngle(MOON_ELEMENTS.M0 + MOON_ELEMENTS.MDot * d);
    const F = normalizeAngle(MOON_ELEMENTS.F0 + MOON_ELEMENTS.FDot * d);
    const D = normalizeAngle(MOON_ELEMENTS.D0 + MOON_ELEMENTS.DDot * d);
    const Om = normalizeAngle(MOON_ELEMENTS.Om0 + MOON_ELEMENTS.OmDot * d);
    const Lr = L * DEG_TO_RAD; const Mr = M * DEG_TO_RAD;
    const Fr = F * DEG_TO_RAD; const Dr = D * DEG_TO_RAD; const Omr = Om * DEG_TO_RAD;
    const Msun = normalizeAngle(SUN_MEAN.M0 + SUN_MEAN.MDot * d) * DEG_TO_RAD;
    const lon = L
      + 6.289 * Math.sin(Mr)
      + 1.274 * Math.sin(2 * Dr - Mr)
      + 0.658 * Math.sin(2 * Dr)
      + 0.214 * Math.sin(2 * Mr)
      - 0.186 * Math.sin(Msun)
      - 0.114 * Math.sin(2 * Fr);
    const lat = 5.128 * Math.sin(Fr)
      + 0.281 * Math.sin(Mr + Fr)
      + 0.278 * Math.sin(Mr - Fr);
    const dist = 385001 - 20905 * Math.cos(Mr)
      - 3699 * Math.cos(2 * Dr - Mr) - 2956 * Math.cos(2 * Dr);
    const eclLon = normalizeAngle(lon);
    const distAU = dist / 149597870.7;
    const ternLon = stdDegToTernaryDeg(eclLon);
    const ternLat = stdDegToTernaryDeg(lat);
    return {
      ecliptic_longitude: eclLon, ecliptic_latitude: lat, distance_au: distAU,
      ternary_longitude: ternLon, ternary_latitude: ternLat,
      ternary_rad: ternLon / TERNARY_RADIAN_DEG,
      resonance: ternaryDegToResonance(ternLon), nearest_z28: nearestZ28(ternLon)
    };
  }

  const el = PLANET_ELEMENTS[planet];
  if (!el) throw new Error(`Unknown planet: ${planet}`);

  const a = el.a + el.aDot * T;
  const e = el.e + el.eDot * T;
  const I = (el.I + el.IDot * T) * DEG_TO_RAD;
  const L = normalizeAngle(el.L + el.LDot * T);
  const longPeri = normalizeAngle(el.longPeri + el.longPeriDot * T);
  const longNode = normalizeAngle(el.longNode + el.longNodeDot * T);

  const omega = (longPeri - longNode) * DEG_TO_RAD;
  const Omega = longNode * DEG_TO_RAD;
  const M = normalizeAngle(L - longPeri) * DEG_TO_RAD;

  const E = solveKepler(M, e);
  const v = trueAnomalyFromE(E, e);
  const r = a * (1 - e * Math.cos(E));

  const cosO = Math.cos(Omega); const sinO = Math.sin(Omega);
  const cosI = Math.cos(I); const sinI = Math.sin(I);
  const cosW = Math.cos(omega); const sinW = Math.sin(omega);
  const cosVW = Math.cos(v + omega); const sinVW = Math.sin(v + omega);

  const xEcl = r * (cosO * cosVW - sinO * sinVW * cosI);
  const yEcl = r * (sinO * cosVW + cosO * sinVW * cosI);
  const zEcl = r * sinVW * sinI;

  let eclLon: number, eclLat: number, dist: number;

  if (planet === "earth") {
    eclLon = normalizeAngle((Math.atan2(yEcl, xEcl) / DEG_TO_RAD) + 180);
    eclLat = -Math.atan2(zEcl, Math.sqrt(xEcl * xEcl + yEcl * yEcl)) / DEG_TO_RAD;
    dist = r;
  } else {
    const earthEl = PLANET_ELEMENTS.earth;
    const aE = earthEl.a + earthEl.aDot * T;
    const eE = earthEl.e + earthEl.eDot * T;
    const LE = normalizeAngle(earthEl.L + earthEl.LDot * T);
    const lpE = normalizeAngle(earthEl.longPeri + earthEl.longPeriDot * T);
    const ME = normalizeAngle(LE - lpE) * DEG_TO_RAD;
    const EE = solveKepler(ME, eE);
    const vE = trueAnomalyFromE(EE, eE);
    const rE = aE * (1 - eE * Math.cos(EE));
    const omegaE = lpE * DEG_TO_RAD;
    const xE = rE * Math.cos(vE + omegaE);
    const yE = rE * Math.sin(vE + omegaE);

    const dx = xEcl - xE;
    const dy = yEcl - yE;
    const dz = zEcl;

    eclLon = normalizeAngle(Math.atan2(dy, dx) / DEG_TO_RAD);
    eclLat = Math.atan2(dz, Math.sqrt(dx * dx + dy * dy)) / DEG_TO_RAD;
    dist = Math.sqrt(dx * dx + dy * dy + dz * dz);
  }

  const ternLon = stdDegToTernaryDeg(eclLon);
  const ternLat = stdDegToTernaryDeg(eclLat);
  return {
    ecliptic_longitude: eclLon, ecliptic_latitude: eclLat, distance_au: dist,
    ternary_longitude: ternLon, ternary_latitude: ternLat,
    ternary_rad: ternLon / TERNARY_RADIAN_DEG,
    resonance: ternaryDegToResonance(ternLon), nearest_z28: nearestZ28(ternLon)
  };
}

export function getEphemeris(planet: string, jd: number): EphemerisResult {
  if (jd < 2400000 || jd > 2500000) {
    throw new Error(`JD ${jd} out of valid range [2400000, 2500000]`);
  }
  return computePlanetPosition(planet.toLowerCase(), jd);
}

export function convertDegrees(
  type: "std_deg" | "std_rad" | "ternary_deg",
  value: number,
  returnResonance = false
) {
  let stdDeg: number;
  let ternDeg: number;

  if (type === "std_deg") {
    stdDeg = value;
    ternDeg = stdDegToTernaryDeg(value);
  } else if (type === "std_rad") {
    stdDeg = value / DEG_TO_RAD;
    ternDeg = stdDegToTernaryDeg(stdDeg);
  } else {
    ternDeg = value;
    stdDeg = ternaryDegToStdDeg(value);
  }

  const result: Record<string, number> = {
    std_deg: normalizeAngle(stdDeg),
    ternary_deg: ((ternDeg % FULL_CIRCLE_DEG) + FULL_CIRCLE_DEG) % FULL_CIRCLE_DEG,
    ternary_rad: ternDeg / TERNARY_RADIAN_DEG
  };

  if (returnResonance) {
    result.resonance = ternaryDegToResonance(ternDeg);
    result.nearest_z28 = nearestZ28(ternDeg);
  }

  return result;
}

export const SUPPORTED_PLANETS = Object.keys(PLANET_ELEMENTS).concat(["sun", "moon"]);
