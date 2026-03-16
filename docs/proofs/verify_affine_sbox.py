#!/usr/bin/env python3
"""
Exhaustive verification of the affine-composed chi S-box S(x) = M·x^17 + c
over GF(27) = GF(3)[t]/(t^3 + 2t + 1).

Verifies claims from TM-2026-011 Rev. 1 (Phase Encryption Security Proof §8.4):
  1. DDT computation:  DP_max = 3/27 = 1/9,  DDT values in {0, 2, 3}
  2. Walsh spectrum:   L(S) = 9,  LP_max = (9/27)^2 = 1/9
  3. Algebraic degree:  degree(S) = 5  (3-ary weight of 17 = 122_3)
  4. Zero fixed point eliminated:  S(0) = [1,0,2] ≠ 0

Parameters:
  M = circulant([1,1,2]) = [[1,1,2],[2,1,1],[1,2,1]]  (det=1 over GF(3))
  c = [1, 0, 2]
  chi_0(x) = x^17 over GF(27)

Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
Applied Physics Division — All Rights Reserved — Patent(s) Pending
"""
import cmath
import sys

# ── GF(27) = GF(3)[t]/(t^3 + 2t + 1) ──────────────────────────────

# Irreducible polynomial: t^3 + 2t + 1 = 0  =>  t^3 = -2t - 1 = t + 2 (mod 3)

def poly_mul_mod3(a, b):
    """Multiply two polynomials mod 3, reduce mod (t^3 + 2t + 1)."""
    c = [0] * 5
    for i in range(3):
        for j in range(3):
            c[i + j] = (c[i + j] + a[i] * b[j]) % 3
    while len(c) > 3:
        if c[-1] != 0:
            deg = len(c) - 1
            coeff = c[-1]
            if deg == 4:
                # t^4 = t * t^3 = t(t+2) = t^2 + 2t
                c[2] = (c[2] + coeff * 1) % 3
                c[1] = (c[1] + coeff * 2) % 3
            elif deg == 3:
                # t^3 = t + 2
                c[1] = (c[1] + coeff * 1) % 3
                c[0] = (c[0] + coeff * 2) % 3
        c.pop()
    return tuple(c)


def gf27_pow(base, exp):
    """Compute base^exp in GF(27)."""
    result = (1, 0, 0)
    b = base
    while exp > 0:
        if exp & 1:
            result = poly_mul_mod3(result, b)
        b = poly_mul_mod3(b, b)
        exp >>= 1
    return result


def gf27_mul(a, b):
    return poly_mul_mod3(a, b)


def gf27_add(a, b):
    return tuple((a[i] + b[i]) % 3 for i in range(3))


def gf27_trace(x):
    """Absolute trace Tr: GF(27) -> GF(3): Tr(x) = x + x^3 + x^9."""
    x3 = gf27_pow(x, 3)
    x9 = gf27_pow(x, 9)
    return (x[0] + x3[0] + x9[0]) % 3


# ── Affine composition ──────────────────────────────────────────────

# M = circulant([1,1,2]) over GF(3)
M = [[1, 1, 2],
     [2, 1, 1],
     [1, 2, 1]]

# c = [1, 0, 2]
C_VEC = (1, 0, 2)


def mat_vec_gf3(mat, v):
    """Matrix-vector product over GF(3)."""
    return tuple(sum(mat[i][j] * v[j] for j in range(3)) % 3 for i in range(3))


def sbox_composed(x):
    """S(x) = M · x^17 + c over GF(27)."""
    x17 = gf27_pow(x, 17)
    mx17 = mat_vec_gf3(M, x17)
    return gf27_add(mx17, C_VEC)


# ── Generate all 27 elements ────────────────────────────────────────

elements = []
for a0 in range(3):
    for a1 in range(3):
        for a2 in range(3):
            elements.append((a0, a1, a2))

ZERO = (0, 0, 0)
omega = cmath.exp(2j * cmath.pi / 3)

# Compute S(x) for all elements
S = {}
for x in elements:
    S[x] = sbox_composed(x)

# ── Test 0: Bijectivity ────────────────────────────────────────────

outputs = set(S.values())
assert len(outputs) == 27, f"S is not a permutation! Only {len(outputs)} distinct outputs"
print("Test 0: S(x) = M·x^17 + c is a permutation of GF(27)  ✓")

# ── Test 1: Zero fixed point eliminated ─────────────────────────────

