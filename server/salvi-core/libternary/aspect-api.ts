/**
 * Ternary Aspect API
 * Bijective ternary calculations for aspect angles in the 364° Prime Circle system
 * 
 * Part of the Salvi Framework - Bijective Ternary Logic
 * https://github.com/SigmaWolf-8/Ternary
 */
// ============================================
// TYPES
// ============================================
export type TernaryA = -1 | 0 | 1;  // Computational: internal processing
export type TernaryB = 0 | 1 | 2;   // Network: data transmission
export type TernaryC = 1 | 2 | 3;   // Human: display/logging
export interface TernaryTriple {
  a: TernaryA;
  b: TernaryB;
  c: TernaryC;
}
export interface AspectTernarySignature {
  phaseCount: number;
  ternary: string;
  bijectiveTernary: string;
  resonance: TernaryTriple;
}
export interface AspectResonance {
  gf3Value: TernaryB;
  resonanceType: string;
  cosmicFrequency: number;
}
export interface TernaryHarmony {
  difference: number;
  phaseOffset: number;
  harmonicResonance: 'aligned' | 'dynamic' | 'transcendent';
  ternarySignature: string;
}
// ============================================
// CONSTANTS
// ============================================
export const PRIME_CIRCLE = 364;
export const PHASE_SIZE = 28;
export const NUM_PHASES = 13;
export const RESONANCE_TYPES: Record<TernaryB, string> = {
  0: 'Completion (Return to Source)',
  1: 'Initiation (New Beginning)',
  2: 'Manifestation (Creative Expression)'
};
// ============================================
// BIJECTIVE MAPPING FUNCTIONS
// ============================================
export function ternaryAtoB(a: TernaryA): TernaryB {
  return (a + 1) as TernaryB;
}
export function ternaryAtoC(a: TernaryA): TernaryC {
  return (a + 2) as TernaryC;
}
export function ternaryBtoC(b: TernaryB): TernaryC {
  return (b + 1) as TernaryC;
}
export function ternaryBtoA(b: TernaryB): TernaryA {
  return (b - 1) as TernaryA;
}
export function ternaryCtoA(c: TernaryC): TernaryA {
  return (c - 2) as TernaryA;
}
export function ternaryCtoB(c: TernaryC): TernaryB {
  return (c - 1) as TernaryB;
}
// ============================================
// BIJECTIVE TERNARY CONVERSION
// ============================================
/**
 * Convert decimal to bijective ternary array (base-3 with digits 1,2,3)
 * Unlike standard ternary (0,1,2), bijective uses (1,2,3) with no zero
 */
export function toBijectiveTernary(n: number): TernaryC[] {
  if (n <= 0) return [1];
  const digits: TernaryC[] = [];
  let num = n;
  while (num > 0) {
    let remainder = num % 3;
    if (remainder === 0) {
      remainder = 3;
      num = Math.floor(num / 3) - 1;
    } else {
      num = Math.floor(num / 3);
    }
    digits.unshift(remainder as TernaryC);
  }
  return digits;
}
/**
 * Convert bijective ternary array back to decimal
 */
export function fromBijectiveTernary(digits: TernaryC[]): number {
  let result = 0;
  for (let i = 0; i < digits.length; i++) {
    result = result * 3 + digits[i];
  }
  return result;
}
/**
 * Display bijective ternary as string with subscript notation
 * Example: 4 → "11₃ᵇ"
 */
export function bijectiveTernaryString(n: number): string {
  const digits = toBijectiveTernary(n);
  return digits.join('') + '₃ᵇ';
}
/**
 * Convert to standard ternary with subscript
 * Example: 4 → "11₃"
 */
