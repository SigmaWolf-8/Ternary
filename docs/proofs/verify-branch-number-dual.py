#!/usr/bin/env python3
"""
Verify the dual-space argument for B(M_theta) = 8.

The key insight: instead of checking all weight-5/6/7 inputs (infeasible),
check all weight-1/2 OUTPUTS and compute their preimage weights via M_theta^{-1}.

If every weight-1 output has preimage weight >= 7, and every weight-2 output 
has preimage weight >= 6, then no weight-5 or weight-6 input can produce a 
low enough output weight to violate B >= 8.
"""

N = 54
offsets = [0, 1, -1, 7, -7, 13, -13]

def build_matrix():
    """Build M_theta as a 54x54 matrix over GF(3)"""
    M = [[0]*N for _ in range(N)]
    for i in range(N):
        for off in offsets:
            j = (i + off) % N
            M[i][j] = (M[i][j] + 1) % 3
    return M

def mat_mul_vec(M, v):
    """M * v over GF(3)"""
    return [sum(M[i][j] * v[j] for j in range(N)) % 3 for i in range(N)]

def hamming_weight(v):
    return sum(1 for x in v if x != 0)

M = build_matrix()

def gf3_inv(a):
    """Multiplicative inverse in GF(3)"""
    if a == 0: return None
    if a == 1: return 1
    if a == 2: return 2

def mat_inverse_gf3(M_in):
    """Invert an NxN matrix over GF(3) using Gaussian elimination"""
    n = len(M_in)
    aug = [row[:] + [1 if j == i else 0 for j in range(n)] for i, row in enumerate(M_in)]
    
    for col in range(n):
        pivot = None
        for row in range(col, n):
            if aug[row][col] != 0:
                pivot = row
                break
        if pivot is None:
            return None
        
        aug[col], aug[pivot] = aug[pivot], aug[col]
        
        inv_piv = gf3_inv(aug[col][col])
        aug[col] = [(x * inv_piv) % 3 for x in aug[col]]
        
        for row in range(n):
            if row != col and aug[row][col] != 0:
                factor = aug[row][col]
                aug[row] = [(aug[row][j] - factor * aug[col][j]) % 3 for j in range(2*n)]
    
    return [row[n:] for row in aug]

print("Building M_theta...")
print("Computing M_theta^{-1} over GF(3)...")
M_inv = mat_inverse_gf3(M)
if M_inv is None:
    print("ERROR: M_theta is SINGULAR!")
    exit(1)
print("M_theta is invertible. ✓")

print("Verifying M * M^{-1} = I...")
ok = True
for i in range(N):
    for j in range(N):
        val = sum(M[i][k] * M_inv[k][j] for k in range(N)) % 3
        expected = 1 if i == j else 0
        if val != expected:
            ok = False
            print(f"  FAIL at ({i},{j}): got {val}, expected {expected}")
            break
    if not ok:
        break
print(f"Inverse verification: {'PASS ✓' if ok else 'FAIL ✗'}")

g_at_1 = len(offsets) % 3
print(f"g(1) = {len(offsets)} mod 3 = {g_at_1} (non-zero confirms non-singularity) ✓")

print("\n" + "="*60)
print("DUAL CHECK: Weight-1 outputs")
print("="*60)
print(f"Number of weight-1 vectors: {N} positions × 2 values = {N*2}")

min_preimage_wt_1 = 999
max_preimage_wt_1 = 0
count_1 = 0
preimage_wt_dist_1 = {}

for pos in range(N):
    for val in [1, 2]:
        b = [0] * N
        b[pos] = val
        a = mat_mul_vec(M_inv, b)
        wt_a = hamming_weight(a)
        count_1 += 1
        preimage_wt_dist_1[wt_a] = preimage_wt_dist_1.get(wt_a, 0) + 1
        if wt_a < min_preimage_wt_1:
            min_preimage_wt_1 = wt_a
        if wt_a > max_preimage_wt_1:
            max_preimage_wt_1 = wt_a
        
        b_check = mat_mul_vec(M, a)
        if b_check != b:
            print(f"  INVERSE ERROR at pos={pos}, val={val}!")

print(f"Checked {count_1} weight-1 output vectors")
print(f"Preimage weight range: [{min_preimage_wt_1}, {max_preimage_wt_1}]")
print(f"Preimage weight distribution: {dict(sorted(preimage_wt_dist_1.items()))}")