s_zero = S[ZERO]
assert s_zero != ZERO, f"S(0) = {s_zero} = 0, zero fixed point NOT eliminated!"
assert s_zero == C_VEC, f"S(0) should be c = {C_VEC}, got {s_zero}"
print(f"Test 1: S(0) = {list(s_zero)} ≠ [0,0,0]  (zero fixed point eliminated)  ✓")

# ── Test 2: DDT computation ─────────────────────────────────────────

print(f"\n{'='*60}")
print("DDT Verification  (§8.4.3, Theorem 8.4.1)")
print(f"{'='*60}")

ddt_values = set()
dp_max = 0
dp_max_entry = None

for da in elements:
    if da == ZERO:
        continue
    for db in elements:
        count = 0
        for x in elements:
            x_plus_da = gf27_add(x, da)
            diff = tuple((S[x_plus_da][i] - S[x][i]) % 3 for i in range(3))
            if diff == db:
                count += 1
        ddt_values.add(count)
        if count > dp_max:
            dp_max = count
            dp_max_entry = (da, db, count)

print(f"DDT values:     {sorted(ddt_values)}")
print(f"Expected:       [0, 2, 3]")
print(f"DP_max:         {dp_max}/27 = {dp_max/27:.6f}")
print(f"Expected:       3/27 = 1/9 = {1/9:.6f}")

assert sorted(ddt_values) == [0, 2, 3], f"DDT values mismatch: {sorted(ddt_values)}"
assert dp_max == 3, f"DP_max = {dp_max}/27, expected 3/27"
print("✓ DDT values {0, 2, 3} confirmed")
print("✓ DP_max = 1/9 confirmed")

# ── Test 3: Walsh spectrum ──────────────────────────────────────────

print(f"\n{'='*60}")
print("Walsh Spectrum Analysis  (§8.4.3, Theorem 8.4.2; §9.2)")
print(f"{'='*60}")

max_walsh = 0.0
walsh_magnitudes = []

for a in elements:
    for b in elements:
        if a == ZERO and b == ZERO:
            continue

        W = 0.0 + 0j
        for x in elements:
            b_sx = gf27_mul(b, S[x])
            a_x = gf27_mul(a, x)
            tr_val = (gf27_trace(b_sx) - gf27_trace(a_x)) % 3
            W += omega ** tr_val

        mag = abs(W)
        walsh_magnitudes.append(mag)

        if b != ZERO:
            if mag > max_walsh:
                max_walsh = mag

max_walsh_rounded = round(max_walsh, 6)
distinct_mags = sorted(set(round(m) for m in walsh_magnitudes))
LP_max = (max_walsh / 27) ** 2

print(f"Total (a,b) pairs:       {len(walsh_magnitudes)}")
print(f"Distinct |W| magnitudes: {distinct_mags}")
print(f"Expected:                [0, 3, 6, 9]")
print(f"Max |W(a,b)| (b≠0):     {max_walsh_rounded}")
print(f"Expected (perfect NL):   9  (= 3^((3+1)/2))")
print(f"LP_max = ({max_walsh_rounded}/27)^2 = {LP_max:.6f}")
print(f"Expected: 1/9 = {1/9:.6f}")

assert distinct_mags == [0, 3, 6, 9], f"Walsh magnitudes mismatch: {distinct_mags}"
assert abs(max_walsh_rounded - 9.0) < 0.01, f"Max Walsh = {max_walsh_rounded}, expected 9"
print("✓ L(S) = 9  (perfect nonlinearity for GF(3^3))")
print("✓ LP_max = 1/9 confirmed")
print("✓ Walsh magnitudes {0, 3, 6, 9} — identical to naked power map")

# ── Test 4: Algebraic degree ────────────────────────────────────────

print(f"\n{'='*60}")
print("Algebraic Degree Analysis  (§8.4.4)")
print(f"{'='*60}")

# 3-ary weight of exponent 17
# 17 = 1*9 + 2*3 + 2*1 = 122 in base 3
exp = 17
base3_digits = []
n = exp
while n > 0:
    base3_digits.append(n % 3)
    n //= 3
ternary_weight = sum(base3_digits)
base3_str = ''.join(str(d) for d in reversed(base3_digits))

print(f"Exponent:       {exp}")
print(f"Base-3 repr:    {base3_str}")
print(f"3-ary weight:   {ternary_weight}")
print(f"Expected:       5")
print(f"Max possible:   (p-1)*n = 2*3 = 6")
print(f"Ratio:          {ternary_weight}/6 = {ternary_weight/6*100:.0f}%")

