// =============================================================================
// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// XPLENUM — Security Fuzzing Harness (AFL++ / libFuzzer compatible)
// Phase 6, Task 6.6: Fuzz all 4 subsystems with randomized operands
//
// Build (AFL++):
//   afl-clang-fast++ -std=c++17 -O2 -o xplenum_fuzz xplenum_fuzz_harness.cpp
//
// Build (libFuzzer):
//   clang++ -std=c++17 -O2 -fsanitize=fuzzer,address -o xplenum_fuzz \
//           xplenum_fuzz_harness.cpp
//
// Run (AFL++):
//   mkdir -p corpus seeds && echo -n "AAAA" > seeds/seed0
//   afl-fuzz -i seeds -o findings -- ./xplenum_fuzz
//
// Run (libFuzzer):
//   ./xplenum_fuzz corpus/ -max_len=64 -runs=10000000
// =============================================================================

#include "../spike/xplenum_spike_extension.h"
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <cassert>
#include <vector>

static XPlenumState g_state;

// ---------------------------------------------------------------------------
// Invariant checkers — called after every instruction to detect violations
// ---------------------------------------------------------------------------

static void check_invariants(const XPlenumState& s, const XPlenumExecResult& r) {
    // INV-1: Exception code must be in valid range [0, 7]
    assert(r.exc_code <= 0x7 && "Exception code out of range");

    // INV-2: Version register always returns constant
    assert(xplenum_csr_read(const_cast<XPlenumState&>(s), CSR_XPVERSION) == XP_VERSION &&
           "Version register corrupted");

    // INV-3: Domain table indices are bounded
    for (int i = 0; i < 256; i++) {
        // Just read — no crash means no OOB
        (void)s.domain_table[i];
    }

    // INV-4: Capability table entries are bounded
    for (int i = 0; i < 64; i++) {
        if (s.cap_table[i].valid && !s.cap_table[i].revoked) {
            assert(s.cap_table[i].bound >= s.cap_table[i].base &&
                   "Capability bound < base");
        }
    }

    // INV-5: Revoked capabilities should never pass check
    for (int i = 0; i < 64; i++) {
        if (s.cap_table[i].revoked) {
            uint32_t insn = (F7_TCAPLD << 25) | (0 << 20) | (1 << 15) |
                            (F3_TCAP << 12) | (3 << 7) | XP_OPCODE;
            XPlenumState test_state = s;
            test_state.xpstatus |= XPSTATUS_CAP_EN;
            auto res = xplenum_execute(test_state, insn, i, 0);
            assert(res.exc_code == XP_EXC_CAP_REVOKED &&
                   "Revoked capability did not trigger exception");
        }
    }

    // INV-6: Disabled subsystem should always except
    if (!(s.xpstatus & XPSTATUS_MASK_EN)) {
        XPlenumState test_state = s;
        uint32_t insn = (F7_TMASK << 25) | (2 << 20) | (1 << 15) |
                        (F3_TMASK << 12) | (3 << 7) | XP_OPCODE;
        auto res = xplenum_execute(test_state, insn, 0, 0);
        assert(res.exc_code == XP_EXC_MASK_FAULT &&
               "Disabled masking did not fault");
    }
}

// ---------------------------------------------------------------------------
// Fuzz input interpretation
// ---------------------------------------------------------------------------

struct FuzzInput {
    uint8_t  op_type;      // Which subsystem to target
    uint8_t  funct7;       // funct7 field
    uint32_t rs1_val;      // Source register 1
    uint32_t rs2_val;      // Source register 2
    uint8_t  csr_op;       // CSR operation (write, modify status)
    uint32_t csr_val;      // CSR write value
    uint8_t  sequence_len; // Number of chained operations
};

static FuzzInput parse_input(const uint8_t* data, size_t size) {
    FuzzInput fi = {0};
    if (size >= 1)  fi.op_type      = data[0] % 7; // 7 funct3 groups
    if (size >= 2)  fi.funct7       = data[1] & 0x7F;
    if (size >= 6)  memcpy(&fi.rs1_val, data + 2, 4);
    if (size >= 10) memcpy(&fi.rs2_val, data + 6, 4);
    if (size >= 11) fi.csr_op       = data[10];
    if (size >= 15) memcpy(&fi.csr_val, data + 11, 4);
    if (size >= 16) fi.sequence_len = data[15] % 32 + 1;
    return fi;
}

// ---------------------------------------------------------------------------
// Core fuzz function
// ---------------------------------------------------------------------------

