// =============================================================================
// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// XPLENUM — Spike ISS Standalone Test Harness
// Phase 6, Task 6.1a: Validates all 21 instructions in the Spike emulation model
//
// Compile: g++ -std=c++17 -O2 -o xplenum_spike_test xplenum_spike_test.cpp
// Run:     ./xplenum_spike_test
// =============================================================================

#include "xplenum_spike_extension.h"
#include <cstdio>
#include <cstdlib>
#include <cassert>

static int tests_passed = 0;
static int tests_failed = 0;

#define TEST_ASSERT(cond, msg) do { \
    if (!(cond)) { \
        printf("  FAIL: %s (line %d)\n", msg, __LINE__); \
        tests_failed++; \
    } else { \
        tests_passed++; \
    } \
} while(0)

static uint32_t encode_r_type(uint32_t f7, uint32_t rs2_idx, uint32_t rs1_idx,
                               uint32_t f3, uint32_t rd_idx, uint32_t opcode) {
    return (f7 << 25) | (rs2_idx << 20) | (rs1_idx << 15) |
           (f3 << 12) | (rd_idx << 7) | opcode;
}

// ---------------------------------------------------------------------------
// Test Groups
// ---------------------------------------------------------------------------

static void test_masking(XPlenumState& state) {
    printf("\n=== Masking Tests ===\n");

    state.xpstatus |= XPSTATUS_MASK_EN;

    // TMASK: rd = rs1 XOR rs2
    {
        uint32_t insn = encode_r_type(F7_TMASK, 2, 1, F3_TMASK, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn, 0xAABBCCDD, 0x11223344);
        TEST_ASSERT(r.writes_rd, "TMASK writes rd");
        TEST_ASSERT(r.rd_val == (0xAABBCCDD ^ 0x11223344), "TMASK XOR correct");
        TEST_ASSERT(r.exc_code == XP_EXC_NONE, "TMASK no exception");
    }

    // TUNMASK: rd = rs1 XOR rs2 (same operation, inverse intent)
    {
        uint32_t insn = encode_r_type(F7_TUNMASK, 2, 1, F3_TMASK, 3, XP_OPCODE);
        uint32_t masked = 0xAABBCCDD ^ 0x11223344;
        auto r = xplenum_execute(state, insn, masked, 0x11223344);
        TEST_ASSERT(r.rd_val == 0xAABBCCDD, "TUNMASK recovers original");
    }

    // TMASKR: generate random mask from DRBG
    {
        uint32_t insn = encode_r_type(F7_TMASKR, 0, 0, F3_TMASK, 3, XP_OPCODE);
        auto r1 = xplenum_execute(state, insn, 0, 0);
        auto r2 = xplenum_execute(state, insn, 0, 0);
        TEST_ASSERT(r1.writes_rd, "TMASKR writes rd");
        TEST_ASSERT(r1.rd_val != r2.rd_val, "TMASKR produces different values");
    }

    // TMASKRF: refresh mask
    {
        uint32_t insn = encode_r_type(F7_TMASKRF, 0, 1, F3_TMASK, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn, 0x12345678, 0);
        TEST_ASSERT(r.writes_rd, "TMASKRF writes rd");
        TEST_ASSERT(r.rd_val != 0x12345678, "TMASKRF changes value");
    }

    // Disabled subsystem test
    state.xpstatus &= ~XPSTATUS_MASK_EN;
    {
        uint32_t insn = encode_r_type(F7_TMASK, 2, 1, F3_TMASK, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn, 0, 0);
        TEST_ASSERT(r.exc_code == XP_EXC_MASK_FAULT, "Disabled masking raises exception");
    }
    state.xpstatus |= XPSTATUS_MASK_EN;
}

