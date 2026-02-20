// =============================================================================
// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// XPLENUM — RISC-V Ternary Security Extension
// Pipeline Stall Controller (xplenum_stall_controller.v)
// Phase 2: Tasks 2.4–2.5 — Stall/hazard handling, exception support
//
// Manages pipeline stalls for multi-cycle XPlenum operations, data hazard
// detection, result forwarding, and exception delivery to CVA6 pipeline.
// =============================================================================

`include "xplenum_pkg.vh"

module xplenum_stall_controller (
    input  wire        clk,
    input  wire        rst_n,

    // -----------------------------------------------------------------------
    // Issue Stage Interface
    // -----------------------------------------------------------------------
    input  wire        issue_valid_i,      // Instruction issued from scoreboard
    input  wire        issue_is_xplenum_i, // Issued instruction targets XPlenum
    input  wire [4:0]  issue_rs1_addr_i,   // Source register 1 address
    input  wire [4:0]  issue_rs2_addr_i,   // Source register 2 address
    input  wire [4:0]  issue_rd_addr_i,    // Destination register address

    // -----------------------------------------------------------------------
    // XPlenum Status
    // -----------------------------------------------------------------------
    input  wire        xp_busy_i,          // XPlenum is executing multi-cycle op
    input  wire        xp_valid_i,         // XPlenum result valid this cycle
    input  wire [4:0]  xp_rd_addr_i,       // XPlenum result destination register
    input  wire [63:0] xp_result_i,        // XPlenum result value (for forwarding)

    // -----------------------------------------------------------------------
    // Stall and Forwarding Outputs
    // -----------------------------------------------------------------------
    output wire        stall_issue_o,      // Stall Issue stage (do not dispatch)
    output wire        insert_bubble_o,    // Insert NOP into Execute stage
    output wire        forward_valid_o,    // Result forwarding available
    output wire [4:0]  forward_rd_addr_o,  // Forwarded destination register
    output wire [63:0] forward_data_o,     // Forwarded result data

    // -----------------------------------------------------------------------
    // Exception Interface
    // -----------------------------------------------------------------------
    input  wire        xp_exception_i,     // XPlenum exception asserted
    input  wire [63:0] xp_exc_cause_i,     // mcause value
    input  wire [63:0] xp_exc_tval_i,      // mtval value

    output reg         trap_valid_o,       // Trap request to controller
    output reg  [63:0] trap_cause_o,       // mcause
    output reg  [63:0] trap_tval_o,        // mtval
    output wire        flush_request_o,    // Pipeline flush requested

    // -----------------------------------------------------------------------
    // Pipeline Control
    // -----------------------------------------------------------------------
    input  wire        flush_i             // External flush (branch mispredict, etc.)
);

    // -----------------------------------------------------------------------
    // In-flight destination register tracking
    // -----------------------------------------------------------------------
    reg        inflight_valid;
    reg [4:0]  inflight_rd;

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            inflight_valid <= 1'b0;
            inflight_rd    <= 5'h0;
        end else if (flush_i) begin
            inflight_valid <= 1'b0;
            inflight_rd    <= 5'h0;
        end else if (issue_valid_i && issue_is_xplenum_i) begin
            inflight_valid <= 1'b1;
            inflight_rd    <= issue_rd_addr_i;
        end else if (xp_valid_i) begin
            inflight_valid <= 1'b0;
        end
    end

    // -----------------------------------------------------------------------
    // RAW Hazard Detection
    //
    // A RAW hazard exists when:
    //   - An XPlenum instruction is in-flight (result not yet produced)
    //   - The next instruction reads from XPlenum's destination register
    //   - x0 is excluded (hardwired zero, no hazard possible)
    // -----------------------------------------------------------------------
    wire raw_hazard_rs1 = inflight_valid &&
                          (issue_rs1_addr_i == inflight_rd) &&
                          (inflight_rd != 5'h0);

    wire raw_hazard_rs2 = inflight_valid &&
                          (issue_rs2_addr_i == inflight_rd) &&
                          (inflight_rd != 5'h0);

    wire raw_hazard = (raw_hazard_rs1 || raw_hazard_rs2) && issue_valid_i;

    // -----------------------------------------------------------------------
    // WAW Hazard Detection
    //
    // A WAW hazard exists when:
    //   - An XPlenum instruction is in-flight
    //   - The next instruction writes to the same destination register
    //   - Both instructions are XPlenum (non-XPlenum uses separate write port)
    // -----------------------------------------------------------------------
    wire waw_hazard = inflight_valid &&
                      issue_is_xplenum_i &&
                      (issue_rd_addr_i == inflight_rd) &&
                      (inflight_rd != 5'h0) &&
                      issue_valid_i;

    // -----------------------------------------------------------------------
    // Structural Hazard
    //
    // XPlenum is a single functional unit — only one XPlenum instruction
    // can be in-flight at a time.
    // -----------------------------------------------------------------------
    wire structural_hazard = xp_busy_i && issue_is_xplenum_i && issue_valid_i;

    // -----------------------------------------------------------------------
    // Stall and Bubble Generation
    // -----------------------------------------------------------------------
    assign stall_issue_o  = raw_hazard || waw_hazard || structural_hazard;
    assign insert_bubble_o = stall_issue_o;

    // -----------------------------------------------------------------------
    // Result Forwarding
    //
    // When XPlenum produces a result, forward it to the Issue stage so
    // dependent instructions don't need to wait for register file writeback.
    // -----------------------------------------------------------------------
    assign forward_valid_o   = xp_valid_i && !xp_exception_i;
    assign forward_rd_addr_o = xp_rd_addr_i;
    assign forward_data_o    = xp_result_i;

    // -----------------------------------------------------------------------
    // Exception Delivery
    //
    // XPlenum exceptions are delivered to CVA6's controller as precise traps.
    // The controller will flush the pipeline and redirect to MTVEC.
    // -----------------------------------------------------------------------
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            trap_valid_o <= 1'b0;
            trap_cause_o <= 64'h0;
            trap_tval_o  <= 64'h0;
        end else if (flush_i) begin
            trap_valid_o <= 1'b0;
        end else if (xp_exception_i) begin
            trap_valid_o <= 1'b1;
            trap_cause_o <= xp_exc_cause_i;
            trap_tval_o  <= xp_exc_tval_i;
        end else begin
            trap_valid_o <= 1'b0;
        end
    end

    assign flush_request_o = xp_exception_i;

    // -----------------------------------------------------------------------
    // Performance Counters (for Phase 3.5 benchmarking)
    // -----------------------------------------------------------------------
    reg [31:0] stall_cycles;
    reg [31:0] raw_hazard_count;
    reg [31:0] structural_hazard_count;
    reg [31:0] instructions_executed;

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            stall_cycles           <= 32'h0;
            raw_hazard_count       <= 32'h0;
            structural_hazard_count <= 32'h0;
            instructions_executed  <= 32'h0;
        end else if (!flush_i) begin
            if (stall_issue_o)
                stall_cycles <= stall_cycles + 1;
            if (raw_hazard)
                raw_hazard_count <= raw_hazard_count + 1;
            if (structural_hazard)
                structural_hazard_count <= structural_hazard_count + 1;
            if (xp_valid_i && !xp_exception_i)
                instructions_executed <= instructions_executed + 1;
        end
    end

endmodule
