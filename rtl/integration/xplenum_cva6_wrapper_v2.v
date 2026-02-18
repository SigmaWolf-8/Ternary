// =============================================================================
// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// XPLENUM — CVA6 Integration Wrapper v2.0 (Phase 8)
//
// Updated wrapper for 64-bit data path integration between
// CVA6 execute stage and xplenum_top_v2.
//
// Changes from v1.0:
//   - 64-bit data path (was 32-bit)
//   - Dual-opcode recognition (Custom-0 + Custom-1)
//   - Tamper lockdown signal propagated to core exception logic
//   - Sign-extension updated for 64-bit results
// =============================================================================

`include "xplenum_pkg.vh"

module xplenum_cva6_wrapper_v2 (
    input  wire         clk,
    input  wire         rst_n,

    // From CVA6 decode/issue stage
    input  wire [31:0]  issue_instruction,
    input  wire         issue_valid,
    input  wire [63:0]  issue_rs1_data,
    input  wire [63:0]  issue_rs2_data,
    input  wire [1:0]   priv_mode,

    // To CVA6 writeback stage
    output wire [63:0]  wb_result,
    output wire         wb_valid,
    output wire [4:0]   wb_rd_addr,

    // To CVA6 exception logic
    output wire         wb_exception,
    output wire [63:0]  wb_exception_cause,
    output wire         tamper_lockdown,

    // External entropy (from platform TRNG)
    input  wire [255:0] entropy,
    input  wire         entropy_valid,
    input  wire         reseed_request,

    // Status (to platform)
    output wire         drbg_health_error,
    output wire         drbg_ready
);

    // -- Custom instruction recognition --
    wire [6:0] opcode = issue_instruction[6:0];
    wire is_xplenum_insn = (opcode == `XP_OPCODE) || (opcode == `XP_OPCODE_PQC);

    // -- Stall logic (for multi-cycle operations) --
    wire xp_busy;  // From xplenum_top_v2 when HO mask or PQC is computing

    // -- XPlenum core --
    wire [63:0] xp_rd_data;
    wire        xp_rd_wen;
    wire [4:0]  xp_rd_addr;
    wire        xp_exception;
    wire [3:0]  xp_exc_code;
    wire        xp_lockdown;

    xplenum_top_v2 u_xplenum (
        .clk(clk),
        .rst_n(rst_n),
        .instruction(issue_instruction),
        .instr_valid(issue_valid && is_xplenum_insn),
        .rs1_data(issue_rs1_data),
        .rs2_data(issue_rs2_data),
        .rd_data(xp_rd_data),
        .rd_write_en(xp_rd_wen),
        .rd_addr(xp_rd_addr),
        .xp_exception(xp_exception),
        .xp_exc_code(xp_exc_code),
        .entropy_i(entropy),
        .entropy_valid_i(entropy_valid),
        .reseed_req_i(reseed_request),
        .drbg_health_err_o(drbg_health_error),
        .drbg_ready_o(drbg_ready),
        .tamper_lockdown_o(xp_lockdown)
    );

    // -- Result to writeback (64-bit, no sign extension needed) --
    assign wb_result    = xp_rd_data;
    assign wb_valid     = xp_rd_wen;
    assign wb_rd_addr   = xp_rd_addr;

    // -- Exception mapping to RISC-V standard cause codes --
    // XPlenum exceptions map to custom cause = 24 + xp_exc_code
    assign wb_exception       = xp_exception;
    assign wb_exception_cause = {56'd0, 4'd0, xp_exc_code} + 64'd24;

    // -- Tamper lockdown propagation --
    assign tamper_lockdown = xp_lockdown;

endmodule
