// =============================================================================
// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// XPLENUM — Spike ISS Custom Instruction Extension
// Phase 6, Task 6.1a: Spike instruction handlers for all 21 XPlenum instructions
//
// This header defines the XPlenum Spike extension, enabling ISA-level validation
// of all 21 custom instructions before full-system QEMU integration.
// =============================================================================

#ifndef XPLENUM_SPIKE_EXTENSION_H
#define XPLENUM_SPIKE_EXTENSION_H

#include <cstdint>
#include <cstring>
#include <array>
#include <random>

// ---------------------------------------------------------------------------
// Opcode / funct3 / funct7 constants (mirror xplenum_pkg.vh)
// ---------------------------------------------------------------------------

static constexpr uint32_t XP_OPCODE = 0x0B; // custom-0

// funct3 groups
static constexpr uint32_t F3_TMASK = 0;
static constexpr uint32_t F3_TDOM  = 1;
static constexpr uint32_t F3_TCAP  = 2;
static constexpr uint32_t F3_TROT  = 3;
static constexpr uint32_t F3_TENC  = 4;
static constexpr uint32_t F3_TSIG  = 5;
static constexpr uint32_t F3_RSVD  = 6;
static constexpr uint32_t F3_TCSR  = 7;

// funct7 — Masking
static constexpr uint32_t F7_TMASK   = 0x00;
static constexpr uint32_t F7_TUNMASK = 0x01;
static constexpr uint32_t F7_TMASKR  = 0x02;
static constexpr uint32_t F7_TMASKRF = 0x03;

// funct7 — Domain
static constexpr uint32_t F7_TDOMSET = 0x00;
static constexpr uint32_t F7_TDOMCHK = 0x01;
static constexpr uint32_t F7_TDOMCLR = 0x02;
static constexpr uint32_t F7_TDOMXFR = 0x03;

// funct7 — Capability
static constexpr uint32_t F7_TCAPLD  = 0x00;
static constexpr uint32_t F7_TCAPCHK = 0x01;
static constexpr uint32_t F7_TCAPST  = 0x02;
static constexpr uint32_t F7_TCAPREV = 0x03;

// funct7 — Crypto/Rotation
static constexpr uint32_t F7_TROTL = 0x00;
static constexpr uint32_t F7_TROTR = 0x01;
static constexpr uint32_t F7_TTBOX = 0x02;
static constexpr uint32_t F7_TPERM = 0x03;

// funct7 — Encoding
static constexpr uint32_t F7_TTRIT   = 0x00;
static constexpr uint32_t F7_TDETRIT = 0x01;

// funct7 — Signal
static constexpr uint32_t F7_TSIGFLT = 0x00;
static constexpr uint32_t F7_TSIGCMP = 0x01;
static constexpr uint32_t F7_TSIGACC = 0x02;

// CSR addresses
static constexpr uint32_t CSR_XPSTATUS    = 0x7C0;
static constexpr uint32_t CSR_XPDOMID     = 0x7C1;
static constexpr uint32_t CSR_XPCAPBASE   = 0x7C2;
static constexpr uint32_t CSR_XPCAPBOUND  = 0x7C3;
static constexpr uint32_t CSR_XPMASK_SEED = 0x7C4;
static constexpr uint32_t CSR_XPMASK_STATE= 0x7C5;
static constexpr uint32_t CSR_XPTRIT_MODE = 0x7C6;
static constexpr uint32_t CSR_XPSIG_CFG   = 0x7C7;
static constexpr uint32_t CSR_XPEXC_CAUSE = 0x7C8;
static constexpr uint32_t CSR_XPEXC_ADDR  = 0x7C9;
static constexpr uint32_t CSR_XPPERF_CNT  = 0x7CA;
static constexpr uint32_t CSR_XPVERSION   = 0x7CB;

// Status register bits
static constexpr uint32_t XPSTATUS_MASK_EN = (1u << 0);
static constexpr uint32_t XPSTATUS_DOM_EN  = (1u << 1);
static constexpr uint32_t XPSTATUS_CAP_EN  = (1u << 2);
static constexpr uint32_t XPSTATUS_SIG_EN  = (1u << 3);

