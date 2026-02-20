// =============================================================================
// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// XPLENUM — QEMU TCG Helper Functions
// Phase 6, Task 6.1b: QEMU RISC-V target extension for all 21 XPlenum instructions
//
// Integration path:
//   1. Copy this file to qemu/target/riscv/xplenum_helper.c
//   2. Add instruction decode to qemu/target/riscv/insn_trans/trans_xplenum.c.inc
//   3. Register helpers in qemu/target/riscv/helper.h
//   4. Update qemu/target/riscv/meson.build
//
// Each helper implements the exact semantics from xplenum_pkg.vh and
// xplenum_top.v, enabling full-system emulation with kernel boot.
// =============================================================================

#include <stdint.h>
#include <string.h>

// ---------------------------------------------------------------------------
// XPlenum emulation state (stored in CPURISCVState extension)
// ---------------------------------------------------------------------------

#define XP_NUM_DOMAINS    256
#define XP_NUM_CAPS       64
#define XP_VERSION_CONST  0x010000

typedef struct XPlenumCapEntry {
    uint32_t base;
    uint32_t bound;
    uint32_t perms;
    uint8_t  valid;
    uint8_t  revoked;
} XPlenumCapEntry;

typedef struct XPlenumEmulState {
    // CSR file
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

    // Domain table
    uint32_t domain_table[XP_NUM_DOMAINS];

    // Capability table
    XPlenumCapEntry cap_table[XP_NUM_CAPS];

    // Signal accumulator
    uint32_t sig_accumulator;

    // DRBG state
    //
    // EMULATOR SIMPLIFICATION NOTE (vs RTL xplenum_ctr_drbg.v):
    //   RTL implements full SP 800-90A CTR_DRBG with AES-256:
    //     - 14-cycle AES pipeline, drbg_ready_o deasserted during generate
    //     - SP 800-90B health tests (rep count cutoff=5, adaptive proportion
    //       window=64, cutoff=9) with drbg_health_err_o
    //     - TMASKR/TMASKRF gated on drbg_buffer_valid
    //   This QEMU helper uses a simplified PRNG for ISA-level validation.
    //   Differences from RTL:
    //     1. TMASKR completes in 1 emulated step (RTL: 15 cycles)
    //     2. No health test gating (drbg_health_err never asserted)
    //     3. Output distribution is NOT NIST-compliant
    //   For RTL-accurate behavior, use Verilator simulation or
    //   cross-verify against RTL traces.
    //
    uint32_t drbg_v[4];
    uint32_t drbg_counter;
    uint8_t  drbg_init;
    uint8_t  drbg_ready;      // mirrors RTL drbg_ready_o (always 1 in emulator)
    uint8_t  drbg_health_err; // mirrors RTL drbg_health_err_o (always 0 in emulator)
} XPlenumEmulState;

// ---------------------------------------------------------------------------
// DRBG simplified model
// ---------------------------------------------------------------------------
static uint32_t xp_drbg_generate(XPlenumEmulState *s) {
    if (!s->drbg_init) {
        s->drbg_v[0] = s->xpmask_seed;
        s->drbg_v[1] = ~s->xpmask_seed;
        s->drbg_v[2] = s->xpmask_seed ^ 0xA5A5A5A5;
        s->drbg_v[3] = s->xpmask_seed + 1;
        s->drbg_init = 1;
    }
    s->drbg_v[0]++;
    uint32_t out = s->drbg_v[0] ^ s->drbg_v[1] ^ s->drbg_v[2] ^ s->drbg_v[3];
    s->drbg_v[1] = (s->drbg_v[1] << 13) | (s->drbg_v[1] >> 19);
    s->drbg_v[2] ^= s->drbg_v[0];
    s->drbg_v[3] += out;
    s->drbg_counter++;
    s->xpmask_state = out;
    return out;
}