static void test_domain(XPlenumState& state) {
    printf("\n=== Domain Tests ===\n");

    state.xpstatus |= XPSTATUS_DOM_EN;

    // TDOMSET: set domain tag
    {
        uint32_t insn = encode_r_type(F7_TDOMSET, 2, 1, F3_TDOM, 0, XP_OPCODE);
        xplenum_execute(state, insn, 0x05, 0xDEADBEEF);
        TEST_ASSERT(state.domain_table[0x05] == 0xDEADBEEF, "TDOMSET stores tag");
    }

    // TDOMCHK: check domain — match
    {
        uint32_t insn = encode_r_type(F7_TDOMCHK, 2, 1, F3_TDOM, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn, 0x05, 0xDEADBEEF);
        TEST_ASSERT(r.rd_val == 1, "TDOMCHK match returns 1");
        TEST_ASSERT(r.exc_code == XP_EXC_NONE, "TDOMCHK match no exception");
    }

    // TDOMCHK: check domain — mismatch
    {
        uint32_t insn = encode_r_type(F7_TDOMCHK, 2, 1, F3_TDOM, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn, 0x05, 0xBAADF00D);
        TEST_ASSERT(r.rd_val == 0, "TDOMCHK mismatch returns 0");
        TEST_ASSERT(r.exc_code == XP_EXC_DOM_VIOLATION, "TDOMCHK mismatch raises violation");
    }

    // TDOMCLR: clear domain tag
    {
        uint32_t insn = encode_r_type(F7_TDOMCLR, 0, 1, F3_TDOM, 0, XP_OPCODE);
        xplenum_execute(state, insn, 0x05, 0);
        TEST_ASSERT(state.domain_table[0x05] == 0, "TDOMCLR clears tag");
    }

    // TDOMXFR: transfer domain
    {
        state.domain_table[0x10] = 0xCAFE;
        uint32_t insn = encode_r_type(F7_TDOMXFR, 2, 1, F3_TDOM, 0, XP_OPCODE);
        xplenum_execute(state, insn, 0x10, 0x20);
        TEST_ASSERT(state.domain_table[0x20] == 0xCAFE, "TDOMXFR destination set");
        TEST_ASSERT(state.domain_table[0x10] == 0, "TDOMXFR source cleared");
    }

    // Disabled subsystem
    state.xpstatus &= ~XPSTATUS_DOM_EN;
    {
        uint32_t insn = encode_r_type(F7_TDOMSET, 2, 1, F3_TDOM, 0, XP_OPCODE);
        auto r = xplenum_execute(state, insn, 0, 0);
        TEST_ASSERT(r.exc_code == XP_EXC_DOM_VIOLATION, "Disabled domain raises exception");
    }
    state.xpstatus |= XPSTATUS_DOM_EN;
}

static void test_capability(XPlenumState& state) {
    printf("\n=== Capability Tests ===\n");

    state.xpstatus |= XPSTATUS_CAP_EN;

    // TCAPST: create capability
    {
        uint32_t insn = encode_r_type(F7_TCAPST, 2, 1, F3_TCAP, 0, XP_OPCODE);
        xplenum_execute(state, insn, 0x03, 0x1000);
        TEST_ASSERT(state.cap_table[3].valid, "TCAPST creates valid entry");
        TEST_ASSERT(state.cap_table[3].base == 0x1000, "TCAPST base correct");
    }

    // TCAPLD: load capability permissions
    {
        uint32_t insn = encode_r_type(F7_TCAPLD, 0, 1, F3_TCAP, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn, 0x03, 0);
        TEST_ASSERT(r.writes_rd, "TCAPLD writes rd");
        TEST_ASSERT(r.rd_val == 0x7, "TCAPLD returns permissions");
    }

    // TCAPCHK: check — in bounds
    {
        uint32_t insn = encode_r_type(F7_TCAPCHK, 2, 1, F3_TCAP, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn, 0x03, 0x1500);
        TEST_ASSERT(r.rd_val == 1, "TCAPCHK in-bounds returns 1");
    }

    // TCAPCHK: check — out of bounds
    {
        uint32_t insn = encode_r_type(F7_TCAPCHK, 2, 1, F3_TCAP, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn, 0x03, 0x3000);
        TEST_ASSERT(r.rd_val == 0, "TCAPCHK out-of-bounds returns 0");
    }

    // TCAPREV: revoke
    {
        uint32_t insn = encode_r_type(F7_TCAPREV, 0, 1, F3_TCAP, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn, 0x03, 0);
        TEST_ASSERT(r.rd_val == 1, "TCAPREV returns 1");
        TEST_ASSERT(state.cap_table[3].revoked, "TCAPREV marks revoked");
    }

    // TCAPLD after revoke
    {
        uint32_t insn = encode_r_type(F7_TCAPLD, 0, 1, F3_TCAP, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn, 0x03, 0);
        TEST_ASSERT(r.exc_code == XP_EXC_CAP_REVOKED, "TCAPLD revoked raises exception");
    }

    // TCAPLD invalid index
    {
        uint32_t insn = encode_r_type(F7_TCAPLD, 0, 1, F3_TCAP, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn, 0x3F, 0);
        TEST_ASSERT(r.exc_code == XP_EXC_CAP_INVALID, "TCAPLD invalid raises exception");
    }
}

