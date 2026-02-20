// =============================================================================
// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// XPLENUM — RISC-V Ternary Security Extension
// Domain Isolation Unit (xplenum_domain_unit.v)
// Stage 4 — 256-entry hardware isolation domain table
// =============================================================================

`include "xplenum_pkg.vh"

module xplenum_domain_unit (
    input  wire        clk,
    input  wire        rst_n,

    // Control
    input  wire        dom_en,         // XPSTATUS[1]
    input  wire [6:0]  funct7,
    input  wire        valid,

    // Current domain context
    input  wire [7:0]  current_dom_id, // From CSR_XPDOMID

    // Operands
    input  wire [31:0] rs1_data,
    input  wire [31:0] rs2_data,

    // Outputs
    output reg  [31:0] result,
    output reg         result_valid,
    output reg  [3:0]  exc_code
);

    // -----------------------------------------------------------------------
    // 256-entry domain tag table
    // Each entry: [31:24] owner, [23:16] perms, [15:8] xfer_auth, [7:0] state
    // -----------------------------------------------------------------------
    reg [31:0] dom_table [0:`DOM_TABLE_SIZE-1];

    integer i;

    // -----------------------------------------------------------------------
    // Initialization
    // -----------------------------------------------------------------------
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            for (i = 0; i < `DOM_TABLE_SIZE; i = i + 1) begin
                dom_table[i] <= 32'h0; // All entries INVALID
            end
        end
    end

    // -----------------------------------------------------------------------
    // Field extraction helpers
    // -----------------------------------------------------------------------
    wire [7:0] idx       = rs1_data[7:0];
    wire [31:0] tag      = dom_table[idx];
    wire [7:0] tag_owner = tag[`DOM_OWNER_HI:`DOM_OWNER_LO];
    wire [7:0] tag_perms = tag[`DOM_PERM_HI:`DOM_PERM_LO];
    wire [7:0] tag_xfer  = tag[`DOM_XFER_HI:`DOM_XFER_LO];
    wire [7:0] tag_state = tag[`DOM_STATE_HI:`DOM_STATE_LO];

    wire owner_match     = (tag_owner == current_dom_id);
    wire is_active       = (tag_state == `DOM_ACTIVE);

    // -----------------------------------------------------------------------
    // Execution logic
    // -----------------------------------------------------------------------
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            result       <= 32'h0;
            result_valid <= 1'b0;
            exc_code     <= `XP_EXC_NONE;
        end else if (valid) begin
            result_valid <= 1'b0;
            exc_code     <= `XP_EXC_NONE;

            if (!dom_en) begin
                exc_code     <= `XP_EXC_DOM_VIOLATION;
                result_valid <= 1'b1;
                result       <= 32'h0;
            end else begin
                result_valid <= 1'b1;
                case (funct7)
                    `F7_TDOMSET: begin
                        // TDOMSET rd, rs1, rs2 — set domain tag
                        if (tag_state == `DOM_INVALID || owner_match) begin
                            result <= tag; // Return previous
                            dom_table[idx] <= rs2_data;
                        end else begin
                            exc_code <= `XP_EXC_DOM_VIOLATION;
                            result   <= 32'h0;
                        end
                    end

                    `F7_TDOMCHK: begin
                        // TDOMCHK rd, rs1, rs2 — check permission
                        if (owner_match && ((tag_perms & rs2_data[7:0]) == rs2_data[7:0])) begin
                            result <= 32'h1;
                        end else begin
                            result <= 32'h0;
                        end
                    end

                    `F7_TDOMCLR: begin
                        // TDOMCLR rd, rs1 — clear domain tag
                        if (owner_match) begin
                            result <= tag;
                            dom_table[idx] <= 32'h0; // Set to INVALID
                        end else begin
                            exc_code <= `XP_EXC_DOM_VIOLATION;
                            result   <= 32'h0;
                        end
                    end

                    `F7_TDOMXFR: begin
                        // TDOMXFR rd, rs1, rs2 — transfer ownership
                        if (owner_match && is_active &&
                            (tag_xfer & (8'h1 << rs2_data[2:0])) != 8'h0) begin
                            result <= {rs2_data[7:0], tag_perms, tag_xfer, `DOM_TRANSFER};
                            dom_table[idx] <= {rs2_data[7:0], tag_perms, tag_xfer, `DOM_TRANSFER};
                        end else begin
                            exc_code <= `XP_EXC_DOM_VIOLATION;
                            result   <= 32'h0;
                        end
                    end

                    default: begin
                        result_valid <= 1'b0;
                    end
                endcase
            end
        end else begin
            result_valid <= 1'b0;
        end
    end

endmodule