// ---------------------------------------------------------------------------
// Ternary S-Box (matches RTL)
// ---------------------------------------------------------------------------
static const uint8_t trit_sbox[243] = {
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
// TCG Helper Prototypes (registered in helper.h as):
//
// DEF_HELPER_3(xplenum_tmask,    tl, env, tl, tl)
// DEF_HELPER_3(xplenum_tunmask,  tl, env, tl, tl)
// DEF_HELPER_1(xplenum_tmaskr,   tl, env)
// DEF_HELPER_2(xplenum_tmaskrf,  tl, env, tl)
// DEF_HELPER_3(xplenum_tdomset,  void, env, tl, tl)
// DEF_HELPER_3(xplenum_tdomchk,  tl, env, tl, tl)
// DEF_HELPER_2(xplenum_tdomclr,  void, env, tl)
// DEF_HELPER_3(xplenum_tdomxfr,  void, env, tl, tl)
// DEF_HELPER_2(xplenum_tcapld,   tl, env, tl)
// DEF_HELPER_3(xplenum_tcapchk,  tl, env, tl, tl)
// DEF_HELPER_3(xplenum_tcapst,   void, env, tl, tl)
// DEF_HELPER_2(xplenum_tcaprev,  tl, env, tl)
// DEF_HELPER_3(xplenum_trotl,    tl, env, tl, tl)
// DEF_HELPER_3(xplenum_trotr,    tl, env, tl, tl)
// DEF_HELPER_2(xplenum_ttbox,    tl, env, tl)
// DEF_HELPER_3(xplenum_tperm,    tl, env, tl, tl)
// DEF_HELPER_2(xplenum_ttrit,    tl, env, tl)
// DEF_HELPER_2(xplenum_tdetrit,  tl, env, tl)
// DEF_HELPER_3(xplenum_tsigflt,  tl, env, tl, tl)
// DEF_HELPER_3(xplenum_tsigcmp,  tl, env, tl, tl)
// DEF_HELPER_3(xplenum_tsigacc,  tl, env, tl, tl)
// ---------------------------------------------------------------------------

// In actual QEMU integration, XPlenumEmulState would be accessed via:
//   XPlenumEmulState *xps = &env->xplenum_state;
// For this reference implementation, we use a static instance.

static XPlenumEmulState g_xps = {0};

// ===== Masking =====

uint64_t helper_xplenum_tmask(void *env, uint64_t rs1, uint64_t rs2) {
    XPlenumEmulState *s = &g_xps;
    s->xpperf_cnt++;
    if (!(s->xpstatus & 0x1)) { s->xpexc_cause = 0x5; return 0; }
    return rs1 ^ rs2;
}

uint64_t helper_xplenum_tunmask(void *env, uint64_t rs1, uint64_t rs2) {
    XPlenumEmulState *s = &g_xps;
    s->xpperf_cnt++;
    if (!(s->xpstatus & 0x1)) { s->xpexc_cause = 0x5; return 0; }
    return rs1 ^ rs2;
}

uint64_t helper_xplenum_tmaskr(void *env) {
    XPlenumEmulState *s = &g_xps;
    s->xpperf_cnt++;
    if (!(s->xpstatus & 0x1)) { s->xpexc_cause = 0x5; return 0; }
    return xp_drbg_generate(s);
}

uint64_t helper_xplenum_tmaskrf(void *env, uint64_t rs1) {
    XPlenumEmulState *s = &g_xps;
    s->xpperf_cnt++;
    if (!(s->xpstatus & 0x1)) { s->xpexc_cause = 0x5; return 0; }
    return rs1 ^ xp_drbg_generate(s);
}

// ===== Domain Isolation =====

void helper_xplenum_tdomset(void *env, uint64_t rs1, uint64_t rs2) {
    XPlenumEmulState *s = &g_xps;
    s->xpperf_cnt++;
    if (!(s->xpstatus & 0x2)) { s->xpexc_cause = 0x1; return; }
    s->domain_table[rs1 & 0xFF] = (uint32_t)rs2;
}

uint64_t helper_xplenum_tdomchk(void *env, uint64_t rs1, uint64_t rs2) {
    XPlenumEmulState *s = &g_xps;
    s->xpperf_cnt++;
    if (!(s->xpstatus & 0x2)) { s->xpexc_cause = 0x1; return 0; }
    uint32_t tag = s->domain_table[rs1 & 0xFF];
    if (tag != (uint32_t)rs2) {
        s->xpexc_cause = 0x1;
        return 0;
    }
    return 1;
}

void helper_xplenum_tdomclr(void *env, uint64_t rs1) {
    XPlenumEmulState *s = &g_xps;
    s->xpperf_cnt++;
    if (!(s->xpstatus & 0x2)) { s->xpexc_cause = 0x1; return; }
    s->domain_table[rs1 & 0xFF] = 0;
}

void helper_xplenum_tdomxfr(void *env, uint64_t rs1, uint64_t rs2) {
    XPlenumEmulState *s = &g_xps;
    s->xpperf_cnt++;
    if (!(s->xpstatus & 0x2)) { s->xpexc_cause = 0x1; return; }
    s->domain_table[rs2 & 0xFF] = s->domain_table[rs1 & 0xFF];
    s->domain_table[rs1 & 0xFF] = 0;
}

// ===== Capability =====

uint64_t helper_xplenum_tcapld(void *env, uint64_t rs1) {
    XPlenumEmulState *s = &g_xps;
    s->xpperf_cnt++;
    if (!(s->xpstatus & 0x4)) { s->xpexc_cause = 0x2; return 0; }
    uint32_t idx = rs1 & 0x3F;
    if (!s->cap_table[idx].valid)   { s->xpexc_cause = 0x2; return 0; }
    if (s->cap_table[idx].revoked)  { s->xpexc_cause = 0x3; return 0; }
    return s->cap_table[idx].perms;
}

uint64_t helper_xplenum_tcapchk(void *env, uint64_t rs1, uint64_t rs2) {
    XPlenumEmulState *s = &g_xps;
    s->xpperf_cnt++;
    if (!(s->xpstatus & 0x4)) { s->xpexc_cause = 0x2; return 0; }
    uint32_t idx = rs1 & 0x3F;
    if (!s->cap_table[idx].valid || s->cap_table[idx].revoked) return 0;
    uint32_t addr = (uint32_t)rs2;
    return (addr >= s->cap_table[idx].base && addr < s->cap_table[idx].bound) ? 1 : 0;
}

void helper_xplenum_tcapst(void *env, uint64_t rs1, uint64_t rs2) {
    XPlenumEmulState *s = &g_xps;
    s->xpperf_cnt++;
    if (!(s->xpstatus & 0x4)) { s->xpexc_cause = 0x2; return; }
    uint32_t idx = rs1 & 0x3F;
    s->cap_table[idx].base    = (uint32_t)rs2;
    s->cap_table[idx].bound   = (uint32_t)rs2 + 0x1000;
    s->cap_table[idx].perms   = 0x7;
    s->cap_table[idx].valid   = 1;
    s->cap_table[idx].revoked = 0;
}

uint64_t helper_xplenum_tcaprev(void *env, uint64_t rs1) {
    XPlenumEmulState *s = &g_xps;
    s->xpperf_cnt++;
    if (!(s->xpstatus & 0x4)) { s->xpexc_cause = 0x2; return 0; }
    uint32_t idx = rs1 & 0x3F;
    if (s->cap_table[idx].valid) { s->cap_table[idx].revoked = 1; return 1; }
    return 0;
}

// ===== Crypto / Rotation =====

uint64_t helper_xplenum_trotl(void *env, uint64_t rs1, uint64_t rs2) {
    XPlenumEmulState *s = &g_xps;
    s->xpperf_cnt++;
    uint32_t v = (uint32_t)rs1, sh = (uint32_t)rs2 & 0x1F;
    return (v << sh) | (v >> (32 - sh));
}

uint64_t helper_xplenum_trotr(void *env, uint64_t rs1, uint64_t rs2) {
    XPlenumEmulState *s = &g_xps;
    s->xpperf_cnt++;
    uint32_t v = (uint32_t)rs1, sh = (uint32_t)rs2 & 0x1F;
    return (v >> sh) | (v << (32 - sh));
}

uint64_t helper_xplenum_ttbox(void *env, uint64_t rs1) {
    XPlenumEmulState *s = &g_xps;
    s->xpperf_cnt++;
    uint32_t idx = (uint32_t)rs1 & 0xFF;
    return (idx < 243) ? trit_sbox[idx] : ((uint32_t)rs1 ^ 0xFF);
}

uint64_t helper_xplenum_tperm(void *env, uint64_t rs1, uint64_t rs2) {
    XPlenumEmulState *s = &g_xps;
    s->xpperf_cnt++;
    uint32_t val = (uint32_t)rs1, key = (uint32_t)rs2;
    uint32_t result = 0;
    for (int i = 0; i < 16; i++) {
        uint32_t src_pos = ((key >> (i * 2)) & 0x3) % 16;
        uint32_t trit = (val >> (src_pos * 2)) & 0x3;
        result |= (trit << (i * 2));
    }
    return result;
}

// ===== Trit Encoding =====

uint64_t helper_xplenum_ttrit(void *env, uint64_t rs1) {
    XPlenumEmulState *s = &g_xps;
    s->xpperf_cnt++;
    uint32_t val = (uint32_t)rs1, result = 0;
    for (int i = 0; i < 16; i++) {
        uint32_t pair = (val >> (i * 2)) & 0x3;
        if (pair == 3) pair = 2;
        result |= (pair << (i * 2));
    }
    return result;
}

uint64_t helper_xplenum_tdetrit(void *env, uint64_t rs1) {
    XPlenumEmulState *s = &g_xps;
    s->xpperf_cnt++;
    uint32_t val = (uint32_t)rs1, result = 0;
    for (int i = 0; i < 16; i++) {
        uint32_t trit = (val >> (i * 2)) & 0x3;
        if (trit <= 2) result |= (trit << (i * 2));
    }
    return result;
}

// ===== Signal Processing =====

uint64_t helper_xplenum_tsigflt(void *env, uint64_t rs1, uint64_t rs2) {
    XPlenumEmulState *s = &g_xps;
    s->xpperf_cnt++;
    if (!(s->xpstatus & 0x8)) { s->xpexc_cause = 0x7; return 0; }
    int32_t sig = (int32_t)(uint32_t)rs1;
    int32_t coeff = (int32_t)(uint32_t)rs2;
    int32_t c0 = (coeff >> 0)  & 0xFF, c1 = (coeff >> 8)  & 0xFF;
    int32_t c2 = (coeff >> 16) & 0xFF, c3 = (coeff >> 24) & 0xFF;
    int32_t s0 = (sig >> 0)  & 0xFF, s1 = (sig >> 8)  & 0xFF;
    int32_t s2 = (sig >> 16) & 0xFF, s3 = (sig >> 24) & 0xFF;
    return (uint32_t)((s0*c0 + s1*c1 + s2*c2 + s3*c3) >> 8);
}

uint64_t helper_xplenum_tsigcmp(void *env, uint64_t rs1, uint64_t rs2) {
    XPlenumEmulState *s = &g_xps;
    s->xpperf_cnt++;
    if (!(s->xpstatus & 0x8)) { s->xpexc_cause = 0x7; return 0; }
    int32_t diff = (int32_t)(uint32_t)rs1 - (int32_t)(uint32_t)rs2;
    if (diff > 0) return 1;
    if (diff < 0) return 0xFFFFFFFF;
    return 0;
}

uint64_t helper_xplenum_tsigacc(void *env, uint64_t rs1, uint64_t rs2) {
    XPlenumEmulState *s = &g_xps;
    s->xpperf_cnt++;
    if (!(s->xpstatus & 0x8)) { s->xpexc_cause = 0x7; return 0; }
    s->sig_accumulator += (uint32_t)rs1 * (uint32_t)rs2;
    return s->sig_accumulator;
}

// ---------------------------------------------------------------------------
// QEMU TCG instruction decode template (trans_xplenum.c.inc)
// ---------------------------------------------------------------------------
//
// static bool trans_xplenum(DisasContext *ctx, arg_r *a, uint32_t insn)
// {
//     uint32_t funct3 = extract32(insn, 12, 3);
//     uint32_t funct7 = extract32(insn, 25, 7);
//     TCGv src1 = get_gpr(ctx, a->rs1, EXT_NONE);
//     TCGv src2 = get_gpr(ctx, a->rs2, EXT_NONE);
//     TCGv dest = dest_gpr(ctx, a->rd);
//
//     switch (funct3) {
//     case 0: /* TMASK group */
//         switch (funct7) {
//         case 0: gen_helper_xplenum_tmask(dest, tcg_env, src1, src2); break;
//         case 1: gen_helper_xplenum_tunmask(dest, tcg_env, src1, src2); break;
//         case 2: gen_helper_xplenum_tmaskr(dest, tcg_env); break;
//         case 3: gen_helper_xplenum_tmaskrf(dest, tcg_env, src1); break;
//         default: return false;
//         }
//         break;
//     case 1: /* TDOM group */
//         switch (funct7) {
//         case 0: gen_helper_xplenum_tdomset(tcg_env, src1, src2); return true;
//         case 1: gen_helper_xplenum_tdomchk(dest, tcg_env, src1, src2); break;
//         case 2: gen_helper_xplenum_tdomclr(tcg_env, src1); return true;
//         case 3: gen_helper_xplenum_tdomxfr(tcg_env, src1, src2); return true;
//         default: return false;
//         }
//         break;
//     /* ... remaining groups follow same pattern ... */
//     default:
//         return false;
//     }
//     gen_set_gpr(ctx, a->rd, dest);
//     return true;
// }
