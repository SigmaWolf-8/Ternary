/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * GF(3) Arithmetic Known-Answer Tests (KAT)
 * Exhaustive verification of the ternary field operations
 */

import { describe, it, expect } from "vitest";
import {
  ternaryAdd,
  ternaryMultiply,
  ternaryRotate,
  ternaryNot,
  ternaryXor,
  batchTernaryAdd,
  calculateInformationDensity,
} from "../server/salvi-core/ternary-operations";
import type { TritA } from "../server/salvi-core/ternary-types";
import {
  convertTrit,
  isValidTrit,
  getTritMeaning,
} from "../server/salvi-core/ternary-types";

const TRITS: TritA[] = [-1, 0, 1];

describe("GF(3) Addition (ternaryAdd)", () => {
  const ADDITION_TABLE: [TritA, TritA, TritA][] = [
    [-1, -1,  1],
    [-1,  0, -1],
    [-1,  1,  0],
    [ 0, -1, -1],
    [ 0,  0,  0],
    [ 0,  1,  1],
    [ 1, -1,  0],
    [ 1,  0,  1],
    [ 1,  1, -1],
  ];

  it.each(ADDITION_TABLE)("(%d) + (%d) = %d", (a, b, expected) => {
    const result = ternaryAdd(a, b);
    expect(result.result).toBe(expected);
    expect(result.constantTime).toBe(true);
  });

  it("is commutative: a+b = b+a for all trits", () => {
    for (const a of TRITS) {
      for (const b of TRITS) {
        expect(ternaryAdd(a, b).result).toBe(ternaryAdd(b, a).result);
      }
    }
  });

  it("has 0 as identity: a+0 = a", () => {
    for (const a of TRITS) {
      expect(ternaryAdd(a, 0).result).toBe(a);
    }
  });

  it("is associative: (a+b)+c = a+(b+c)", () => {
    for (const a of TRITS) {
      for (const b of TRITS) {
        for (const c of TRITS) {
          const lhs = ternaryAdd(ternaryAdd(a, b).result as TritA, c).result;
          const rhs = ternaryAdd(a, ternaryAdd(b, c).result as TritA).result;
          expect(lhs).toBe(rhs);
        }
      }
    }
  });
});

describe("GF(3) Multiplication (ternaryMultiply)", () => {
  const MULTIPLICATION_TABLE: [TritA, TritA, TritA][] = [
    [-1, -1,  1],
    [-1,  0,  0],
    [-1,  1, -1],
    [ 0, -1,  0],
    [ 0,  0,  0],
    [ 0,  1,  0],
    [ 1, -1, -1],
    [ 1,  0,  0],
    [ 1,  1,  1],
  ];

  it.each(MULTIPLICATION_TABLE)("(%d) * (%d) = %d", (a, b, expected) => {
    const result = ternaryMultiply(a, b);
    expect(result.result).toBe(expected);
    expect(result.constantTime).toBe(true);
  });

  it("is commutative: a*b = b*a", () => {
    for (const a of TRITS) {
      for (const b of TRITS) {
        expect(ternaryMultiply(a, b).result).toBe(ternaryMultiply(b, a).result);
      }
    }
  });

  it("has 1 as identity: a*1 = a", () => {
    for (const a of TRITS) {
      expect(ternaryMultiply(a, 1).result).toBe(a);
    }
  });

  it("0 is absorbing: a*0 = 0", () => {
    for (const a of TRITS) {
      expect(ternaryMultiply(a, 0).result).toBe(0);
    }
  });

  it("distributes over addition: a*(b+c) = a*b + a*c", () => {
    for (const a of TRITS) {
      for (const b of TRITS) {
        for (const c of TRITS) {
          const lhs = ternaryMultiply(a, ternaryAdd(b, c).result as TritA).result;
          const rhs = ternaryAdd(
            ternaryMultiply(a, b).result as TritA,
            ternaryMultiply(a, c).result as TritA
          ).result;
          expect(lhs).toBe(rhs);
        }
      }
    }
  });
});

describe("Ternary Rotation (ternaryRotate)", () => {
  it("rotate by 0 is identity", () => {
    for (const a of TRITS) {
      expect(ternaryRotate(a, 0).result).toBe(a);
    }
  });

  it("rotate by 3 is identity (period 3)", () => {
    for (const a of TRITS) {
      expect(ternaryRotate(a, 3).result).toBe(a);
    }
  });

  it("three rotations by 1 return to original", () => {
    for (const a of TRITS) {
      let val = a as TritA;
      val = ternaryRotate(val, 1).result as TritA;
      val = ternaryRotate(val, 1).result as TritA;
      val = ternaryRotate(val, 1).result as TritA;
      expect(val).toBe(a);
    }
  });
});