static void test_crypto_rotation(XPlenumState& state) {
    printf("\n=== Crypto/Rotation Tests ===\n");

    // TROTL
    {
        uint32_t insn = encode_r_type(F7_TROTL, 2, 1, F3_TROT, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn, 0x80000001, 1);
        TEST_ASSERT(r.writes_rd, "TROTL writes rd");
        TEST_ASSERT(r.rd_val == 0x00000003, "TROTL rotate left by 1");
    }

    // TROTR
    {
        uint32_t insn = encode_r_type(F7_TROTR, 2, 1, F3_TROT, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn, 0x80000001, 1);
        TEST_ASSERT(r.rd_val == 0xC0000000, "TROTR rotate right by 1");
    }

    // TTBOX
    {
        uint32_t insn = encode_r_type(F7_TTBOX, 0, 1, F3_TROT, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn, 0x00, 0);
        TEST_ASSERT(r.writes_rd, "TTBOX writes rd");
        TEST_ASSERT(r.rd_val == TRIT_SBOX[0], "TTBOX S-box lookup index 0");
    }

    // TPERM
    {
        uint32_t insn = encode_r_type(F7_TPERM, 2, 1, F3_TROT, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn, 0xAABBCCDD, 0);
        TEST_ASSERT(r.writes_rd, "TPERM writes rd");
    }
}

static void test_trit_encoding(XPlenumState& state) {
    printf("\n=== Trit Encoding Tests ===\n");

    // TTRIT: binary to ternary
    {
        uint32_t insn = encode_r_type(F7_TTRIT, 0, 1, F3_TENC, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn, 0x05, 0); // 0b0101 → trits: 1,1
        TEST_ASSERT(r.writes_rd, "TTRIT writes rd");
    }

    // TDETRIT: ternary to binary
    {
        uint32_t insn = encode_r_type(F7_TDETRIT, 0, 1, F3_TENC, 3, XP_OPCODE);
        uint32_t trit_val = binary_to_trit(0x42);
        auto r = xplenum_execute(state, insn, trit_val, 0);
        TEST_ASSERT(r.writes_rd, "TDETRIT writes rd");
    }

    // Round-trip: encode then decode
    {
        uint32_t insn_enc = encode_r_type(F7_TTRIT, 0, 1, F3_TENC, 3, XP_OPCODE);
        uint32_t insn_dec = encode_r_type(F7_TDETRIT, 0, 1, F3_TENC, 3, XP_OPCODE);
        uint32_t original = 0x55; // 0b01010101 — all valid trits
        auto r1 = xplenum_execute(state, insn_enc, original, 0);
        auto r2 = xplenum_execute(state, insn_dec, r1.rd_val, 0);
        TEST_ASSERT(r2.rd_val == original, "TTRIT/TDETRIT round-trip");
    }
}

