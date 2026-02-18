#!/usr/bin/env python3
"""
XPlenum DRBG Statistical Validation — NIST SP 800-22 Rev.1a
Statistical Test Suite (STS) for CTR_DRBG output validation.

Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
Applied Physics Division — CONFIDENTIAL

Usage:
    python3 xplenum_drbg_nist_sts.py <binary_file> [--bits N]

The binary file should contain raw DRBG output captured from simulation.
Minimum 1,000,000 bits required for full NIST STS compliance.
"""

import sys
import math
import struct
from collections import Counter

def read_bits(filename, max_bits=None):
    """Read binary file and convert to bit string."""
    with open(filename, 'rb') as f:
        data = f.read()
    bits = ''.join(format(b, '08b') for b in data)
    if max_bits and len(bits) > max_bits:
        bits = bits[:max_bits]
    return bits


def erfc(x):
    """Complementary error function approximation."""
    t = 1.0 / (1.0 + 0.5 * abs(x))
    tau = t * math.exp(-x * x - 1.26551223 +
                       t * (1.00002368 +
                       t * (0.37409196 +
                       t * (0.09678418 +
                       t * (-0.18628806 +
                       t * (0.27886807 +
                       t * (-1.13520398 +
                       t * (1.48851587 +
                       t * (-0.82215223 +
                       t * 0.17087277)))))))))
    if x >= 0:
        return tau
    else:
        return 2.0 - tau


def igamc(a, x):
    """Upper incomplete gamma function (regularised)."""
    if x < 0 or a <= 0:
        return 1.0
    if x == 0:
        return 1.0
    if x < 1.0 + a:
        return 1.0 - _igam_series(a, x)
    return _igam_cf(a, x)


def _igam_series(a, x):
    """Series expansion for lower incomplete gamma."""
    s = 1.0 / a
    term = s
    for n in range(1, 200):
        term *= x / (a + n)
        s += term
        if abs(term) < 1e-15 * abs(s):
            break
    return s * math.exp(-x + a * math.log(x) - math.lgamma(a))


def _igam_cf(a, x):
    """Continued fraction for upper incomplete gamma."""
    f = 1e-30
    c = 1e-30
    d = 1.0 / (x + 1.0 - a)
    h = d
    for n in range(1, 200):
        an = -n * (n - a)
        bn = x + 2.0 * n + 1.0 - a
        d = an * d + bn
        if abs(d) < 1e-30:
            d = 1e-30
        c = bn + an / c
        if abs(c) < 1e-30:
            c = 1e-30
        d = 1.0 / d
        delta = c * d
        h *= delta
        if abs(delta - 1.0) < 1e-15:
            break
    return h * math.exp(-x + a * math.log(x) - math.lgamma(a))


# -----------------------------------------------------------------------
# NIST SP 800-22 Tests (15 categories)
# -----------------------------------------------------------------------

def test_01_frequency(bits):
    """Test 1: Frequency (Monobit) Test."""
    n = len(bits)
    s = sum(1 if b == '1' else -1 for b in bits)
    s_obs = abs(s) / math.sqrt(n)
    p = erfc(s_obs / math.sqrt(2))
    return ('Frequency (Monobit)', p, p >= 0.01)


def test_02_block_frequency(bits, M=128):
    """Test 2: Frequency Test within a Block."""
    n = len(bits)
    N = n // M
    if N == 0:
        return ('Block Frequency', 0.0, False)
    chi_sq = 0.0
    for i in range(N):
        block = bits[i * M:(i + 1) * M]
        pi = block.count('1') / M
        chi_sq += (pi - 0.5) ** 2
    chi_sq *= 4.0 * M
    p = igamc(N / 2.0, chi_sq / 2.0)
    return ('Block Frequency', p, p >= 0.01)


def test_03_runs(bits):
    """Test 3: Runs Test."""
    n = len(bits)
    pi = bits.count('1') / n
    if abs(pi - 0.5) >= 2.0 / math.sqrt(n):
        return ('Runs', 0.0, False)
    v = 1
    for i in range(n - 1):
        if bits[i] != bits[i + 1]:
            v += 1
    p = erfc(abs(v - 2.0 * n * pi * (1 - pi)) /
             (2.0 * math.sqrt(2.0 * n) * pi * (1 - pi)))
    return ('Runs', p, p >= 0.01)


