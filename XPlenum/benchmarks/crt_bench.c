/*
 * CRT Fast Path Benchmark
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. — Applied Physics Division
 *
 * Measures the actual CPU performance difference between:
 *   FAST: mod-28 decomposition (mod-4 bitmask + mod-7 + CRT reconstruct)
 *   SLOW: mod-13 (full modular division)
 *   FULL: progressive_route (both paths + CRT reconstruction)
 *   BASELINE: direct mod-364
 *
 * Compile: gcc -O2 -o crt_bench crt_bench.c -lm
 * Run:     ./crt_bench
 */

#include <stdio.h>
#include <stdint.h>
#include <time.h>
#include <stdlib.h>

#define FULL_CIRCLE 364
#define MOD_MOON    13
#define MOD_DAY     28
#define COEFF_FINE  196   /* 28 × 7 */
#define COEFF_FAST  169   /* 13 × 13 */

/* Number of iterations per benchmark (enough to average out noise) */
#define ITERATIONS  100000000ULL  /* 100 million */
#define WARMUP      10000000ULL   /* 10 million warmup */

/* ══════════════════════════════════════════════════════════════
 * FAST PATH: mod-28 via sub-CRT (mod-4 bitmask + mod-7)
 * Target: 2-3 CPU cycles
 * ══════════════════════════════════════════════════════════════ */

static inline uint64_t fast_mod_28(uint64_t position) {
    uint64_t r4 = position & 0x03;         /* mod 4: AND (1 cycle) */
    uint64_t r7 = position % 7;            /* mod 7: compiler optimizes to mul+shift */
    return (21 * r4 + 8 * r7) % 28;        /* CRT reconstruct */
}

static inline uint8_t fast_day_component(uint64_t position) {
    return (uint8_t)fast_mod_28(position);
}

/* ══════════════════════════════════════════════════════════════
 * SLOW PATH: mod-13 (full modular division)
 * Target: 20-40 CPU cycles
 * ══════════════════════════════════════════════════════════════ */

static inline uint8_t fine_moon_component(uint64_t position) {
    return (uint8_t)(position % MOD_MOON);
}

/* ══════════════════════════════════════════════════════════════
 * CRT RECONSTRUCTION
 * ══════════════════════════════════════════════════════════════ */

static inline uint16_t reconstruct(uint8_t fine, uint8_t fast) {
    return (uint16_t)((COEFF_FINE * fine + COEFF_FAST * fast) % FULL_CIRCLE);
}

/* ══════════════════════════════════════════════════════════════
 * PROGRESSIVE ROUTE (both paths)
 * ══════════════════════════════════════════════════════════════ */

typedef struct {
    uint8_t day_component;
    uint8_t moon_component;
    uint16_t circle_position;
} RouteResult;

static inline RouteResult progressive_route(uint64_t position) {
    uint64_t pos = position % FULL_CIRCLE;
    RouteResult r;
    r.day_component = fast_day_component(pos);
    r.moon_component = fine_moon_component(pos);
    r.circle_position = reconstruct(r.moon_component, r.day_component);
    return r;
}

/* ══════════════════════════════════════════════════════════════
 * HIGH-RESOLUTION TIMER
 * ══════════════════════════════════════════════════════════════ */

static inline double now_ns(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1e9 + ts.tv_nsec;
}

/* Volatile sink to prevent dead-code elimination */
volatile uint64_t sink = 0;

/* ══════════════════════════════════════════════════════════════
 * BENCHMARK HARNESS
 * ══════════════════════════════════════════════════════════════ */