static void test_signal(XPlenumState& state) {
    printf("\n=== Signal Processing Tests ===\n");

    state.xpstatus |= XPSTATUS_SIG_EN;

    // TSIGFLT: signal filter
    {
        uint32_t insn = encode_r_type(F7_TSIGFLT, 2, 1, F3_TSIG, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn, 0x01020304, 0x01010101);
        TEST_ASSERT(r.writes_rd, "TSIGFLT writes rd");
        TEST_ASSERT(r.exc_code == XP_EXC_NONE, "TSIGFLT no exception");
    }

    // TSIGCMP: signal compare
    {
        uint32_t insn = encode_r_type(F7_TSIGCMP, 2, 1, F3_TSIG, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn, 100, 50);
        TEST_ASSERT(r.rd_val == 1, "TSIGCMP a > b returns 1");
    }
    {
        uint32_t insn = encode_r_type(F7_TSIGCMP, 2, 1, F3_TSIG, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn, 50, 100);
        TEST_ASSERT(r.rd_val == 0xFFFFFFFF, "TSIGCMP a < b returns -1");
    }
    {
        uint32_t insn = encode_r_type(F7_TSIGCMP, 2, 1, F3_TSIG, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn, 42, 42);
        TEST_ASSERT(r.rd_val == 0, "TSIGCMP a == b returns 0");
    }

    // TSIGACC: signal accumulate
    {
        state.sig_accumulator = 0;
        uint32_t insn = encode_r_type(F7_TSIGACC, 2, 1, F3_TSIG, 3, XP_OPCODE);
        xplenum_execute(state, insn, 10, 5);
        TEST_ASSERT(state.sig_accumulator == 50, "TSIGACC first accumulate");
        xplenum_execute(state, insn, 3, 7);
        TEST_ASSERT(state.sig_accumulator == 71, "TSIGACC second accumulate");
    }

    // Disabled subsystem
    state.xpstatus &= ~XPSTATUS_SIG_EN;
    {
        uint32_t insn = encode_r_type(F7_TSIGFLT, 2, 1, F3_TSIG, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn, 0, 0);
        TEST_ASSERT(r.exc_code == XP_EXC_PRIV_FAULT, "Disabled signal raises exception");
    }
    state.xpstatus |= XPSTATUS_SIG_EN;
}