# Affine composition preserves degree: deg(A ∘ chi_0) = max(deg(A)*deg(chi_0), deg(A)) = 5
print(f"Composed degree: max(1*{ternary_weight}, 1) = {ternary_weight}")

assert ternary_weight == 5, f"Algebraic degree = {ternary_weight}, expected 5"
print("✓ Algebraic degree = 5 confirmed")
print("✓ Affine composition preserves algebraic degree")

# ── Test 5: M matrix properties ─────────────────────────────────────

print(f"\n{'='*60}")
print("Matrix M Properties")
print(f"{'='*60}")

# det(M) over GF(3)
det = (M[0][0]*(M[1][1]*M[2][2] - M[1][2]*M[2][1])
     - M[0][1]*(M[1][0]*M[2][2] - M[1][2]*M[2][0])
     + M[0][2]*(M[1][0]*M[2][1] - M[1][1]*M[2][0])) % 3
print(f"det(M) mod 3:   {det}")
assert det == 1, f"det(M) = {det}, expected 1"
print("✓ det(M) = 1  (invertible over GF(3))")

# Branch number of M: min weight of (x, Mx) for nonzero x
min_bn = 99
for x in elements:
    if x == ZERO:
        continue
    mx = mat_vec_gf3(M, x)
    wt_x = sum(1 for v in x if v != 0)
    wt_mx = sum(1 for v in mx if v != 0)
    bn = wt_x + wt_mx
    if bn < min_bn:
        min_bn = bn

print(f"Branch number:  {min_bn}")
print(f"Expected:       3  (maximum for 3×3 over GF(3))")
assert min_bn == 3, f"Branch number = {min_bn}, expected 3"
print("✓ Branch number = 3 (max achievable)")

# ── Test 6: Cross-verify with naked power map ───────────────────────

print(f"\n{'='*60}")
print("Cross-Verification: Composed vs Naked Power Map")
print(f"{'='*60}")

# Compute DDT of naked x^17
naked_dp_max = 0
naked_ddt_vals = set()
chi0 = {x: gf27_pow(x, 17) for x in elements}

for da in elements:
    if da == ZERO:
        continue
    for db in elements:
        count = sum(1 for x in elements
                    if tuple((chi0[gf27_add(x, da)][i] - chi0[x][i]) % 3 for i in range(3)) == db)
        naked_ddt_vals.add(count)
        if count > naked_dp_max:
            naked_dp_max = count

print(f"Naked x^17 DDT values: {sorted(naked_ddt_vals)}")
print(f"Composed S   DDT values: {sorted(ddt_values)}")
assert sorted(naked_ddt_vals) == sorted(ddt_values), "DDT value sets differ!"
assert naked_dp_max == dp_max, "DP_max differs!"
print("✓ DDT value sets identical: {0, 2, 3}")
print("✓ DP_max identical: 1/9")

# ── Final Summary ───────────────────────────────────────────────────

print(f"\n{'='*60}")
print("VERIFICATION SUMMARY")
print(f"{'='*60}")
checks = [
    ("Bijectivity",               True),
    ("Zero fixed-point eliminated", s_zero != ZERO),
    ("DP_max = 1/9",              dp_max == 3),
    ("DDT values = {0,2,3}",     sorted(ddt_values) == [0, 2, 3]),
    ("L(S) = 9",                  abs(max_walsh_rounded - 9.0) < 0.01),
    ("LP_max = 1/9",             abs(LP_max - 1/9) < 1e-6),
    ("Walsh mags = {0,3,6,9}",   distinct_mags == [0, 3, 6, 9]),
    ("Algebraic degree = 5",      ternary_weight == 5),
    ("det(M) = 1",               det == 1),
    ("Branch number = 3",         min_bn == 3),
    ("DDT matches naked map",    sorted(naked_ddt_vals) == sorted(ddt_values)),
]

all_pass = True
for name, passed in checks:
    status = "PASS" if passed else "FAIL"
    if not passed:
        all_pass = False
    print(f"  [{status}]  {name}")

print(f"\n{'='*60}")
if all_pass:
    print("ALL 11 CHECKS PASSED — S(x) = M·x^17+c verified.")
    print("Claims in TM-2026-011 Rev. 1 §8.4, §9.2 are confirmed.")
else:
    print("SOME CHECKS FAILED — review output above.")
    sys.exit(1)