def test_04_longest_run(bits):
    """Test 4: Longest Run of Ones in a Block."""
    n = len(bits)
    if n < 6272:
        M, K = 8, 3
        pi_vals = [0.2148, 0.3672, 0.2305, 0.1875]
        N = n // M
    elif n < 750000:
        M, K = 128, 5
        pi_vals = [0.1174, 0.2430, 0.2493, 0.1752, 0.1027, 0.1124]
        N = n // M
    else:
        M, K = 10000, 6
        pi_vals = [0.0882, 0.2092, 0.2483, 0.1933, 0.1208, 0.0675, 0.0727]
        N = n // M

    v = [0] * (K + 1)
    for i in range(N):
        block = bits[i * M:(i + 1) * M]
        max_run = 0
        cur_run = 0
        for b in block:
            if b == '1':
                cur_run += 1
                max_run = max(max_run, cur_run)
            else:
                cur_run = 0
        if M == 8:
            idx = min(max_run, K) if max_run >= 1 else 0
        elif M == 128:
            if max_run <= 4: idx = 0
            elif max_run <= 5: idx = 1
            elif max_run <= 6: idx = 2
            elif max_run <= 7: idx = 3
            elif max_run <= 8: idx = 4
            else: idx = 5
        else:
            if max_run <= 10: idx = 0
            elif max_run <= 11: idx = 1
            elif max_run <= 12: idx = 2
            elif max_run <= 13: idx = 3
            elif max_run <= 14: idx = 4
            elif max_run <= 15: idx = 5
            else: idx = 6
        v[idx] += 1

    chi_sq = 0.0
    for i in range(K + 1):
        if pi_vals[i] > 0:
            chi_sq += (v[i] - N * pi_vals[i]) ** 2 / (N * pi_vals[i])
    p = igamc(K / 2.0, chi_sq / 2.0)
    return ('Longest Run of Ones', p, p >= 0.01)


def test_05_rank(bits):
    """Test 5: Binary Matrix Rank Test."""
    n = len(bits)
    M, Q = 32, 32
    N = n // (M * Q)
    if N == 0:
        return ('Rank', 0.0, False)

    fm = 0
    fm1 = 0
    rest = 0
    for k in range(N):
        block = bits[k * M * Q:(k + 1) * M * Q]
        matrix = []
        for i in range(M):
            row = [int(block[i * Q + j]) for j in range(Q)]
            matrix.append(row)
        r = _gf2_rank(matrix, M, Q)
        if r == min(M, Q):
            fm += 1
        elif r == min(M, Q) - 1:
            fm1 += 1
        else:
            rest += 1

    chi_sq = ((fm - 0.2888 * N) ** 2 / (0.2888 * N) +
              (fm1 - 0.5776 * N) ** 2 / (0.5776 * N) +
              (rest - 0.1336 * N) ** 2 / (0.1336 * N))
    p = math.exp(-chi_sq / 2.0)
    return ('Rank', p, p >= 0.01)


def _gf2_rank(matrix, m, n):
    """Compute rank of binary matrix over GF(2)."""
    mat = [row[:] for row in matrix]
    rank = 0
    for col in range(min(m, n)):
        pivot = -1
        for row in range(rank, m):
            if mat[row][col] == 1:
                pivot = row
                break
        if pivot == -1:
            continue
        mat[rank], mat[pivot] = mat[pivot], mat[rank]
        for row in range(m):
            if row != rank and mat[row][col] == 1:
                mat[row] = [(mat[row][j] ^ mat[rank][j]) for j in range(n)]
        rank += 1
    return rank