static void test_csr_comprehensive(XPlenumState& state) {
    printf("\n=== Comprehensive CSR Tests (all 12 registers) ===\n");

    // Helper: CSR write uses funct7 bit[6]=1, CSR read uses funct7 bit[6]=0
    // rs2_val provides CSR index (0x0–0xB)

    // --- CSR 0x7C0: XPSTATUS (R/W) ---
    {
        uint32_t insn_w = encode_r_type(0x40, 0, 1, F3_TCSR, 0, XP_OPCODE);
        xplenum_execute(state, insn_w, 0x0F, 0x00);
        TEST_ASSERT(state.xpstatus == 0x0F, "CSR write XPSTATUS = 0x0F");

        uint32_t insn_r = encode_r_type(0x00, 0, 0, F3_TCSR, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn_r, 0, 0x00);
        TEST_ASSERT(r.rd_val == 0x0F, "CSR read XPSTATUS returns 0x0F");
        TEST_ASSERT(r.writes_rd, "CSR read writes rd");
    }

    // --- CSR 0x7C1: XPDOMID (R/W) ---
    {
        uint32_t insn_w = encode_r_type(0x40, 0, 1, F3_TCSR, 0, XP_OPCODE);
        xplenum_execute(state, insn_w, 0xAB, 0x01);
        TEST_ASSERT(state.xpdomid == 0xAB, "CSR write XPDOMID");

        uint32_t insn_r = encode_r_type(0x00, 0, 0, F3_TCSR, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn_r, 0, 0x01);
        TEST_ASSERT(r.rd_val == 0xAB, "CSR read XPDOMID");
    }

    // --- CSR 0x7C2: XPCAPBASE (R/W) ---
    {
        uint32_t insn_w = encode_r_type(0x40, 0, 1, F3_TCSR, 0, XP_OPCODE);
        xplenum_execute(state, insn_w, 0x80000000, 0x02);
        TEST_ASSERT(state.xpcapbase == 0x80000000, "CSR write XPCAPBASE");

        uint32_t insn_r = encode_r_type(0x00, 0, 0, F3_TCSR, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn_r, 0, 0x02);
        TEST_ASSERT(r.rd_val == 0x80000000, "CSR read XPCAPBASE");
    }

    // --- CSR 0x7C3: XPCAPBOUND (R/W) ---
    {
        uint32_t insn_w = encode_r_type(0x40, 0, 1, F3_TCSR, 0, XP_OPCODE);
        xplenum_execute(state, insn_w, 0x90000000, 0x03);
        TEST_ASSERT(state.xpcapbound == 0x90000000, "CSR write XPCAPBOUND");

        uint32_t insn_r = encode_r_type(0x00, 0, 0, F3_TCSR, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn_r, 0, 0x03);
        TEST_ASSERT(r.rd_val == 0x90000000, "CSR read XPCAPBOUND");
    }

    // --- CSR 0x7C4: XPMASK_SEED (R/W, write triggers DRBG re-instantiation) ---
    {
        state.drbg_instantiated = true;
        uint32_t insn_w = encode_r_type(0x40, 0, 1, F3_TCSR, 0, XP_OPCODE);
        xplenum_execute(state, insn_w, 0xDEADBEEF, 0x04);
        TEST_ASSERT(state.xpmask_seed == 0xDEADBEEF, "CSR write XPMASK_SEED");
        TEST_ASSERT(!state.drbg_instantiated, "XPMASK_SEED write resets DRBG instantiation");

        uint32_t insn_r = encode_r_type(0x00, 0, 0, F3_TCSR, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn_r, 0, 0x04);
        TEST_ASSERT(r.rd_val == 0xDEADBEEF, "CSR read XPMASK_SEED");
    }

    // --- CSR 0x7C5: XPMASK_STATE (RO) ---
    {
        state.xpmask_state = 0x12345678;
        uint32_t insn_r = encode_r_type(0x00, 0, 0, F3_TCSR, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn_r, 0, 0x05);
        TEST_ASSERT(r.rd_val == 0x12345678, "CSR read XPMASK_STATE (RO)");
    }

    // --- CSR 0x7C6: XPTRIT_MODE (R/W) ---
    {
        uint32_t insn_w = encode_r_type(0x40, 0, 1, F3_TCSR, 0, XP_OPCODE);
        xplenum_execute(state, insn_w, 0x02, 0x06);
        TEST_ASSERT(state.xptrit_mode == 0x02, "CSR write XPTRIT_MODE");

        uint32_t insn_r = encode_r_type(0x00, 0, 0, F3_TCSR, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn_r, 0, 0x06);
        TEST_ASSERT(r.rd_val == 0x02, "CSR read XPTRIT_MODE");
    }

    // --- CSR 0x7C7: XPSIG_CFG (R/W) ---
    {
        uint32_t insn_w = encode_r_type(0x40, 0, 1, F3_TCSR, 0, XP_OPCODE);
        xplenum_execute(state, insn_w, 0xFF00FF, 0x07);
        TEST_ASSERT(state.xpsig_cfg == 0xFF00FF, "CSR write XPSIG_CFG");

        uint32_t insn_r = encode_r_type(0x00, 0, 0, F3_TCSR, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn_r, 0, 0x07);
        TEST_ASSERT(r.rd_val == 0xFF00FF, "CSR read XPSIG_CFG");
    }

    // --- CSR 0x7C8: XPEXC_CAUSE (RO) ---
    {
        state.xpexc_cause = XP_EXC_DOM_VIOLATION;
        uint32_t insn_r = encode_r_type(0x00, 0, 0, F3_TCSR, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn_r, 0, 0x08);
        TEST_ASSERT(r.rd_val == XP_EXC_DOM_VIOLATION, "CSR read XPEXC_CAUSE (RO)");
    }

    // --- CSR 0x7C9: XPEXC_ADDR (RO) ---
    {
        state.xpexc_addr = 0xDEAD0000;
        uint32_t insn_r = encode_r_type(0x00, 0, 0, F3_TCSR, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn_r, 0, 0x09);
        TEST_ASSERT(r.rd_val == 0xDEAD0000, "CSR read XPEXC_ADDR (RO)");
    }

    // --- CSR 0x7CA: XPPERF_CNT (R/W) ---
    {
        uint32_t insn_w = encode_r_type(0x40, 0, 1, F3_TCSR, 0, XP_OPCODE);
        xplenum_execute(state, insn_w, 0, 0x0A);
        TEST_ASSERT(state.xpperf_cnt == 0, "CSR write XPPERF_CNT (reset to 0)");

        uint32_t insn_r = encode_r_type(0x00, 0, 0, F3_TCSR, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn_r, 0, 0x0A);
        TEST_ASSERT(r.rd_val == 0, "CSR read XPPERF_CNT");
    }

    // --- CSR 0x7CB: XPVERSION (RO, always 0x010000) ---
    {
        uint32_t insn_r = encode_r_type(0x00, 0, 0, F3_TCSR, 3, XP_OPCODE);
        auto r = xplenum_execute(state, insn_r, 0, 0x0B);
        TEST_ASSERT(r.rd_val == XP_VERSION, "CSR read XPVERSION = 0x010000");
        TEST_ASSERT(r.rd_val == 0x010000, "CSR XPVERSION constant value");
    }

    // --- xplenum_csr_read/xplenum_csr_write API tests ---
    {
        xplenum_csr_write(state, CSR_XPSTATUS, 0x07);
        TEST_ASSERT(xplenum_csr_read(state, CSR_XPSTATUS) == 0x07, "csr_read/write XPSTATUS");

        xplenum_csr_write(state, CSR_XPDOMID, 0x55);
        TEST_ASSERT(xplenum_csr_read(state, CSR_XPDOMID) == 0x55, "csr_read/write XPDOMID");

        TEST_ASSERT(xplenum_csr_read(state, CSR_XPVERSION) == XP_VERSION, "csr_read XPVERSION via API");

        xplenum_csr_write(state, CSR_XPMASK_SEED, 0x11111111);
        TEST_ASSERT(xplenum_csr_read(state, CSR_XPMASK_SEED) == 0x11111111, "csr_read/write XPMASK_SEED via API");
        TEST_ASSERT(!state.drbg_instantiated, "XPMASK_SEED API write resets DRBG");

        TEST_ASSERT(xplenum_csr_read(state, CSR_XPMASK_STATE) == state.xpmask_state, "csr_read XPMASK_STATE via API");
        TEST_ASSERT(xplenum_csr_read(state, CSR_XPEXC_CAUSE) == state.xpexc_cause, "csr_read XPEXC_CAUSE via API");
        TEST_ASSERT(xplenum_csr_read(state, CSR_XPEXC_ADDR) == state.xpexc_addr, "csr_read XPEXC_ADDR via API");

        xplenum_csr_write(state, CSR_XPPERF_CNT, 42);
        TEST_ASSERT(xplenum_csr_read(state, CSR_XPPERF_CNT) == 42, "csr_read/write XPPERF_CNT via API");

        xplenum_csr_write(state, CSR_XPTRIT_MODE, 0x03);
        TEST_ASSERT(xplenum_csr_read(state, CSR_XPTRIT_MODE) == 0x03, "csr_read/write XPTRIT_MODE via API");

        xplenum_csr_write(state, CSR_XPSIG_CFG, 0xBEEF);
        TEST_ASSERT(xplenum_csr_read(state, CSR_XPSIG_CFG) == 0xBEEF, "csr_read/write XPSIG_CFG via API");

        xplenum_csr_write(state, CSR_XPCAPBASE, 0x40000000);
        TEST_ASSERT(xplenum_csr_read(state, CSR_XPCAPBASE) == 0x40000000, "csr_read/write XPCAPBASE via API");

        xplenum_csr_write(state, CSR_XPCAPBOUND, 0x50000000);
        TEST_ASSERT(xplenum_csr_read(state, CSR_XPCAPBOUND) == 0x50000000, "csr_read/write XPCAPBOUND via API");
    }

    // Restore status for subsequent tests
    state.xpstatus = 0x0F;
}