// Exception codes
static constexpr uint32_t XP_EXC_NONE          = 0x0;
static constexpr uint32_t XP_EXC_DOM_VIOLATION = 0x1;
static constexpr uint32_t XP_EXC_CAP_INVALID   = 0x2;
static constexpr uint32_t XP_EXC_CAP_REVOKED   = 0x3;
static constexpr uint32_t XP_EXC_CAP_BOUNDS    = 0x4;
static constexpr uint32_t XP_EXC_MASK_FAULT    = 0x5;
static constexpr uint32_t XP_EXC_TRIT_OVERFLOW = 0x6;
static constexpr uint32_t XP_EXC_PRIV_FAULT    = 0x7;

// Version constant
static constexpr uint32_t XP_VERSION = 0x010000; // v1.0.0

// ---------------------------------------------------------------------------
// Ternary S-Box (GF(3) substitution table, 243 entries for 5-trit input)
// Precomputed table matching xplenum_trit_unit.v
// ---------------------------------------------------------------------------
static constexpr uint8_t TRIT_SBOX[243] = {
    2,0,1,1,2,0,0,1,2,1,0,2,2,1,0,0,2,1,0,1,2,2,0,1,1,2,0,
    0,2,1,2,0,1,1,2,0,2,1,0,0,2,1,1,0,2,1,2,0,0,1,2,2,0,1,
    1,0,2,0,1,2,2,0,1,0,2,1,1,0,2,2,1,0,2,0,1,1,2,0,0,1,2,
    2,1,0,1,2,0,0,1,2,1,0,2,2,1,0,0,2,1,0,1,2,2,0,1,1,2,0,
    0,2,1,2,0,1,1,2,0,2,1,0,0,2,1,1,0,2,1,2,0,0,1,2,2,0,1,
    1,0,2,0,1,2,2,0,1,0,2,1,1,0,2,2,1,0,2,0,1,1,2,0,0,1,2,
    2,1,0,1,2,0,0,1,2,1,0,2,2,1,0,0,2,1,0,1,2,2,0,1,1,2,0,
    0,2,1,2,0,1,1,2,0,2,1,0,0,2,1,1,0,2,1,2,0,0,1,2,2,0,1,
    1,0,2,0,1,2,2,0,1,0,2,1,1,0,2,2,1,0,2,0,1,1,2,0,0,1,2
};

// ---------------------------------------------------------------------------
// XPlenum Spike State — emulator-side hardware state
// ---------------------------------------------------------------------------
struct XPlenumState {
    // CSR registers
    uint32_t xpstatus;
    uint32_t xpdomid;
    uint32_t xpcapbase;
    uint32_t xpcapbound;
    uint32_t xpmask_seed;
    uint32_t xpmask_state;
    uint32_t xptrit_mode;
    uint32_t xpsig_cfg;
    uint32_t xpexc_cause;
    uint32_t xpexc_addr;
    uint32_t xpperf_cnt;

    // Domain isolation table (256 entries, 8-bit domain ID)
    std::array<uint32_t, 256> domain_table;

    // Capability table (64 entries: base, bound, permissions, valid, revoked)
    struct CapEntry {
        uint32_t base;
        uint32_t bound;
        uint32_t perms;
        bool valid;
        bool revoked;
    };
    std::array<CapEntry, 64> cap_table;

    // Signal accumulator
    uint32_t sig_accumulator;

    // DRBG state
    //
    // EMULATOR SIMPLIFICATION NOTE (vs RTL xplenum_ctr_drbg.v):
    //   The RTL CTR_DRBG implements full SP 800-90A with AES-256, including:
    //     - 14-cycle AES pipeline latency (drbg_ready_o deasserted during generate)
    //     - SP 800-90B health tests (repetition count cutoff=5, adaptive proportion
    //       window=64 cutoff=9) with drbg_health_err_o output
    //     - TMASKR/TMASKRF gated on drbg_buffer_valid in RTL
    //   This emulator uses a simplified PRNG for functional ISA-level validation.
    //   For RTL-accurate DRBG behavior (timing, health gating), use the Verilator
    //   RTL simulation or cross-verify against RTL traces.
    //
    //   Differences from RTL:
    //     1. TMASKR always completes in 1 emulated cycle (RTL: 15 cycles)
    //     2. No health test gating (drbg_health_err never asserted in emulator)
    //     3. DRBG output distribution differs (emulator is NOT NIST-compliant)
    //
    uint32_t drbg_v[4];
    uint32_t drbg_key[8];
    uint32_t drbg_reseed_counter;
    bool     drbg_instantiated;
    bool     drbg_ready;       // mirrors RTL drbg_ready_o (always true in emulator)
    bool     drbg_health_err;  // mirrors RTL drbg_health_err_o (always false in emulator)

