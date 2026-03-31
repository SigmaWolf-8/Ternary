/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved — Applied Physics Division
 *
 * TM-2026-025 v3 — Coprime Polygon Pair Structural Identity Tests
 * Complete suite: Generator Duality, CRT, Interleave, Unified Equation,
 * PlenumColor harmonics (all five), ARC_BLUE derivation, 2× scaling.
 */

import { describe, it, expect } from 'vitest';

// ── Constants ────────────────────────────────────────────────────────

const FULL_CIRCLE = 364;
const TWO_PI = 28;
const RADIAN = 13;
const SPONGE_W = 54;
const BRANCH_NUMBER = 8;
const T8 = 24;

// Coprime polygon pair
const ARC = 143;
const H = 11, T = 13;
const VERTICES = 23;
const PHI_143 = 120;
const GAP = 286;
const SQRT_D = 468;
const INTERLEAVE = [1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1];
const BEZOUT = [6, -5];
const CF = [2, 1, 1, 5];

// PlenumColor harmonics
const RED = 182, BLUE = 240, GREEN = 650;
const COPRIME = 286;

// ── Helpers ──────────────────────────────────────────────────────────

const gcd = (a: number, b: number): number => { while (b) { [a,b]=[b,a%b]; } return a; };
const phi = (n: number): number => {
  let r=n,p=2,t=n;
  while(p*p<=t){if(t%p===0){while(t%p===0)t/=p;r-=r/p;}p++;}
  if(t>1)r-=r/t; return r;
};
const walk = (s: number, m: number) => Array.from({length:m},(_,k)=>(s*k)%m);

// ── Tests ────────────────────────────────────────────────────────────