static void test_perf_counter(XPlenumState& state) {
    printf("\n=== Performance Counter Tests ===\n");

    uint32_t cnt_before = state.xpperf_cnt;
    uint32_t insn = encode_r_type(F7_TROTL, 2, 1, F3_TROT, 3, XP_OPCODE);
    xplenum_execute(state, insn, 1, 1);
    xplenum_execute(state, insn, 1, 1);
    xplenum_execute(state, insn, 1, 1);
    TEST_ASSERT(state.xpperf_cnt == cnt_before + 3, "Perf counter increments per instruction");
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------
int main() {
    printf("XPlenum Spike ISS — Instruction Validation Suite\n");
    printf("================================================\n");

    XPlenumState state;

    test_masking(state);
    test_domain(state);
    test_capability(state);
    test_crypto_rotation(state);
    test_trit_encoding(state);
    test_signal(state);
    test_csr_comprehensive(state);
    test_perf_counter(state);

    printf("\n================================================\n");
    printf("Results: %d passed, %d failed (total: %d)\n",
           tests_passed, tests_failed, tests_passed + tests_failed);

    if (tests_failed > 0) {
        printf("FAIL — %d test(s) failed\n", tests_failed);
        return 1;
    }

    printf("PASS — All %d tests passed\n", tests_passed);
    return 0;
}