    // RNG for mask generation
    std::mt19937 rng;

    XPlenumState() { reset(); }

    void reset() {
        xpstatus    = 0;
        xpdomid     = 0;
        xpcapbase   = 0;
        xpcapbound  = 0;
        xpmask_seed = 0;
        xpmask_state= 0;
        xptrit_mode = 0;
        xpsig_cfg   = 0;
        xpexc_cause = 0;
        xpexc_addr  = 0;
        xpperf_cnt  = 0;
        sig_accumulator = 0;
        drbg_reseed_counter = 0;
        drbg_instantiated = false;
        drbg_ready = true;
        drbg_health_err = false;
        domain_table.fill(0);
        for (auto& c : cap_table) {
            c.base = c.bound = c.perms = 0;
            c.valid = false;
            c.revoked = false;
        }
        std::memset(drbg_v, 0, sizeof(drbg_v));
        std::memset(drbg_key, 0, sizeof(drbg_key));
        rng.seed(0xDEADBEEF);
    }

    // Generate DRBG output (simplified model — matches behavioral semantics)
    uint32_t drbg_generate() {
        if (!drbg_instantiated) {
            drbg_v[0] = xpmask_seed;
            drbg_v[1] = ~xpmask_seed;
            drbg_v[2] = xpmask_seed ^ 0xA5A5A5A5;
            drbg_v[3] = xpmask_seed + 1;
            drbg_instantiated = true;
        }
        // Simplified CTR increment + mix
        drbg_v[0]++;
        uint32_t out = drbg_v[0] ^ drbg_v[1] ^ drbg_v[2] ^ drbg_v[3];
        drbg_v[1] = (drbg_v[1] << 13) | (drbg_v[1] >> 19);
        drbg_v[2] ^= drbg_v[0];
        drbg_v[3] += out;
        drbg_reseed_counter++;
        xpmask_state = out;
        return out;
    }
};

// ---------------------------------------------------------------------------
// Instruction field extraction
// ---------------------------------------------------------------------------
static inline uint32_t xp_opcode(uint32_t insn) { return insn & 0x7F; }
static inline uint32_t xp_rd(uint32_t insn)     { return (insn >> 7) & 0x1F; }
static inline uint32_t xp_funct3(uint32_t insn) { return (insn >> 12) & 0x7; }
static inline uint32_t xp_rs1(uint32_t insn)    { return (insn >> 15) & 0x1F; }
static inline uint32_t xp_rs2(uint32_t insn)    { return (insn >> 20) & 0x1F; }
static inline uint32_t xp_funct7(uint32_t insn) { return (insn >> 25) & 0x7F; }

// ---------------------------------------------------------------------------
// GF(3) rotate left/right helpers
// ---------------------------------------------------------------------------
static inline uint32_t ternary_rotl(uint32_t val, uint32_t shift) {
    shift &= 0x1F;
    return (val << shift) | (val >> (32 - shift));
}

static inline uint32_t ternary_rotr(uint32_t val, uint32_t shift) {
    shift &= 0x1F;
    return (val >> shift) | (val << (32 - shift));
}

// Binary to balanced ternary encoding
static inline uint32_t binary_to_trit(uint32_t val) {
    uint32_t result = 0;
    for (int i = 0; i < 16; i++) {
        uint32_t pair = (val >> (i * 2)) & 0x3;
        if (pair == 3) pair = 2; // clamp invalid trit
        result |= (pair << (i * 2));
    }
    return result;
}

// Balanced ternary to binary decoding
static inline uint32_t trit_to_binary(uint32_t val) {
    uint32_t result = 0;
    for (int i = 0; i < 16; i++) {
        uint32_t trit = (val >> (i * 2)) & 0x3;
        if (trit <= 2) {
            result |= (trit << (i * 2));
        }
    }
    return result;
}

// Ternary permutation
static inline uint32_t ternary_perm(uint32_t val, uint32_t key) {
    uint32_t result = 0;
    for (int i = 0; i < 16; i++) {
        uint32_t src_pos = ((key >> (i * 2)) & 0x3) % 16;
        uint32_t trit = (val >> (src_pos * 2)) & 0x3;
        result |= (trit << (i * 2));
    }
    return result;
}

// T-Box substitution
static inline uint32_t trit_sbox_lookup(uint32_t val) {
    uint32_t idx = val & 0xFF;
    if (idx < 243) {
        return TRIT_SBOX[idx];
    }
    return val ^ 0xFF;
}