int main(void) {
    uint64_t i;
    double start, end, elapsed;
    double ns_per_op;

    /* Generate test positions (pseudo-random to prevent branch prediction gaming) */
    uint64_t *positions = malloc(ITERATIONS * sizeof(uint64_t));
    if (!positions) { perror("malloc"); return 1; }

    srand(42); /* deterministic seed */
    for (i = 0; i < ITERATIONS; i++) {
        positions[i] = (uint64_t)rand() % (FULL_CIRCLE * 1000); /* varied positions */
    }

    printf("╔══════════════════════════════════════════════════════════════╗\n");
    printf("║     CRT FAST PATH BENCHMARK — PlenumNET / Salvi Framework  ║\n");
    printf("║     364 = 13 × 28 Progressive Refinement Timing            ║\n");
    printf("╠══════════════════════════════════════════════════════════════╣\n");
    printf("║  Iterations: %llu (100M)                          ║\n", (unsigned long long)ITERATIONS);
    printf("║  Compiler:   GCC -O2                                       ║\n");
    printf("╚══════════════════════════════════════════════════════════════╝\n\n");

    /* ── WARMUP ── */
    for (i = 0; i < WARMUP; i++) {
        sink += fast_day_component(positions[i % ITERATIONS]);
    }

    /* ═══════════════════════════════════════════════
     * TEST 1: FAST PATH ONLY (mod-28 decomposition)
     * Expected: 2-5 ns/op (~3-10 cycles at 2 GHz)
     * ═══════════════════════════════════════════════ */
    start = now_ns();
    for (i = 0; i < ITERATIONS; i++) {
        sink += fast_day_component(positions[i]);
    }
    end = now_ns();
    elapsed = end - start;
    ns_per_op = elapsed / ITERATIONS;
    printf("FAST PATH (mod-28 via bitmask + mod-7 + CRT)\n");
    printf("  Total:  %.2f ms\n", elapsed / 1e6);
    printf("  Per-op: %.2f ns  (≈ %.1f cycles @ 2 GHz)\n\n", ns_per_op, ns_per_op * 2.0);

    double fast_ns = ns_per_op;

    /* ═══════════════════════════════════════════════
     * TEST 2: SLOW PATH ONLY (mod-13)
     * Expected: 5-20 ns/op (~10-40 cycles at 2 GHz)
     * ═══════════════════════════════════════════════ */
    start = now_ns();
    for (i = 0; i < ITERATIONS; i++) {
        sink += fine_moon_component(positions[i]);
    }
    end = now_ns();
    elapsed = end - start;
    ns_per_op = elapsed / ITERATIONS;
    printf("SLOW PATH (mod-13, full modular division)\n");
    printf("  Total:  %.2f ms\n", elapsed / 1e6);
    printf("  Per-op: %.2f ns  (≈ %.1f cycles @ 2 GHz)\n\n", ns_per_op, ns_per_op * 2.0);

    double slow_ns = ns_per_op;

    /* ═══════════════════════════════════════════════
     * TEST 3: DIRECT MOD-364 (baseline)
     * Expected: similar to or slower than mod-13
     * ═══════════════════════════════════════════════ */
    start = now_ns();
    for (i = 0; i < ITERATIONS; i++) {
        sink += positions[i] % FULL_CIRCLE;
    }
    end = now_ns();
    elapsed = end - start;
    ns_per_op = elapsed / ITERATIONS;
    printf("BASELINE (direct mod-364)\n");
    printf("  Total:  %.2f ms\n", elapsed / 1e6);
    printf("  Per-op: %.2f ns  (≈ %.1f cycles @ 2 GHz)\n\n", ns_per_op, ns_per_op * 2.0);

    double baseline_ns = ns_per_op;

    /* ═══════════════════════════════════════════════
     * TEST 4: FULL PROGRESSIVE ROUTE (both paths + CRT)
     * Expected: slightly more than slow path alone
     * ═══════════════════════════════════════════════ */
    start = now_ns();
    for (i = 0; i < ITERATIONS; i++) {
        RouteResult r = progressive_route(positions[i]);
        sink += r.circle_position;
    }
    end = now_ns();
    elapsed = end - start;
    ns_per_op = elapsed / ITERATIONS;
    printf("PROGRESSIVE ROUTE (fast + slow + CRT reconstruct)\n");
    printf("  Total:  %.2f ms\n", elapsed / 1e6);
    printf("  Per-op: %.2f ns  (≈ %.1f cycles @ 2 GHz)\n\n", ns_per_op, ns_per_op * 2.0);

    double progressive_ns = ns_per_op;

    /* ═══════════════════════════════════════════════
     * TEST 5: FAST PATH mod-4 ONLY (pure bitmask)
     * Expected: ~1 ns (1-2 cycles — the floor)
     * ═══════════════════════════════════════════════ */
    start = now_ns();
    for (i = 0; i < ITERATIONS; i++) {
        sink += positions[i] & 0x03;
    }
    end = now_ns();
    elapsed = end - start;
    ns_per_op = elapsed / ITERATIONS;
    printf("BITMASK ONLY (mod-4 via AND — the hardware floor)\n");
    printf("  Total:  %.2f ms\n", elapsed / 1e6);
    printf("  Per-op: %.2f ns  (≈ %.1f cycles @ 2 GHz)\n\n", ns_per_op, ns_per_op * 2.0);

    /* ═══════════════════════════════════════════════
     * SUMMARY
     * ═══════════════════════════════════════════════ */
    printf("╔══════════════════════════════════════════════════════════════╗\n");
    printf("║                        RESULTS                             ║\n");
    printf("╠══════════════════════════════════════════════════════════════╣\n");
    printf("║  Fast path (mod-28):    %6.2f ns                           ║\n", fast_ns);
    printf("║  Slow path (mod-13):    %6.2f ns                           ║\n", slow_ns);
    printf("║  Baseline  (mod-364):   %6.2f ns                           ║\n", baseline_ns);
    printf("║  Progressive (full):    %6.2f ns                           ║\n", progressive_ns);
    printf("╠══════════════════════════════════════════════════════════════╣\n");

    if (slow_ns > 0 && fast_ns > 0) {
        printf("║  Fast/Slow ratio:       %6.2fx faster                     ║\n", slow_ns / fast_ns);
        printf("║  Fast/Baseline ratio:   %6.2fx faster                     ║\n", baseline_ns / fast_ns);
        printf("║  Head start:            %6.2f ns                          ║\n", slow_ns - fast_ns);
    }
    printf("╠══════════════════════════════════════════════════════════════╣\n");
    printf("║  At 1 GHz:  %.1f cycles fast, %.1f cycles slow              ║\n",
           fast_ns * 1.0, slow_ns * 1.0);
    printf("║  At 3 GHz:  %.1f cycles fast, %.1f cycles slow             ║\n",
           fast_ns * 3.0, slow_ns * 3.0);
    printf("╚══════════════════════════════════════════════════════════════╝\n");

    /* Correctness verification */
    printf("\nVerifying correctness (all 364 positions)...\n");
    int errors = 0;
    for (uint64_t p = 0; p < FULL_CIRCLE; p++) {
        RouteResult r = progressive_route(p);
        if (r.circle_position != p) {
            printf("  FAIL at p=%llu: got %u\n", (unsigned long long)p, r.circle_position);
            errors++;
        }
        if (r.day_component != p % 28) {
            printf("  FAIL fast at p=%llu: got %u, expected %llu\n",
                   (unsigned long long)p, r.day_component, (unsigned long long)(p % 28));
            errors++;
        }
        if (r.moon_component != p % 13) {
            printf("  FAIL slow at p=%llu: got %u, expected %llu\n",
                   (unsigned long long)p, r.moon_component, (unsigned long long)(p % 13));
            errors++;
        }
    }
    if (errors == 0) {
        printf("  All 364 positions: CRT round-trip ✓, fast_mod_28 ✓, fine_mod_13 ✓\n");
    } else {
        printf("  %d ERRORS FOUND\n", errors);
    }

    free(positions);
    return errors;
}