export function toTernaryString(n: number): string {
  if (n === 0) return '0₃';
  const digits: string[] = [];
  let num = n;
  while (num > 0) {
    digits.unshift(String(num % 3));
    num = Math.floor(num / 3);
  }
  return digits.join('') + '₃';
}
// ============================================
// GF(3) FIELD OPERATIONS
// ============================================
export const GF3 = {
  /** Addition in GF(3): (a + b) mod 3 */
  add: (a: TernaryB, b: TernaryB): TernaryB => ((a + b) % 3) as TernaryB,
  
  /** Subtraction in GF(3): (a - b) mod 3 */
  subtract: (a: TernaryB, b: TernaryB): TernaryB => (((a - b) % 3 + 3) % 3) as TernaryB,
  
  /** Multiplication in GF(3): (a × b) mod 3 */
  multiply: (a: TernaryB, b: TernaryB): TernaryB => ((a * b) % 3) as TernaryB,
  
  /** Negation in GF(3): (3 - a) mod 3 */
  negate: (a: TernaryB): TernaryB => ((3 - a) % 3) as TernaryB
};
// ============================================
// ASPECT ANGLE ANALYSIS
// ============================================
/**
 * Get complete ternary signature for an aspect angle
 * @param angle - Aspect angle in degrees (364° system)
 */
export function getAspectTernarySignature(angle: number): AspectTernarySignature {
  const phaseCount = Math.round(angle / PHASE_SIZE);
  const mod3 = phaseCount % 3;
  
  return {
    phaseCount,
    ternary: toTernaryString(phaseCount),
    bijectiveTernary: bijectiveTernaryString(phaseCount),
    resonance: {
      a: (mod3 === 0 ? 0 : mod3 === 1 ? 1 : -1) as TernaryA,
      b: mod3 as TernaryB,
      c: (mod3 === 0 ? 3 : mod3) as TernaryC
    }
  };
}
/**
 * Calculate aspect resonance using GF(3) classification
 * @param aspectAngle - Aspect angle in degrees
 */
export function calculateAspectResonance(aspectAngle: number): AspectResonance {
  const phaseCount = Math.round(aspectAngle / PHASE_SIZE);
  const gf3Value = (phaseCount % 3) as TernaryB;
  
  return {
    gf3Value,
    resonanceType: RESONANCE_TYPES[gf3Value],
    cosmicFrequency: phaseCount > 0 ? PRIME_CIRCLE / phaseCount : PRIME_CIRCLE
  };
}
/**
 * Calculate ternary harmony between two angular positions
 * @param angle1 - First position in degrees (360° tropical)
 * @param angle2 - Second position in degrees (360° tropical)
 */
export function calculateTernaryHarmony(angle1: number, angle2: number): TernaryHarmony {
  // Convert 360° to 364° (Prime system)
  const prime1 = (angle1 * PRIME_CIRCLE) / 360;
  const prime2 = (angle2 * PRIME_CIRCLE) / 360;
  
  let diff = Math.abs(prime1 - prime2) % PRIME_CIRCLE;
  if (diff > PRIME_CIRCLE / 2) diff = PRIME_CIRCLE - diff;
  
  const phaseOffset = Math.round(diff / PHASE_SIZE);
  const mod3 = phaseOffset % 3;
  
  const harmonicResonance = mod3 === 0 ? 'transcendent' : 
                            mod3 === 1 ? 'aligned' : 'dynamic';
  
  return {
    difference: diff,
    phaseOffset,
    harmonicResonance,
    ternarySignature: bijectiveTernaryString(phaseOffset)
  };
}
// ============================================
// ASPECT DEFINITIONS (364° SYSTEM)
// ============================================
export const ASPECTS_364 = {
  conjunction:  { angle: 0,       phases: 0, gf3: 0 as const, resonance: 'Completion' },
  tredecile:    { angle: 28,      phases: 1, gf3: 1 as const, resonance: 'Initiation' },
  novile:       { angle: 40.444,  phases: 1, gf3: 1 as const, resonance: 'Initiation' },
  septile:      { angle: 52,      phases: 2, gf3: 2 as const, resonance: 'Manifestation' },
  sextile:      { angle: 60.667,  phases: 2, gf3: 2 as const, resonance: 'Manifestation' },
  quintile:     { angle: 72.8,    phases: 3, gf3: 0 as const, resonance: 'Completion' },
  square:       { angle: 91,      phases: 3, gf3: 0 as const, resonance: 'Completion' },
  trine:        { angle: 121.333, phases: 4, gf3: 1 as const, resonance: 'Initiation' },
  quincunx:     { angle: 151.667, phases: 5, gf3: 2 as const, resonance: 'Manifestation' },
  opposition:   { angle: 182,     phases: 6, gf3: 0 as const, resonance: 'Completion' }
} as const;
export type AspectName = keyof typeof ASPECTS_364;