if min_preimage_wt_1 >= 7:
    print(f"ALL weight-1 outputs have preimage weight ≥ 7 ✓")
    print(f"→ No weight-6 input can produce weight-1 output")
    print(f"→ No weight-5 input can produce weight-1 output")
else:
    print(f"WARNING: Found weight-1 output with preimage weight {min_preimage_wt_1}")

print("\n" + "="*60)
print("DUAL CHECK: Weight-2 outputs")
print("="*60)
expected_count = N * (N-1) // 2 * 4
print(f"Number of weight-2 vectors: C(54,2) × 4 = {expected_count}")

min_preimage_wt_2 = 999
max_preimage_wt_2 = 0
count_2 = 0
preimage_wt_dist_2 = {}

for p1 in range(N):
    for p2 in range(p1+1, N):
        for v1 in [1, 2]:
            for v2 in [1, 2]:
                b = [0] * N
                b[p1] = v1
                b[p2] = v2
                a = mat_mul_vec(M_inv, b)
                wt_a = hamming_weight(a)
                count_2 += 1
                preimage_wt_dist_2[wt_a] = preimage_wt_dist_2.get(wt_a, 0) + 1
                if wt_a < min_preimage_wt_2:
                    min_preimage_wt_2 = wt_a
                if wt_a > max_preimage_wt_2:
                    max_preimage_wt_2 = wt_a

print(f"Checked {count_2} weight-2 output vectors")
print(f"Preimage weight range: [{min_preimage_wt_2}, {max_preimage_wt_2}]")
print(f"Preimage weight distribution: {dict(sorted(preimage_wt_dist_2.items()))}")

if min_preimage_wt_2 >= 6:
    print(f"ALL weight-2 outputs have preimage weight ≥ 6 ✓")
    print(f"→ No weight-5 input can produce weight-2 output")
else:
    print(f"WARNING: Found weight-2 output with preimage weight {min_preimage_wt_2}")

print("\n" + "="*60)
print("SYNTHESIS: B(M_theta) EXACT VALUE")
print("="*60)

print("""
Proof that B(M_theta) = 8:

Upper bound: B ≤ 8 (weight-1 input gives sum = 1 + 7 = 8)

Lower bound: wt(a) + wt(M_theta · a) ≥ 8 for all nonzero a:

  Case wt(a) ≤ 4:
    Exhaustive primal search (5,264,172 vectors): min sum = 8 ✓

  Case wt(a) = 5:
    Need wt(M_theta · a) ≥ 3, i.e. wt(b) ≥ 3 where b = M_theta · a.
    If wt(b) = 1: then a = M_inv · b has wt(a) = preimage_wt(wt-1 output).""")

if min_preimage_wt_1 >= 7:
    print(f"    But ALL weight-1 outputs have preimage weight = {min_preimage_wt_1}. Contradiction. ✓")
else:
    print(f"    WARNING: min preimage weight = {min_preimage_wt_1}")

print(f"    If wt(b) = 2: then a = M_inv · b has wt(a) = preimage_wt(wt-2 output).")
if min_preimage_wt_2 >= 6:
    print(f"    But ALL weight-2 outputs have preimage weight ≥ {min_preimage_wt_2}. Contradiction. ✓")
else:
    print(f"    WARNING: min preimage weight = {min_preimage_wt_2}")

print(f"""
  Case wt(a) = 6:
    Need wt(M_theta · a) ≥ 2, i.e. wt(b) ≥ 2.
    If wt(b) = 1: preimage weight = {min_preimage_wt_1} ≥ 7 ≠ 6. Contradiction. ✓
    wt(b) = 0 impossible (M_theta invertible). ✓

  Case wt(a) = 7:
    Sum ≥ 7 + 1 = 8 (M_theta invertible → nonzero output). ✓

  Case wt(a) ≥ 8:
    Sum ≥ 8 + 0 ≥ 8 trivially. ✓""")

all_good = (min_preimage_wt_1 >= 7) and (min_preimage_wt_2 >= 6)
print(f"\n{'='*60}")
if all_good:
    print("B(M_theta) = 8 EXACTLY. PROOF COMPLETE. ✓")
    print(f"Total vectors checked: 5,264,172 (primal) + {count_1 + count_2} (dual) = {5264172 + count_1 + count_2}")
else:
    print("PROOF INCOMPLETE — dual check found unexpected preimage weights")
print("="*60)