// Signal filter (FIR-like)
static inline uint32_t signal_filter(uint32_t signal, uint32_t coeffs) {
    int32_t s = static_cast<int32_t>(signal);
    int32_t c = static_cast<int32_t>(coeffs);
    int32_t c0 = (c >> 0) & 0xFF;
    int32_t c1 = (c >> 8) & 0xFF;
    int32_t c2 = (c >> 16) & 0xFF;
    int32_t c3 = (c >> 24) & 0xFF;
    int32_t s0 = (s >> 0) & 0xFF;
    int32_t s1 = (s >> 8) & 0xFF;
    int32_t s2 = (s >> 16) & 0xFF;
    int32_t s3 = (s >> 24) & 0xFF;
    int32_t result = (s0 * c0 + s1 * c1 + s2 * c2 + s3 * c3) >> 8;
    return static_cast<uint32_t>(result) & 0xFFFFFFFF;
}

// Signal compare
static inline uint32_t signal_compare(uint32_t a, uint32_t b) {
    int32_t diff = static_cast<int32_t>(a) - static_cast<int32_t>(b);
    if (diff > 0) return 1;
    if (diff < 0) return 0xFFFFFFFF; // -1 in two's complement
    return 0;
}

// ---------------------------------------------------------------------------
// Main instruction execution function
// Returns: {result, exception_code, writes_rd}
// ---------------------------------------------------------------------------
struct XPlenumExecResult {
    uint32_t rd_val;
    uint32_t exc_code;
    bool     writes_rd;
};

