/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved — Applied Physics Division
 *
 * TM-2026-017 v5.0 + TM-2026-026 v1.2
 * Extended Geometric Framework & UV Spectral Protocol Integration Tests
 *
 * Validates all new PLATFORM sub-blocks: REPUNITS, SQUARED_CIRCLE,
 * POLYGON_CENTRAL_ANGLES, NODE_CENSUS, SUPERHUB_ZONES, TORUS_KNOTS,
 * BRIESKORN_SPHERE, UV_SPECTRAL, ATMOSPHERIC_FILTER.
 */

import { describe, it, expect } from 'vitest';
import { PLATFORM } from '../shared/constants';

const gcd = (a: number, b: number): number => { while (b) { [a,b]=[b,a%b]; } return a; };
const isPrime = (n: number): boolean => {
  if (n < 2) return false;
  for (let d = 2; d * d <= n; d++) if (n % d === 0) return false;
  return true;
};

describe('TM-2026-017: Extended Geometric Framework', () => {

  describe('§2.1 Repunits (R_n = (3ⁿ − 1) / 2)', () => {
    const R = PLATFORM.REPUNITS;
    it('R3 = 13', () => expect(R.R3).toBe((27 - 1) / 2));
    it('R4 = 40', () => expect(R.R4).toBe((81 - 1) / 2));
    it('R5 = 121', () => expect(R.R5).toBe((243 - 1) / 2));
    it('R6 = 364', () => expect(R.R6).toBe((729 - 1) / 2));
    it('R6 = full circle', () => expect(R.R6).toBe(PLATFORM.PLENUM_SQUARE.FULL_CIRCLE));
    it('R4 = sum of roots', () => expect(R.R4).toBe(14 + 26));
    it('R5 = 11²', () => expect(R.R5).toBe(121));
  });

  describe('§3 Squared Circle', () => {
    const SC = PLATFORM.SQUARED_CIRCLE;
    it('area = 182 = π × radian', () => expect(SC.AREA).toBe(14 * 13));
    it('side² = area', () => expect(SC.SIDE_SQUARED).toBe(SC.AREA));
    it('factorization 14 × 13', () => expect(SC.FACTORIZATION_A * SC.FACTORIZATION_B).toBe(SC.AREA));
    it('quarter points at 0, 91, 182, 273', () => {
      expect(SC.QUARTER_POINTS).toEqual([0, 91, 182, 273]);
      expect(SC.QUARTER_POINTS[1]).toBe(91);
      expect(SC.QUARTER_POINTS[2]).toBe(182);
    });
  });

  describe('§4 Polygon Central Angles', () => {
    const P = PLATFORM.POLYGON_CENTRAL_ANGLES;
    it('square = 91°', () => expect(P.SQUARE.ANGLE).toBe(91));
    it('heptagon = 52°', () => expect(P.HEPTAGON.ANGLE).toBe(52));
    it('tridecagon = 28° = 2π', () => expect(P.TRIDECAGON.ANGLE).toBe(28));
    it('7 × 52 = 364', () => expect(7 * P.HEPTAGON.ANGLE).toBe(364));
    it('4 × 91 = 364', () => expect(4 * P.SQUARE.ANGLE).toBe(364));
    it('13 × 28 = 364', () => expect(13 * P.TRIDECAGON.ANGLE).toBe(364));
    it('hexagon: 182/3', () => expect(P.HEXAGON.ANGLE_NUM).toBe(182));
    it('triangle: 364/3', () => expect(P.TRIANGLE.ANGLE_NUM).toBe(364));
  });

  describe('§11.1 Node Census', () => {
    const NC = PLATFORM.NODE_CENSUS;
    it('total = rim + interior', () => expect(NC.RIM_VERTICES + NC.INTERIOR_INTERSECTIONS).toBe(NC.TOTAL));
    it('rim = 58', () => expect(NC.RIM_VERTICES).toBe(58));
    it('interior = 446', () => expect(NC.INTERIOR_INTERSECTIONS).toBe(446));
    it('total = 504', () => expect(NC.TOTAL).toBe(504));
  });

  describe('§11 Superhub Zones', () => {
    const SH = PLATFORM.SUPERHUB_ZONES;
    it('zones A & B have polygon 7', () => {
      expect(SH.A.POLYGONS).toContain(7);
      expect(SH.B.POLYGONS).toContain(7);
    });
    it('zones C & D have polygon 8', () => {
      expect(SH.C.POLYGONS).toContain(8);
      expect(SH.D.POLYGONS).toContain(8);
    });
    it('11, 12, 13 in all zones', () => {
      [SH.A, SH.B, SH.C, SH.D].forEach(z => {
        expect(z.POLYGONS).toContain(11);
        expect(z.POLYGONS).toContain(12);
        expect(z.POLYGONS).toContain(13);
      });
    });
    it('discriminant polygon = 12', () => expect(SH.DISCRIMINANT_POLYGON).toBe(12));
    it('A & B mirror (same distance)', () => expect(SH.A.DISTANCE).toBe(SH.B.DISTANCE));
    it('C & D mirror (same distance)', () => expect(SH.C.DISTANCE).toBe(SH.D.DISTANCE));
  });

  describe('§10 Torus Knots', () => {
    const TK = PLATFORM.TORUS_KNOTS;
    it('primary knot (7, 11)', () => {
      expect(TK.PRIMARY.P).toBe(7);
      expect(TK.PRIMARY.Q).toBe(11);
    });
    it('all pairs coprime', () => {
      expect(gcd(TK.PRIMARY.P, TK.PRIMARY.Q)).toBe(1);
      expect(gcd(TK.RED_RADIAN.P, TK.RED_RADIAN.Q)).toBe(1);
      expect(gcd(TK.GREEN_RADIAN.P, TK.GREEN_RADIAN.Q)).toBe(1);
      expect(gcd(TK.PI_RADIAN.P, TK.PI_RADIAN.Q)).toBe(1);
      expect(gcd(TK.FULL_CIRCLE_RADIAN.P, TK.FULL_CIRCLE_RADIAN.Q)).toBe(1);
    });
    it('Hamiltonian = 7 × 11 × 13 = 1001', () => {
      expect(TK.HAMILTONIAN_LENGTH).toBe(7 * 11 * 13);
    });
  });
});