describe("Ternary XOR (Kleene min)", () => {
  it("min(-1,-1) = -1", () => expect(ternaryXor(-1, -1).result).toBe(-1));
  it("min(-1, 0) = -1", () => expect(ternaryXor(-1, 0).result).toBe(-1));
  it("min(-1, 1) = -1", () => expect(ternaryXor(-1, 1).result).toBe(-1));
  it("min( 0, 0) =  0", () => expect(ternaryXor(0, 0).result).toBe(0));
  it("min( 0, 1) =  0", () => expect(ternaryXor(0, 1).result).toBe(0));
  it("min( 1, 1) =  1", () => expect(ternaryXor(1, 1).result).toBe(1));

  it("is commutative", () => {
    for (const a of TRITS) {
      for (const b of TRITS) {
        expect(ternaryXor(a, b).result).toBe(ternaryXor(b, a).result);
      }
    }
  });
});

describe("Ternary NOT (negation)", () => {
  it("-1 -> 1", () => expect(ternaryNot(-1).result).toBe(1));
  it(" 0 -> 0", () => expect(ternaryNot(0).result).toBe(-0));
  it(" 1 -> -1", () => expect(ternaryNot(1).result).toBe(-1));

  it("double negation is identity", () => {
    for (const a of TRITS) {
      expect(ternaryNot(ternaryNot(a).result as TritA).result).toBe(a);
    }
  });
});

describe("Batch Operations", () => {
  it("batchTernaryAdd processes all pairs correctly", () => {
    const pairs = [
      { a: -1 as TritA, b: 1 as TritA },
      { a: 1 as TritA, b: 1 as TritA },
      { a: 0 as TritA, b: 0 as TritA },
    ];
    const results = batchTernaryAdd(pairs);
    expect(results).toHaveLength(3);
    expect(results[0].result).toBe(0);
    expect(results[1].result).toBe(-1);
    expect(results[2].result).toBe(0);
  });
});

describe("Trit Conversion (bijections)", () => {
  it("A(-1) -> B(0) and A(-1) -> C(1)", () => {
    const bResult = convertTrit(-1, "A", "B");
    expect(bResult.converted.value).toBe(0);
    const cResult = convertTrit(-1, "A", "C");
    expect(cResult.converted.value).toBe(1);
  });

  it("A(0) -> B(1)", () => {
    const bResult = convertTrit(0, "A", "B");
    expect(bResult.converted.value).toBe(1);
  });

  it("round-trip A -> B -> A preserves value", () => {
    for (const a of TRITS) {
      const b = convertTrit(a, "A", "B").converted.value;
      const roundTrip = convertTrit(b, "B", "A").converted.value;
      expect(roundTrip).toBe(a);
    }
  });

  it("round-trip A -> C -> A preserves value", () => {
    for (const a of TRITS) {
      const c = convertTrit(a, "A", "C").converted.value;
      const roundTrip = convertTrit(c, "C", "A").converted.value;
      expect(roundTrip).toBe(a);
    }
  });
});

describe("Trit Validation", () => {
  it("validates A range {-1, 0, 1}", () => {
    expect(isValidTrit(-1, "A")).toBe(true);
    expect(isValidTrit(0, "A")).toBe(true);
    expect(isValidTrit(1, "A")).toBe(true);
    expect(isValidTrit(2, "A")).toBe(false);
    expect(isValidTrit(-2, "A")).toBe(false);
  });

  it("validates B range {0, 1, 2}", () => {
    expect(isValidTrit(0, "B")).toBe(true);
    expect(isValidTrit(1, "B")).toBe(true);
    expect(isValidTrit(2, "B")).toBe(true);
    expect(isValidTrit(-1, "B")).toBe(false);
    expect(isValidTrit(3, "B")).toBe(false);
  });

  it("validates C range {1, 2, 3}", () => {
    expect(isValidTrit(1, "C")).toBe(true);
    expect(isValidTrit(2, "C")).toBe(true);
    expect(isValidTrit(3, "C")).toBe(true);
    expect(isValidTrit(0, "C")).toBe(false);
    expect(isValidTrit(4, "C")).toBe(false);
  });
});

describe("Trit Meaning", () => {
  it("maps A values to semantics", () => {
    expect(getTritMeaning(-1, "A")).toBe("False");
    expect(getTritMeaning(0, "A")).toBe("Neutral");
    expect(getTritMeaning(1, "A")).toBe("True");
  });

  it("maps B values to semantics", () => {
    expect(getTritMeaning(0, "B")).toBe("False");
    expect(getTritMeaning(1, "B")).toBe("Neutral");
    expect(getTritMeaning(2, "B")).toBe("True");
  });
});

describe("Information Density", () => {
  it("calculates density for valid trit counts", () => {
    const density = calculateInformationDensity(10);
    expect(density.trits).toBe(10);
    expect(density.bitsEquivalent).toBeGreaterThan(10);
    expect(typeof density.efficiencyGain).toBe("string");
  });
});
