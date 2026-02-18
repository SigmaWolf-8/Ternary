// =============================================================================
// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// XPLENUM — QEMU TCG Instruction Translation Template
// Phase 6, Task 6.1b
//
// Include this file in qemu/target/riscv/insn_trans/ and add to translate.c
// Include path: #include "insn_trans/trans_xplenum.c.inc"
//
// Decoder match: opcode == 0x0B (custom-0)
// =============================================================================

// Decode entry point — called from riscv translate.c custom opcode handler
static bool trans_xplenum_custom0(DisasContext *ctx, uint32_t insn)
{
    uint32_t funct3 = extract32(insn, 12, 3);
    uint32_t funct7 = extract32(insn, 25, 7);
    int rd  = extract32(insn, 7, 5);
    int rs1 = extract32(insn, 15, 5);
    int rs2 = extract32(insn, 20, 5);

    TCGv src1 = get_gpr(ctx, rs1, EXT_NONE);
    TCGv src2 = get_gpr(ctx, rs2, EXT_NONE);
    TCGv dest = dest_gpr(ctx, rd);

    switch (funct3) {

    // ---- Masking (funct3 = 000) ----
    case 0:
        switch (funct7) {
        case 0x00: // TMASK
            gen_helper_xplenum_tmask(dest, tcg_env, src1, src2);
            break;
        case 0x01: // TUNMASK
            gen_helper_xplenum_tunmask(dest, tcg_env, src1, src2);
            break;
        case 0x02: // TMASKR
            gen_helper_xplenum_tmaskr(dest, tcg_env);
            break;
        case 0x03: // TMASKRF
            gen_helper_xplenum_tmaskrf(dest, tcg_env, src1);
            break;
        default:
            return false;
        }
        gen_set_gpr(ctx, rd, dest);
        return true;

    // ---- Domain Isolation (funct3 = 001) ----
    case 1:
        switch (funct7) {
        case 0x00: // TDOMSET (no rd write)
            gen_helper_xplenum_tdomset(tcg_env, src1, src2);
            return true;
        case 0x01: // TDOMCHK
            gen_helper_xplenum_tdomchk(dest, tcg_env, src1, src2);
            break;
        case 0x02: // TDOMCLR (no rd write)
            gen_helper_xplenum_tdomclr(tcg_env, src1);
            return true;
        case 0x03: // TDOMXFR (no rd write)
            gen_helper_xplenum_tdomxfr(tcg_env, src1, src2);
            return true;
        default:
            return false;
        }
        gen_set_gpr(ctx, rd, dest);
        return true;

    // ---- Capability (funct3 = 010) ----
    case 2:
        switch (funct7) {
        case 0x00: // TCAPLD
            gen_helper_xplenum_tcapld(dest, tcg_env, src1);
            break;
        case 0x01: // TCAPCHK
            gen_helper_xplenum_tcapchk(dest, tcg_env, src1, src2);
            break;
        case 0x02: // TCAPST (no rd write)
            gen_helper_xplenum_tcapst(tcg_env, src1, src2);
            return true;
        case 0x03: // TCAPREV
            gen_helper_xplenum_tcaprev(dest, tcg_env, src1);
            break;
        default:
            return false;
        }
        gen_set_gpr(ctx, rd, dest);
        return true;

    // ---- Crypto / Rotation (funct3 = 011) ----
    case 3:
        switch (funct7) {
        case 0x00: // TROTL
            gen_helper_xplenum_trotl(dest, tcg_env, src1, src2);
            break;
        case 0x01: // TROTR
            gen_helper_xplenum_trotr(dest, tcg_env, src1, src2);
            break;
        case 0x02: // TTBOX
            gen_helper_xplenum_ttbox(dest, tcg_env, src1);
            break;
        case 0x03: // TPERM
            gen_helper_xplenum_tperm(dest, tcg_env, src1, src2);
            break;
        default:
            return false;
        }
        gen_set_gpr(ctx, rd, dest);
        return true;

    // ---- Trit Encoding (funct3 = 100) ----
    case 4:
        switch (funct7) {
        case 0x00: // TTRIT
            gen_helper_xplenum_ttrit(dest, tcg_env, src1);
            break;
        case 0x01: // TDETRIT
            gen_helper_xplenum_tdetrit(dest, tcg_env, src1);
            break;
        default:
            return false;
        }
        gen_set_gpr(ctx, rd, dest);
        return true;

    // ---- Signal Processing (funct3 = 101) ----
    case 5:
        switch (funct7) {
        case 0x00: // TSIGFLT
            gen_helper_xplenum_tsigflt(dest, tcg_env, src1, src2);
            break;
        case 0x01: // TSIGCMP
            gen_helper_xplenum_tsigcmp(dest, tcg_env, src1, src2);
            break;
        case 0x02: // TSIGACC
            gen_helper_xplenum_tsigacc(dest, tcg_env, src1, src2);
            break;
        default:
            return false;
        }
        gen_set_gpr(ctx, rd, dest);
        return true;

    // ---- CSR Access (funct3 = 111) ----
    case 7:
        // CSR operations handled through standard CSR path
        // with xplenum_csr_read/write hooks
        return false;

    default:
        return false;
    }
}