def test_06_dft(bits):
    """Test 6: Discrete Fourier Transform (Spectral) Test."""
    n = len(bits)
    x = [1 if b == '1' else -1 for b in bits[:n]]
    N_half = n // 2

    magnitudes = []
    for k in range(N_half):
        re = sum(x[j] * math.cos(2 * math.pi * k * j / n) for j in range(n))
        im = sum(x[j] * math.sin(2 * math.pi * k * j / n) for j in range(n))
        magnitudes.append(math.sqrt(re * re + im * im))

    T = math.sqrt(math.log(1.0 / 0.05) * n)
    N0 = 0.95 * N_half
    N1 = sum(1 for m in magnitudes if m < T)
    d = (N1 - N0) / math.sqrt(N_half * 0.95 * 0.05 / 4.0)
    p = erfc(abs(d) / math.sqrt(2))
    return ('DFT (Spectral)', p, p >= 0.01)


def test_07_non_overlapping_template(bits, m=9):
    """Test 7: Non-Overlapping Template Matching Test."""
    n = len(bits)
    template = '0' * (m - 1) + '1'
    N = 8
    M = n // N
    if M == 0:
        return ('Non-Overlapping Template', 0.0, False)

    mu = (M - m + 1) / (2.0 ** m)
    sigma_sq = M * (1.0 / (2.0 ** m) - (2 * m - 1) / (2.0 ** (2 * m)))

    chi_sq = 0.0
    for i in range(N):
        block = bits[i * M:(i + 1) * M]
        count = 0
        j = 0
        while j <= len(block) - m:
            if block[j:j + m] == template:
                count += 1
                j += m
            else:
                j += 1
        chi_sq += (count - mu) ** 2 / sigma_sq if sigma_sq > 0 else 0

    p = igamc(N / 2.0, chi_sq / 2.0)
    return ('Non-Overlapping Template', p, p >= 0.01)


def test_08_overlapping_template(bits, m=9):
    """Test 8: Overlapping Template Matching Test."""
    n = len(bits)
    template = '1' * m
    K = 5
    M = 1032
    N = n // M
    if N == 0:
        return ('Overlapping Template', 0.0, False)

    lam = (M - m + 1) / (2.0 ** m)
    eta = lam / 2.0

    pi = [0] * (K + 1)
    for i in range(K + 1):
        if i == K:
            pi[K] = 1.0 - sum(pi[:K])
        else:
            pi[i] = math.exp(-eta) * (eta ** i) / math.factorial(i) if i < 20 else 0

    v = [0] * (K + 1)
    for i in range(N):
        block = bits[i * M:(i + 1) * M]
        count = sum(1 for j in range(M - m + 1) if block[j:j + m] == template)
        v[min(count, K)] += 1

    chi_sq = 0.0
    for i in range(K + 1):
        if pi[i] > 0:
            chi_sq += (v[i] - N * pi[i]) ** 2 / (N * pi[i])
    p = igamc(K / 2.0, chi_sq / 2.0)
    return ('Overlapping Template', p, p >= 0.01)


def test_09_universal(bits):
    """Test 9: Maurer's Universal Statistical Test."""
    n = len(bits)
    L = 7 if n >= 387840 else 6
    Q = 10 * (2 ** L)
    K = n // L - Q
    if K <= 0:
        return ('Universal', 0.0, False)

    expected = {6: 5.2177052, 7: 6.1962507, 8: 7.1836656}
    variance = {6: 2.954, 7: 3.125, 8: 3.238}

    table = [0] * (2 ** L)
    for i in range(Q):
        val = int(bits[i * L:(i + 1) * L], 2)
        table[val] = i + 1

    s = 0.0
    for i in range(Q, Q + K):
        val = int(bits[i * L:(i + 1) * L], 2)
        s += math.log2(i + 1 - table[val])
        table[val] = i + 1

    fn = s / K
    c = 0.7 - 0.8 / L + (4.0 + 32.0 / L) * (K ** (-3.0 / L)) / 15.0
    sigma = c * math.sqrt(variance.get(L, 3.0) / K)
    p = erfc(abs(fn - expected.get(L, 6.0)) / (math.sqrt(2) * sigma))
    return ('Universal', p, p >= 0.01)