static inline XPlenumExecResult xplenum_execute(
    XPlenumState& state,
    uint32_t insn,
    uint32_t rs1_val,
    uint32_t rs2_val
) {
    XPlenumExecResult res = {0, XP_EXC_NONE, false};
    uint32_t f3 = xp_funct3(insn);
    uint32_t f7 = xp_funct7(insn);

    state.xpperf_cnt++;

    switch (f3) {
    // ==== MASKING (funct3 = 000) ====
    case F3_TMASK:
        if (!(state.xpstatus & XPSTATUS_MASK_EN)) {
            res.exc_code = XP_EXC_MASK_FAULT;
            state.xpexc_cause = XP_EXC_MASK_FAULT;
            return res;
        }
        switch (f7) {
        case F7_TMASK:   // TMASK: apply mask
            res.rd_val = rs1_val ^ rs2_val;
            res.writes_rd = true;
            break;
        case F7_TUNMASK: // TUNMASK: remove mask
            res.rd_val = rs1_val ^ rs2_val;
            res.writes_rd = true;
            break;
        case F7_TMASKR:  // TMASKR: generate random mask from DRBG
            res.rd_val = state.drbg_generate();
            res.writes_rd = true;
            break;
        case F7_TMASKRF: // TMASKRF: refresh mask
            res.rd_val = rs1_val ^ state.drbg_generate();
            res.writes_rd = true;
            break;
        default:
            res.exc_code = XP_EXC_PRIV_FAULT;
            state.xpexc_cause = XP_EXC_PRIV_FAULT;
            break;
        }
        break;

    // ==== DOMAIN ISOLATION (funct3 = 001) ====
    case F3_TDOM:
        if (!(state.xpstatus & XPSTATUS_DOM_EN)) {
            res.exc_code = XP_EXC_DOM_VIOLATION;
            state.xpexc_cause = XP_EXC_DOM_VIOLATION;
            return res;
        }
        switch (f7) {
        case F7_TDOMSET: // Set domain tag
            state.domain_table[rs1_val & 0xFF] = rs2_val;
            break;
        case F7_TDOMCHK: { // Check domain permission
            uint32_t tag = state.domain_table[rs1_val & 0xFF];
            if (tag != (rs2_val & 0xFFFFFFFF)) {
                res.exc_code = XP_EXC_DOM_VIOLATION;
                state.xpexc_cause = XP_EXC_DOM_VIOLATION;
            }
            res.rd_val = (tag == rs2_val) ? 1 : 0;
            res.writes_rd = true;
            break;
        }
        case F7_TDOMCLR: // Clear domain tag
            state.domain_table[rs1_val & 0xFF] = 0;
            break;
        case F7_TDOMXFR: // Transfer domain ownership
            state.domain_table[rs2_val & 0xFF] = state.domain_table[rs1_val & 0xFF];
            state.domain_table[rs1_val & 0xFF] = 0;
            break;
        default:
            res.exc_code = XP_EXC_PRIV_FAULT;
            state.xpexc_cause = XP_EXC_PRIV_FAULT;
            break;
        }
        break;

    // ==== CAPABILITY (funct3 = 010) ====
    case F3_TCAP:
        if (!(state.xpstatus & XPSTATUS_CAP_EN)) {
            res.exc_code = XP_EXC_CAP_INVALID;
            state.xpexc_cause = XP_EXC_CAP_INVALID;
            return res;
        }
        switch (f7) {
        case F7_TCAPLD: { // Load capability
            uint32_t idx = rs1_val & 0x3F;
            if (!state.cap_table[idx].valid) {
                res.exc_code = XP_EXC_CAP_INVALID;
                state.xpexc_cause = XP_EXC_CAP_INVALID;
            } else if (state.cap_table[idx].revoked) {
                res.exc_code = XP_EXC_CAP_REVOKED;
                state.xpexc_cause = XP_EXC_CAP_REVOKED;
            } else {
                res.rd_val = state.cap_table[idx].perms;
                res.writes_rd = true;
            }
            break;
        }
        case F7_TCAPCHK: { // Check capability
            uint32_t idx = rs1_val & 0x3F;
            if (!state.cap_table[idx].valid) {
                res.rd_val = 0;
            } else if (state.cap_table[idx].revoked) {
                res.rd_val = 0;
            } else {
                uint32_t addr = rs2_val;
                bool in_bounds = (addr >= state.cap_table[idx].base &&
                                  addr < state.cap_table[idx].bound);
                res.rd_val = in_bounds ? 1 : 0;
            }
            res.writes_rd = true;
            break;
        }
        case F7_TCAPST: { // Store/create capability
            uint32_t idx = rs1_val & 0x3F;
            state.cap_table[idx].base  = rs2_val;
            state.cap_table[idx].bound = rs2_val + 0x1000;
            state.cap_table[idx].perms = 0x7; // RWX
            state.cap_table[idx].valid = true;
            state.cap_table[idx].revoked = false;
            break;
        }
        case F7_TCAPREV: { // Revoke capability
            uint32_t idx = rs1_val & 0x3F;
            if (state.cap_table[idx].valid) {
                state.cap_table[idx].revoked = true;
                res.rd_val = 1;
            } else {
                res.rd_val = 0;
            }
            res.writes_rd = true;
            break;
        }
        default:
            res.exc_code = XP_EXC_PRIV_FAULT;
            state.xpexc_cause = XP_EXC_PRIV_FAULT;
            break;
        }
        break;

    // ==== CRYPTO / ROTATION (funct3 = 011) ====
    case F3_TROT:
        switch (f7) {
        case F7_TROTL:
            res.rd_val = ternary_rotl(rs1_val, rs2_val);
            res.writes_rd = true;
            break;
        case F7_TROTR:
            res.rd_val = ternary_rotr(rs1_val, rs2_val);
            res.writes_rd = true;
            break;
        case F7_TTBOX:
            res.rd_val = trit_sbox_lookup(rs1_val);
            res.writes_rd = true;
            break;
        case F7_TPERM:
            res.rd_val = ternary_perm(rs1_val, rs2_val);
            res.writes_rd = true;
            break;
        default:
            res.exc_code = XP_EXC_PRIV_FAULT;
            state.xpexc_cause = XP_EXC_PRIV_FAULT;
            break;
        }
        break;

    // ==== TRIT ENCODING (funct3 = 100) ====
    case F3_TENC:
        switch (f7) {
        case F7_TTRIT:
            res.rd_val = binary_to_trit(rs1_val);
            res.writes_rd = true;
            break;
        case F7_TDETRIT:
            res.rd_val = trit_to_binary(rs1_val);
            res.writes_rd = true;
            break;
        default:
            res.exc_code = XP_EXC_TRIT_OVERFLOW;
            state.xpexc_cause = XP_EXC_TRIT_OVERFLOW;
            break;
        }
        break;

    // ==== SIGNAL PROCESSING (funct3 = 101) ====
    case F3_TSIG:
        if (!(state.xpstatus & XPSTATUS_SIG_EN)) {
            res.exc_code = XP_EXC_PRIV_FAULT;
            state.xpexc_cause = XP_EXC_PRIV_FAULT;
            return res;
        }
        switch (f7) {
        case F7_TSIGFLT:
            res.rd_val = signal_filter(rs1_val, rs2_val);
            res.writes_rd = true;
            break;
        case F7_TSIGCMP:
            res.rd_val = signal_compare(rs1_val, rs2_val);
            res.writes_rd = true;
            break;
        case F7_TSIGACC:
            state.sig_accumulator += rs1_val * rs2_val;
            res.rd_val = state.sig_accumulator;
            res.writes_rd = true;
            break;
        default:
            res.exc_code = XP_EXC_PRIV_FAULT;
            state.xpexc_cause = XP_EXC_PRIV_FAULT;
            break;
        }
        break;

    // ==== CSR ACCESS (funct3 = 111) ====
    case F3_TCSR: {
        uint32_t csr_idx = rs2_val & 0xF;
        bool is_write = (f7 >> 6) & 1;
        if (is_write) {
            switch (csr_idx) {
            case 0: state.xpstatus    = rs1_val; break;
            case 1: state.xpdomid     = rs1_val; break;
            case 2: state.xpcapbase   = rs1_val; break;
            case 3: state.xpcapbound  = rs1_val; break;
            case 4: state.xpmask_seed = rs1_val;
                    state.drbg_instantiated = false; break;
            case 6: state.xptrit_mode = rs1_val; break;
            case 7: state.xpsig_cfg   = rs1_val; break;
            case 0xA: state.xpperf_cnt = rs1_val; break;
            default: break;
            }
        } else {
            switch (csr_idx) {
            case 0:  res.rd_val = state.xpstatus;    break;
            case 1:  res.rd_val = state.xpdomid;     break;
            case 2:  res.rd_val = state.xpcapbase;   break;
            case 3:  res.rd_val = state.xpcapbound;  break;
            case 4:  res.rd_val = state.xpmask_seed; break;
            case 5:  res.rd_val = state.xpmask_state; break;
            case 6:  res.rd_val = state.xptrit_mode; break;
            case 7:  res.rd_val = state.xpsig_cfg;   break;
            case 8:  res.rd_val = state.xpexc_cause; break;
            case 9:  res.rd_val = state.xpexc_addr;  break;
            case 0xA: res.rd_val = state.xpperf_cnt; break;
            case 0xB: res.rd_val = XP_VERSION;       break;
            default: res.rd_val = 0;                 break;
            }
            res.writes_rd = true;
        }
        break;
    }

    default:
        res.exc_code = XP_EXC_PRIV_FAULT;
        state.xpexc_cause = XP_EXC_PRIV_FAULT;
        break;
    }

    return res;
}

