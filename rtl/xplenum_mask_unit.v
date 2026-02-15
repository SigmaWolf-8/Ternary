// =============================================================================
// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// XPLENUM — RISC-V Ternary Security Extension
// Ternary Masking Unit (xplenum_mask_unit.v)
// Stage 3 — Side-channel resistant ternary masking with hardware TRNG
// =============================================================================

`include "xplenum_pkg.vh"

module xplenum_mask_unit (
    input  wire        clk,
    input  wire        rst_n,

    // Control
    input  wire        mask_en,        // XPSTATUS[0]
    input  wire [6:0]  funct7,
    input  wire        valid,          // Instruction valid

    // Operands
    input  wire [31:0] rs1_data,
    input  wire [31:0] rs2_data,

    // LFSR seed interface
    input  wire        seed_wr,
    input  wire [31:0] seed_data,

    // Outputs
    output reg  [31:0] result,
    output reg         result_valid,
    output wire [31:0] mask_state,     // CSR_XPMASK_STATE read-back
    output reg  [3:0]  exc_code
);

    // -----------------------------------------------------------------------
    // LFSR-based TRNG — 32-bit maximal-length
    // Polynomial: x^32 + x^22 + x^2 + x + 1
    // -----------------------------------------------------------------------
    reg [31:0] lfsr;

    wire lfsr_feedback = lfsr[31] ^ lfsr[21] ^ lfsr[1] ^ lfsr[0];

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            lfsr <= 32'hDEAD_BEEF;
        end else if (seed_wr) begin
            lfsr <= (seed_data != 32'h0) ? seed_data : 32'hDEAD_BEEF;
        end else if (valid && mask_en) begin
            lfsr <= {lfsr[30:0], lfsr_feedback};
        end
    end

    // -----------------------------------------------------------------------
    // Mask state register (stores last generated random mask)
    // -----------------------------------------------------------------------
    reg [31:0] mask_state_reg;
    assign mask_state = mask_state_reg;

    // -----------------------------------------------------------------------
    // Trit-wise modular arithmetic (mod 3)
    // Each trit is encoded as 2-bit pair: 00=0, 01=+1, 10=-1, 11=invalid
    // -----------------------------------------------------------------------

    // Trit addition: (a + b) mod 3 in balanced ternary
    function [1:0] trit_add;
        input [1:0] a;
        input [1:0] b;
        reg [2:0] sum;
        begin
            case ({a, b})
                4'b00_00: trit_add = `TRIT_ZERO;  //  0 + 0  =  0
                4'b00_01: trit_add = `TRIT_POS;   //  0 + 1  = +1
                4'b00_10: trit_add = `TRIT_NEG;   //  0 + -1 = -1
                4'b01_00: trit_add = `TRIT_POS;   // +1 + 0  = +1
                4'b01_01: trit_add = `TRIT_NEG;   // +1 + 1  = -1 (mod 3 wrap)
                4'b01_10: trit_add = `TRIT_ZERO;  // +1 + -1 =  0
                4'b10_00: trit_add = `TRIT_NEG;   // -1 + 0  = -1
                4'b10_01: trit_add = `TRIT_ZERO;  // -1 + 1  =  0
                4'b10_10: trit_add = `TRIT_POS;   // -1 + -1 = +1 (mod 3 wrap)
                default:  trit_add = `TRIT_ZERO;  // Invalid → 0
            endcase
        end
    endfunction

    // Trit subtraction: (a - b) mod 3 = a + (-b)
    function [1:0] trit_sub;
        input [1:0] a;
        input [1:0] b;
        reg [1:0] neg_b;
        begin
            case (b)
                `TRIT_ZERO: neg_b = `TRIT_ZERO;
                `TRIT_POS:  neg_b = `TRIT_NEG;
                `TRIT_NEG:  neg_b = `TRIT_POS;
                default:    neg_b = `TRIT_ZERO;
            endcase
            trit_sub = trit_add(a, neg_b);
        end
    endfunction

    // -----------------------------------------------------------------------
    // 16-trit parallel mask/unmask operations
    // -----------------------------------------------------------------------
    function [31:0] apply_mask;
        input [31:0] data;
        input [31:0] mask;
        integer i;
        begin
            for (i = 0; i < 16; i = i + 1) begin
                apply_mask[2*i +: 2] = trit_add(data[2*i +: 2], mask[2*i +: 2]);
            end
        end
    endfunction

    function [31:0] remove_mask;
        input [31:0] data;
        input [31:0] mask;
        integer i;
        begin
            for (i = 0; i < 16; i = i + 1) begin
                remove_mask[2*i +: 2] = trit_sub(data[2*i +: 2], mask[2*i +: 2]);
            end
        end
    endfunction

    // -----------------------------------------------------------------------
    // Convert raw LFSR bits to valid trit encoding
    // Map 2-bit pairs: 00→00, 01→01, 10→10, 11→00 (invalid→zero)
    // -----------------------------------------------------------------------
    function [31:0] lfsr_to_trits;
        input [31:0] raw;
        integer i;
        begin
            for (i = 0; i < 16; i = i + 1) begin
                if (raw[2*i +: 2] == `TRIT_INVALID)
                    lfsr_to_trits[2*i +: 2] = `TRIT_ZERO;
                else
                    lfsr_to_trits[2*i +: 2] = raw[2*i +: 2];
            end
        end
    endfunction

    wire [31:0] random_mask = lfsr_to_trits(lfsr);

    // -----------------------------------------------------------------------
    // Main execution logic
    // -----------------------------------------------------------------------
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            result       <= 32'h0;
            result_valid <= 1'b0;
            mask_state_reg <= 32'h0;
            exc_code     <= `XP_EXC_NONE;
        end else if (valid) begin
            result_valid <= 1'b0;
            exc_code     <= `XP_EXC_NONE;

            if (!mask_en) begin
                exc_code     <= `XP_EXC_MASK_FAULT;
                result_valid <= 1'b1;
                result       <= 32'h0;
            end else begin
                result_valid <= 1'b1;
                case (funct7)
                    `F7_TMASK: begin
                        // TMASK rd, rs1, rs2 — apply mask
                        result <= apply_mask(rs1_data, rs2_data);
                    end

                    `F7_TUNMASK: begin
                        // TUNMASK rd, rs1, rs2 — remove mask
                        result <= remove_mask(rs1_data, rs2_data);
                    end

                    `F7_TMASKR: begin
                        // TMASKR rd, rs1 — generate random mask + apply
                        mask_state_reg <= random_mask;
                        result <= apply_mask(rs1_data, random_mask);
                    end

                    `F7_TMASKRF: begin
                        // TMASKRF rd, rs1 — unmask with old, remask with new
                        result <= apply_mask(
                            remove_mask(rs1_data, mask_state_reg),
                            random_mask
                        );
                        mask_state_reg <= random_mask;
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
