// =============================================================================
// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// XPLENUM — RISC-V Ternary Security Extension
// CVA6 Integration Top Module (xplenum_cva6_top.v)
// Phase 2: Task 2.6 — Complete integration top-level with clock domain alignment
//
// Instantiates xplenum_cva6_wrapper and xplenum_stall_controller,
// providing a single integration point for CVA6's Execute stage.
// =============================================================================

`include "xplenum_pkg.vh"

module xplenum_cva6_top (
    input  wire        clk,
    input  wire        rst_n,

    // -----------------------------------------------------------------------
    // From CVA6 Issue Stage
    // -----------------------------------------------------------------------
    input  wire        issue_valid_i,
    input  wire        issue_is_xplenum_i,
    input  wire [31:0] instruction_i,
    input  wire [63:0] rs1_data_i,
    input  wire [63:0] rs2_data_i,
    input  wire [4:0]  rs1_addr_i,
    input  wire [4:0]  rs2_addr_i,
    input  wire [4:0]  rd_addr_i,
    input  wire [3:0]  trans_id_i,

    // -----------------------------------------------------------------------
    // To CVA6 Commit Stage
    // -----------------------------------------------------------------------
    output wire [63:0] result_o,
    output wire        result_valid_o,
    output wire [4:0]  result_rd_addr_o,
    output wire [3:0]  result_trans_id_o,

    // -----------------------------------------------------------------------
    // Pipeline Control
    // -----------------------------------------------------------------------
    output wire        ready_o,
    output wire        busy_o,
    output wire        stall_issue_o,
    output wire        insert_bubble_o,
    input  wire        flush_i,

    // -----------------------------------------------------------------------
    // Result Forwarding (to Issue Stage forwarding mux)
    // -----------------------------------------------------------------------
    output wire        fwd_valid_o,
    output wire [4:0]  fwd_rd_addr_o,
    output wire [63:0] fwd_data_o,

    // -----------------------------------------------------------------------
    // Exception/Trap (to CVA6 Controller)
    // -----------------------------------------------------------------------
    output wire        trap_valid_o,
    output wire [63:0] trap_cause_o,
    output wire [63:0] trap_tval_o,
    output wire        flush_request_o,

    // -----------------------------------------------------------------------
    // CSR Access (from CVA6 CSR Register File)
    // -----------------------------------------------------------------------
    input  wire [11:0] csr_addr_i,
    input  wire [63:0] csr_wdata_i,
    input  wire        csr_wen_i,
    output wire [63:0] csr_rdata_o,
    output wire        csr_valid_o
);

    // -----------------------------------------------------------------------
    // Clock Domain Alignment (Task 2.6)
    //
    // XPlenum operates in the same clock domain as CVA6. No clock domain
    // crossing is required. This module serves as documentation of that
    // design decision and provides a clean integration boundary.
    //
    // If future revisions require a separate clock domain (e.g., for a
    // high-frequency AES core in the DRBG), CDC synchronisers should be
    // inserted here, replacing the direct wire connections below.
    // -----------------------------------------------------------------------

    // Internal signals from wrapper
    wire [63:0] wrap_result;
    wire        wrap_valid;
    wire [4:0]  wrap_rd_addr;
    wire [3:0]  wrap_trans_id;
    wire        wrap_ready;
    wire        wrap_busy;
    wire        wrap_exception;
    wire [63:0] wrap_exc_cause;
    wire [63:0] wrap_exc_tval;

    // -----------------------------------------------------------------------
    // XPlenum CVA6 Wrapper Instance
    // -----------------------------------------------------------------------
    xplenum_cva6_wrapper u_wrapper (
        .clk              (clk),
        .rst_n            (rst_n),

        .xp_valid_i       (issue_valid_i && issue_is_xplenum_i),
        .xp_instruction_i (instruction_i),
        .xp_rs1_data_i    (rs1_data_i),
        .xp_rs2_data_i    (rs2_data_i),
        .xp_rd_addr_i     (rd_addr_i),
        .xp_trans_id_i    (trans_id_i),

        .xp_result_o      (wrap_result),
        .xp_valid_o       (wrap_valid),
        .xp_rd_addr_o     (wrap_rd_addr),
        .xp_trans_id_o    (wrap_trans_id),

        .xp_ready_o       (wrap_ready),
        .xp_busy_o        (wrap_busy),
        .flush_i          (flush_i),

        .xp_exception_o   (wrap_exception),
        .xp_exc_cause_o   (wrap_exc_cause),
        .xp_exc_tval_o    (wrap_exc_tval),

        .csr_xp_addr_i    (csr_addr_i),
        .csr_xp_wdata_i   (csr_wdata_i),
        .csr_xp_wen_i     (csr_wen_i),
        .csr_xp_rdata_o   (csr_rdata_o),
        .csr_xp_valid_o   (csr_valid_o)
    );

    // -----------------------------------------------------------------------
    // Stall Controller Instance
    // -----------------------------------------------------------------------
    xplenum_stall_controller u_stall_ctrl (
        .clk                    (clk),
        .rst_n                  (rst_n),

        .issue_valid_i          (issue_valid_i),
        .issue_is_xplenum_i     (issue_is_xplenum_i),
        .issue_rs1_addr_i       (rs1_addr_i),
        .issue_rs2_addr_i       (rs2_addr_i),
        .issue_rd_addr_i        (rd_addr_i),

        .xp_busy_i              (wrap_busy),
        .xp_valid_i             (wrap_valid),
        .xp_rd_addr_i           (wrap_rd_addr),
        .xp_result_i            (wrap_result),

        .stall_issue_o          (stall_issue_o),
        .insert_bubble_o        (insert_bubble_o),
        .forward_valid_o        (fwd_valid_o),
        .forward_rd_addr_o      (fwd_rd_addr_o),
        .forward_data_o         (fwd_data_o),

        .xp_exception_i         (wrap_exception),
        .xp_exc_cause_i         (wrap_exc_cause),
        .xp_exc_tval_i          (wrap_exc_tval),

        .trap_valid_o           (trap_valid_o),
        .trap_cause_o           (trap_cause_o),
        .trap_tval_o            (trap_tval_o),
        .flush_request_o        (flush_request_o),

        .flush_i                (flush_i)
    );

    // -----------------------------------------------------------------------
    // Output assignments
    // -----------------------------------------------------------------------
    assign result_o          = wrap_result;
    assign result_valid_o    = wrap_valid;
    assign result_rd_addr_o  = wrap_rd_addr;
    assign result_trans_id_o = wrap_trans_id;
    assign ready_o           = wrap_ready;
    assign busy_o            = wrap_busy;

endmodule