// ---------------------------------------------------------------------------
// CSR read/write interface (for Spike CSR hooking)
// ---------------------------------------------------------------------------
static inline uint32_t xplenum_csr_read(XPlenumState& state, uint32_t addr) {
    switch (addr) {
    case CSR_XPSTATUS:    return state.xpstatus;
    case CSR_XPDOMID:     return state.xpdomid;
    case CSR_XPCAPBASE:   return state.xpcapbase;
    case CSR_XPCAPBOUND:  return state.xpcapbound;
    case CSR_XPMASK_SEED: return state.xpmask_seed;
    case CSR_XPMASK_STATE:return state.xpmask_state;
    case CSR_XPTRIT_MODE: return state.xptrit_mode;
    case CSR_XPSIG_CFG:   return state.xpsig_cfg;
    case CSR_XPEXC_CAUSE: return state.xpexc_cause;
    case CSR_XPEXC_ADDR:  return state.xpexc_addr;
    case CSR_XPPERF_CNT:  return state.xpperf_cnt;
    case CSR_XPVERSION:   return XP_VERSION;
    default:              return 0;
    }
}

static inline void xplenum_csr_write(XPlenumState& state, uint32_t addr, uint32_t val) {
    switch (addr) {
    case CSR_XPSTATUS:    state.xpstatus    = val; break;
    case CSR_XPDOMID:     state.xpdomid     = val; break;
    case CSR_XPCAPBASE:   state.xpcapbase   = val; break;
    case CSR_XPCAPBOUND:  state.xpcapbound  = val; break;
    case CSR_XPMASK_SEED: state.xpmask_seed = val;
                          state.drbg_instantiated = false; break;
    case CSR_XPTRIT_MODE: state.xptrit_mode = val; break;
    case CSR_XPSIG_CFG:   state.xpsig_cfg   = val; break;
    case CSR_XPPERF_CNT:  state.xpperf_cnt  = val; break;
    default: break;
    }
}

#endif // XPLENUM_SPIKE_EXTENSION_H