describe('TM-2026-025 v3: (11, 13) Coprime Polygon Pair', () => {

  describe('§1 Generator Duality', () => {
    it('143 = 11 × 13', () => expect(H*T).toBe(ARC));
    it('143 = 11 ternary radians', () => expect(ARC).toBe(11*RADIAN));
    it('11 generates Z₃₆₄', () => expect(new Set(walk(11,364)).size).toBe(364));
    it('13 does NOT generate Z₃₆₄', () => expect(new Set(walk(13,364)).size).toBe(28));
    it('gcd(11,364)=1', () => expect(gcd(11,364)).toBe(1));
    it('gcd(13,364)=13', () => expect(gcd(13,364)).toBe(13));
    it('both generate Z₂₈', () => { expect(new Set(walk(13,28)).size).toBe(28); expect(new Set(walk(11,28)).size).toBe(28); });
    it('different orderings', () => expect(walk(11,28)).not.toEqual(walk(13,28)));
  });

  describe('§2 Euclidean Ladder', () => {
    it('gcd(364,143) = 13', () => expect(gcd(364,143)).toBe(13));
    it('remainders [78,65,13] all ×13', () => {
      let a=364,b=143; const r: number[]=[]; while(b){const rem=a%b; if(rem)r.push(rem); a=b; b=rem;}
      expect(r).toEqual([78,65,13]); r.forEach(v=>expect(v%13).toBe(0));
    });
    it('CF=[2;1,1,5] → 28/11', () => {
      const c: number[]=[]; let a=364,b=143; while(b){c.push(Math.floor(a/b));[a,b]=[b,a%b];}
      expect(c).toEqual(CF);
      let [hp,hc]=[1,c[0]],[kp,kc]=[0,1];
      for(let i=1;i<c.length;i++){[hp,hc]=[hc,c[i]*hc+hp];[kp,kc]=[kc,c[i]*kc+kp];}
      expect([hc,kc]).toEqual([28,11]);
    });
  });

  describe('§3 CRT', () => {
    it('Z₃₆₄ → (3,3,0)', () => { expect(ARC%4).toBe(3); expect(ARC%7).toBe(3); expect(ARC%13).toBe(0); });
    it('Z₇₅₆ → (8,3)', () => { expect(ARC%27).toBe(BRANCH_NUMBER); expect(ARC%28).toBe(3); });
  });

  describe('§4 Combined Vertices', () => {
    it('23 = 11+13−1', () => expect(VERTICES).toBe(H+T-1));
    it('11⁻¹ mod 28 = 23', () => expect((11*23)%28).toBe(1));
    it('gcd(23,54)=1', () => expect(gcd(23,54)).toBe(1));
    it('13 self-inverse', () => expect((13*13)%28).toBe(1));
    it('Theorem 4.1: 143−φ(143)=23', () => expect(ARC - PHI_143).toBe(VERTICES));
    it('φ(143)=120', () => expect(phi(143)).toBe(PHI_143));
  });

  describe('§5 Interleave', () => {
    it('length=11, sum=12, palindromic', () => {
      expect(INTERLEAVE.length).toBe(11);
      expect(INTERLEAVE.reduce((a,b)=>a+b,0)).toBe(12);
      INTERLEAVE.forEach((v,i)=>expect(v).toBe(INTERLEAVE[10-i]));
    });
    it('single "2" at center', () => expect(INTERLEAVE[5]).toBe(2));
  });

  describe('§6 Unified Equation', () => {
    it('650−364=286=2×143', () => { expect(GREEN-FULL_CIRCLE).toBe(GAP); expect(GAP).toBe(2*ARC); });
    it('182−143=39=3rad', () => expect(RED-ARC).toBe(3*13));
    it('182+143=325=25rad', () => expect(RED+ARC).toBe(25*13));
    it('650=364+2×143', () => expect(FULL_CIRCLE+2*ARC).toBe(GREEN));
    it('Vieta: sum=832, product=118300', () => { expect(RED+GREEN).toBe(832); expect(RED*GREEN).toBe(118300); });
    it('φ(364)−φ(143)=24=T₈', () => expect(phi(364)-phi(143)).toBe(T8));
  });

  describe('§6.1 PlenumColor: ARC_COPRIME & √Δ', () => {
    it('ARC_GREEN = FC + ARC_COPRIME', () => expect(FULL_CIRCLE+COPRIME).toBe(GREEN));
    it('ARC_RED + ARC_COPRIME = √Δ', () => expect(RED+COPRIME).toBe(SQRT_D));
    it('ARC_COPRIME = 2×143', () => expect(COPRIME).toBe(2*ARC));
    it('√Δ = GREEN−RED', () => expect(GREEN-RED).toBe(SQRT_D));
    it('468²=219024=Δ', () => { expect(SQRT_D*SQRT_D).toBe(219024); expect(832*832-4*118300).toBe(219024); });
    it('roots from formula', () => { expect((832-SQRT_D)/2).toBe(RED); expect((832+SQRT_D)/2).toBe(GREEN); });
    it('√Δ = 36×13 = 36 radians', () => expect(SQRT_D).toBe(36*13));
    it('FC = 2×RED', () => expect(FULL_CIRCLE).toBe(2*RED));
  });

  describe('§6.1 PlenumColor: ARC_BLUE Derivation', () => {
    it('ARC_BLUE = 2×φ(143) = 240', () => expect(2*PHI_143).toBe(BLUE));
    it('ARC_BLUE = 3⁵−3 = 240', () => expect(243-3).toBe(BLUE));
    it('ARC_BLUE = 3(3⁴−1)', () => expect(3*(81-1)).toBe(BLUE));
    it('ARC_COPRIME − ARC_BLUE = 2×23', () => expect(COPRIME-BLUE).toBe(2*VERTICES));
    it('CRT Z₇₅₆: (24,16) = (T₈, 2⁴)', () => { expect(BLUE%27).toBe(24); expect(BLUE%28).toBe(16); });
    it('CRT Z₁₃: 6 = φ(7) = Bézout[0]', () => { expect(BLUE%13).toBe(6); expect(phi(7)).toBe(6); });
  });

  describe('§6.2 The 2× Scaling Pattern', () => {
    it('2×143=286 (ARC_COPRIME)', () => expect(2*ARC).toBe(COPRIME));
    it('2×φ(143)=240 (ARC_BLUE)', () => expect(2*PHI_143).toBe(BLUE));
    it('2×23=46 (gap)', () => expect(2*VERTICES).toBe(COPRIME-BLUE));
  });

  describe('§6.3 System Closure', () => {
    it('three routes to GREEN', () => {
      expect(FULL_CIRCLE+COPRIME).toBe(GREEN);
      expect(RED+SQRT_D).toBe(GREEN);
      expect((832+SQRT_D)/2).toBe(GREEN);
    });
    it('BLUE = COPRIME − 2×VERTICES', () => expect(COPRIME-2*VERTICES).toBe(BLUE));
  });

  describe('§7 DDT', () => {
    it('364+312+26=702=26×27', () => { expect(364+312+26).toBe(702); expect(26*27).toBe(702); });
  });

  describe('§8 Bézout', () => {
    it('11×6+13×(−5)=1', () => expect(H*BEZOUT[0]+T*BEZOUT[1]).toBe(1));
    it('6=φ(7)', () => expect(phi(7)).toBe(BEZOUT[0]));
  });

  describe('Sponge Strides', () => {
    it('{11,13,23} coprime to 54', () => [11,13,23].forEach(s=>expect(gcd(s,SPONGE_W)).toBe(1)));
    it('7×11×13=1001', () => expect(7*11*13).toBe(1001));
  });
});