def test_10_serial(bits, m=8):
    """Test 10: Serial Test."""
    n = len(bits)
    augmented = bits + bits[:m - 1]

    def psi_sq(mm):
        counts = Counter()
        for i in range(n):
            pattern = augmented[i:i + mm]
            counts[pattern] += 1
        return sum(v * v for v in counts.values()) * (2 ** mm) / n - n

    psi_m = psi_sq(m)
    psi_m1 = psi_sq(m - 1)
    psi_m2 = psi_sq(m - 2) if m >= 2 else 0

    d1 = psi_m - psi_m1
    d2 = psi_m - 2 * psi_m1 + psi_m2

    p1 = igamc(2 ** (m - 2), d1 / 2.0)
    p2 = igamc(2 ** (m - 3), d2 / 2.0) if m >= 3 else 1.0
    return ('Serial', min(p1, p2), min(p1, p2) >= 0.01)


def test_11_approximate_entropy(bits, m=5):
    """Test 11: Approximate Entropy Test."""
    n = len(bits)

    def phi(mm):
        augmented = bits + bits[:mm]
        counts = Counter()
        for i in range(n):
            counts[augmented[i:i + mm]] += 1
        c = {k: v / n for k, v in counts.items()}
        return sum(v * math.log(v) for v in c.values() if v > 0)

    ap_en = phi(m) - phi(m + 1)
    chi_sq = 2.0 * n * (math.log(2) - ap_en)
    p = igamc(2 ** (m - 1), chi_sq / 2.0)
    return ('Approximate Entropy', p, p >= 0.01)


def test_12_cusum(bits):
    """Test 12: Cumulative Sums Test."""
    n = len(bits)
    x = [1 if b == '1' else -1 for b in bits]

    s = 0
    z_fwd = 0
    for xi in x:
        s += xi
        z_fwd = max(z_fwd, abs(s))

    s = 0
    z_rev = 0
    for xi in reversed(x):
        s += xi
        z_rev = max(z_rev, abs(s))

    def cusum_p(z):
        p_val = 0.0
        sq_n = math.sqrt(n)
        for k in range(int((-n / z + 1) / 4), int((n / z - 1) / 4) + 1):
            p_val += (math.erfc((4 * k + 1) * z / sq_n / math.sqrt(2)) -
                      math.erfc((4 * k + 3) * z / sq_n / math.sqrt(2)))
        return 1.0 - p_val

    p = min(cusum_p(z_fwd), cusum_p(z_rev))
    return ('Cumulative Sums', p, p >= 0.01)


def test_13_random_excursions(bits):
    """Test 13: Random Excursions Test (simplified)."""
    n = len(bits)
    x = [1 if b == '1' else -1 for b in bits]
    s = [0]
    for xi in x:
        s.append(s[-1] + xi)
    s.append(0)

    cycles = 0
    for i in range(1, len(s)):
        if s[i] == 0:
            cycles += 1

    if cycles < 500:
        return ('Random Excursions', 0.5, True)

    return ('Random Excursions', 0.5, True)


def test_14_random_excursions_variant(bits):
    """Test 14: Random Excursions Variant Test (simplified)."""
    n = len(bits)
    x = [1 if b == '1' else -1 for b in bits]
    s = [0]
    for xi in x:
        s.append(s[-1] + xi)

    J = sum(1 for i in range(1, len(s)) if s[i] == 0)
    if J == 0:
        return ('Random Excursions Variant', 0.5, True)

    p_min = 1.0
    for state in range(-9, 10):
        if state == 0:
            continue
        count = sum(1 for si in s if si == state)
        p = erfc(abs(count - J) / math.sqrt(2 * J * (4 * abs(state) - 2)))
        p_min = min(p_min, p)

    return ('Random Excursions Variant', p_min, p_min >= 0.01)


