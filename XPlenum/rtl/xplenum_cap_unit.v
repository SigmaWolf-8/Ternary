// =============================================================================
// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// XPLENUM — RISC-V Ternary Security Extension
// Capability Unit (xplenum_cap_unit.v)
// Stage 5 — CHERI-inspired 64-entry capability table with O(1) revocation
// =============================================================================

`include "xplenum_pkg.vh"

module xplenum_cap_unit (
    input  wire        clk,
    input  wire        rst_n,

    // Control
    input  wire        cap_en,         // XPSTATUS[2]
    input  wire [6:0]  funct7,
    input  wire        valid,

    // Operands
    input  wire [31:0] rs1_data,
    input  wire [31:0] rs2_data,

    // Outputs
    output reg  [31:0] result,
    output reg         result_valid,
    output reg  [3:0]  exc_code
);

    // -----------------------------------------------------------------------
    // 64-entry capability table (each entry 64 bits)
    // [63:56] tag, [55:48] perms, [47:32] base, [31:16] bound,
    // [15:8] otype, [7:0] seal
    // -----------------------------------------------------------------------
    reg [63:0] cap_table [0:`CAP_TABLE_SIZE-1];

    // -----------------------------------------------------------------------
    // 64-bit revocation bitmap — O(1) revocation
    // -----------------------------------------------------------------------
    reg [63:0] revoke_bitmap;

    integer i;

    // -----------------------------------------------------------------------
    // Initialization
    // -----------------------------------------------------------------------
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            revoke_bitmap <= 64'h0;
            for (i = 0; i < `CAP_TABLE_SIZE; i = i + 1) begin
                cap_table[i] <= 64'h0;
            end
        end
    end

    // -----------------------------------------------------------------------
    // Index and validity checks
    // -----------------------------------------------------------------------
    wire [5:0]  cap_idx     = rs1_data[5:0];
    wire        idx_valid   = (cap_idx < `CAP_TABLE_SIZE);
    wire        is_revoked  = revoke_bitmap[cap_idx];
    wire [63:0] cap_entry   = cap_table[cap_idx];

    wire [7:0]  cap_tag     = cap_entry[`CAP_TAG_HI:`CAP_TAG_LO];
    wire [7:0]  cap_perms   = cap_entry[`CAP_PERM_HI:`CAP_PERM_LO];
    wire [15:0] cap_base    = cap_entry[`CAP_BASE_HI:`CAP_BASE_LO];
    wire [15:0] cap_bound   = cap_entry[`CAP_BOUND_HI:`CAP_BOUND_LO];
    wire [7:0]  cap_otype   = cap_entry[`CAP_OTYPE_HI:`CAP_OTYPE_LO];
    wire [7:0]  cap_seal    = cap_entry[`CAP_SEAL_HI:`CAP_SEAL_LO];

    wire        cap_is_valid = (cap_tag != 8'h0);
    wire        cap_is_open  = (cap_seal == `SEAL_OPEN);

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

            if (!cap_en) begin
                exc_code     <= `XP_EXC_CAP_INVALID;
                result_valid <= 1'b1;
                result       <= 32'h0;
            end else if (!idx_valid) begin
                exc_code     <= `XP_EXC_CAP_INVALID;
                result_valid <= 1'b1;
                result       <= 32'h0;
            end else begin
                result_valid <= 1'b1;
                case (funct7)
                    `F7_TCAPLD: begin
                        // TCAPLD rd, rs1, rs2 — load capability half
                        if (is_revoked) begin
                            exc_code <= `XP_EXC_CAP_REVOKED;
                            result   <= 32'h0;
                        end else begin
                            // rs2[0] selects half: 0=lower, 1=upper
                            result <= rs2_data[0] ? cap_entry[63:32] : cap_entry[31:0];
                        end
                    end

                    `F7_TCAPCHK: begin
                        // TCAPCHK rd, rs1, rs2 — check permissions
                        if (cap_is_valid && !is_revoked &&
                            ((cap_perms & rs2_data[7:0]) == rs2_data[7:0])) begin
                            result <= 32'h1;
                        end else begin
                            result <= 32'h0;
                        end
                    end

                    `F7_TCAPST: begin
                        // TCAPST rd, rs1, rs2 — store capability half
                        if (!cap_is_open && cap_is_valid) begin
                            exc_code <= `XP_EXC_PRIV_FAULT;
                            result   <= 32'h0;
                        end else begin
                            // Store to lower half (funct7 bit pattern selects)
                            result <= cap_entry[31:0];
                            cap_table[cap_idx][31:0] <= rs2_data;
                        end
                    end

                    `F7_TCAPREV: begin
                        // TCAPREV rd, rs1 — revoke capability (O(1) bitmap)
                        result <= {31'h0, revoke_bitmap[cap_idx]};
                        revoke_bitmap[cap_idx] <= 1'b1;
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