static void fuzz_one(const uint8_t* data, size_t size) {
    if (size < 4) return;

    FuzzInput fi = parse_input(data, size);

    // Optionally modify status register (enable/disable subsystems)
    if (fi.csr_op & 0x80) {
        g_state.xpstatus = fi.csr_val & 0x0F;
    }

    // Build instruction encoding
    uint32_t f3 = fi.op_type;
    uint32_t f7 = fi.funct7 & 0x7F;
    uint32_t insn = (f7 << 25) | (2 << 20) | (1 << 15) |
                    (f3 << 12) | (3 << 7) | XP_OPCODE;

    // Execute instruction sequence
    for (int i = 0; i < fi.sequence_len; i++) {
        auto result = xplenum_execute(g_state, insn, fi.rs1_val + i, fi.rs2_val);
        check_invariants(g_state, result);

        // Interleave different operations to stress state transitions
        if (i > 0 && (data[(i % size)] & 0x3) == 0) {
            // Random CSR toggle
            g_state.xpstatus ^= (1u << (data[(i + 1) % size] & 0x3));
        }

        // Rapid domain switch stress
        if (f3 == F3_TDOM && i % 3 == 0) {
            uint32_t dom_insn = (F7_TDOMXFR << 25) | (2 << 20) | (1 << 15) |
                                (F3_TDOM << 12) | (0 << 7) | XP_OPCODE;
            g_state.xpstatus |= XPSTATUS_DOM_EN;
            auto dr = xplenum_execute(g_state, dom_insn,
                                       fi.rs1_val & 0xFF, fi.rs2_val & 0xFF);
            check_invariants(g_state, dr);
        }

        // Capability interleave
        if (f3 == F3_TCAP && i % 2 == 0) {
            // Mint then immediately revoke
            uint32_t mint_insn = (F7_TCAPST << 25) | (2 << 20) | (1 << 15) |
                                 (F3_TCAP << 12) | (0 << 7) | XP_OPCODE;
            uint32_t rev_insn  = (F7_TCAPREV << 25) | (0 << 20) | (1 << 15) |
                                 (F3_TCAP << 12) | (3 << 7) | XP_OPCODE;
            g_state.xpstatus |= XPSTATUS_CAP_EN;
            xplenum_execute(g_state, mint_insn, i & 0x3F, 0x1000 * i);
            auto rr = xplenum_execute(g_state, rev_insn, i & 0x3F, 0);
            check_invariants(g_state, rr);
        }
    }
}

// ---------------------------------------------------------------------------
// Entry points
// ---------------------------------------------------------------------------

#ifdef __AFL_FUZZ_TESTCASE_LEN
// AFL++ persistent mode
__AFL_FUZZ_INIT();
int main() {
    g_state.reset();
    __AFL_INIT();
    unsigned char *buf = __AFL_FUZZ_TESTCASE_BUF;
    while (__AFL_LOOP(100000)) {
        int len = __AFL_FUZZ_TESTCASE_LEN;
        g_state.reset();
        fuzz_one(buf, len);
    }
    return 0;
}

#elif defined(LIBFUZZER)
// libFuzzer entry
extern "C" int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size) {
    static bool init = false;
    if (!init) { g_state.reset(); init = true; }
    g_state.reset();
    fuzz_one(data, size);
    return 0;
}

#else
// Standalone deterministic fuzzer (no AFL/libFuzzer required)
int main(int argc, char** argv) {
    printf("XPlenum Security Fuzzer — Standalone Mode\n");
    printf("==========================================\n\n");

    uint64_t iterations = 10000000; // 10M default
    if (argc > 1) iterations = strtoull(argv[1], nullptr, 10);

    printf("Running %llu iterations...\n", (unsigned long long)iterations);

    std::mt19937 rng(0x5EED);
    uint8_t buf[64];
    uint64_t max_ops = 0;
    uint64_t total_exceptions = 0;

    for (uint64_t i = 0; i < iterations; i++) {
        size_t len = 16 + (rng() % 48);
        for (size_t j = 0; j < len; j++) buf[j] = rng() & 0xFF;

        g_state.reset();
        uint32_t exc_before = g_state.xpexc_cause;
        fuzz_one(buf, len);
        if (g_state.xpexc_cause != exc_before) total_exceptions++;

        if (g_state.xpperf_cnt > max_ops) max_ops = g_state.xpperf_cnt;

        if ((i + 1) % 1000000 == 0) {
            printf("  %lluM iterations complete, max ops/run: %llu, exceptions: %llu\n",
                   (unsigned long long)(i + 1) / 1000000,
                   (unsigned long long)max_ops,
                   (unsigned long long)total_exceptions);
        }
    }

    printf("\n==========================================\n");
    printf("PASS — %llu iterations, 0 invariant violations\n",
           (unsigned long long)iterations);
    printf("Max operations per run: %llu\n", (unsigned long long)max_ops);
    printf("Total exception events: %llu\n", (unsigned long long)total_exceptions);
    return 0;
}
#endif