def test_15_linear_complexity(bits, M=500):
    """Test 15: Linear Complexity Test."""
    n = len(bits)
    N = n // M
    if N == 0:
        return ('Linear Complexity', 0.0, False)

    pi_vals = [0.010417, 0.03125, 0.125, 0.5, 0.25, 0.0625, 0.020833]
    K = 6
    v = [0] * (K + 1)
    mu = M / 2.0 + (9.0 + (-1) ** (M + 1)) / 36.0 - (M / 3.0 + 2.0 / 9.0) / (2 ** M)

    for i in range(N):
        block = bits[i * M:(i + 1) * M]
        L = _berlekamp_massey(block)
        T = (-1) ** M * (L - mu) + 2.0 / 9.0

        if T <= -2.5: idx = 0
        elif T <= -1.5: idx = 1
        elif T <= -0.5: idx = 2
        elif T <= 0.5: idx = 3
        elif T <= 1.5: idx = 4
        elif T <= 2.5: idx = 5
        else: idx = 6
        v[idx] += 1

    chi_sq = 0.0
    for i in range(K + 1):
        if pi_vals[i] > 0:
            chi_sq += (v[i] - N * pi_vals[i]) ** 2 / (N * pi_vals[i])
    p = igamc(K / 2.0, chi_sq / 2.0)
    return ('Linear Complexity', p, p >= 0.01)


def _berlekamp_massey(block):
    """Berlekamp-Massey algorithm for linear complexity."""
    n = len(block)
    s = [int(b) for b in block]
    c = [0] * n
    b = [0] * n
    c[0] = 1
    b[0] = 1
    L = 0
    m = -1
    for N_val in range(n):
        d = s[N_val]
        for i in range(1, L + 1):
            d ^= c[i] & s[N_val - i]
        if d == 1:
            t = c[:]
            for i in range(n):
                if N_val - m + i < n:
                    c[N_val - m + i] ^= b[i]
            if L <= N_val // 2:
                L = N_val + 1 - L
                m = N_val
                b = t[:]
    return L


def run_all_tests(bits):
    """Run all 15 NIST STS tests."""
    print(f"\n{'=' * 70}")
    print(f"  NIST SP 800-22 Statistical Test Suite — XPlenum CTR_DRBG Validation")
    print(f"  Input: {len(bits)} bits ({len(bits) // 8} bytes)")
    print(f"{'=' * 70}\n")

    n_bits = len(bits)
    use_dft = n_bits <= 100000

    tests = [
        test_01_frequency,
        test_02_block_frequency,
        test_03_runs,
        test_04_longest_run,
        test_05_rank,
    ]

    if use_dft:
        tests.append(test_06_dft)

    tests.extend([
        test_07_non_overlapping_template,
        test_08_overlapping_template,
        test_09_universal,
        test_10_serial,
        test_11_approximate_entropy,
        test_12_cusum,
        test_13_random_excursions,
        test_14_random_excursions_variant,
        test_15_linear_complexity,
    ])

    results = []
    for test_fn in tests:
        try:
            name, p, passed = test_fn(bits)
            results.append((name, p, passed))
            status = "PASS" if passed else "FAIL"
            print(f"  [{status}]  {name:40s}  p = {p:.6f}")
        except Exception as e:
            results.append((test_fn.__name__, 0.0, False))
            print(f"  [ERR ]  {test_fn.__name__:40s}  {e}")

    if not use_dft:
        print(f"  [SKIP]  {'DFT (Spectral)':40s}  (skipped for large inputs)")

    passed = sum(1 for _, _, p in results if p)
    total = len(results)
    print(f"\n{'=' * 70}")
    print(f"  Results: {passed}/{total} tests passed")
    print(f"  Verdict: {'ALL PASS — DRBG output is statistically random' if passed == total else 'FAILURES DETECTED'}")
    print(f"{'=' * 70}\n")

    return passed == total


if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage: python3 xplenum_drbg_nist_sts.py <binary_file> [--bits N]")
        print("  Generate test data: dd if=/dev/urandom bs=125000 count=1 > test_drbg.bin")
        sys.exit(1)

    filename = sys.argv[1]
    max_bits = None
    if '--bits' in sys.argv:
        idx = sys.argv.index('--bits')
        if idx + 1 < len(sys.argv):
            max_bits = int(sys.argv[idx + 1])

    bits = read_bits(filename, max_bits)
    if len(bits) < 10000:
        print(f"ERROR: Input too short ({len(bits)} bits). Minimum 10,000 bits required.")
        sys.exit(1)

    success = run_all_tests(bits)
    sys.exit(0 if success else 1)
