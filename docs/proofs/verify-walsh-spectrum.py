#!/usr/bin/env python3
"""
Exhaustive Walsh spectrum computation for chi(x) = x^17 over GF(27) = GF(3)[t]/(t^3 + 2t + 1).

Computes the full Walsh transform matrix and determines:
- Maximum linearity L(chi)
- Maximum linear probability LP_max
- Whether chi achieves perfect nonlinearity for GF(3^3)
"""
import cmath

# GF(27) = GF(3)[t]/(t^3 + 2t + 1)
# Elements represented as (a0, a1, a2) where element = a0 + a1*t + a2*t^2
# Coefficients in {0, 1, 2} (mod 3)

MOD_POLY = (2, 0, 2, 1)  # t^3 + 2t + 1 -> coefficients [const, t, t^2, t^3] = [1, 2, 0, 1]
# Actually: t^3 + 2t + 1 means t^3 = -2t - 1 = t - 1 = t + 2 (mod 3)
# So t^3 ≡ t + 2 (mod 3)

def poly_mul_mod3(a, b):
    """Multiply two polynomials mod 3, reduce mod (t^3 + 2t + 1)"""
    # a, b are tuples of length 3: (a0, a1, a2)
    # product before reduction: degree up to 4
    c = [0] * 5
    for i in range(3):
        for j in range(3):
            c[i+j] = (c[i+j] + a[i] * b[j]) % 3
    # Reduce: t^3 = t + 2 (mod 3) since t^3 + 2t + 1 = 0 => t^3 = -2t - 1 = t + 2
    # t^4 = t * t^3 = t(t+2) = t^2 + 2t
    while len(c) > 3:
        if c[-1] != 0:
            deg = len(c) - 1
            coeff = c[-1]
            if deg == 4:
                # t^4 = t^2 + 2t
                c[2] = (c[2] + coeff * 1) % 3
                c[1] = (c[1] + coeff * 2) % 3
            elif deg == 3:
                # t^3 = t + 2
                c[1] = (c[1] + coeff * 1) % 3
                c[0] = (c[0] + coeff * 2) % 3
        c.pop()
    return tuple(c)

def gf27_pow(base, exp):
    """Compute base^exp in GF(27)"""
    result = (1, 0, 0)  # 1
    b = base
    while exp > 0:
        if exp & 1:
            result = poly_mul_mod3(result, b)
        b = poly_mul_mod3(b, b)
        exp >>= 1
    return result

def gf27_trace(x):
    """Absolute trace Tr: GF(27) -> GF(3): Tr(x) = x + x^3 + x^9"""
    x3 = gf27_pow(x, 3)
    x9 = gf27_pow(x, 9)
    return (x[0] + x3[0] + x9[0]) % 3

def gf27_mul(a, b):
    return poly_mul_mod3(a, b)

# Generate all 27 elements
elements = []
for a0 in range(3):
    for a1 in range(3):
        for a2 in range(3):
            elements.append((a0, a1, a2))

ZERO = (0, 0, 0)
omega = cmath.exp(2j * cmath.pi / 3)  # primitive 3rd root of unity

# Compute chi(x) = x^17 for all elements
chi = {}
for x in elements:
    chi[x] = gf27_pow(x, 17)

# Verify chi is a permutation
chi_outputs = set(chi.values())
assert len(chi_outputs) == 27, f"chi is not a permutation! Only {len(chi_outputs)} distinct outputs"
print("chi(x) = x^17 is a permutation of GF(27) ✓")

# Compute full Walsh transform
# W(a, b) = sum_{x in GF(27)} omega^{Tr(b*chi(x)) - Tr(a*x)}
# = sum_{x in GF(27)} omega^{Tr(b*x^17 - a*x)}

max_walsh = 0.0
walsh_magnitudes = []

for a in elements:
    for b in elements:
        if a == ZERO and b == ZERO:
            continue  # Skip trivial case
        
        W = 0.0 + 0j
        for x in elements:
            bx17 = gf27_mul(b, chi[x])
            ax = gf27_mul(a, x)
            # Tr(b*x^17) - Tr(a*x) mod 3
            tr_val = (gf27_trace(bx17) - gf27_trace(ax)) % 3
            W += omega ** tr_val
        
        mag = abs(W)
        walsh_magnitudes.append(mag)
        
        if b != ZERO:  # Only count nonzero output masks for linearity
            if mag > max_walsh:
                max_walsh = mag

# Round to handle floating point
max_walsh_rounded = round(max_walsh, 6)

print(f"\nWalsh Spectrum Analysis for chi(x) = x^17 over GF(27)")
print(f"=" * 60)
print(f"Total (a,b) pairs evaluated: {len(walsh_magnitudes)}")
print(f"Maximum |W(a,b)| (b ≠ 0): {max_walsh_rounded}")
print(f"Expected for perfect nonlinearity: {3**((3+1)/2)} = {3**2} = 9")

# Check all distinct magnitudes
distinct_mags = sorted(set(round(m, 4) for m in walsh_magnitudes))
print(f"Distinct Walsh magnitudes: {distinct_mags}")

# Compute LP_max
LP_max = (max_walsh / 27) ** 2
print(f"\nLP_max = (L(chi)/3^3)^2 = ({max_walsh_rounded}/27)^2 = {round(LP_max, 6)}")
print(f"Expected: 1/9 = {round(1/9, 6)}")

if abs(max_walsh_rounded - 9.0) < 0.01:
    print(f"\n✓ CONFIRMED: L(chi) = 9 (perfect nonlinearity for GF(3^3))")
    print(f"✓ CONFIRMED: LP_max = 1/9 (matches DP_max)")
    print(f"✓ Linear and differential bounds are SYMMETRIC")
else:
    print(f"\n✗ L(chi) = {max_walsh_rounded} ≠ 9")

# Also verify DP for completeness
print(f"\n{'='*60}")
print("DDT Verification (cross-check with TM-2026-008)")
print(f"{'='*60}")
ddt_values = set()
dp_max = 0
for da in elements:
    if da == ZERO:
        continue
    for db in elements:
        count = 0
        for x in elements:
            # Check if chi(x + da) - chi(x) = db
            x_plus_da = tuple((x[i] + da[i]) % 3 for i in range(3))
            diff = tuple((chi[x_plus_da][i] - chi[x][i]) % 3 for i in range(3))
            if diff == db:
                count += 1
        ddt_values.add(count)
        if count > dp_max:
            dp_max = count

print(f"DDT values: {sorted(ddt_values)}")
print(f"DP_max = {dp_max}/27 = {round(dp_max/27, 6)}")
if dp_max == 3:
    print("✓ DP_max = 3/27 = 1/9 confirmed (matches TM-2026-008)")