describe('TM-2026-026: UV Spectral Protocol', () => {

  describe('§7.3 Brieskorn Sphere Σ(7, 11, 13)', () => {
    const BS = PLATFORM.BRIESKORN_SPHERE;
    it('exponents = (7, 11, 13)', () => expect(BS.EXPONENTS).toEqual([7, 11, 13]));
    it('pairwise products: 143, 91, 77', () => {
      expect(BS.ORBIFOLD.PAIRWISE_PRODUCTS).toEqual([143, 91, 77]);
      expect(BS.ORBIFOLD.PAIRWISE_PRODUCTS[0]).toBe(11 * 13);
      expect(BS.ORBIFOLD.PAIRWISE_PRODUCTS[1]).toBe(7 * 13);
      expect(BS.ORBIFOLD.PAIRWISE_PRODUCTS[2]).toBe(7 * 11);
    });
    it('pairwise sum = 311', () => {
      expect(143 + 91 + 77).toBe(BS.ORBIFOLD.PAIRWISE_SUM);
    });
    it('orbifold χ = −690/1001', () => {
      expect(BS.ORBIFOLD.NUMERATOR).toBe(-690);
      expect(BS.ORBIFOLD.DENOMINATOR).toBe(1001);
    });
    it('690 = 650 + 40', () => {
      const { ARC_ROOT: a, REPUNIT_R4: b } = BS.ORBIFOLD.DECOMPOSITION_690;
      expect(a + b).toBe(690);
      expect(a).toBe(650);
      expect(b).toBe(40);
    });

    describe('Symmetric Polynomials', () => {
      const SP = BS.SYMMETRIC_POLYNOMIALS;
      it('e₁ = 7+11+13 = 31', () => expect(SP.E1).toBe(7 + 11 + 13));
      it('e₂ = 77+91+143 = 311', () => expect(SP.E2).toBe(77 + 91 + 143));
      it('e₃ = 7×11×13 = 1001', () => expect(SP.E3).toBe(7 * 11 * 13));
      it('e₁ is prime', () => expect(isPrime(SP.E1)).toBe(true));
      it('e₂ is prime', () => expect(isPrime(SP.E2)).toBe(true));
      it('e₃ − e₂ = 690', () => expect(SP.E3 - SP.E2).toBe(690));
    });
  });

  describe('§2 Four System Wavelengths', () => {
    const UV = PLATFORM.UV_SPECTRAL;
    it('EUV = 91 = 7 × 13', () => expect(UV.PRIMARY_BANDS.EUV).toBe(7 * 13));
    it('UVC = 182 = 14 × 13', () => expect(UV.PRIMARY_BANDS.UVC).toBe(14 * 13));
    it('UVB = 286 = 22 × 13', () => expect(UV.PRIMARY_BANDS.UVB).toBe(22 * 13));
    it('UVA = 364 = 28 × 13', () => expect(UV.PRIMARY_BANDS.UVA).toBe(28 * 13));
    it('all multiples of 13', () => {
      const bands = UV.PRIMARY_BANDS;
      [bands.EUV, bands.UVC, bands.UVB, bands.UVA].forEach(v =>
        expect(v % 13).toBe(0)
      );
    });
    it('gcd = 13', () => expect(UV.GCD).toBe(13));
  });

  describe('§2.2 Exact Ratios', () => {
    const R = PLATFORM.UV_SPECTRAL.EXACT_RATIOS;
    const B = PLATFORM.UV_SPECTRAL.PRIMARY_BANDS;
    it('UVC/EUV = 2/1', () => expect(B.UVC * R.UVC_EUV[1]).toBe(B.EUV * R.UVC_EUV[0]));
    it('UVB/EUV = 22/7 (Archimedean π)', () => expect(B.UVB * R.UVB_EUV[1]).toBe(B.EUV * R.UVB_EUV[0]));
    it('UVA/EUV = 4/1', () => expect(B.UVA * R.UVA_EUV[1]).toBe(B.EUV * R.UVA_EUV[0]));
    it('UVB/UVC = 11/7', () => expect(B.UVB * R.UVB_UVC[1]).toBe(B.UVC * R.UVB_UVC[0]));
    it('UVA/UVB = 14/11', () => expect(B.UVA * R.UVA_UVB[1]).toBe(B.UVB * R.UVA_UVB[0]));
  });

  describe('§7.4 Secondary System Integers', () => {
    const SI = PLATFORM.UV_SPECTRAL.SECONDARY_INTEGERS;
    it('222 = 2 × center = 2 × 111', () => expect(SI.FAR_UVC_GERMICIDAL).toBe(2 * 111));
    it('308 = 4 × 77 = 4 × 7 × 11', () => expect(SI.EXCIMER_THERAPEUTIC).toBe(4 * 7 * 11));
    it('311 = e₂ (symmetric polynomial)', () => expect(SI.NARROWBAND_UVB).toBe(77 + 91 + 143));
    it('311 is prime', () => expect(isPrime(SI.NARROWBAND_UVB)).toBe(true));
  });

  describe('§8.3 Band Boundaries', () => {
    const BB = PLATFORM.UV_SPECTRAL.BAND_BOUNDARIES;
    const PB = PLATFORM.UV_SPECTRAL.PRIMARY_BANDS;
    it('EUV|UVC = floor((91+182)/2) = 136', () => expect(BB.EUV_UVC).toBe(Math.floor((PB.EUV + PB.UVC) / 2)));
    it('UVC|UVB = (182+286)/2 = 234', () => expect(BB.UVC_UVB).toBe((PB.UVC + PB.UVB) / 2));
    it('UVB|UVA = (286+364)/2 = 325', () => expect(BB.UVB_UVA).toBe((PB.UVB + PB.UVA) / 2));
    it('UV|Vis = 400', () => expect(BB.UV_VISIBLE).toBe(400));
    it('anchors within their bands', () => {
      expect(PB.EUV).toBeLessThanOrEqual(BB.EUV_UVC);
      expect(PB.UVC).toBeGreaterThan(BB.EUV_UVC);
      expect(PB.UVC).toBeLessThanOrEqual(BB.UVC_UVB);
      expect(PB.UVB).toBeGreaterThan(BB.UVC_UVB);
      expect(PB.UVB).toBeLessThanOrEqual(BB.UVB_UVA);
      expect(PB.UVA).toBeGreaterThan(BB.UVB_UVA);
      expect(PB.UVA).toBeLessThanOrEqual(BB.UV_VISIBLE);
    });
  });

  describe('§8.4 Vacuum Bias', () => {
    const UV = PLATFORM.UV_SPECTRAL;
    it('VACUUM_BIAS = 0.00194', () => expect(UV.VACUUM_BIAS).toBeCloseTo(0.00194, 5));
    it('UNIVERSAL_BIAS = 0.00139', () => expect(UV.UNIVERSAL_BIAS).toBeCloseTo(0.00139, 5));
    it('VACUUM_BIAS > UNIVERSAL_BIAS', () => expect(UV.VACUUM_BIAS).toBeGreaterThan(UV.UNIVERSAL_BIAS));
    it('91 × (1 + bias) ≈ 91.176', () => {
      const vacuum = 91 * (1 + UV.VACUUM_BIAS);
      expect(vacuum).toBeCloseTo(91.176, 2);
    });
  });

  describe('§2.3 Radian Multipliers', () => {
    const UV = PLATFORM.UV_SPECTRAL;
    it('multipliers = [7, 14, 22, 28]', () => expect(UV.RADIAN_MULTIPLIERS).toEqual([7, 14, 22, 28]));
    it('multipliers × 13 = bands', () => {
      const expected = [UV.PRIMARY_BANDS.EUV, UV.PRIMARY_BANDS.UVC, UV.PRIMARY_BANDS.UVB, UV.PRIMARY_BANDS.UVA];
      UV.RADIAN_MULTIPLIERS.forEach((m, i) => expect(m * 13).toBe(expected[i]));
    });
  });

  describe('§5 Atmospheric Filter', () => {
    const AF = PLATFORM.UV_SPECTRAL.ATMOSPHERIC_FILTER;
    it('EUV transmission = 0', () => expect(AF.EUV_91.TRANSMISSION).toBe(0));
    it('UVC transmission = 0', () => expect(AF.UVC_182.TRANSMISSION).toBe(0));
    it('UVB transmission ≈ 0.4%', () => expect(AF.UVB_286.TRANSMISSION).toBeCloseTo(0.004, 3));
    it('UVA transmission ≈ 80%', () => expect(AF.UVA_364.TRANSMISSION).toBeCloseTo(0.80, 2));
    it('monotonically increasing', () => {
      expect(AF.EUV_91.TRANSMISSION).toBeLessThanOrEqual(AF.UVC_182.TRANSMISSION);
      expect(AF.UVC_182.TRANSMISSION).toBeLessThan(AF.UVB_286.TRANSMISSION);
      expect(AF.UVB_286.TRANSMISSION).toBeLessThan(AF.UVA_364.TRANSMISSION);
    });
  });

  describe('Center Constant', () => {
    it('center = 111 = (182 + 40) / 2', () => expect(PLATFORM.UV_SPECTRAL.CENTER_CONSTANT).toBe((182 + 40) / 2));
    it('center matches PLENUM_SQUARE.CENTER', () => expect(PLATFORM.UV_SPECTRAL.CENTER_CONSTANT).toBe(PLATFORM.PLENUM_SQUARE.CENTER));
  });

  describe('Cross-monograph Consistency', () => {
    it('REPUNITS.R6 = FULL_CIRCLE = UVA', () => {
      expect(PLATFORM.REPUNITS.R6).toBe(PLATFORM.PLENUM_SQUARE.FULL_CIRCLE);
      expect(PLATFORM.REPUNITS.R6).toBe(PLATFORM.UV_SPECTRAL.PRIMARY_BANDS.UVA);
    });
    it('REPUNITS.R3 = radian = GCD', () => {
      expect(PLATFORM.REPUNITS.R3).toBe(PLATFORM.UV_SPECTRAL.GCD);
    });
    it('Brieskorn e₃ = Torus Hamiltonian', () => {
      expect(PLATFORM.BRIESKORN_SPHERE.SYMMETRIC_POLYNOMIALS.E3).toBe(PLATFORM.TORUS_KNOTS.HAMILTONIAN_LENGTH);
    });
    it('Coprime polygon ARC (143) = orbifold pairwise product [0]', () => {
      expect(PLATFORM.COPRIME_POLYGON_PAIR.ARC).toBe(PLATFORM.BRIESKORN_SPHERE.ORBIFOLD.PAIRWISE_PRODUCTS[0]);
    });
    it('Node census total = 504', () => {
      expect(PLATFORM.NODE_CENSUS.TOTAL).toBe(504);
    });
    it('unified equation: 182 + 650 = 832', () => {
      const area = PLATFORM.SQUARED_CIRCLE.AREA;
      const arc_root = PLATFORM.BRIESKORN_SPHERE.ORBIFOLD.DECOMPOSITION_690.ARC_ROOT;
      expect(area + arc_root).toBe(832);
    });
    it('unified equation: 182 × 650 = 118300', () => {
      const area = PLATFORM.SQUARED_CIRCLE.AREA;
      const arc_root = PLATFORM.BRIESKORN_SPHERE.ORBIFOLD.DECOMPOSITION_690.ARC_ROOT;
      expect(area * arc_root).toBe(118300);
    });
    it('BRIESKORN_SPHERE is homology sphere', () => {
      expect(PLATFORM.BRIESKORN_SPHERE.HOMOLOGY_SPHERE).toBe(true);
    });
    it('BRIESKORN_SPHERE has hyperbolic base', () => {
      expect(PLATFORM.BRIESKORN_SPHERE.HYPERBOLIC_BASE).toBe(true);
    });
    it('DECOMPOSITION_690: ARC_ROOT + REPUNIT_R4 = 690', () => {
      const d = PLATFORM.BRIESKORN_SPHERE.ORBIFOLD.DECOMPOSITION_690;
      expect(d.ARC_ROOT + d.REPUNIT_R4).toBe(690);
    });
    it('UV_SPECTRAL.REDUCED_MASS_COMPONENT defined', () => {
      expect(PLATFORM.UV_SPECTRAL.REDUCED_MASS_COMPONENT).toBeCloseTo(0.00055, 5);
    });
    it('ATMOSPHERIC_FILTER nested inside UV_SPECTRAL', () => {
      expect(PLATFORM.UV_SPECTRAL.ATMOSPHERIC_FILTER).toBeDefined();
      expect(PLATFORM.UV_SPECTRAL.ATMOSPHERIC_FILTER.EUV_91.TRANSMISSION).toBe(0);
    });
  });
});
