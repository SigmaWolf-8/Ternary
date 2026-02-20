// =============================================================================
// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// XPLENUM — RISC-V Ternary Security Extension
// Trit Encoding, Crypto, and Signal Processing Unit (xplenum_trit_unit.v)
// Stage 6 — Binary-ternary conversion, T-box, rotation, permutation, signals
// =============================================================================

`include "xplenum_pkg.vh"

module xplenum_trit_unit (
    input  wire        clk,
    input  wire        rst_n,

    // Control
    input  wire        sig_en,         // XPSTATUS[3] for signal ops
    input  wire [2:0]  funct3,
    input  wire [6:0]  funct7,
    input  wire        valid,

    // Operands
    input  wire [31:0] rs1_data,
    input  wire [31:0] rs2_data,

    // Signal config
    input  wire [31:0] sig_cfg,        // From CSR_XPSIG_CFG

    // Outputs
    output reg  [31:0] result,
    output reg         result_valid,
    output reg  [3:0]  exc_code
);

    // -----------------------------------------------------------------------
    // T-Box: 27-entry substitution table (3^3 entries)
    // Cryptographically designed nonlinear 3-trit-to-3-trit mapping
    // -----------------------------------------------------------------------
    reg [5:0] tbox [0:`TBOX_SIZE-1];

    integer i;

    initial begin
        //  Input (decimal)  →  Output (packed 3-trit)
        //  Nonlinear permutation over GF(3)^3
        tbox[ 0] = 6'b01_10_00;  // (-1,-1,-1) → (+1,-1, 0)
        tbox[ 1] = 6'b10_00_01;  // (-1,-1, 0) → (-1, 0,+1)
        tbox[ 2] = 6'b00_01_10;  // (-1,-1,+1) → ( 0,+1,-1)
        tbox[ 3] = 6'b10_01_00;  // (-1, 0,-1) → (-1,+1, 0)
        tbox[ 4] = 6'b01_00_10;  // (-1, 0, 0) → (+1, 0,-1)
        tbox[ 5] = 6'b00_10_01;  // (-1, 0,+1) → ( 0,-1,+1)
        tbox[ 6] = 6'b01_01_01;  // (-1,+1,-1) → (+1,+1,+1)
        tbox[ 7] = 6'b10_10_10;  // (-1,+1, 0) → (-1,-1,-1)
        tbox[ 8] = 6'b00_00_00;  // (-1,+1,+1) → ( 0, 0, 0)
        tbox[ 9] = 6'b10_01_01;  // ( 0,-1,-1) → (-1,+1,+1)
        tbox[10] = 6'b01_10_10;  // ( 0,-1, 0) → (+1,-1,-1)
        tbox[11] = 6'b00_00_01;  // ( 0,-1,+1) → ( 0, 0,+1)
        tbox[12] = 6'b01_00_00;  // ( 0, 0,-1) → (+1, 0, 0)
        tbox[13] = 6'b10_10_01;  // ( 0, 0, 0) → (-1,-1,+1)
        tbox[14] = 6'b00_01_10;  // ( 0, 0,+1) → ( 0,+1,-1)
        tbox[15] = 6'b10_00_10;  // ( 0,+1,-1) → (-1, 0,-1)
        tbox[16] = 6'b00_10_00;  // ( 0,+1, 0) → ( 0,-1, 0)
        tbox[17] = 6'b01_01_00;  // ( 0,+1,+1) → (+1,+1, 0)
        tbox[18] = 6'b00_10_10;  // (+1,-1,-1) → ( 0,-1,-1)
        tbox[19] = 6'b01_00_01;  // (+1,-1, 0) → (+1, 0,+1)
        tbox[20] = 6'b10_01_10;  // (+1,-1,+1) → (-1,+1,-1)
        tbox[21] = 6'b00_00_10;  // (+1, 0,-1) → ( 0, 0,-1)
        tbox[22] = 6'b10_10_00;  // (+1, 0, 0) → (-1,-1, 0)
        tbox[23] = 6'b01_01_10;  // (+1, 0,+1) → (+1,+1,-1)
        tbox[24] = 6'b10_00_00;  // (+1,+1,-1) → (-1, 0, 0)
        tbox[25] = 6'b00_01_01;  // (+1,+1, 0) → ( 0,+1,+1)
        tbox[26] = 6'b01_10_01;  // (+1,+1,+1) → (+1,-1,+1)
    end

    // -----------------------------------------------------------------------
    // Trit-to-index conversion (3-trit group → 0..26)
    // -----------------------------------------------------------------------
    function [4:0] trit3_to_idx;
        input [5:0] trits; // 3 trit pairs (6 bits)
        reg signed [2:0] t0, t1, t2;
        reg signed [7:0] val;
        begin
            case (trits[1:0])
                `TRIT_ZERO: t0 = 0;
                `TRIT_POS:  t0 = 1;
                `TRIT_NEG:  t0 = -1;
                default:    t0 = 0;
            endcase
            case (trits[3:2])
                `TRIT_ZERO: t1 = 0;
                `TRIT_POS:  t1 = 1;
                `TRIT_NEG:  t1 = -1;
                default:    t1 = 0;
            endcase
            case (trits[5:4])
                `TRIT_ZERO: t2 = 0;
                `TRIT_POS:  t2 = 1;
                `TRIT_NEG:  t2 = -1;
                default:    t2 = 0;
            endcase
            val = (t2 + 1) * 9 + (t1 + 1) * 3 + (t0 + 1);
            trit3_to_idx = val[4:0];
        end
    endfunction

    // -----------------------------------------------------------------------
    // Check for invalid trit encoding (11 in any pair)
    // -----------------------------------------------------------------------
    function trit_invalid;
        input [31:0] data;
        integer j;
        begin
            trit_invalid = 1'b0;
            for (j = 0; j < 16; j = j + 1) begin
                if (data[2*j +: 2] == `TRIT_INVALID)
                    trit_invalid = 1'b1;
            end
        end
    endfunction

    // -----------------------------------------------------------------------
    // Binary to balanced ternary conversion
    // -----------------------------------------------------------------------
    function [31:0] bin_to_bt;
        input [31:0] val;
        reg [31:0] out;
        reg [31:0] remaining;
        reg [1:0] rem;
        integer j;
        begin
            out = 32'h0;
            remaining = val;
            for (j = 0; j < 16; j = j + 1) begin
                rem = remaining % 3;
                case (rem)
                    2'd0: out[2*j +: 2] = `TRIT_ZERO;
                    2'd1: begin
                        out[2*j +: 2] = `TRIT_POS;
                        remaining = remaining - 1;
                    end
                    2'd2: begin
                        out[2*j +: 2] = `TRIT_NEG;
                        remaining = remaining + 1;
                    end
                    default: out[2*j +: 2] = `TRIT_ZERO;
                endcase
                remaining = remaining / 3;
            end
            bin_to_bt = out;
        end
    endfunction

    // -----------------------------------------------------------------------
    // Balanced ternary to binary conversion
    // -----------------------------------------------------------------------
    function [31:0] bt_to_bin;
        input [31:0] bt;
        reg signed [31:0] accum;
        reg signed [31:0] power;
        reg signed [2:0] tval;
        integer j;
        begin
            accum = 0;
            power = 1;
            for (j = 0; j < 16; j = j + 1) begin
                case (bt[2*j +: 2])
                    `TRIT_ZERO: tval = 0;
                    `TRIT_POS:  tval = 1;
                    `TRIT_NEG:  tval = -1;
                    default:    tval = 0;
                endcase
                accum = accum + tval * power;
                power = power * 3;
            end
            bt_to_bin = accum;
        end
    endfunction

    // -----------------------------------------------------------------------
    // Ternary rotation
    // -----------------------------------------------------------------------
    wire [3:0] rot_amt = rs2_data[3:0];
    wire [4:0] bit_rot = {rot_amt, 1'b0}; // *2 for trit pairs

    wire [31:0] rotl_result = (rs1_data << bit_rot) | (rs1_data >> (32 - bit_rot));
    wire [31:0] rotr_result = (rs1_data >> bit_rot) | (rs1_data << (32 - bit_rot));

    // -----------------------------------------------------------------------
    // Signal processing accumulator
    // -----------------------------------------------------------------------
    reg [31:0] sig_accum;

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n)
            sig_accum <= 32'h0;
    end

    // -----------------------------------------------------------------------
    // Main execution logic
    // -----------------------------------------------------------------------
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            result       <= 32'h0;
            result_valid <= 1'b0;
            exc_code     <= `XP_EXC_NONE;
        end else if (valid) begin
            result_valid <= 1'b0;
            exc_code     <= `XP_EXC_NONE;

            case (funct3)
                // =============================================================
                // Ternary Rotation / Crypto (funct3 = 011)
                // =============================================================
                `F3_TROT: begin
                    result_valid <= 1'b1;
                    case (funct7)
                        `F7_TROTL: begin
                            result <= rotl_result;
                        end

                        `F7_TROTR: begin
                            result <= rotr_result;
                        end

                        `F7_TTBOX: begin
                            // Apply T-box substitution to each 3-trit group
                            if (trit_invalid(rs1_data)) begin
                                exc_code <= `XP_EXC_TRIT_OVERFLOW;
                                result   <= 32'h0;
                            end else begin
                                result[5:0]   <= tbox[trit3_to_idx(rs1_data[5:0])];
                                result[11:6]  <= tbox[trit3_to_idx(rs1_data[11:6])];
                                result[17:12] <= tbox[trit3_to_idx(rs1_data[17:12])];
                                result[23:18] <= tbox[trit3_to_idx(rs1_data[23:18])];
                                result[29:24] <= tbox[trit3_to_idx(rs1_data[29:24])];
                                result[31:30] <= 2'b00; // Pad last 2 bits
                            end
                        end

                        `F7_TPERM: begin
                            // Trit permutation: each 4-bit nibble in rs2 is dest index
                            for (i = 0; i < 8; i = i + 1) begin
                                result[rs2_data[4*i +: 4] * 2 +: 2] <= rs1_data[2*i +: 2];
                            end
                        end

                        default: result_valid <= 1'b0;
                    endcase
                end

                // =============================================================
                // Trit Encoding / Decoding (funct3 = 100)
                // =============================================================
                `F3_TENC: begin
                    result_valid <= 1'b1;
                    case (funct7)
                        `F7_TTRIT: begin
                            // Binary → balanced ternary
                            result <= bin_to_bt(rs1_data);
                        end

                        `F7_TDETRIT: begin
                            // Balanced ternary → binary
                            if (trit_invalid(rs1_data)) begin
                                exc_code <= `XP_EXC_TRIT_OVERFLOW;
                                result   <= 32'h0;
                            end else begin
                                result <= bt_to_bin(rs1_data);
                            end
                        end

                        default: result_valid <= 1'b0;
                    endcase
                end

                // =============================================================
                // Signal Processing (funct3 = 101)
                // =============================================================
                `F3_TSIG: begin
                    if (!sig_en) begin
                        exc_code     <= `XP_EXC_NONE;
                        result_valid <= 1'b1;
                        result       <= 32'h0;
                    end else begin
                        result_valid <= 1'b1;
                        case (funct7)
                            `F7_TSIGFLT: begin
                                // IIR/FIR filter: simple multiply-accumulate
                                result <= (rs1_data * rs2_data[15:0]) >> sig_cfg[3:0];
                            end

                            `F7_TSIGCMP: begin
                                // Threshold compare → ternary classification
                                if (rs1_data > rs2_data + sig_cfg[15:8])
                                    result <= {30'h0, `TRIT_POS};   // Above
                                else if (rs1_data + sig_cfg[15:8] < rs2_data)
                                    result <= {30'h0, `TRIT_NEG};   // Below
                                else
                                    result <= {30'h0, `TRIT_ZERO};  // Within deadband
                            end

                            `F7_TSIGACC: begin
                                // EWMA accumulate
                                sig_accum <= (sig_accum * rs2_data[7:0] +
                                             rs1_data * (8'd255 - rs2_data[7:0])) >> 8;
                                result <= sig_accum;
                            end

                            default: result_valid <= 1'b0;
                        endcase
                    end
                end

                default: begin
                    result_valid <= 1'b0;
                end
            endcase
        end else begin
            result_valid <= 1'b0;
        end
    end

endmodule
